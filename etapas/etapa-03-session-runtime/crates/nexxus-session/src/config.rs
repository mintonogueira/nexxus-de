//! Versioned configuration owned by the Session Runtime.
//!
//! The Session Runtime intentionally requires an explicit backend choice. An
//! absent choice is an error rather than an implicit X11/Wayland preference.

use nexxus_backend_api::BackendKind;
use nexxus_config::{ConfigEnvelope, ConfigError, TomlConfigStore};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const SESSION_CONFIG_SCHEMA: u32 = 1;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Backend requested for this session. No fallback is inferred when absent.
    pub backend: Option<BackendKind>,
}

#[derive(Debug, Error)]
pub enum SessionConfigError {
    #[error("session backend must be selected explicitly by --backend or configuration")]
    BackendRequired,
    #[error("invalid backend '{0}'; expected 'x11' or 'wayland'")]
    InvalidBackend(String),
    #[error(transparent)]
    Store(#[from] ConfigError),
}

impl SessionConfig {
    /// Loads the configured backend when the file exists; a missing file is
    /// equivalent to an empty configuration and never invents a backend.
    pub fn load_optional(path: &Path) -> Result<Self, SessionConfigError> {
        let store = TomlConfigStore::new(path, SESSION_CONFIG_SCHEMA);
        match store.load::<SessionConfig>() {
            Ok(envelope) => Ok(envelope.data),
            Err(ConfigError::Io { source, .. }) if source.kind() == ErrorKind::NotFound => {
                Ok(Self::default())
            }
            Err(error) => Err(error.into()),
        }
    }

    /// Writes the default schema through the atomic configuration primitive
    /// provided by Etapa 01.
    pub fn save(&self, path: &Path) -> Result<(), SessionConfigError> {
        let store = TomlConfigStore::new(path, SESSION_CONFIG_SCHEMA);
        store.save(&ConfigEnvelope {
            schema_version: SESSION_CONFIG_SCHEMA,
            data: self.clone(),
        })?;
        Ok(())
    }

    pub fn resolve_backend(
        &self,
        cli_backend: Option<BackendKind>,
    ) -> Result<BackendKind, SessionConfigError> {
        cli_backend.or(self.backend).ok_or(SessionConfigError::BackendRequired)
    }
}

pub fn parse_backend(value: &str) -> Result<BackendKind, SessionConfigError> {
    match value {
        "x11" => Ok(BackendKind::X11),
        "wayland" => Ok(BackendKind::Wayland),
        other => Err(SessionConfigError::InvalidBackend(other.to_owned())),
    }
}

pub fn default_config_path(config_dir: &Path) -> PathBuf {
    config_dir.join("session.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_backend_has_precedence_without_silent_fallback() {
        let config = SessionConfig {
            backend: Some(BackendKind::Wayland),
        };
        assert_eq!(
            config.resolve_backend(Some(BackendKind::X11)).unwrap(),
            BackendKind::X11
        );
    }

    #[test]
    fn missing_backend_is_explicit_error() {
        assert!(matches!(
            SessionConfig::default().resolve_backend(None),
            Err(SessionConfigError::BackendRequired)
        ));
    }
}
