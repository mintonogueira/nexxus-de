//! Logical events received from graphics adapters and commands emitted to them.

use crate::{Geometry, SizeConstraints, WindowId, WindowMetadata};
use nexxus_backend_api::BackendError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WmEvent {
    WindowCreated {
        id: WindowId,
        geometry: Geometry,
        constraints: SizeConstraints,
        metadata: WindowMetadata,
    },
    WindowDestroyed {
        id: WindowId,
    },
    WindowMapped {
        id: WindowId,
    },
    WindowUnmapped {
        id: WindowId,
    },
    WindowGeometryChanged {
        id: WindowId,
        geometry: Geometry,
    },
    FocusChanged {
        id: Option<WindowId>,
    },
    WindowMetadataChanged {
        id: WindowId,
        metadata: WindowMetadata,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WmCommand {
    RequestFocus {
        window: WindowId,
    },
    RequestMove {
        window: WindowId,
        x: i32,
        y: i32,
    },
    RequestResize {
        window: WindowId,
        width: u32,
        height: u32,
    },
    RequestMaximize {
        window: WindowId,
    },
    RequestRestore {
        window: WindowId,
    },
    RequestFullscreen {
        window: WindowId,
        enabled: bool,
    },
    RequestClose {
        window: WindowId,
    },
}

/// Thin output boundary implemented by future concrete graphics backends.
/// No protocol-native handle crosses this interface.
pub trait BackendCommandSink {
    fn submit(&mut self, command: &WmCommand) -> Result<(), BackendError>;
}
