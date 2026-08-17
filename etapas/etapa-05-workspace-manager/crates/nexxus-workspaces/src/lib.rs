//! Nexxus Workspace Manager.
//!
//! This crate owns logical workspaces, deterministic MRU, initial application
//! placement and workspace configuration. It intentionally contains no X11,
//! Wayland or monitor-specific handles, preserving the backend-neutral contract
//! required by the Nexxus architecture.

#![forbid(unsafe_code)]

mod manager;
mod types;

pub use manager::{WORKSPACE_CONFIG_SCHEMA, WorkspaceError, WorkspaceManager};
pub use types::{
    DynamicPolicy, PlacementRule, Workspace, WorkspaceConfig, WorkspaceDefinition, WorkspaceEvent,
    WorkspaceId, WorkspaceIdError, WorkspaceKind,
};
