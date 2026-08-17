//! XDG application discovery, parsing, indexing and live refresh for Nexxus.
//!
//! This crate owns the desktop-entry catalog contract consumed by later Menu,
//! Desktop Shell and Application Finder stages. It deliberately contains no UI.
#![forbid(unsafe_code)]

mod category;
mod config;
mod exec;
mod icon;
mod model;
mod scanner;
mod service;

pub use category::MainCategory;
pub use config::{ApplicationIndexConfig, ApplicationRoot, ApplicationSource, ConfigError};
pub use exec::{ExecArgument, ExecError, ExecTemplate, LaunchCommand, LaunchContext};
pub use icon::{IconReference, resolve_icon_reference};
pub use model::{
    ApplicationRecord, DesktopId, IndexDelta, IndexDiagnostic, IndexDiagnosticKind, IndexSnapshot,
};
pub use scanner::{ScanError, scan};
pub use service::{ApplicationIndexEvent, ApplicationIndexService, ServiceError};
