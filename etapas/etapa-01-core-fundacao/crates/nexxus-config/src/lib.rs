//! Configuration persistence primitives for Nexxus.
//!
//! This crate owns file-format, schema and atomic-write mechanics only. It does
//! not define settings belonging to later Nexxus modules.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

/// Defensive upper bound for one persisted configuration document.
///
/// Configuration is control-plane data, not bulk storage. Bounding it prevents
/// corrupted or hostile files from causing unbounded allocation during startup.
pub const MAX_CONFIG_FILE_SIZE: u64 = 4 * 1024 * 1024;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConfigEnvelope<T> {
    pub schema_version: u32,
    pub data: T,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("configuration I/O error at '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid TOML configuration at '{path}': {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("could not serialize configuration: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("unsupported configuration schema {found}; maximum supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("configuration at '{path}' exceeds the {max} byte limit ({actual} bytes)")]
    Oversized {
        path: PathBuf,
        actual: u64,
        max: u64,
    },
}

pub struct TomlConfigStore {
    path: PathBuf,
    supported_schema: u32,
}

impl TomlConfigStore {
    pub fn new(path: impl Into<PathBuf>, supported_schema: u32) -> Self {
        Self {
            path: path.into(),
            supported_schema,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Loads a complete versioned TOML document with a strict size bound.
    pub fn load<T: DeserializeOwned>(&self) -> Result<ConfigEnvelope<T>, ConfigError> {
        let file = File::open(&self.path).map_err(|source| ConfigError::Io {
            path: self.path.clone(),
            source,
        })?;
        let metadata = file.metadata().map_err(|source| ConfigError::Io {
            path: self.path.clone(),
            source,
        })?;
        if metadata.len() > MAX_CONFIG_FILE_SIZE {
            return Err(ConfigError::Oversized {
                path: self.path.clone(),
                actual: metadata.len(),
                max: MAX_CONFIG_FILE_SIZE,
            });
        }

        // The extra byte closes the race where the file grows after metadata
        // inspection but before or during the read.
        let mut limited = file.take(MAX_CONFIG_FILE_SIZE + 1);
        let mut text = String::new();
        limited
            .read_to_string(&mut text)
            .map_err(|source| ConfigError::Io {
                path: self.path.clone(),
                source,
            })?;
        if text.len() as u64 > MAX_CONFIG_FILE_SIZE {
            return Err(ConfigError::Oversized {
                path: self.path.clone(),
                actual: text.len() as u64,
                max: MAX_CONFIG_FILE_SIZE,
            });
        }

        let envelope: ConfigEnvelope<T> =
            toml::from_str(&text).map_err(|source| ConfigError::Parse {
                path: self.path.clone(),
                source,
            })?;
        self.validate_schema(envelope.schema_version)?;
        Ok(envelope)
    }

    /// Writes a complete configuration transactionally in the destination
    /// directory, avoiding a partially written final file after a crash.
    pub fn save<T: Serialize>(&self, envelope: &ConfigEnvelope<T>) -> Result<(), ConfigError> {
        self.validate_schema(envelope.schema_version)?;
        let parent = self
            .path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700)).map_err(|source| {
                ConfigError::Io {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }

        let serialized = toml::to_string_pretty(envelope)?;
        if serialized.len() as u64 > MAX_CONFIG_FILE_SIZE {
            return Err(ConfigError::Oversized {
                path: self.path.clone(),
                actual: serialized.len() as u64,
                max: MAX_CONFIG_FILE_SIZE,
            });
        }

        let temp = self.temp_path(parent);
        let result = self.write_and_commit(&temp, serialized.as_bytes());
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        result
    }

    fn validate_schema(&self, found: u32) -> Result<(), ConfigError> {
        if found > self.supported_schema {
            return Err(ConfigError::UnsupportedSchema {
                found,
                supported: self.supported_schema,
            });
        }
        Ok(())
    }

    fn temp_path(&self, parent: &Path) -> PathBuf {
        let name = self
            .path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("config.toml");
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        parent.join(format!(".{name}.{}.{}.tmp", std::process::id(), counter))
    }

    fn write_and_commit(&self, temp: &Path, bytes: &[u8]) -> Result<(), ConfigError> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(temp)
            .map_err(|source| ConfigError::Io {
                path: temp.to_path_buf(),
                source,
            })?;
        file.write_all(bytes).map_err(|source| ConfigError::Io {
            path: temp.to_path_buf(),
            source,
        })?;
        file.sync_all().map_err(|source| ConfigError::Io {
            path: temp.to_path_buf(),
            source,
        })?;
        drop(file);

        // rename(2) is atomic within the same filesystem; the temporary file is
        // deliberately created in the destination directory for this reason.
        fs::rename(temp, &self.path).map_err(|source| ConfigError::Io {
            path: self.path.clone(),
            source,
        })?;

        if let Some(parent) = self.path.parent() {
            File::open(parent)
                .and_then(|dir| dir.sync_all())
                .map_err(|source| ConfigError::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct Demo {
        enabled: bool,
        label: String,
    }

    fn temp_path(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "nexxus-config-test-{}-{nonce}-{name}",
            std::process::id()
        ))
    }

    #[test]
    fn saves_and_loads_versioned_toml() {
        let dir = temp_path("roundtrip");
        let path = dir.join("settings.toml");
        let store = TomlConfigStore::new(&path, 1);
        let expected = ConfigEnvelope {
            schema_version: 1,
            data: Demo {
                enabled: true,
                label: "nexxus".into(),
            },
        };
        store.save(&expected).unwrap();
        let actual: ConfigEnvelope<Demo> = store.load().unwrap();
        assert_eq!(actual, expected);
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_future_schema() {
        let dir = temp_path("future");
        let path = dir.join("settings.toml");
        let store = TomlConfigStore::new(&path, 1);
        let future = ConfigEnvelope {
            schema_version: 2,
            data: Demo {
                enabled: false,
                label: String::new(),
            },
        };
        assert!(matches!(
            store.save(&future),
            Err(ConfigError::UnsupportedSchema { .. })
        ));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_oversized_file_before_deserialization() {
        let dir = temp_path("oversized");
        fs::create_dir(&dir).unwrap();
        let path = dir.join("settings.toml");
        let file = File::create(&path).unwrap();
        file.set_len(MAX_CONFIG_FILE_SIZE + 1).unwrap();
        let store = TomlConfigStore::new(&path, 1);
        assert!(matches!(
            store.load::<Demo>(),
            Err(ConfigError::Oversized { .. })
        ));
        let _ = fs::remove_dir_all(dir);
    }
}
