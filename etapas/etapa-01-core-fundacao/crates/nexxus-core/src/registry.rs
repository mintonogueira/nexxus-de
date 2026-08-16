//! Dependency and capability registry for Nexxus modules.

use crate::{ApiVersion, CapabilityId, Dependency, ModuleDescriptor, ModuleId};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Clone, Debug, Default)]
pub struct CapabilitySelections(BTreeMap<CapabilityId, ModuleId>);

impl CapabilitySelections {
    /// Selects one provider when a capability has multiple valid providers.
    pub fn select(&mut self, capability: CapabilityId, module: ModuleId) {
        self.0.insert(capability, module);
    }

    fn get(&self, capability: &CapabilityId) -> Option<&ModuleId> {
        self.0.get(capability)
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RegistryError {
    #[error("module '{0}' is already registered")]
    DuplicateModule(ModuleId),
    #[error(
        "module '{module}' requires incompatible Core API {required}; current API is {current}"
    )]
    IncompatibleCoreApi {
        module: ModuleId,
        required: ApiVersion,
        current: ApiVersion,
    },
    #[error("module '{module}' depends on missing module '{dependency}'")]
    MissingModule {
        module: ModuleId,
        dependency: ModuleId,
    },
    #[error("module '{module}' depends on unavailable capability '{capability}'")]
    MissingCapability {
        module: ModuleId,
        capability: CapabilityId,
    },
    #[error("capability '{capability}' has multiple providers and no valid selection")]
    AmbiguousCapability { capability: CapabilityId },
    #[error("selected provider '{provider}' does not provide capability '{capability}'")]
    InvalidCapabilitySelection {
        capability: CapabilityId,
        provider: ModuleId,
    },
    #[error("dependency cycle detected involving module '{0}'")]
    DependencyCycle(ModuleId),
}

#[derive(Clone, Debug)]
pub struct ModuleRegistry {
    core_api: ApiVersion,
    modules: BTreeMap<ModuleId, ModuleDescriptor>,
}

impl ModuleRegistry {
    pub fn new(core_api: ApiVersion) -> Self {
        Self {
            core_api,
            modules: BTreeMap::new(),
        }
    }

    /// Registers metadata only after validating Core API compatibility.
    pub fn register(&mut self, descriptor: ModuleDescriptor) -> Result<(), RegistryError> {
        if self.modules.contains_key(&descriptor.id) {
            return Err(RegistryError::DuplicateModule(descriptor.id));
        }
        if !self.core_api.supports(descriptor.required_core_api) {
            return Err(RegistryError::IncompatibleCoreApi {
                module: descriptor.id,
                required: descriptor.required_core_api,
                current: self.core_api,
            });
        }
        self.modules.insert(descriptor.id.clone(), descriptor);
        Ok(())
    }

    pub fn descriptor(&self, id: &ModuleId) -> Option<&ModuleDescriptor> {
        self.modules.get(id)
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &ModuleDescriptor> {
        self.modules.values()
    }

    /// Resolves a deterministic dependency-first order and rejects incomplete,
    /// ambiguous or cyclic graphs before lifecycle initialization begins.
    pub fn resolve_order(
        &self,
        selections: &CapabilitySelections,
    ) -> Result<Vec<ModuleId>, RegistryError> {
        let providers = self.capability_providers();
        let mut graph: BTreeMap<ModuleId, Vec<ModuleId>> = BTreeMap::new();

        for descriptor in self.modules.values() {
            let mut dependencies = Vec::new();
            for dependency in &descriptor.requires {
                match dependency {
                    Dependency::Module(id) => {
                        if !self.modules.contains_key(id) {
                            return Err(RegistryError::MissingModule {
                                module: descriptor.id.clone(),
                                dependency: id.clone(),
                            });
                        }
                        dependencies.push(id.clone());
                    }
                    Dependency::Capability(capability) => {
                        dependencies.push(self.resolve_capability(
                            &descriptor.id,
                            capability,
                            &providers,
                            selections,
                        )?);
                    }
                }
            }
            graph.insert(descriptor.id.clone(), dependencies);
        }

        let mut temporary = BTreeSet::new();
        let mut permanent = BTreeSet::new();
        let mut order = Vec::with_capacity(graph.len());
        for id in graph.keys() {
            Self::visit(id, &graph, &mut temporary, &mut permanent, &mut order)?;
        }
        Ok(order)
    }

    fn capability_providers(&self) -> BTreeMap<CapabilityId, Vec<ModuleId>> {
        let mut providers: BTreeMap<CapabilityId, Vec<ModuleId>> = BTreeMap::new();
        for descriptor in self.modules.values() {
            for capability in &descriptor.provides {
                providers
                    .entry(capability.clone())
                    .or_default()
                    .push(descriptor.id.clone());
            }
        }
        providers
    }

    fn resolve_capability(
        &self,
        module: &ModuleId,
        capability: &CapabilityId,
        providers: &BTreeMap<CapabilityId, Vec<ModuleId>>,
        selections: &CapabilitySelections,
    ) -> Result<ModuleId, RegistryError> {
        let candidates =
            providers
                .get(capability)
                .ok_or_else(|| RegistryError::MissingCapability {
                    module: module.clone(),
                    capability: capability.clone(),
                })?;

        if let Some(selected) = selections.get(capability) {
            if candidates.contains(selected) {
                return Ok(selected.clone());
            }
            return Err(RegistryError::InvalidCapabilitySelection {
                capability: capability.clone(),
                provider: selected.clone(),
            });
        }

        if candidates.len() == 1 {
            return Ok(candidates[0].clone());
        }
        Err(RegistryError::AmbiguousCapability {
            capability: capability.clone(),
        })
    }

    fn visit(
        id: &ModuleId,
        graph: &BTreeMap<ModuleId, Vec<ModuleId>>,
        temporary: &mut BTreeSet<ModuleId>,
        permanent: &mut BTreeSet<ModuleId>,
        order: &mut Vec<ModuleId>,
    ) -> Result<(), RegistryError> {
        if permanent.contains(id) {
            return Ok(());
        }
        if !temporary.insert(id.clone()) {
            return Err(RegistryError::DependencyCycle(id.clone()));
        }
        if let Some(dependencies) = graph.get(id) {
            for dependency in dependencies {
                Self::visit(dependency, graph, temporary, permanent, order)?;
            }
        }
        temporary.remove(id);
        permanent.insert(id.clone());
        order.push(id.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IsolationMode, ModuleDescriptor};

    fn id(value: &str) -> ModuleId {
        ModuleId::new(value).unwrap()
    }

    fn cap(value: &str) -> CapabilityId {
        CapabilityId::new(value).unwrap()
    }

    fn descriptor(name: &str) -> ModuleDescriptor {
        ModuleDescriptor {
            id: id(name),
            name: name.into(),
            version: "0.1.0".into(),
            required_core_api: ApiVersion::new(1, 0),
            provides: Vec::new(),
            requires: Vec::new(),
            optional: false,
            isolation: IsolationMode::InProcess,
        }
    }

    #[test]
    fn rejects_module_requiring_newer_core_minor_api() {
        let mut module = descriptor("future");
        module.required_core_api = ApiVersion::new(1, 1);
        let mut registry = ModuleRegistry::new(ApiVersion::new(1, 0));
        assert!(matches!(
            registry.register(module),
            Err(RegistryError::IncompatibleCoreApi { .. })
        ));
    }

    #[test]
    fn rejects_duplicate_module() {
        let mut registry = ModuleRegistry::new(ApiVersion::new(1, 0));
        registry.register(descriptor("alpha")).unwrap();
        assert!(matches!(
            registry.register(descriptor("alpha")),
            Err(RegistryError::DuplicateModule(_))
        ));
    }

    #[test]
    fn rejects_missing_dependency() {
        let mut module = descriptor("alpha");
        module.requires.push(Dependency::Module(id("missing")));
        let mut registry = ModuleRegistry::new(ApiVersion::new(1, 0));
        registry.register(module).unwrap();
        assert!(matches!(
            registry.resolve_order(&CapabilitySelections::default()),
            Err(RegistryError::MissingModule { .. })
        ));
    }

    #[test]
    fn detects_dependency_cycle() {
        let mut first = descriptor("a");
        let mut second = descriptor("b");
        first.requires.push(Dependency::Module(id("b")));
        second.requires.push(Dependency::Module(id("a")));
        let mut registry = ModuleRegistry::new(ApiVersion::new(1, 0));
        registry.register(first).unwrap();
        registry.register(second).unwrap();
        assert!(matches!(
            registry.resolve_order(&CapabilitySelections::default()),
            Err(RegistryError::DependencyCycle(_))
        ));
    }

    #[test]
    fn selection_resolves_ambiguous_capability() {
        let graphics = cap("graphics.backend");
        let mut first = descriptor("backend-x");
        first.provides.push(graphics.clone());
        let mut second = descriptor("backend-y");
        second.provides.push(graphics.clone());
        let mut consumer = descriptor("consumer");
        consumer
            .requires
            .push(Dependency::Capability(graphics.clone()));

        let mut registry = ModuleRegistry::new(ApiVersion::new(1, 0));
        registry.register(first).unwrap();
        registry.register(second).unwrap();
        registry.register(consumer).unwrap();

        assert!(matches!(
            registry.resolve_order(&CapabilitySelections::default()),
            Err(RegistryError::AmbiguousCapability { .. })
        ));
        let mut selections = CapabilitySelections::default();
        selections.select(graphics, id("backend-x"));
        let order = registry.resolve_order(&selections).unwrap();
        assert!(
            order
                .iter()
                .position(|value| value == &id("backend-x"))
                .unwrap()
                < order
                    .iter()
                    .position(|value| value == &id("consumer"))
                    .unwrap()
        );
    }
}
