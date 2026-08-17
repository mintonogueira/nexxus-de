//! Persisted desktop-shell state owned by Stage 13.
//!
//! The settings UI remains a later stage; this module only defines the stable
//! storage contract required to keep wallpaper and desktop launchers across
//! sessions. Atomic writes are delegated to the Stage 01 `nexxus-config` crate.

use std::env;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use nexxus_assets::wallpaper;
use nexxus_config::{ConfigEnvelope, ConfigError, TomlConfigStore};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const DESKTOP_CONFIG_SCHEMA: u32 = 1;
pub const DEFAULT_WALLPAPER: &str = "10-dark-mountain";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum WallpaperSelection {
    Builtin { name: String },
    File { path: PathBuf },
}

impl Default for WallpaperSelection {
    fn default() -> Self {
        Self::Builtin {
            name: DEFAULT_WALLPAPER.to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LauncherPlacement {
    pub desktop_id: String,
    pub x: f32,
    pub y: f32,
}

impl LauncherPlacement {
    pub fn new(desktop_id: impl Into<String>, x: f32, y: f32) -> Result<Self, DesktopConfigError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(DesktopConfigError::InvalidCoordinate);
        }
        Ok(Self {
            desktop_id: desktop_id.into(),
            x,
            y,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DesktopConfig {
    #[serde(default)]
    pub wallpaper: WallpaperSelection,
    #[serde(default)]
    pub launchers: Vec<LauncherPlacement>,
}

#[derive(Debug, Error)]
pub enum DesktopConfigError {
    #[error("desktop configuration path cannot be resolved because HOME is unavailable")]
    HomeUnavailable,
    #[error("wallpaper '{0}' is not present in the Stage 08 asset catalog")]
    UnknownBuiltinWallpaper(String),
    #[error("wallpaper file '{0}' does not exist or is not a regular file")]
    InvalidWallpaperFile(PathBuf),
    #[error("desktop launcher coordinate must be finite")]
    InvalidCoordinate,
    #[error(transparent)]
    Store(#[from] ConfigError),
}

/// Thin wrapper around the common atomic TOML store from Stage 01.
pub struct DesktopConfigStore {
    store: TomlConfigStore,
}

impl DesktopConfigStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            store: TomlConfigStore::new(path, DESKTOP_CONFIG_SCHEMA),
        }
    }

    pub fn from_environment() -> Result<Self, DesktopConfigError> {
        Ok(Self::new(default_config_path()?))
    }

    pub fn path(&self) -> &Path {
        self.store.path()
    }

    /// Missing configuration is the normal first-run state; every other I/O or
    /// parse error remains visible instead of silently discarding user state.
    pub fn load_or_default(&self) -> Result<DesktopConfig, DesktopConfigError> {
        match self.store.load::<DesktopConfig>() {
            Ok(envelope) => Ok(envelope.data),
            Err(ConfigError::Io { source, .. }) if source.kind() == ErrorKind::NotFound => {
                Ok(DesktopConfig::default())
            }
            Err(error) => Err(error.into()),
        }
    }

    pub fn save(&self, config: &DesktopConfig) -> Result<(), DesktopConfigError> {
        validate_wallpaper(&config.wallpaper)?;
        if config
            .launchers
            .iter()
            .any(|launcher| !launcher.x.is_finite() || !launcher.y.is_finite())
        {
            return Err(DesktopConfigError::InvalidCoordinate);
        }
        self.store
            .save(&ConfigEnvelope {
                schema_version: DESKTOP_CONFIG_SCHEMA,
                data: config,
            })
            .map_err(Into::into)
    }
}

pub fn validate_wallpaper(selection: &WallpaperSelection) -> Result<(), DesktopConfigError> {
    match selection {
        WallpaperSelection::Builtin { name } => wallpaper(name)
            .map(|_| ())
            .ok_or_else(|| DesktopConfigError::UnknownBuiltinWallpaper(name.clone())),
        WallpaperSelection::File { path } => {
            if path.is_file() {
                Ok(())
            } else {
                Err(DesktopConfigError::InvalidWallpaperFile(path.clone()))
            }
        }
    }
}

pub fn default_config_path() -> Result<PathBuf, DesktopConfigError> {
    if let Some(path) = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
    {
        return Ok(path.join("nexxus/desktop.toml"));
    }
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or(DesktopConfigError::HomeUnavailable)?;
    Ok(home.join(".config/nexxus/desktop.toml"))
}
