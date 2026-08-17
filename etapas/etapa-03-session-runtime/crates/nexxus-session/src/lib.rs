//! Nexxus Session Runtime.
//!
//! This crate coordinates session startup and shutdown over contracts produced
//! by Etapas 01 and 02. It intentionally contains no X11, Wayland, compositor,
//! workspace, tiling, greeter or persistent Session State implementation.

#![forbid(unsafe_code)]

mod config;
mod runtime;

pub use config::{
    SessionConfig, SessionConfigError, SESSION_CONFIG_SCHEMA, default_config_path, parse_backend,
};
pub use runtime::{
    BackendModule, SessionEnvironment, SessionRuntime, SessionRuntimeError, query_status,
    request_shutdown,
};

use nexxus_backend_api::BackendKind;
use nexxus_core::{ModuleState, RegistryError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Converts registry validation failures into the lifecycle error channel used
/// by the Session Runtime. The Core already defines RegistryError as a
/// LifecycleError source, so this preserves the foundation contract instead of
/// creating a parallel error model in Etapa 03.
impl From<RegistryError> for SessionRuntimeError {
    fn from(error: RegistryError) -> Self {
        Self::Lifecycle(error.into())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "kebab-case")]
pub enum SessionControlRequest {
    Status,
    Shutdown { reason: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "result", content = "data", rename_all = "kebab-case")]
pub enum SessionControlResponse {
    Accepted,
    Status(SessionStatus),
    Error(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionStatus {
    pub backend: BackendKind,
    pub control_socket: PathBuf,
    pub modules: Vec<(String, Option<ModuleState>)>,
}
