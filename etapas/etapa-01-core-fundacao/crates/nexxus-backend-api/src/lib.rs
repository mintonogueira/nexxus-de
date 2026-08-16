//! Backend-neutral graphics contracts for Nexxus.
//!
//! No X11 or Wayland implementation belongs in this crate. Concrete backends
//! will be developed in their own stages and must implement these common
//! contracts rather than leaking protocol-native handles to higher modules.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackendKind {
    X11,
    Wayland,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BackendCapabilities {
    pub multi_monitor: bool,
    pub hotplug: bool,
    pub xwayland_compatibility: bool,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OutputId(String);

impl OutputId {
    /// Creates a backend-neutral output identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, &'static str> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err("output id cannot be empty");
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OutputInfo {
    pub id: OutputId,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub scale_milli: u32,
    pub primary: bool,
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("graphics backend is unavailable: {0}")]
    Unavailable(String),
    #[error("graphics backend operation failed: {0}")]
    Operation(String),
}

/// Minimal contract exposed by concrete graphics backends to the Core.
///
/// Window-management, tiling and workspace behavior are intentionally not
/// defined here yet; those contracts belong to their dedicated stages.
pub trait GraphicsBackend: Send {
    fn kind(&self) -> BackendKind;
    fn capabilities(&self) -> BackendCapabilities;
    fn start(&mut self) -> Result<(), BackendError>;
    fn stop(&mut self) -> Result<(), BackendError>;
    fn outputs(&self) -> Result<Vec<OutputInfo>, BackendError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyBackend;

    impl GraphicsBackend for DummyBackend {
        fn kind(&self) -> BackendKind {
            BackendKind::X11
        }

        fn capabilities(&self) -> BackendCapabilities {
            BackendCapabilities {
                multi_monitor: true,
                hotplug: true,
                xwayland_compatibility: false,
            }
        }

        fn start(&mut self) -> Result<(), BackendError> {
            Ok(())
        }

        fn stop(&mut self) -> Result<(), BackendError> {
            Ok(())
        }

        fn outputs(&self) -> Result<Vec<OutputInfo>, BackendError> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn a_backend_can_satisfy_the_neutral_contract_without_native_handles() {
        let mut backend = DummyBackend;
        backend.start().unwrap();
        assert_eq!(backend.kind(), BackendKind::X11);
        assert!(backend.outputs().unwrap().is_empty());
        backend.stop().unwrap();
    }
}
