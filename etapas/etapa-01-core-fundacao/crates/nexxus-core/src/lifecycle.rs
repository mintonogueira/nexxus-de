//! Module lifecycle orchestration for the Nexxus Core.
//!
//! The manager resolves dependencies before any module receives control and
//! performs reverse-order cleanup on failures. A failing module remains marked
//! `Failed` even when its partial resources are successfully cleaned up.

use crate::{
    CapabilitySelections, CoreEvent, EventBus, ModuleDescriptor, ModuleId, ModuleRegistry,
    ModuleState, RegistryError,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Clone)]
pub struct ModuleContext {
    runtime_dir: PathBuf,
    events: EventBus,
}

impl ModuleContext {
    pub fn new(runtime_dir: impl Into<PathBuf>, events: EventBus) -> Self {
        Self {
            runtime_dir: runtime_dir.into(),
            events,
        }
    }

    pub fn runtime_dir(&self) -> &Path {
        &self.runtime_dir
    }

    pub fn events(&self) -> &EventBus {
        &self.events
    }
}

#[derive(Debug, Error)]
#[error("{message}")]
pub struct ModuleFailure {
    message: String,
}

impl ModuleFailure {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Runtime implementation of one descriptor registered in the Core.
///
/// `stop` must tolerate partial initialization because rollback may call it
/// after an `initialize` or `start` failure.
pub trait NexxusModule: Send {
    fn descriptor(&self) -> &ModuleDescriptor;
    fn initialize(&mut self, context: &ModuleContext) -> Result<(), ModuleFailure>;
    fn start(&mut self, context: &ModuleContext) -> Result<(), ModuleFailure>;
    fn stop(&mut self, context: &ModuleContext) -> Result<(), ModuleFailure>;
}

#[derive(Debug, Error)]
pub enum LifecycleError {
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error("module implementation '{0}' is already installed")]
    DuplicateImplementation(ModuleId),
    #[error("no module implementation exists for descriptor '{0}'")]
    MissingImplementation(ModuleId),
    #[error("implementation descriptor for '{module}' differs from the registered descriptor")]
    DescriptorMismatch { module: ModuleId },
    #[error("module '{module}' is in state {state:?} and cannot perform {operation}")]
    InvalidState {
        module: ModuleId,
        state: ModuleState,
        operation: &'static str,
    },
    #[error("module '{module}' failed during {phase}: {source}")]
    Module {
        module: ModuleId,
        phase: &'static str,
        #[source]
        source: ModuleFailure,
    },
}

pub struct LifecycleManager {
    registry: ModuleRegistry,
    modules: BTreeMap<ModuleId, Box<dyn NexxusModule>>,
    states: BTreeMap<ModuleId, ModuleState>,
    context: ModuleContext,
}

impl LifecycleManager {
    pub fn new(registry: ModuleRegistry, context: ModuleContext) -> Self {
        Self {
            registry,
            modules: BTreeMap::new(),
            states: BTreeMap::new(),
            context,
        }
    }

    /// Installs an implementation only when it exactly matches its registered
    /// descriptor, preventing runtime code from changing validated metadata.
    pub fn install(&mut self, module: Box<dyn NexxusModule>) -> Result<(), LifecycleError> {
        let id = module.descriptor().id.clone();
        if self.modules.contains_key(&id) {
            return Err(LifecycleError::DuplicateImplementation(id));
        }
        let registered = self
            .registry
            .descriptor(&id)
            .ok_or_else(|| LifecycleError::MissingImplementation(id.clone()))?;
        if module.descriptor() != registered {
            return Err(LifecycleError::DescriptorMismatch { module: id });
        }

        self.states.insert(id.clone(), ModuleState::Registered);
        self.modules.insert(id, module);
        Ok(())
    }

    pub fn state(&self, id: &ModuleId) -> Option<ModuleState> {
        self.states.get(id).copied()
    }

    /// Starts all resolved modules only after complete implementation/state
    /// preflight. No module is initialized while a later one is still missing.
    pub fn start_all(
        &mut self,
        selections: &CapabilitySelections,
    ) -> Result<Vec<ModuleId>, LifecycleError> {
        let order = self.registry.resolve_order(selections)?;
        self.preflight_start(&order)?;

        let mut initialized = Vec::with_capacity(order.len());
        for id in &order {
            let initialize_result = self
                .modules
                .get_mut(id)
                .expect("preflight guarantees module existence")
                .initialize(&self.context);
            if let Err(source) = initialize_result {
                self.cleanup_failed_module(id);
                self.rollback(&initialized);
                self.mark(id, ModuleState::Failed);
                return Err(LifecycleError::Module {
                    module: id.clone(),
                    phase: "initialize",
                    source,
                });
            }
            self.mark(id, ModuleState::Initialized);
            initialized.push(id.clone());

            let start_result = self
                .modules
                .get_mut(id)
                .expect("preflight guarantees module existence")
                .start(&self.context);
            if let Err(source) = start_result {
                initialized.pop();
                self.cleanup_failed_module(id);
                self.rollback(&initialized);
                self.mark(id, ModuleState::Failed);
                return Err(LifecycleError::Module {
                    module: id.clone(),
                    phase: "start",
                    source,
                });
            }
            self.mark(id, ModuleState::Running);
        }
        Ok(initialized)
    }

    /// Stops modules in reverse dependency order and reports every failure so
    /// one broken module does not suppress cleanup of the remaining modules.
    pub fn stop_all(&mut self, order: &[ModuleId]) -> Vec<LifecycleError> {
        let mut errors = Vec::new();
        for id in order.iter().rev() {
            let Some(state) = self.state(id) else {
                errors.push(LifecycleError::MissingImplementation(id.clone()));
                continue;
            };
            if !matches!(state, ModuleState::Running | ModuleState::Initialized) {
                continue;
            }

            self.mark(id, ModuleState::Stopping);
            let stop_result = match self.modules.get_mut(id) {
                Some(module) => module.stop(&self.context),
                None => {
                    errors.push(LifecycleError::MissingImplementation(id.clone()));
                    continue;
                }
            };
            match stop_result {
                Ok(()) => self.mark(id, ModuleState::Stopped),
                Err(source) => {
                    self.mark(id, ModuleState::Failed);
                    errors.push(LifecycleError::Module {
                        module: id.clone(),
                        phase: "stop",
                        source,
                    });
                }
            }
        }
        errors
    }

    fn preflight_start(&self, order: &[ModuleId]) -> Result<(), LifecycleError> {
        for id in order {
            let module = self
                .modules
                .get(id)
                .ok_or_else(|| LifecycleError::MissingImplementation(id.clone()))?;
            let registered = self
                .registry
                .descriptor(id)
                .ok_or_else(|| LifecycleError::MissingImplementation(id.clone()))?;
            if module.descriptor() != registered {
                return Err(LifecycleError::DescriptorMismatch { module: id.clone() });
            }

            let state = self
                .states
                .get(id)
                .copied()
                .ok_or_else(|| LifecycleError::MissingImplementation(id.clone()))?;
            if !matches!(state, ModuleState::Registered | ModuleState::Stopped) {
                return Err(LifecycleError::InvalidState {
                    module: id.clone(),
                    state,
                    operation: "start",
                });
            }
        }
        Ok(())
    }

    fn rollback(&mut self, initialized: &[ModuleId]) {
        let errors = self.stop_all(initialized);
        for error in errors {
            tracing::warn!(%error, "module rollback cleanup failed");
        }
    }

    /// Cleans resources from the module that produced the primary failure but
    /// leaves the final observable state as `Failed` for diagnostics.
    fn cleanup_failed_module(&mut self, id: &ModuleId) {
        if let Some(module) = self.modules.get_mut(id) {
            if let Err(error) = module.stop(&self.context) {
                tracing::warn!(module = %id, %error, "failed module cleanup also failed");
            }
        }
    }

    fn mark(&mut self, id: &ModuleId, state: ModuleState) {
        self.states.insert(id.clone(), state);
        self.context.events.publish(CoreEvent::ModuleStateChanged {
            module: id.clone(),
            state,
        });
        tracing::debug!(module = %id, ?state, "Nexxus module state changed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ApiVersion, CORE_API_VERSION, IsolationMode};
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Copy)]
    enum FailurePoint {
        None,
        Initialize,
        Start,
    }

    struct DummyModule {
        descriptor: ModuleDescriptor,
        calls: Arc<Mutex<Vec<&'static str>>>,
        failure: FailurePoint,
    }

    impl NexxusModule for DummyModule {
        fn descriptor(&self) -> &ModuleDescriptor {
            &self.descriptor
        }

        fn initialize(&mut self, _context: &ModuleContext) -> Result<(), ModuleFailure> {
            self.calls.lock().unwrap().push("initialize");
            if matches!(self.failure, FailurePoint::Initialize) {
                return Err(ModuleFailure::new("initialize failed"));
            }
            Ok(())
        }

        fn start(&mut self, _context: &ModuleContext) -> Result<(), ModuleFailure> {
            self.calls.lock().unwrap().push("start");
            if matches!(self.failure, FailurePoint::Start) {
                return Err(ModuleFailure::new("start failed"));
            }
            Ok(())
        }

        fn stop(&mut self, _context: &ModuleContext) -> Result<(), ModuleFailure> {
            self.calls.lock().unwrap().push("stop");
            Ok(())
        }
    }

    fn id(value: &str) -> ModuleId {
        ModuleId::new(value).unwrap()
    }

    fn descriptor(value: &str) -> ModuleDescriptor {
        ModuleDescriptor {
            id: id(value),
            name: value.into(),
            version: "0.1.0".into(),
            required_core_api: ApiVersion::new(1, 0),
            provides: Vec::new(),
            requires: Vec::new(),
            optional: false,
            isolation: IsolationMode::InProcess,
        }
    }

    fn manager_with(descriptors: &[ModuleDescriptor]) -> LifecycleManager {
        let mut registry = ModuleRegistry::new(CORE_API_VERSION);
        for descriptor in descriptors {
            registry.register(descriptor.clone()).unwrap();
        }
        LifecycleManager::new(
            registry,
            ModuleContext::new("/tmp/nexxus-lifecycle-test", EventBus::new()),
        )
    }

    #[test]
    fn missing_implementation_fails_before_any_module_is_initialized() {
        let first = descriptor("first");
        let second = descriptor("second");
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mut manager = manager_with(&[first.clone(), second]);
        manager
            .install(Box::new(DummyModule {
                descriptor: first,
                calls: Arc::clone(&calls),
                failure: FailurePoint::None,
            }))
            .unwrap();

        assert!(matches!(
            manager.start_all(&CapabilitySelections::default()),
            Err(LifecycleError::MissingImplementation(_))
        ));
        assert!(calls.lock().unwrap().is_empty());
    }

    #[test]
    fn start_failure_cleans_resources_but_preserves_failed_state() {
        let desc = descriptor("broken");
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mut manager = manager_with(std::slice::from_ref(&desc));
        manager
            .install(Box::new(DummyModule {
                descriptor: desc.clone(),
                calls: Arc::clone(&calls),
                failure: FailurePoint::Start,
            }))
            .unwrap();

        assert!(manager.start_all(&CapabilitySelections::default()).is_err());
        assert_eq!(manager.state(&desc.id), Some(ModuleState::Failed));
        assert_eq!(
            calls.lock().unwrap().as_slice(),
            &["initialize", "start", "stop"]
        );
    }

    #[test]
    fn initialize_failure_attempts_cleanup_and_preserves_failed_state() {
        let desc = descriptor("broken-init");
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mut manager = manager_with(std::slice::from_ref(&desc));
        manager
            .install(Box::new(DummyModule {
                descriptor: desc.clone(),
                calls: Arc::clone(&calls),
                failure: FailurePoint::Initialize,
            }))
            .unwrap();

        assert!(manager.start_all(&CapabilitySelections::default()).is_err());
        assert_eq!(manager.state(&desc.id), Some(ModuleState::Failed));
        assert_eq!(calls.lock().unwrap().as_slice(), &["initialize", "stop"]);
    }
}
