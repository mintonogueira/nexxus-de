//! XDG and per-user runtime path resolution for the Nexxus Core.
//!
//! Runtime paths are treated as a trust boundary because local IPC sockets and
//! session state will live below them. The implementation therefore rejects
//! symlinks, foreign ownership and group/other permissions.

use std::env;
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PathError {
    #[error("HOME is not available as an absolute path and no valid XDG override was provided")]
    MissingHome,
    #[error("could not determine the current Unix uid from /proc/self: {0}")]
    CurrentUid(std::io::Error),
    #[error("runtime directory '{path}' has insecure ownership, permissions or file type")]
    InsecureRuntime { path: PathBuf },
    #[error("could not prepare runtime directory '{path}': {source}")]
    RuntimeIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Resolved per-user locations following the XDG base-directory contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NexxusPaths {
    config_home: PathBuf,
    data_home: PathBuf,
    cache_home: PathBuf,
    state_home: PathBuf,
    runtime_base: PathBuf,
    runtime_fallback: bool,
}

impl NexxusPaths {
    /// Resolves absolute XDG overrides and falls back to HOME-based paths.
    pub fn from_environment() -> Result<Self, PathError> {
        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute());
        let config_home = xdg_or_home("XDG_CONFIG_HOME", home.as_deref(), ".config")?;
        let data_home = xdg_or_home("XDG_DATA_HOME", home.as_deref(), ".local/share")?;
        let cache_home = xdg_or_home("XDG_CACHE_HOME", home.as_deref(), ".cache")?;
        let state_home = xdg_or_home("XDG_STATE_HOME", home.as_deref(), ".local/state")?;

        let runtime = env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute());
        let (runtime_base, runtime_fallback) = match runtime {
            Some(path) => (path, false),
            None => {
                let uid = current_uid()?;
                (env::temp_dir().join(format!("nexxus-runtime-{uid}")), true)
            }
        };

        Ok(Self {
            config_home,
            data_home,
            cache_home,
            state_home,
            runtime_base,
            runtime_fallback,
        })
    }

    pub fn config_dir(&self) -> PathBuf {
        self.config_home.join("nexxus")
    }
    pub fn data_dir(&self) -> PathBuf {
        self.data_home.join("nexxus")
    }
    pub fn cache_dir(&self) -> PathBuf {
        self.cache_home.join("nexxus")
    }
    pub fn state_dir(&self) -> PathBuf {
        self.state_home.join("nexxus")
    }
    pub fn runtime_dir(&self) -> PathBuf {
        self.runtime_base.join("nexxus")
    }
    pub fn uses_runtime_fallback(&self) -> bool {
        self.runtime_fallback
    }

    /// Creates the Nexxus runtime namespace and verifies it is private.
    ///
    /// Existing paths are never chmod'ed into compliance because that could
    /// hide an ownership or symlink problem.
    pub fn prepare_runtime_dir(&self) -> Result<PathBuf, PathError> {
        let uid = current_uid()?;
        if self.runtime_fallback {
            create_private_dir(&self.runtime_base)?;
            validate_private_dir(&self.runtime_base, uid)?;
        } else {
            validate_private_dir(&self.runtime_base, uid)?;
        }

        let runtime_dir = self.runtime_dir();
        create_private_dir(&runtime_dir)?;
        validate_private_dir(&runtime_dir, uid)?;
        Ok(runtime_dir)
    }
}

fn xdg_or_home(variable: &str, home: Option<&Path>, fallback: &str) -> Result<PathBuf, PathError> {
    if let Some(value) = env::var_os(variable)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
    {
        return Ok(value);
    }
    home.map(|home| home.join(fallback))
        .ok_or(PathError::MissingHome)
}

fn current_uid() -> Result<u32, PathError> {
    fs::metadata("/proc/self")
        .map(|metadata| metadata.uid())
        .map_err(PathError::CurrentUid)
}

/// Creates exactly one private runtime directory without following an
/// attacker-controlled symlink through `create_dir_all`.
fn create_private_dir(path: &Path) -> Result<(), PathError> {
    match fs::symlink_metadata(path) {
        Ok(_) => return Ok(()),
        Err(source) if source.kind() == ErrorKind::NotFound => {}
        Err(source) => {
            return Err(PathError::RuntimeIo {
                path: path.to_path_buf(),
                source,
            });
        }
    }

    match fs::create_dir(path) {
        Ok(()) => {}
        Err(source) if source.kind() == ErrorKind::AlreadyExists => {}
        Err(source) => {
            return Err(PathError::RuntimeIo {
                path: path.to_path_buf(),
                source,
            });
        }
    }

    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|source| {
        PathError::RuntimeIo {
            path: path.to_path_buf(),
            source,
        }
    })
}

fn validate_private_dir(path: &Path, uid: u32) -> Result<(), PathError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| PathError::RuntimeIo {
        path: path.to_path_buf(),
        source,
    })?;
    let mode = metadata.mode() & 0o777;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != uid
        || mode & 0o077 != 0
    {
        return Err(PathError::InsecureRuntime {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_path(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!(
            "nexxus-path-test-{}-{nonce}-{label}",
            std::process::id()
        ))
    }

    #[test]
    fn private_directory_requires_user_only_permissions() {
        let path = unique_path("mode");
        fs::create_dir(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        let uid = current_uid().unwrap();
        assert!(matches!(
            validate_private_dir(&path, uid),
            Err(PathError::InsecureRuntime { .. })
        ));
        let _ = fs::remove_dir(path);
    }

    #[test]
    fn private_directory_rejects_symlink_even_to_private_target() {
        let target = unique_path("target");
        let link = unique_path("link");
        fs::create_dir(&target).unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(0o700)).unwrap();
        symlink(&target, &link).unwrap();
        let uid = current_uid().unwrap();
        assert!(matches!(
            validate_private_dir(&link, uid),
            Err(PathError::InsecureRuntime { .. })
        ));
        let _ = fs::remove_file(link);
        let _ = fs::remove_dir(target);
    }
}
