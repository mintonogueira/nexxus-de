//! Fundamental runtime contracts for the Nexxus desktop environment.
//!
//! This crate deliberately contains no X11, Wayland, UI toolkit, device,
//! network, storage or privileged-operation implementation. Its role is to
//! provide the small, backend-agnostic core required by the project
//! architecture: module identity, dependency resolution, lifecycle control
//! and typed internal events.

#![forbid(unsafe_code)]

mod event;
mod lifecycle;
mod paths;
mod registry;
mod types;

pub use event::{CoreEvent, EventBus, EventSubscription};
pub use lifecycle::{LifecycleError, LifecycleManager, ModuleContext, ModuleFailure, NexxusModule};
pub use paths::{NexxusPaths, PathError};
pub use registry::{CapabilitySelections, ModuleRegistry, RegistryError};
pub use types::{
    ApiVersion, CapabilityId, Dependency, IsolationMode, ModuleDescriptor, ModuleId, ModuleState,
};

/// API version implemented by this initial Core foundation.
pub const CORE_API_VERSION: ApiVersion = ApiVersion::new(1, 0);
