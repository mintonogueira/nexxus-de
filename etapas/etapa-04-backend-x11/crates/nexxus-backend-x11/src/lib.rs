//! Concrete X11 backend for the Nexxus Desktop Environment.
//!
//! The crate translates X11/ICCCM/EWMH state into the backend-neutral window
//! contracts from Etapa 02. It deliberately does not reparent/decorate client
//! windows and does not start an X11 compositor: neither is required to provide
//! the functional window-management contract of Etapa 04.

#![forbid(unsafe_code)]

mod atoms;
mod module;
mod runtime;
mod service;

pub use module::{X11BackendModule, module_descriptor, module_id};
pub use service::{X11Controller, X11Service};

use thiserror::Error;

/// Errors surfaced by the X11 adapter without leaking protocol-native error
/// types across the module boundary.
#[derive(Debug, Error)]
pub enum X11BackendError {
    #[error("X11 backend is unavailable: {0}")]
    Unavailable(String),
    #[error("X11 backend operation failed: {0}")]
    Operation(String),
    #[error("X11 worker is not running")]
    WorkerStopped,
    #[error("X11 worker returned an unexpected response")]
    UnexpectedResponse,
}

pub(crate) fn operation_error(error: impl std::fmt::Display) -> X11BackendError {
    X11BackendError::Operation(error.to_string())
}
