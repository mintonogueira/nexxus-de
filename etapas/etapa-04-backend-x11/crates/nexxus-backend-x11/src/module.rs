//! Adapters for the Core lifecycle and backend-neutral graphics contract.

use crate::runtime::inspect_output;
use crate::{X11BackendError, X11Service};
use nexxus_backend_api::{
    BackendCapabilities, BackendError, BackendKind, GraphicsBackend, OutputInfo,
};
use nexxus_core::{
    ApiVersion, CapabilityId, IsolationMode, ModuleContext, ModuleDescriptor, ModuleFailure,
    ModuleId, NexxusModule,
};

const GRAPHICS_CAPABILITY: &str = "graphics.backend";

pub fn module_id() -> ModuleId {
    ModuleId::new("nexxus-backend-x11").expect("canonical module id is valid")
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor {
        id: module_id(),
        name: "Nexxus X11 Backend".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        required_core_api: ApiVersion::new(1, 0),
        provides: vec![
            CapabilityId::new(GRAPHICS_CAPABILITY).expect("canonical capability is valid"),
        ],
        requires: Vec::new(),
        optional: false,
        isolation: IsolationMode::InProcess,
    }
}

/// Concrete module consumed by Session Runtime. `initialize` performs only a
/// non-invasive display preflight; claiming the WM role happens in `start` so
/// lifecycle rollback remains correct.
pub struct X11BackendModule {
    descriptor: ModuleDescriptor,
    display: Option<String>,
    preflight_output: Option<OutputInfo>,
    service: Option<X11Service>,
}

impl X11BackendModule {
    pub fn new(display: Option<String>) -> Self {
        Self {
            descriptor: module_descriptor(),
            display,
            preflight_output: None,
            service: None,
        }
    }

    pub fn controller(&self) -> Option<crate::X11Controller> {
        self.service.as_ref().map(X11Service::controller)
    }

    fn start_service(&mut self) -> Result<(), X11BackendError> {
        if self.service.is_none() {
            self.service = Some(X11Service::start(self.display.clone())?);
        }
        Ok(())
    }

    fn stop_service(&mut self) -> Result<(), X11BackendError> {
        if let Some(mut service) = self.service.take() {
            service.stop()?;
        }
        Ok(())
    }
}

impl NexxusModule for X11BackendModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn initialize(&mut self, _context: &ModuleContext) -> Result<(), ModuleFailure> {
        self.preflight_output = Some(
            inspect_output(self.display.as_deref())
                .map_err(|error| ModuleFailure::new(error.to_string()))?,
        );
        Ok(())
    }

    fn start(&mut self, _context: &ModuleContext) -> Result<(), ModuleFailure> {
        self.start_service()
            .map_err(|error| ModuleFailure::new(error.to_string()))
    }

    fn stop(&mut self, _context: &ModuleContext) -> Result<(), ModuleFailure> {
        self.stop_service()
            .map_err(|error| ModuleFailure::new(error.to_string()))
    }
}

impl GraphicsBackend for X11BackendModule {
    fn kind(&self) -> BackendKind {
        BackendKind::X11
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            multi_monitor: false,
            hotplug: false,
            xwayland_compatibility: false,
        }
    }

    fn start(&mut self) -> Result<(), BackendError> {
        self.start_service().map_err(to_backend_error)
    }

    fn stop(&mut self) -> Result<(), BackendError> {
        self.stop_service().map_err(to_backend_error)
    }

    fn outputs(&self) -> Result<Vec<OutputInfo>, BackendError> {
        if let Some(service) = &self.service {
            return Ok(vec![service.output()]);
        }
        if let Some(output) = &self.preflight_output {
            return Ok(vec![output.clone()]);
        }
        inspect_output(self.display.as_deref())
            .map(|output| vec![output])
            .map_err(to_backend_error)
    }
}

fn to_backend_error(error: X11BackendError) -> BackendError {
    match error {
        X11BackendError::Unavailable(message) => BackendError::Unavailable(message),
        other => BackendError::Operation(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_matches_session_runtime_contract() {
        let descriptor = module_descriptor();
        assert_eq!(descriptor.id.as_str(), "nexxus-backend-x11");
        assert!(
            descriptor
                .provides
                .iter()
                .any(|capability| capability.as_str() == GRAPHICS_CAPABILITY)
        );
    }
}
