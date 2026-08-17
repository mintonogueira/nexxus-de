//! Deterministic workspace lifecycle, membership, MRU and persistence logic.

use crate::{
    DynamicPolicy, PlacementRule, Workspace, WorkspaceConfig, WorkspaceDefinition, WorkspaceEvent,
    WorkspaceId, WorkspaceKind,
};
use nexxus_config::{ConfigEnvelope, ConfigError, TomlConfigStore};
use nexxus_wm::WindowId;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::path::Path;
use thiserror::Error;

pub const WORKSPACE_CONFIG_SCHEMA: u32 = 1;
const MAX_WORKSPACE_NAME_BYTES: usize = 128;

/// Owns workspace state while keeping graphics backends and monitor topology out
/// of the model. Window geometry remains the WM's responsibility.
pub struct WorkspaceManager {
    workspaces: BTreeMap<WorkspaceId, Workspace>,
    order: Vec<WorkspaceId>,
    active: WorkspaceId,
    mru: VecDeque<WorkspaceId>,
    windows: HashMap<WindowId, WorkspaceId>,
    dynamic_policy: DynamicPolicy,
    placement_rules: Vec<PlacementRule>,
    next_id: Option<u32>,
    events: VecDeque<WorkspaceEvent>,
}

impl WorkspaceManager {
    /// Creates a minimal valid manager without assuming a product-level default
    /// name. Callers choose the visible name explicitly.
    pub fn with_single_fixed(name: impl Into<String>) -> Result<Self, WorkspaceError> {
        let id = WorkspaceId::new(1).map_err(|_| WorkspaceError::IdExhausted)?;
        Self::from_config(WorkspaceConfig {
            active: id,
            dynamic_policy: DynamicPolicy::default(),
            workspaces: vec![WorkspaceDefinition {
                id,
                name: name.into(),
                kind: WorkspaceKind::Fixed,
            }],
            placement_rules: Vec::new(),
        })
    }

    /// Reconstructs persistent workspace definitions. Runtime window membership
    /// is deliberately empty because full session restoration is a later stage.
    pub fn from_config(config: WorkspaceConfig) -> Result<Self, WorkspaceError> {
        validate_config(&config)?;

        let mut workspaces = BTreeMap::new();
        let mut order = Vec::with_capacity(config.workspaces.len());
        let mut maximum_id = 0u32;
        for definition in config.workspaces {
            maximum_id = maximum_id.max(definition.id.get());
            order.push(definition.id);
            workspaces.insert(definition.id, Workspace::from_definition(definition));
        }

        let mut mru = VecDeque::with_capacity(order.len());
        mru.push_back(config.active);
        for id in &order {
            if *id != config.active {
                mru.push_back(*id);
            }
        }

        Ok(Self {
            workspaces,
            order,
            active: config.active,
            mru,
            windows: HashMap::new(),
            dynamic_policy: config.dynamic_policy,
            placement_rules: config.placement_rules,
            next_id: maximum_id.checked_add(1),
            events: VecDeque::new(),
        })
    }

    /// Loads schema-versioned TOML using the atomic configuration primitive from
    /// the foundation stage.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, WorkspaceError> {
        let store = TomlConfigStore::new(path.as_ref(), WORKSPACE_CONFIG_SCHEMA);
        let envelope: ConfigEnvelope<WorkspaceConfig> = store.load()?;
        Self::from_config(envelope.data)
    }

    /// Persists definitions, policy, rules and active workspace. Runtime windows
    /// are excluded so this file does not become an implicit Session State store.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), WorkspaceError> {
        let store = TomlConfigStore::new(path.as_ref(), WORKSPACE_CONFIG_SCHEMA);
        store.save(&ConfigEnvelope {
            schema_version: WORKSPACE_CONFIG_SCHEMA,
            data: self.config_snapshot(),
        })?;
        Ok(())
    }

    pub fn active_id(&self) -> WorkspaceId {
        self.active
    }

    pub fn active(&self) -> &Workspace {
        // `active` is validated on construction and never removed without first
        // selecting a replacement, so this lookup is an internal invariant.
        &self.workspaces[&self.active]
    }

    pub fn workspace(&self, id: WorkspaceId) -> Option<&Workspace> {
        self.workspaces.get(&id)
    }

    pub fn workspaces(&self) -> impl Iterator<Item = &Workspace> {
        self.order.iter().filter_map(|id| self.workspaces.get(id))
    }

    /// Exposes deterministic MRU order for the future Super+Tab controller.
    pub fn mru_order(&self) -> impl Iterator<Item = WorkspaceId> + '_ {
        self.mru.iter().copied()
    }

    pub fn previous_mru(&self) -> Option<WorkspaceId> {
        self.mru.iter().copied().find(|id| *id != self.active)
    }

    /// Returns only windows belonging to the active workspace. Focus/MRU ordering
    /// inside this set remains owned by `nexxus-wm` for future Alt+Tab handling.
    pub fn active_windows(&self) -> impl Iterator<Item = WindowId> + '_ {
        self.active().windows()
    }

    pub fn workspace_of(&self, window: WindowId) -> Option<WorkspaceId> {
        self.windows.get(&window).copied()
    }

    pub fn dynamic_policy(&self) -> DynamicPolicy {
        self.dynamic_policy
    }

    pub fn set_dynamic_policy(&mut self, policy: DynamicPolicy) {
        self.dynamic_policy = policy;
    }

    pub fn placement_rules(&self) -> &[PlacementRule] {
        &self.placement_rules
    }

    /// Replaces initial-placement rules only after every target has been
    /// validated. This avoids partially applying an invalid configuration.
    pub fn set_placement_rules(&mut self, rules: Vec<PlacementRule>) -> Result<(), WorkspaceError> {
        validate_rules(&rules, &self.workspaces)?;
        self.placement_rules = rules;
        Ok(())
    }

    pub fn create_fixed(&mut self, name: impl Into<String>) -> Result<WorkspaceId, WorkspaceError> {
        self.create(name.into(), WorkspaceKind::Fixed)
    }

    pub fn create_dynamic(
        &mut self,
        name: impl Into<String>,
    ) -> Result<WorkspaceId, WorkspaceError> {
        self.create(name.into(), WorkspaceKind::Dynamic)
    }

    fn create(&mut self, name: String, kind: WorkspaceKind) -> Result<WorkspaceId, WorkspaceError> {
        let name = normalize_name(name)?;
        if self.workspaces.values().any(|workspace| workspace.name == name) {
            return Err(WorkspaceError::DuplicateName(name));
        }

        let raw = self.next_id.ok_or(WorkspaceError::IdExhausted)?;
        let id = WorkspaceId::new(raw).map_err(|_| WorkspaceError::IdExhausted)?;
        self.next_id = raw.checked_add(1);
        self.order.push(id);
        self.workspaces.insert(
            id,
            Workspace::from_definition(WorkspaceDefinition {
                id,
                name: name.clone(),
                kind,
            }),
        );
        self.mru.push_back(id);
        self.events.push_back(WorkspaceEvent::Created {
            workspace: id,
            name,
            kind,
        });
        Ok(id)
    }

    /// Removes a workspace without losing windows. Any resident windows are
    /// reassigned to a deterministic surviving workspace before removal.
    pub fn remove(&mut self, id: WorkspaceId) -> Result<(), WorkspaceError> {
        if !self.workspaces.contains_key(&id) {
            return Err(WorkspaceError::WorkspaceNotFound(id));
        }
        if self.workspaces.len() == 1 {
            return Err(WorkspaceError::LastWorkspace);
        }

        let destination = self.replacement_for(id)?;
        let resident_windows: Vec<WindowId> = self
            .workspaces
            .get(&id)
            .ok_or(WorkspaceError::WorkspaceNotFound(id))?
            .windows()
            .collect();

        for window in resident_windows {
            if let Some(target) = self.workspaces.get_mut(&destination) {
                target.insert_window(window);
            }
            self.windows.insert(window, destination);
            self.events.push_back(WorkspaceEvent::WindowMoved {
                window,
                from: Some(id),
                to: destination,
            });
        }

        self.workspaces.remove(&id);
        self.order.retain(|candidate| *candidate != id);
        self.mru.retain(|candidate| *candidate != id);
        self.placement_rules.retain(|rule| rule.workspace != id);
        self.events
            .push_back(WorkspaceEvent::Removed { workspace: id });

        if self.active == id {
            let previous = self.active;
            self.active = destination;
            self.touch_mru(destination);
            self.events.push_back(WorkspaceEvent::Activated {
                previous,
                current: destination,
            });
        }
        Ok(())
    }

    pub fn rename(
        &mut self,
        id: WorkspaceId,
        new_name: impl Into<String>,
    ) -> Result<(), WorkspaceError> {
        let new_name = normalize_name(new_name.into())?;
        if self
            .workspaces
            .values()
            .any(|workspace| workspace.id != id && workspace.name == new_name)
        {
            return Err(WorkspaceError::DuplicateName(new_name));
        }
        let workspace = self
            .workspaces
            .get_mut(&id)
            .ok_or(WorkspaceError::WorkspaceNotFound(id))?;
        if workspace.name == new_name {
            return Ok(());
        }
        let old_name = std::mem::replace(&mut workspace.name, new_name.clone());
        self.events.push_back(WorkspaceEvent::Renamed {
            workspace: id,
            old_name,
            new_name,
        });
        Ok(())
    }

    pub fn activate(&mut self, id: WorkspaceId) -> Result<(), WorkspaceError> {
        if !self.workspaces.contains_key(&id) {
            return Err(WorkspaceError::WorkspaceNotFound(id));
        }
        if self.active == id {
            self.touch_mru(id);
            return Ok(());
        }
        let previous = self.active;
        self.active = id;
        self.touch_mru(id);
        self.events.push_back(WorkspaceEvent::Activated {
            previous,
            current: id,
        });
        Ok(())
    }

    /// Assigns a newly managed window exactly once. Rules are consulted only at
    /// this boundary; later manual moves never reapply or lock the window.
    pub fn assign_new_window(
        &mut self,
        window: WindowId,
        application_id: Option<&str>,
    ) -> Result<WorkspaceId, WorkspaceError> {
        if self.windows.contains_key(&window) {
            return Err(WorkspaceError::WindowAlreadyTracked(window));
        }
        let target = application_id
            .and_then(|application_id| {
                self.placement_rules
                    .iter()
                    .find(|rule| rule.application_id == application_id)
                    .map(|rule| rule.workspace)
            })
            .unwrap_or(self.active);

        let workspace = self
            .workspaces
            .get_mut(&target)
            .ok_or(WorkspaceError::WorkspaceNotFound(target))?;
        workspace.insert_window(window);
        self.windows.insert(window, target);
        self.events.push_back(WorkspaceEvent::WindowMoved {
            window,
            from: None,
            to: target,
        });
        Ok(target)
    }

    /// Moves a window unconditionally once both IDs are valid. This is the key
    /// guarantee that placement rules never become workspace imprisonment.
    pub fn move_window(
        &mut self,
        window: WindowId,
        target: WorkspaceId,
    ) -> Result<(), WorkspaceError> {
        if !self.workspaces.contains_key(&target) {
            return Err(WorkspaceError::WorkspaceNotFound(target));
        }
        let source = self
            .windows
            .get(&window)
            .copied()
            .ok_or(WorkspaceError::WindowNotTracked(window))?;
        if source == target {
            return Ok(());
        }

        if let Some(workspace) = self.workspaces.get_mut(&source) {
            workspace.remove_window(window);
        }
        if let Some(workspace) = self.workspaces.get_mut(&target) {
            workspace.insert_window(window);
        }
        self.windows.insert(window, target);
        self.events.push_back(WorkspaceEvent::WindowMoved {
            window,
            from: Some(source),
            to: target,
        });
        self.prune_dynamic_if_needed(source)?;
        Ok(())
    }

    /// Forgets a destroyed/unmanaged window and optionally prunes its now-empty
    /// dynamic workspace according to policy.
    pub fn forget_window(&mut self, window: WindowId) -> Result<(), WorkspaceError> {
        let source = self
            .windows
            .remove(&window)
            .ok_or(WorkspaceError::WindowNotTracked(window))?;
        if let Some(workspace) = self.workspaces.get_mut(&source) {
            workspace.remove_window(window);
        }
        self.events
            .push_back(WorkspaceEvent::WindowForgotten { window, from: source });
        self.prune_dynamic_if_needed(source)?;
        Ok(())
    }

    /// Drains events in generation order so downstream adapters cannot observe a
    /// later activation before the removal/move that caused it.
    pub fn drain_events(&mut self) -> impl Iterator<Item = WorkspaceEvent> + '_ {
        self.events.drain(..)
    }

    pub fn config_snapshot(&self) -> WorkspaceConfig {
        WorkspaceConfig {
            active: self.active,
            dynamic_policy: self.dynamic_policy,
            workspaces: self
                .order
                .iter()
                .filter_map(|id| self.workspaces.get(id))
                .map(Workspace::definition)
                .collect(),
            placement_rules: self.placement_rules.clone(),
        }
    }

    fn touch_mru(&mut self, id: WorkspaceId) {
        self.mru.retain(|candidate| *candidate != id);
        self.mru.push_front(id);
    }

    fn replacement_for(&self, removed: WorkspaceId) -> Result<WorkspaceId, WorkspaceError> {
        if self.active != removed {
            return Ok(self.active);
        }
        self.mru
            .iter()
            .copied()
            .find(|candidate| *candidate != removed && self.workspaces.contains_key(candidate))
            .or_else(|| self.order.iter().copied().find(|candidate| *candidate != removed))
            .ok_or(WorkspaceError::LastWorkspace)
    }

    fn prune_dynamic_if_needed(&mut self, id: WorkspaceId) -> Result<(), WorkspaceError> {
        if self.dynamic_policy != DynamicPolicy::RemoveEmptyInactive || self.active == id {
            return Ok(());
        }
        let should_remove = self
            .workspaces
            .get(&id)
            .is_some_and(|workspace| workspace.kind == WorkspaceKind::Dynamic && workspace.is_empty());
        if should_remove && self.workspaces.len() > 1 {
            self.remove(id)?;
        }
        Ok(())
    }
}

fn normalize_name(name: String) -> Result<String, WorkspaceError> {
    let normalized = name.trim().to_owned();
    if normalized.is_empty() {
        return Err(WorkspaceError::InvalidName);
    }
    if normalized.len() > MAX_WORKSPACE_NAME_BYTES {
        return Err(WorkspaceError::NameTooLong(MAX_WORKSPACE_NAME_BYTES));
    }
    Ok(normalized)
}

fn validate_config(config: &WorkspaceConfig) -> Result<(), WorkspaceError> {
    if config.workspaces.is_empty() {
        return Err(WorkspaceError::NoWorkspaces);
    }

    let mut ids = HashSet::new();
    let mut names = HashSet::new();
    for workspace in &config.workspaces {
        let name = normalize_name(workspace.name.clone())?;
        if workspace.name != name {
            return Err(WorkspaceError::InvalidName);
        }
        if !ids.insert(workspace.id) {
            return Err(WorkspaceError::DuplicateId(workspace.id));
        }
        if !names.insert(workspace.name.clone()) {
            return Err(WorkspaceError::DuplicateName(workspace.name.clone()));
        }
    }
    if !ids.contains(&config.active) {
        return Err(WorkspaceError::ActiveWorkspaceMissing(config.active));
    }

    let map: BTreeMap<WorkspaceId, Workspace> = config
        .workspaces
        .iter()
        .cloned()
        .map(|definition| (definition.id, Workspace::from_definition(definition)))
        .collect();
    validate_rules(&config.placement_rules, &map)
}

fn validate_rules(
    rules: &[PlacementRule],
    workspaces: &BTreeMap<WorkspaceId, Workspace>,
) -> Result<(), WorkspaceError> {
    let mut application_ids = HashSet::new();
    for rule in rules {
        if rule.application_id.trim().is_empty() {
            return Err(WorkspaceError::InvalidApplicationId);
        }
        if !application_ids.insert(rule.application_id.clone()) {
            return Err(WorkspaceError::DuplicateApplicationRule(
                rule.application_id.clone(),
            ));
        }
        if !workspaces.contains_key(&rule.workspace) {
            return Err(WorkspaceError::RuleTargetMissing(rule.workspace));
        }
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error("workspace configuration must contain at least one workspace")]
    NoWorkspaces,
    #[error("workspace {0:?} does not exist")]
    WorkspaceNotFound(WorkspaceId),
    #[error("active workspace {0:?} is not present in the configuration")]
    ActiveWorkspaceMissing(WorkspaceId),
    #[error("workspace id {0:?} is duplicated")]
    DuplicateId(WorkspaceId),
    #[error("workspace name '{0}' is duplicated")]
    DuplicateName(String),
    #[error("workspace name must be non-empty and cannot contain leading/trailing whitespace")]
    InvalidName,
    #[error("workspace name exceeds {0} bytes")]
    NameTooLong(usize),
    #[error("workspace identifier space is exhausted")]
    IdExhausted,
    #[error("the final workspace cannot be removed")]
    LastWorkspace,
    #[error("window {0:?} is already assigned to a workspace")]
    WindowAlreadyTracked(WindowId),
    #[error("window {0:?} is not assigned to any workspace")]
    WindowNotTracked(WindowId),
    #[error("application placement rule requires a non-empty application id")]
    InvalidApplicationId,
    #[error("placement rule for application '{0}' is duplicated")]
    DuplicateApplicationRule(String),
    #[error("placement rule targets missing workspace {0:?}")]
    RuleTargetMissing(WorkspaceId),
}
