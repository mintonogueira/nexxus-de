//! Versioned IPC protocol foundation for Nexxus.
//!
//! The transport is intentionally limited to local Unix domain sockets in this
//! stage. Higher-level services are not implemented here.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const MAX_FRAME_SIZE: usize = 1024 * 1024;
pub const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::new(1, 0);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
}

impl ProtocolVersion {
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Negotiates additive minor revisions inside one major compatibility
    /// boundary. A different major version is rejected before payload use.
    pub fn negotiate(self, peer: Self) -> Result<Self, ProtocolError> {
        if self.major != peer.major {
            return Err(ProtocolError::IncompatibleVersion { local: self, peer });
        }
        Ok(Self::new(self.major, self.minor.min(peer.minor)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MessageKind {
    Request,
    Response,
    Event,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Message<T> {
    pub protocol: ProtocolVersion,
    pub request_id: u64,
    pub kind: MessageKind,
    pub payload: T,
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("IPC frame exceeds the maximum size of {max} bytes: {actual} bytes")]
    OversizedFrame { actual: usize, max: usize },
    #[error("I/O error while processing IPC: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid IPC payload: {0}")]
    InvalidPayload(#[from] serde_json::Error),
    #[error("incompatible protocol version: local {local:?}, peer {peer:?}")]
    IncompatibleVersion {
        local: ProtocolVersion,
        peer: ProtocolVersion,
    },
    #[error("IPC endpoint parent directory is not private: '{0}'")]
    InsecureEndpointDirectory(PathBuf),
    #[error("IPC endpoint path is unsafe or belongs to another user: '{0}'")]
    UnsafeEndpointPath(PathBuf),
    #[error("IPC endpoint is already active: '{0}'")]
    EndpointInUse(PathBuf),
    #[error("IPC endpoint path has no usable parent directory: '{0}'")]
    InvalidEndpointPath(PathBuf),
}

/// Serializes one message into a length-prefixed frame.
pub fn encode<T: Serialize>(message: &Message<T>) -> Result<Vec<u8>, ProtocolError> {
    let payload = serde_json::to_vec(message)?;
    if payload.len() > MAX_FRAME_SIZE {
        return Err(ProtocolError::OversizedFrame {
            actual: payload.len(),
            max: MAX_FRAME_SIZE,
        });
    }
    let mut frame = Vec::with_capacity(4 + payload.len());
    frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

/// Deserializes a payload after framing has already enforced the size limit.
pub fn decode<T: DeserializeOwned>(payload: &[u8]) -> Result<Message<T>, ProtocolError> {
    if payload.len() > MAX_FRAME_SIZE {
        return Err(ProtocolError::OversizedFrame {
            actual: payload.len(),
            max: MAX_FRAME_SIZE,
        });
    }
    Ok(serde_json::from_slice(payload)?)
}

pub struct UnixConnection {
    stream: UnixStream,
}

impl UnixConnection {
    pub fn new(stream: UnixStream) -> Self {
        Self { stream }
    }

    pub fn send<T: Serialize>(&mut self, message: &Message<T>) -> Result<(), ProtocolError> {
        let frame = encode(message)?;
        self.stream.write_all(&frame)?;
        self.stream.flush()?;
        Ok(())
    }

    /// Reads the length prefix before allocating the payload buffer.
    pub fn receive<T: DeserializeOwned>(&mut self) -> Result<Message<T>, ProtocolError> {
        let mut length = [0u8; 4];
        self.stream.read_exact(&mut length)?;
        let length = u32::from_be_bytes(length) as usize;
        if length > MAX_FRAME_SIZE {
            return Err(ProtocolError::OversizedFrame {
                actual: length,
                max: MAX_FRAME_SIZE,
            });
        }
        let mut payload = vec![0u8; length];
        self.stream.read_exact(&mut payload)?;
        decode(&payload)
    }

    pub fn into_inner(self) -> UnixStream {
        self.stream
    }
}

/// Owned local IPC listener with safe stale-socket replacement and cleanup.
///
/// The parent directory must already be owned by the current uid and deny all
/// group/other access. Existing non-socket paths, symlinks or foreign sockets
/// are never removed.
pub struct UnixEndpoint {
    listener: UnixListener,
    path: PathBuf,
    device: u64,
    inode: u64,
}

impl UnixEndpoint {
    pub fn bind_private(path: impl Into<PathBuf>) -> Result<Self, ProtocolError> {
        let path = path.into();
        let parent = path
            .parent()
            .filter(|value| !value.as_os_str().is_empty())
            .ok_or_else(|| ProtocolError::InvalidEndpointPath(path.clone()))?;
        let uid = current_uid()?;
        validate_private_parent(parent, uid)?;
        prepare_socket_path(&path, uid)?;

        let listener = UnixListener::bind(&path)?;
        if let Err(source) = fs::set_permissions(&path, fs::Permissions::from_mode(0o600)) {
            let _ = fs::remove_file(&path);
            return Err(ProtocolError::Io(source));
        }
        let metadata = fs::symlink_metadata(&path)?;
        if !metadata.file_type().is_socket() || metadata.uid() != uid {
            let _ = fs::remove_file(&path);
            return Err(ProtocolError::UnsafeEndpointPath(path));
        }

        Ok(Self {
            listener,
            path,
            device: metadata.dev(),
            inode: metadata.ino(),
        })
    }

    pub fn accept(&self) -> Result<UnixConnection, ProtocolError> {
        let (stream, _) = self.listener.accept()?;
        Ok(UnixConnection::new(stream))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for UnixEndpoint {
    fn drop(&mut self) {
        // Never unlink a path replaced after bind. Matching device and inode
        // proves the pathname still refers to this endpoint instance.
        if let Ok(metadata) = fs::symlink_metadata(&self.path) {
            if metadata.file_type().is_socket()
                && metadata.dev() == self.device
                && metadata.ino() == self.inode
            {
                let _ = fs::remove_file(&self.path);
            }
        }
    }
}

fn current_uid() -> Result<u32, ProtocolError> {
    Ok(fs::metadata("/proc/self")?.uid())
}

fn validate_private_parent(path: &Path, uid: u32) -> Result<(), ProtocolError> {
    let metadata = fs::symlink_metadata(path)?;
    let mode = metadata.mode() & 0o777;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != uid
        || mode & 0o077 != 0
    {
        return Err(ProtocolError::InsecureEndpointDirectory(path.to_path_buf()));
    }
    Ok(())
}

fn prepare_socket_path(path: &Path, uid: u32) -> Result<(), ProtocolError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == ErrorKind::NotFound => return Ok(()),
        Err(source) => return Err(ProtocolError::Io(source)),
    };

    if metadata.file_type().is_symlink()
        || !metadata.file_type().is_socket()
        || metadata.uid() != uid
    {
        return Err(ProtocolError::UnsafeEndpointPath(path.to_path_buf()));
    }

    match UnixStream::connect(path) {
        Ok(_) => Err(ProtocolError::EndpointInUse(path.to_path_buf())),
        Err(source) if source.kind() == ErrorKind::ConnectionRefused => {
            // A same-user socket with a kernel-confirmed absent listener is the
            // only stale endpoint case eligible for automatic removal.
            fs::remove_file(path)?;
            Ok(())
        }
        Err(source) if source.kind() == ErrorKind::NotFound => Ok(()),
        Err(source) => Err(ProtocolError::Io(source)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::os::unix::fs::symlink;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct Ping {
        value: String,
    }

    fn private_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "nexxus-protocol-test-{}-{nonce}-{label}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        path
    }

    #[test]
    fn negotiates_minor_version() {
        assert_eq!(
            ProtocolVersion::new(1, 3)
                .negotiate(ProtocolVersion::new(1, 1))
                .unwrap(),
            ProtocolVersion::new(1, 1)
        );
        assert!(
            ProtocolVersion::new(1, 0)
                .negotiate(ProtocolVersion::new(2, 0))
                .is_err()
        );
    }

    #[test]
    fn round_trip_over_unix_stream_pair() {
        let (left, right) = UnixStream::pair().unwrap();
        let mut sender = UnixConnection::new(left);
        let mut receiver = UnixConnection::new(right);
        let message = Message {
            protocol: PROTOCOL_VERSION,
            request_id: 7,
            kind: MessageKind::Request,
            payload: Ping {
                value: "hello".into(),
            },
        };
        sender.send(&message).unwrap();
        let decoded: Message<Ping> = receiver.receive().unwrap();
        assert_eq!(decoded, message);
    }

    #[test]
    fn rejects_declared_oversized_frame_before_allocating_payload() {
        let (mut writer, reader) = UnixStream::pair().unwrap();
        writer
            .write_all(&((MAX_FRAME_SIZE as u32) + 1).to_be_bytes())
            .unwrap();
        let mut receiver = UnixConnection::new(reader);
        let error = receiver.receive::<Ping>().unwrap_err();
        assert!(matches!(error, ProtocolError::OversizedFrame { .. }));
    }

    #[test]
    fn private_endpoint_uses_mode_0600_and_removes_its_own_socket() {
        let dir = private_temp_dir("private");
        let path = dir.join("core.sock");
        {
            let endpoint = UnixEndpoint::bind_private(&path).unwrap();
            assert_eq!(endpoint.path(), path);
            let mode = fs::symlink_metadata(&path).unwrap().mode() & 0o777;
            assert_eq!(mode, 0o600);
            assert!(matches!(
                UnixEndpoint::bind_private(&path),
                Err(ProtocolError::EndpointInUse(_))
            ));
        }
        assert!(!path.exists());
        let _ = fs::remove_dir(dir);
    }

    #[test]
    fn private_endpoint_replaces_only_same_user_stale_socket() {
        let dir = private_temp_dir("stale");
        let path = dir.join("core.sock");
        let stale = UnixListener::bind(&path).unwrap();
        drop(stale);
        assert!(path.exists());
        let endpoint = UnixEndpoint::bind_private(&path).unwrap();
        drop(endpoint);
        assert!(!path.exists());
        let _ = fs::remove_dir(dir);
    }

    #[test]
    fn private_endpoint_rejects_symlink_path() {
        let dir = private_temp_dir("symlink");
        let target = dir.join("target");
        File::create(&target).unwrap();
        let path = dir.join("core.sock");
        symlink(&target, &path).unwrap();
        assert!(matches!(
            UnixEndpoint::bind_private(&path),
            Err(ProtocolError::UnsafeEndpointPath(_))
        ));
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(target);
        let _ = fs::remove_dir(dir);
    }
}
