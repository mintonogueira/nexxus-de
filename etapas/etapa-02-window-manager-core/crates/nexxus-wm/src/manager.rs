//! Deterministic backend-agnostic window state machine.

use crate::{BackendCommandSink, Geometry, GeometryError, PresentationState, Window, WindowId, WindowMetadata, WindowPlacement, WmCommand, WmEvent};
use nexxus_backend_api::BackendError;
use std::collections::{BTreeMap, VecDeque};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventOutcome { Applied, IgnoredStale }

#[derive(Debug, Error)]
pub enum WmError {
    #[error("window '{0:?}' is already registered")]
    DuplicateWindow(WindowId),
    #[error("window '{0:?}' is not registered")]
    UnknownWindow(WindowId),
    #[error("window '{window:?}' cannot perform {operation} while presentation state is {state:?}")]
    InvalidState { window: WindowId, operation: &'static str, state: PresentationState },
    #[error("window '{0:?}' has no presentation state to restore")]
    NothingToRestore(WindowId),
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    #[error(transparent)]
    Backend(#[from] BackendError),
}

/// Owns logical windows, the active window and most-recently-used focus order.
#[derive(Default)]
pub struct WindowManager {
    windows: BTreeMap<WindowId, Window>,
    mru: VecDeque<WindowId>,
    focused: Option<WindowId>,
}

impl WindowManager {
    pub fn new() -> Self { Self::default() }
    pub fn window(&self, id: WindowId) -> Option<&Window> { self.windows.get(&id) }
    pub fn windows(&self) -> impl ExactSizeIterator<Item = &Window> { self.windows.values() }
    pub const fn focused(&self) -> Option<WindowId> { self.focused }
    pub fn mru(&self) -> impl Iterator<Item = WindowId> + '_ { self.mru.iter().copied() }

    /// Applies one normalized backend event. Stale updates for already removed
    /// windows are ignored because real event queues may legally race destroy.
    pub fn apply_event(&mut self, event: WmEvent) -> Result<EventOutcome, WmError> {
        match event {
            WmEvent::WindowCreated { id, geometry, constraints, metadata } => {
                if self.windows.contains_key(&id) { return Err(WmError::DuplicateWindow(id)); }
                let window = Window::new(id, geometry, constraints, metadata)?;
                self.windows.insert(id, window);
                self.touch_mru(id);
                Ok(EventOutcome::Applied)
            }
            WmEvent::WindowDestroyed { id } => {
                if self.windows.remove(&id).is_none() { return Ok(EventOutcome::IgnoredStale); }
                self.remove_from_mru(id);
                if self.focused == Some(id) { self.focused = None; self.recover_focus(); }
                Ok(EventOutcome::Applied)
            }
            WmEvent::WindowMapped { id } => {
                let Some(window) = self.windows.get_mut(&id) else { return Ok(EventOutcome::IgnoredStale); };
                window.mapped = true; window.visible = true; Ok(EventOutcome::Applied)
            }
            WmEvent::WindowUnmapped { id } => {
                let Some(window) = self.windows.get_mut(&id) else { return Ok(EventOutcome::IgnoredStale); };
                window.mapped = false; window.visible = false; window.active = false;
                if self.focused == Some(id) { self.focused = None; self.recover_focus(); }
                Ok(EventOutcome::Applied)
            }
            WmEvent::WindowGeometryChanged { id, geometry } => {
                let Some(window) = self.windows.get_mut(&id) else { return Ok(EventOutcome::IgnoredStale); };
                window.update_geometry(geometry)?; Ok(EventOutcome::Applied)
            }
            WmEvent::FocusChanged { id } => {
                if let Some(id) = id {
                    if !self.windows.contains_key(&id) { return Ok(EventOutcome::IgnoredStale); }
                    self.set_focus(id)?;
                } else { self.clear_focus(); }
                Ok(EventOutcome::Applied)
            }
            WmEvent::WindowMetadataChanged { id, metadata } => {
                let Some(window) = self.windows.get_mut(&id) else { return Ok(EventOutcome::IgnoredStale); };
                window.metadata = metadata; Ok(EventOutcome::Applied)
            }
        }
    }

    /// Marks one mapped/visible window active and updates MRU deterministically.
    pub fn set_focus(&mut self, id: WindowId) -> Result<(), WmError> {
        let focusable = self.windows.get(&id).map(|w| w.mapped && w.visible).ok_or(WmError::UnknownWindow(id))?;
        if !focusable {
            return Err(WmError::InvalidState { window: id, operation: "focus", state: self.windows[&id].presentation });
        }
        self.clear_focus();
        if let Some(window) = self.windows.get_mut(&id) { window.active = true; }
        self.focused = Some(id); self.touch_mru(id); Ok(())
    }

    pub fn focus_previous(&mut self) -> Option<WindowId> {
        let current = self.focused;
        let next = self.mru.iter().copied().find(|candidate| Some(*candidate) != current && self.is_focusable(*candidate));
        if let Some(next) = next { let _ = self.set_focus(next); }
        next
    }

    pub fn request_move(&self, id: WindowId, x: i32, y: i32) -> Result<WmCommand, WmError> {
        self.require_normal(id, "move")?;
        Ok(WmCommand::RequestMove { window: id, x, y })
    }

    pub fn request_resize(&self, id: WindowId, width: u32, height: u32) -> Result<WmCommand, WmError> {
        let window = self.require_normal(id, "resize")?;
        let constrained = Geometry::new(window.geometry.x, window.geometry.y, width, height)?.constrained(window.constraints)?;
        Ok(WmCommand::RequestResize { window: id, width: constrained.width, height: constrained.height })
    }

    /// Pushes prior state before maximizing so restore is lossless.
    pub fn maximize(&mut self, id: WindowId) -> Result<WmCommand, WmError> {
        let window = self.window_mut(id)?;
        if window.presentation == PresentationState::Maximized {
            return Err(WmError::InvalidState { window: id, operation: "maximize", state: window.presentation });
        }
        window.push_restore_snapshot(); window.presentation = PresentationState::Maximized;
        Ok(WmCommand::RequestMaximize { window: id })
    }

    pub fn set_fullscreen(&mut self, id: WindowId, enabled: bool) -> Result<WmCommand, WmError> {
        if enabled {
            let window = self.window_mut(id)?;
            if window.presentation == PresentationState::Fullscreen {
                return Err(WmError::InvalidState { window: id, operation: "enter fullscreen", state: window.presentation });
            }
            window.push_restore_snapshot(); window.presentation = PresentationState::Fullscreen;
            return Ok(WmCommand::RequestFullscreen { window: id, enabled: true });
        }
        let state = self.windows.get(&id).ok_or(WmError::UnknownWindow(id))?.presentation;
        if state != PresentationState::Fullscreen {
            return Err(WmError::InvalidState { window: id, operation: "leave fullscreen", state });
        }
        self.restore(id)?;
        Ok(WmCommand::RequestFullscreen { window: id, enabled: false })
    }

    /// Restores one transition, including geometry and prior placement.
    pub fn restore(&mut self, id: WindowId) -> Result<WmCommand, WmError> {
        let window = self.window_mut(id)?;
        let snapshot = window.pop_restore_snapshot().ok_or(WmError::NothingToRestore(id))?;
        window.geometry = snapshot.geometry;
        window.placement = snapshot.placement;
        window.presentation = snapshot.presentation;
        if snapshot.placement == WindowPlacement::Floating && snapshot.presentation == PresentationState::Normal {
            window.floating_geometry = snapshot.geometry;
        }
        Ok(WmCommand::RequestRestore { window: id })
    }

    /// Changes only placement state. Actual tiled geometry belongs to Etapa 06.
    pub fn set_placement(&mut self, id: WindowId, placement: WindowPlacement) -> Result<(), WmError> {
        let window = self.window_mut(id)?;
        if window.presentation != PresentationState::Normal {
            return Err(WmError::InvalidState { window: id, operation: "change placement", state: window.presentation });
        }
        if placement == WindowPlacement::Floating { window.geometry = window.floating_geometry; }
        else if window.placement == WindowPlacement::Floating { window.floating_geometry = window.geometry; }
        window.placement = placement; Ok(())
    }

    pub fn request_focus(&self, id: WindowId) -> Result<WmCommand, WmError> {
        self.windows.contains_key(&id).then_some(WmCommand::RequestFocus { window: id }).ok_or(WmError::UnknownWindow(id))
    }
    pub fn request_close(&self, id: WindowId) -> Result<WmCommand, WmError> {
        self.windows.contains_key(&id).then_some(WmCommand::RequestClose { window: id }).ok_or(WmError::UnknownWindow(id))
    }
    pub fn dispatch(&self, sink: &mut impl BackendCommandSink, command: &WmCommand) -> Result<(), WmError> { sink.submit(command)?; Ok(()) }
    pub fn update_metadata(&mut self, id: WindowId, metadata: WindowMetadata) -> Result<(), WmError> { self.window_mut(id)?.metadata = metadata; Ok(()) }

    fn require_normal(&self, id: WindowId, operation: &'static str) -> Result<&Window, WmError> {
        let window = self.windows.get(&id).ok_or(WmError::UnknownWindow(id))?;
        if window.presentation != PresentationState::Normal {
            return Err(WmError::InvalidState { window: id, operation, state: window.presentation });
        }
        Ok(window)
    }
    fn window_mut(&mut self, id: WindowId) -> Result<&mut Window, WmError> { self.windows.get_mut(&id).ok_or(WmError::UnknownWindow(id)) }
    fn clear_focus(&mut self) {
        if let Some(id) = self.focused.take() {
            if let Some(window) = self.windows.get_mut(&id) { window.active = false; }
        }
    }
    fn recover_focus(&mut self) {
        let candidate = self.mru.iter().copied().find(|candidate| self.is_focusable(*candidate));
        if let Some(candidate) = candidate { let _ = self.set_focus(candidate); }
    }
    fn is_focusable(&self, id: WindowId) -> bool { self.windows.get(&id).is_some_and(|w| w.mapped && w.visible) }
    fn touch_mru(&mut self, id: WindowId) { self.remove_from_mru(id); self.mru.push_front(id); }
    fn remove_from_mru(&mut self, id: WindowId) { self.mru.retain(|candidate| *candidate != id); }
}
