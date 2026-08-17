//! Backend-neutral workspace identifiers, configuration and observable events.

use nexxus_wm::WindowId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

/// Stable identifier persisted across renames and configuration reloads.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct WorkspaceId(u32);

impl WorkspaceId {
    /// Creates a workspace identifier; zero is reserved as an invalid sentinel.
    pub fn new(value: u32) -> Result<Self, WorkspaceIdError> {
        if value == 0 {
            return Err(WorkspaceIdError);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
#[error("workspace id cannot be zero")]
pub struct WorkspaceIdError;

/// Persistent workspaces remain until explicitly removed. Dynamic workspaces may
/// be removed by the configured lifecycle policy when they become unused.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceKind {
    Fixed,
    Dynamic,
}

/// Controls only automatic cleanup of dynamic workspaces. Creation remains an
/// explicit manager operation so later UI/settings stages can choose the policy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DynamicPolicy {
    KeepEmpty,
    RemoveEmptyInactive,
}

impl Default for DynamicPolicy {
    fn default() -> Self {
        Self::RemoveEmptyInactive
    }
}

/// Persistent definition. Window membership is intentionally absent because
/// full process/session restoration belongs to the later Session State stage.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkspaceDefinition {
    pub id: WorkspaceId,
    pub name: String,
    pub kind: WorkspaceKind,
}

/// Exact application-id rule used only when a window is first assigned.
/// Manual moves always override this initial placement and are never blocked.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlacementRule {
    pub application_id: String,
    pub workspace: WorkspaceId,
}

/// Schema payload persisted through `nexxus-config`'s atomic TOML store.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkspaceConfig {
    pub active: WorkspaceId,
    #[serde(default)]
    pub dynamic_policy: DynamicPolicy,
    pub workspaces: Vec<WorkspaceDefinition>,
    #[serde(default)]
    pub placement_rules: Vec<PlacementRule>,
}

/// Runtime workspace state. No monitor identifier is stored: one workspace is a
/// logical context whose windows may occupy any monitor coordinates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub kind: WorkspaceKind,
    windows: BTreeSet<WindowId>,
}

impl Workspace {
    pub(crate) fn from_definition(definition: WorkspaceDefinition) -> Self {
        Self {
            id: definition.id,
            name: definition.name,
            kind: definition.kind,
            windows: BTreeSet::new(),
        }
    }

    pub fn windows(&self) -> impl Iterator<Item = WindowId> + '_ {
        self.windows.iter().copied()
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    pub(crate) fn insert_window(&mut self, window: WindowId) {
        self.windows.insert(window);
    }

    pub(crate) fn remove_window(&mut self, window: WindowId) {
        self.windows.remove(&window);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    pub(crate) fn definition(&self) -> WorkspaceDefinition {
        WorkspaceDefinition {
            id: self.id,
            name: self.name.clone(),
            kind: self.kind,
        }
    }
}

/// Events consumed later by workspace bar, shortcuts, backend adapters and
/// session integration without exposing backend-specific handles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkspaceEvent {
    Created {
        workspace: WorkspaceId,
        name: String,
        kind: WorkspaceKind,
    },
    Removed {
        workspace: WorkspaceId,
    },
    Renamed {
        workspace: WorkspaceId,
        old_name: String,
        new_name: String,
    },
    Activated {
        previous: WorkspaceId,
        current: WorkspaceId,
    },
    WindowMoved {
        window: WindowId,
        from: Option<WorkspaceId>,
        to: WorkspaceId,
    },
    WindowForgotten {
        window: WindowId,
        from: WorkspaceId,
    },
}
