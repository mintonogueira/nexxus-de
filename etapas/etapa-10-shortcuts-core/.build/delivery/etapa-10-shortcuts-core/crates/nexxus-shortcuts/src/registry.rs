//! Command/binding registry, conflict validation, persistence and dispatch.

use crate::command::{CommandDescriptor, CommandIdError};
use crate::model::TriggerParseError;
use crate::{CommandId, Trigger};
use nexxus_config::{ConfigEnvelope, ConfigError, TomlConfigStore};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error as StdError;
use std::path::Path;
use thiserror::Error;

pub const SHORTCUT_CONFIG_SCHEMA: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PersistedBinding {
    pub trigger: String,
    pub command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub bindings: Vec<PersistedBinding>,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("shortcut '{trigger}' is already bound to '{existing}', cannot bind it to '{requested}'")]
pub struct BindingConflict {
    pub trigger: Trigger,
    pub existing: CommandId,
    pub requested: CommandId,
}

#[derive(Debug, Error)]
pub enum ShortcutError {
    #[error(transparent)]
    Trigger(#[from] TriggerParseError),
    #[error(transparent)]
    CommandId(#[from] CommandIdError),
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Conflict(#[from] BindingConflict),
    #[error("shortcut command '{0}' is not registered")]
    UnknownCommand(CommandId),
    #[error("shortcut command '{0}' is already registered")]
    DuplicateCommand(CommandId),
    #[error("bare F11 is reserved for applications and cannot be a global Nexxus shortcut")]
    BareF11Reserved,
    #[error("shortcut '{0}' is not currently bound")]
    BindingNotFound(Trigger),
}

/// Consumers implement this boundary to translate a logical shortcut command
/// into their existing module APIs. Shortcuts Core never embeds those modules'
/// runtime state or protocol-native handles.
pub trait ShortcutDispatchSink {
    type Error: StdError + Send + Sync + 'static;

    fn dispatch(&mut self, command: &CommandDescriptor) -> Result<(), Self::Error>;
}

#[derive(Debug, Error)]
pub enum DispatchError<E>
where
    E: StdError + Send + Sync + 'static,
{
    #[error("shortcut registry invariant failed for command '{0}'")]
    RegistryInvariant(CommandId),
    #[error("shortcut consumer rejected dispatch: {0}")]
    Sink(#[source] E),
}

pub struct ShortcutRegistry {
    commands: BTreeMap<CommandId, CommandDescriptor>,
    bindings: BTreeMap<Trigger, CommandId>,
}

impl Default for ShortcutRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl ShortcutRegistry {
    pub fn with_defaults() -> Self {
        let mut registry = Self::commands_only();
        for (trigger, command) in default_bindings() {
            registry
                .bind(trigger, command)
                .expect("static Nexxus default shortcut set is conflict-free");
        }
        registry
    }

    fn commands_only() -> Self {
        let commands = CommandDescriptor::builtins()
            .into_iter()
            .map(|descriptor| (descriptor.id().clone(), descriptor))
            .collect();
        Self {
            commands,
            bindings: BTreeMap::new(),
        }
    }

    pub fn commands(&self) -> impl ExactSizeIterator<Item = &CommandDescriptor> {
        self.commands.values()
    }

    pub fn bindings(&self) -> impl ExactSizeIterator<Item = (&Trigger, &CommandId)> {
        self.bindings.iter()
    }

    pub fn command(&self, id: &CommandId) -> Option<&CommandDescriptor> {
        self.commands.get(id)
    }

    pub fn binding_for(&self, trigger: &Trigger) -> Option<&CommandId> {
        self.bindings.get(trigger)
    }

    /// Adds an extension command without changing any binding. Later modules can
    /// register their logical action here without changing the Shortcuts Core.
    pub fn register_command(&mut self, descriptor: CommandDescriptor) -> Result<(), ShortcutError> {
        let id = descriptor.id().clone();
        if self.commands.contains_key(&id) {
            return Err(ShortcutError::DuplicateCommand(id));
        }
        self.commands.insert(id, descriptor);
        Ok(())
    }

    pub fn bind(&mut self, trigger: Trigger, command: CommandId) -> Result<(), ShortcutError> {
        validate_trigger(&trigger)?;
        self.require_command(&command)?;

        if let Some(existing) = self.bindings.get(&trigger) {
            if existing == &command {
                return Ok(());
            }
            return Err(BindingConflict {
                trigger,
                existing: existing.clone(),
                requested: command,
            }
            .into());
        }
        self.bindings.insert(trigger, command);
        Ok(())
    }

    pub fn unbind(&mut self, trigger: &Trigger) -> Option<CommandId> {
        self.bindings.remove(trigger)
    }

    /// Replaces one binding transactionally: the old binding is left untouched
    /// when the replacement is invalid or conflicts with another command.
    pub fn rebind(&mut self, old: &Trigger, new: Trigger) -> Result<(), ShortcutError> {
        let command = self
            .bindings
            .get(old)
            .cloned()
            .ok_or_else(|| ShortcutError::BindingNotFound(old.clone()))?;
        validate_trigger(&new)?;

        if &new == old {
            return Ok(());
        }
        if let Some(existing) = self.bindings.get(&new) {
            return Err(BindingConflict {
                trigger: new,
                existing: existing.clone(),
                requested: command,
            }
            .into());
        }

        self.bindings.remove(old);
        self.bindings.insert(new, command);
        Ok(())
    }

    pub fn restore_default_bindings(&mut self) {
        let defaults = Self::with_defaults();
        self.bindings = defaults.bindings;
    }

    pub fn config_snapshot(&self) -> ShortcutConfig {
        ShortcutConfig {
            bindings: self
                .bindings
                .iter()
                .map(|(trigger, command)| PersistedBinding {
                    trigger: trigger.to_string(),
                    command: command.to_string(),
                })
                .collect(),
        }
    }

    /// Loads and fully validates a document before exposing any binding. Invalid
    /// conflicts, unknown commands and bare F11 all fail atomically.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ShortcutError> {
        let store = TomlConfigStore::new(path.as_ref(), SHORTCUT_CONFIG_SCHEMA);
        let envelope: ConfigEnvelope<ShortcutConfig> = store.load()?;
        Self::from_config(envelope.data)
    }

    pub fn from_config(config: ShortcutConfig) -> Result<Self, ShortcutError> {
        let mut registry = Self::commands_only();
        for persisted in config.bindings {
            registry.bind(
                Trigger::parse(&persisted.trigger)?,
                CommandId::new(persisted.command)?,
            )?;
        }
        Ok(registry)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ShortcutError> {
        let store = TomlConfigStore::new(path.as_ref(), SHORTCUT_CONFIG_SCHEMA);
        store.save(&ConfigEnvelope {
            schema_version: SHORTCUT_CONFIG_SCHEMA,
            data: self.config_snapshot(),
        })?;
        Ok(())
    }

    /// Resolves a trigger and forwards its stable descriptor to the host. An
    /// unbound trigger is not an error and returns `Ok(false)`.
    pub fn dispatch_trigger<S>(
        &self,
        trigger: &Trigger,
        sink: &mut S,
    ) -> Result<bool, DispatchError<S::Error>>
    where
        S: ShortcutDispatchSink,
    {
        let Some(command_id) = self.bindings.get(trigger) else {
            return Ok(false);
        };
        let command = self
            .commands
            .get(command_id)
            .ok_or_else(|| DispatchError::RegistryInvariant(command_id.clone()))?;
        sink.dispatch(command).map_err(DispatchError::Sink)?;
        Ok(true)
    }

    fn require_command(&self, command: &CommandId) -> Result<(), ShortcutError> {
        if self.commands.contains_key(command) {
            Ok(())
        } else {
            Err(ShortcutError::UnknownCommand(command.clone()))
        }
    }
}

fn validate_trigger(trigger: &Trigger) -> Result<(), ShortcutError> {
    if trigger.is_bare_f11() {
        Err(ShortcutError::BareF11Reserved)
    } else {
        Ok(())
    }
}

fn command(value: &str) -> CommandId {
    CommandId::new(value).expect("static shortcut command identifier is valid")
}

fn default_bindings() -> Vec<(Trigger, CommandId)> {
    let definitions = [
        ("Super", "nexxus.shell.application-menu"),
        ("Super+F", "nexxus.launcher.application-finder"),
        ("Super+T", nexxus_tiling::TILE_FIT_ACTION_ID),
        ("Ctrl+Alt+T", "nexxus.launcher.terminal"),
        ("Alt+Tab", "nexxus.wm.cycle-current-workspace"),
        ("Super+Tab", "nexxus.workspaces.cycle-mru-forward"),
        ("Super+Shift+Tab", "nexxus.workspaces.cycle-mru-backward"),
        ("Super+L", "nexxus.session.lock"),
        ("Ctrl+Escape", "nexxus.shell.desktop-menu"),
        ("Ctrl+Alt+Delete", "nexxus.launcher.bashtop"),
        ("Ctrl+Shift+Escape", "nexxus.launcher.bashtop"),
        ("Alt+F4", "nexxus.wm.close-focused"),
        ("Super+Left", "nexxus.workspaces.previous"),
        ("Super+Right", "nexxus.workspaces.next"),
        (
            "Super+Shift+Left",
            "nexxus.workspaces.move-focused-previous",
        ),
        ("Super+Shift+Right", "nexxus.workspaces.move-focused-next"),
        ("Print", "nexxus.capture.default"),
        ("Alt+Print", "nexxus.capture.alternate"),
        ("Shift+Print", "nexxus.capture.shift"),
        ("XF86AudioLowerVolume", "nexxus.media.volume-down"),
        ("XF86AudioRaiseVolume", "nexxus.media.volume-up"),
        ("XF86AudioMute", "nexxus.media.volume-mute"),
        ("XF86AudioPlay", "nexxus.media.play-pause"),
        ("XF86AudioPrev", "nexxus.media.previous"),
        ("XF86AudioNext", "nexxus.media.next"),
        ("XF86MonBrightnessDown", "nexxus.brightness.down"),
        ("XF86MonBrightnessUp", "nexxus.brightness.up"),
    ];

    definitions
        .into_iter()
        .map(|(trigger, command_id)| {
            (
                Trigger::parse(trigger).expect("static default shortcut trigger is valid"),
                command(command_id),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CommandTarget, WmAction};
    use std::fmt;

    #[derive(Debug)]
    struct TestSinkError;

    impl fmt::Display for TestSinkError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("test sink error")
        }
    }

    impl StdError for TestSinkError {}

    #[derive(Default)]
    struct RecordingSink {
        targets: Vec<CommandTarget>,
    }

    impl ShortcutDispatchSink for RecordingSink {
        type Error = TestSinkError;

        fn dispatch(&mut self, command: &CommandDescriptor) -> Result<(), Self::Error> {
            self.targets.push(command.target());
            Ok(())
        }
    }

    #[test]
    fn defaults_never_reserve_bare_f11() {
        let registry = ShortcutRegistry::with_defaults();
        assert!(
            registry
                .binding_for(&Trigger::parse("F11").unwrap())
                .is_none()
        );
        assert!(
            registry
                .bindings()
                .all(|(trigger, _)| !trigger.is_bare_f11())
        );
    }

    #[test]
    fn rejects_conflict_without_overwriting_existing_binding() {
        let mut registry = ShortcutRegistry::with_defaults();
        let trigger = Trigger::parse("Super+F").unwrap();
        let original = registry.binding_for(&trigger).unwrap().clone();

        let result = registry.bind(trigger.clone(), command("nexxus.wm.close-focused"));
        assert!(matches!(result, Err(ShortcutError::Conflict(_))));
        assert_eq!(registry.binding_for(&trigger), Some(&original));
    }

    #[test]
    fn rejects_bare_f11_even_when_reconfiguring() {
        let mut registry = ShortcutRegistry::with_defaults();
        let old = Trigger::parse("Alt+F4").unwrap();
        let result = registry.rebind(&old, Trigger::parse("F11").unwrap());
        assert!(matches!(result, Err(ShortcutError::BareF11Reserved)));
        assert_eq!(
            registry.binding_for(&old),
            Some(&command("nexxus.wm.close-focused"))
        );
    }

    #[test]
    fn bindings_round_trip_through_versioned_config() {
        let registry = ShortcutRegistry::with_defaults();
        let restored = ShortcutRegistry::from_config(registry.config_snapshot()).unwrap();
        assert_eq!(
            restored.config_snapshot().bindings,
            registry.config_snapshot().bindings
        );
    }

    #[test]
    fn dispatches_resolved_command_target() {
        let registry = ShortcutRegistry::with_defaults();
        let mut sink = RecordingSink::default();
        assert!(
            registry
                .dispatch_trigger(&Trigger::parse("Alt+F4").unwrap(), &mut sink)
                .unwrap()
        );
        assert_eq!(
            sink.targets,
            vec![CommandTarget::Wm(WmAction::CloseFocused)]
        );
    }

    #[test]
    fn saves_and_loads_versioned_configuration_atomically() {
        let path = std::env::temp_dir().join(format!(
            "nexxus-shortcuts-{}-{}.toml",
            std::process::id(),
            "roundtrip"
        ));
        let registry = ShortcutRegistry::with_defaults();
        registry.save(&path).unwrap();
        let restored = ShortcutRegistry::load(&path).unwrap();
        assert_eq!(restored.config_snapshot(), registry.config_snapshot());
        let _ = std::fs::remove_file(path);
    }
}
