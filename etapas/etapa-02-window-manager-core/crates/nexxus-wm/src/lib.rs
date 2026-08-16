//! Backend-agnostic Window Manager Core for Nexxus.
//!
//! This crate owns logical window state, focus/MRU, geometry constraints,
//! lifecycle events and commands. X11, Wayland, rendering, workspaces, tiling
//! algorithms and UI are deliberately excluded and belong to later stages.

#![forbid(unsafe_code)]

mod contract;
mod manager;
mod types;

pub use contract::{BackendCommandSink, WmCommand, WmEvent};
pub use manager::{EventOutcome, WindowManager, WmError};
pub use types::{Geometry, GeometryError, PresentationState, RestoreSnapshot, SizeConstraints, Window, WindowId, WindowIdError, WindowMetadata, WindowPlacement};

use nexxus_core::ModuleId;

/// Returns the module identifier consumed later by Session Runtime/registry.
pub fn module_id() -> ModuleId {
    ModuleId::new("nexxus-wm").expect("static Nexxus module id is valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: u64) -> WindowId { WindowId::new(value).unwrap() }
    fn geometry(x: i32) -> Geometry { Geometry::new(x, 20, 800, 600).unwrap() }
    fn created(value: u64) -> WmEvent {
        WmEvent::WindowCreated { id: id(value), geometry: geometry(value as i32), constraints: SizeConstraints::default(), metadata: WindowMetadata::default() }
    }

    #[test]
    fn module_identifier_is_stable_and_backend_neutral() { assert_eq!(module_id().as_str(), "nexxus-wm"); }

    #[test]
    fn duplicate_ids_are_rejected() {
        let mut wm = WindowManager::new(); wm.apply_event(created(1)).unwrap();
        assert!(matches!(wm.apply_event(created(1)), Err(WmError::DuplicateWindow(_))));
    }

    #[test]
    fn focus_and_mru_are_deterministic() {
        let mut wm = WindowManager::new(); wm.apply_event(created(1)).unwrap(); wm.apply_event(created(2)).unwrap();
        wm.set_focus(id(1)).unwrap(); wm.set_focus(id(2)).unwrap();
        assert_eq!(wm.focused(), Some(id(2)));
        assert_eq!(wm.mru().collect::<Vec<_>>(), vec![id(2), id(1)]);
        assert_eq!(wm.focus_previous(), Some(id(1)));
    }

    #[test]
    fn destroying_focused_window_recovers_previous_focus() {
        let mut wm = WindowManager::new(); wm.apply_event(created(1)).unwrap(); wm.apply_event(created(2)).unwrap();
        wm.set_focus(id(1)).unwrap(); wm.set_focus(id(2)).unwrap();
        wm.apply_event(WmEvent::WindowDestroyed { id: id(2) }).unwrap();
        assert_eq!(wm.focused(), Some(id(1))); assert!(wm.window(id(1)).unwrap().active);
    }

    #[test]
    fn resize_obeys_constraints_without_mutating_until_backend_event() {
        let mut wm = WindowManager::new();
        wm.apply_event(WmEvent::WindowCreated {
            id: id(1), geometry: geometry(10),
            constraints: SizeConstraints { min_width: 320, min_height: 200, max_width: Some(1920), max_height: Some(1080) },
            metadata: WindowMetadata::default(),
        }).unwrap();
        assert_eq!(wm.request_resize(id(1), 100, 5000).unwrap(), WmCommand::RequestResize { window: id(1), width: 320, height: 1080 });
        assert_eq!(wm.window(id(1)).unwrap().geometry, geometry(10));
    }

    #[test]
    fn maximize_and_restore_preserve_floating_geometry() {
        let mut wm = WindowManager::new(); wm.apply_event(created(1)).unwrap();
        let original = wm.window(id(1)).unwrap().geometry;
        wm.maximize(id(1)).unwrap();
        wm.apply_event(WmEvent::WindowGeometryChanged { id: id(1), geometry: Geometry::new(0, 0, 1920, 1080).unwrap() }).unwrap();
        wm.restore(id(1)).unwrap();
        assert_eq!(wm.window(id(1)).unwrap().geometry, original);
        assert_eq!(wm.window(id(1)).unwrap().presentation, PresentationState::Normal);
    }

    #[test]
    fn nested_fullscreen_restore_returns_through_previous_state() {
        let mut wm = WindowManager::new(); wm.apply_event(created(1)).unwrap();
        wm.maximize(id(1)).unwrap(); wm.set_fullscreen(id(1), true).unwrap(); wm.restore(id(1)).unwrap();
        assert_eq!(wm.window(id(1)).unwrap().presentation, PresentationState::Maximized);
        wm.restore(id(1)).unwrap(); assert_eq!(wm.window(id(1)).unwrap().presentation, PresentationState::Normal);
    }

    #[test]
    fn floating_state_survives_tiled_round_trip() {
        let mut wm = WindowManager::new(); wm.apply_event(created(1)).unwrap(); let floating = wm.window(id(1)).unwrap().geometry;
        wm.set_placement(id(1), WindowPlacement::Tiled).unwrap();
        wm.apply_event(WmEvent::WindowGeometryChanged { id: id(1), geometry: Geometry::new(0, 0, 960, 1080).unwrap() }).unwrap();
        wm.set_placement(id(1), WindowPlacement::Floating).unwrap();
        assert_eq!(wm.window(id(1)).unwrap().geometry, floating);
    }

    #[test]
    fn stale_events_are_ignored_after_destruction() {
        let mut wm = WindowManager::new(); wm.apply_event(created(1)).unwrap(); wm.apply_event(WmEvent::WindowDestroyed { id: id(1) }).unwrap();
        assert_eq!(wm.apply_event(WmEvent::WindowGeometryChanged { id: id(1), geometry: geometry(40) }).unwrap(), EventOutcome::IgnoredStale);
        assert_eq!(wm.apply_event(WmEvent::WindowDestroyed { id: id(1) }).unwrap(), EventOutcome::IgnoredStale);
    }

    #[test]
    fn invalid_state_sequences_fail_without_corrupting_restore_state() {
        let mut wm = WindowManager::new(); wm.apply_event(created(1)).unwrap();
        assert!(matches!(wm.restore(id(1)), Err(WmError::NothingToRestore(_))));
        wm.maximize(id(1)).unwrap(); assert!(matches!(wm.maximize(id(1)), Err(WmError::InvalidState { .. })));
        wm.restore(id(1)).unwrap(); assert_eq!(wm.window(id(1)).unwrap().presentation, PresentationState::Normal);
    }
}
