//! Single-threaded X11 protocol engine used by the background service.
//!
//! The X connection and `WindowManager` state machine are intentionally owned
//! by the same worker. This keeps protocol ordering deterministic and avoids
//! locking X11 state while still reusing, rather than duplicating, Etapa 02's
//! window-management logic.

use crate::atoms::Atoms;
use crate::{X11BackendError, operation_error};
use nexxus_backend_api::{OutputId, OutputInfo};
use nexxus_wm::{
    Geometry, PresentationState, SizeConstraints, Window, WindowId, WindowManager, WindowMetadata,
    WmCommand, WmEvent,
};
use std::collections::BTreeSet;
use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::properties::{WmClass, WmSizeHints};
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ChangeWindowAttributesAux, ClientMessageEvent, ConfigureWindowAux,
    ConnectionExt as _, CreateWindowAux, EventMask, InputFocus, MapState, PropMode, StackMode,
    WindowClass,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

const WM_NAME: &[u8] = b"Nexxus";

pub(crate) struct X11Runtime {
    conn: RustConnection,
    screen_num: usize,
    root: u32,
    width: u32,
    height: u32,
    support_window: u32,
    atoms: Atoms,
    manager: WindowManager,
    managed: BTreeSet<WindowId>,
}

impl X11Runtime {
    /// Opens the selected display, claims SubstructureRedirect atomically and
    /// publishes the minimal EWMH root properties before managing clients.
    pub fn connect_and_claim(display: Option<&str>) -> Result<Self, X11BackendError> {
        let (conn, screen_num) = x11rb::connect(display)
            .map_err(|error| X11BackendError::Unavailable(error.to_string()))?;
        let screen = conn.setup().roots.get(screen_num).ok_or_else(|| {
            X11BackendError::Unavailable("selected X11 screen does not exist".into())
        })?;
        let root = screen.root;
        let width = u32::from(screen.width_in_pixels);
        let height = u32::from(screen.height_in_pixels);

        let claim = ChangeWindowAttributesAux::new().event_mask(
            EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::PROPERTY_CHANGE
                | EventMask::STRUCTURE_NOTIFY,
        );
        conn.change_window_attributes(root, &claim)
            .map_err(operation_error)?
            .check()
            .map_err(|error| {
                X11BackendError::Unavailable(format!(
                    "cannot claim the X11 root window; another window manager may already be running: {error}"
                ))
            })?;

        let atoms = Atoms::load(&conn)?;
        let support_window = conn.generate_id().map_err(operation_error)?;
        conn.create_window(
            COPY_DEPTH_FROM_PARENT,
            support_window,
            root,
            -1,
            -1,
            1,
            1,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new(),
        )
        .map_err(operation_error)?;

        let mut runtime = Self {
            conn,
            screen_num,
            root,
            width,
            height,
            support_window,
            atoms,
            manager: WindowManager::new(),
            managed: BTreeSet::new(),
        };
        runtime.publish_wm_identity()?;
        runtime.scan_existing_windows()?;
        runtime.conn.flush().map_err(operation_error)?;
        Ok(runtime)
    }

    pub fn output(&self) -> Result<OutputInfo, X11BackendError> {
        Ok(OutputInfo {
            id: OutputId::new(format!("x11-screen-{}", self.screen_num))
                .map_err(|error| X11BackendError::Operation(error.to_owned()))?,
            name: format!("X11 Screen {}", self.screen_num),
            width: self.width,
            height: self.height,
            scale_milli: 1000,
            primary: true,
        })
    }

    pub fn windows(&self) -> Vec<Window> {
        self.manager.windows().cloned().collect()
    }

    /// Drains all currently queued X11 events. The outer service interleaves
    /// this with command-channel work so one source cannot starve the other.
    pub fn drain_events(&mut self) -> Result<(), X11BackendError> {
        while let Some(event) = self.conn.poll_for_event().map_err(operation_error)? {
            self.handle_event(event)?;
        }
        self.conn.flush().map_err(operation_error)
    }

    pub fn focus(&mut self, id: WindowId) -> Result<(), X11BackendError> {
        let command = self.manager.request_focus(id).map_err(operation_error)?;
        self.execute_command(&command)
    }

    pub fn move_window(&mut self, id: WindowId, x: i32, y: i32) -> Result<(), X11BackendError> {
        let command = self
            .manager
            .request_move(id, x, y)
            .map_err(operation_error)?;
        self.execute_command(&command)
    }

    pub fn resize_window(
        &mut self,
        id: WindowId,
        width: u32,
        height: u32,
    ) -> Result<(), X11BackendError> {
        let command = self
            .manager
            .request_resize(id, width, height)
            .map_err(operation_error)?;
        self.execute_command(&command)
    }

    pub fn maximize(&mut self, id: WindowId) -> Result<(), X11BackendError> {
        let command = self.manager.maximize(id).map_err(operation_error)?;
        self.execute_command(&command)
    }

    pub fn restore(&mut self, id: WindowId) -> Result<(), X11BackendError> {
        let command = self.manager.restore(id).map_err(operation_error)?;
        self.execute_command(&command)
    }

    pub fn fullscreen(&mut self, id: WindowId, enabled: bool) -> Result<(), X11BackendError> {
        let command = self
            .manager
            .set_fullscreen(id, enabled)
            .map_err(operation_error)?;
        self.execute_command(&command)
    }

    pub fn close(&mut self, id: WindowId) -> Result<(), X11BackendError> {
        let command = self.manager.request_close(id).map_err(operation_error)?;
        self.execute_command(&command)
    }

    /// Releases properties created by this WM and destroys only the private
    /// supporting window. Client windows are never destroyed during shutdown.
    pub fn shutdown(&mut self) -> Result<(), X11BackendError> {
        let _ = self
            .conn
            .delete_property(self.root, self.atoms.net_supporting_wm_check);
        let _ = self
            .conn
            .delete_property(self.root, self.atoms.net_active_window);
        let _ = self
            .conn
            .delete_property(self.root, self.atoms.net_client_list);
        let _ = self
            .conn
            .delete_property(self.root, self.atoms.net_client_list_stacking);
        let _ = self.conn.destroy_window(self.support_window);
        self.conn.flush().map_err(operation_error)
    }

    fn publish_wm_identity(&self) -> Result<(), X11BackendError> {
        self.conn
            .change_property32(
                PropMode::REPLACE,
                self.root,
                self.atoms.net_supporting_wm_check,
                AtomEnum::WINDOW,
                &[self.support_window],
            )
            .map_err(operation_error)?;
        self.conn
            .change_property32(
                PropMode::REPLACE,
                self.support_window,
                self.atoms.net_supporting_wm_check,
                AtomEnum::WINDOW,
                &[self.support_window],
            )
            .map_err(operation_error)?;
        self.conn
            .change_property8(
                PropMode::REPLACE,
                self.support_window,
                self.atoms.net_wm_name,
                self.atoms.utf8_string,
                WM_NAME,
            )
            .map_err(operation_error)?;
        self.conn
            .change_property32(
                PropMode::REPLACE,
                self.root,
                self.atoms.net_supported,
                AtomEnum::ATOM,
                &self.atoms.supported(),
            )
            .map_err(operation_error)?;
        Ok(())
    }

    fn scan_existing_windows(&mut self) -> Result<(), X11BackendError> {
        let children = self
            .conn
            .query_tree(self.root)
            .map_err(operation_error)?
            .reply()
            .map_err(operation_error)?
            .children;
        for window in children {
            if window == self.support_window {
                continue;
            }
            let attributes = match self
                .conn
                .get_window_attributes(window)
                .map_err(operation_error)?
                .reply()
            {
                Ok(attributes) => attributes,
                Err(_) => continue,
            };
            if !attributes.override_redirect && attributes.map_state != MapState::UNMAPPED {
                self.manage_window(window, false)?;
            }
        }
        self.publish_client_list()
    }

    fn handle_event(&mut self, event: Event) -> Result<(), X11BackendError> {
        match event {
            Event::MapRequest(event) => self.manage_window(event.window, true)?,
            Event::DestroyNotify(event) => self.destroy_window(event.window)?,
            Event::UnmapNotify(event) => {
                if let Some(id) = self.known_id(event.window) {
                    self.apply(WmEvent::WindowUnmapped { id })?;
                }
            }
            Event::ConfigureRequest(event) => {
                let aux = ConfigureWindowAux::from_configure_request(&event)
                    .sibling(None)
                    .stack_mode(None);
                self.conn
                    .configure_window(event.window, &aux)
                    .map_err(operation_error)?;
            }
            Event::ConfigureNotify(event) => {
                if let Some(id) = self.known_id(event.window) {
                    let geometry = Geometry::new(
                        i32::from(event.x),
                        i32::from(event.y),
                        u32::from(event.width),
                        u32::from(event.height),
                    )
                    .map_err(operation_error)?;
                    self.apply(WmEvent::WindowGeometryChanged { id, geometry })?;
                }
            }
            Event::FocusIn(event) => {
                if let Some(id) = self.known_id(event.event) {
                    self.apply(WmEvent::FocusChanged { id: Some(id) })?;
                    self.publish_active(Some(id))?;
                }
            }
            Event::PropertyNotify(event) => {
                if let Some(id) = self.known_id(event.window) {
                    let metadata = self.read_metadata(event.window)?;
                    self.apply(WmEvent::WindowMetadataChanged { id, metadata })?;
                }
            }
            Event::ClientMessage(event) => self.handle_client_message(event)?,
            _ => {}
        }
        Ok(())
    }

    fn manage_window(&mut self, window: u32, map: bool) -> Result<(), X11BackendError> {
        if window == self.support_window {
            return Ok(());
        }
        let attributes = self
            .conn
            .get_window_attributes(window)
            .map_err(operation_error)?
            .reply()
            .map_err(operation_error)?;
        if attributes.override_redirect {
            if map {
                self.conn.map_window(window).map_err(operation_error)?;
            }
            return Ok(());
        }

        let id = window_id(window)?;
        if !self.managed.contains(&id) {
            let geometry_reply = self
                .conn
                .get_geometry(window)
                .map_err(operation_error)?
                .reply()
                .map_err(operation_error)?;
            let geometry = Geometry::new(
                i32::from(geometry_reply.x),
                i32::from(geometry_reply.y),
                u32::from(geometry_reply.width),
                u32::from(geometry_reply.height),
            )
            .map_err(operation_error)?;
            let constraints = self.read_constraints(window)?;
            let metadata = self.read_metadata(window)?;
            self.apply(WmEvent::WindowCreated {
                id,
                geometry,
                constraints,
                metadata,
            })?;
            self.managed.insert(id);
            self.conn
                .change_window_attributes(
                    window,
                    &ChangeWindowAttributesAux::new().event_mask(
                        EventMask::PROPERTY_CHANGE
                            | EventMask::FOCUS_CHANGE
                            | EventMask::STRUCTURE_NOTIFY,
                    ),
                )
                .map_err(operation_error)?;
        }
        if map {
            self.conn.map_window(window).map_err(operation_error)?;
        }
        self.apply(WmEvent::WindowMapped { id })?;
        self.publish_client_list()
    }

    fn destroy_window(&mut self, window: u32) -> Result<(), X11BackendError> {
        if let Some(id) = self.known_id(window) {
            self.apply(WmEvent::WindowDestroyed { id })?;
            self.managed.remove(&id);
            self.publish_client_list()?;
        }
        Ok(())
    }

    fn handle_client_message(&mut self, event: ClientMessageEvent) -> Result<(), X11BackendError> {
        let Some(id) = self.known_id(event.window) else {
            return Ok(());
        };
        if event.type_ == self.atoms.net_active_window {
            return self.focus(id);
        }
        if event.type_ == self.atoms.net_close_window {
            return self.close(id);
        }
        if event.type_ != self.atoms.net_wm_state || event.format != 32 {
            return Ok(());
        }

        let data = event.data.as_data32();
        let action = data[0];
        let first = data[1];
        let second = data[2];
        let state = self.manager.window(id).map(|window| window.presentation);
        if first == self.atoms.net_wm_state_fullscreen
            || second == self.atoms.net_wm_state_fullscreen
        {
            let current = state == Some(PresentationState::Fullscreen);
            let enabled = requested_state(action, current);
            if enabled != current {
                return self.fullscreen(id, enabled);
            }
            return Ok(());
        }
        let maximize = first == self.atoms.net_wm_state_maximized_vert
            || first == self.atoms.net_wm_state_maximized_horz
            || second == self.atoms.net_wm_state_maximized_vert
            || second == self.atoms.net_wm_state_maximized_horz;
        if maximize {
            let current = state == Some(PresentationState::Maximized);
            let enabled = requested_state(action, current);
            if enabled && !current {
                self.maximize(id)?;
            } else if !enabled && current {
                self.restore(id)?;
            }
        }
        Ok(())
    }

    fn execute_command(&mut self, command: &WmCommand) -> Result<(), X11BackendError> {
        match *command {
            WmCommand::RequestFocus { window } => {
                let xid = xid(window)?;
                self.conn
                    .set_input_focus(InputFocus::PARENT, xid, CURRENT_TIME)
                    .map_err(operation_error)?;
                self.conn
                    .configure_window(xid, &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE))
                    .map_err(operation_error)?;
                self.send_take_focus_if_supported(xid)?;
                self.publish_active(Some(window))?;
            }
            WmCommand::RequestMove { window, x, y } => {
                self.conn
                    .configure_window(xid(window)?, &ConfigureWindowAux::new().x(x).y(y))
                    .map_err(operation_error)?;
            }
            WmCommand::RequestResize {
                window,
                width,
                height,
            } => {
                self.conn
                    .configure_window(
                        xid(window)?,
                        &ConfigureWindowAux::new().width(width).height(height),
                    )
                    .map_err(operation_error)?;
            }
            WmCommand::RequestMaximize { window } => {
                let xid = xid(window)?;
                self.conn
                    .configure_window(
                        xid,
                        &ConfigureWindowAux::new()
                            .x(0)
                            .y(0)
                            .width(self.width)
                            .height(self.height),
                    )
                    .map_err(operation_error)?;
                self.set_state(
                    xid,
                    &[
                        self.atoms.net_wm_state_maximized_vert,
                        self.atoms.net_wm_state_maximized_horz,
                    ],
                )?;
            }
            WmCommand::RequestRestore { window } => {
                let snapshot = self.manager.window(window).ok_or_else(|| {
                    X11BackendError::Operation("restore target disappeared".into())
                })?;
                let geometry = snapshot.geometry;
                let presentation = snapshot.presentation;
                let xid = xid(window)?;
                self.conn
                    .configure_window(
                        xid,
                        &ConfigureWindowAux::new()
                            .x(geometry.x)
                            .y(geometry.y)
                            .width(geometry.width)
                            .height(geometry.height),
                    )
                    .map_err(operation_error)?;
                self.publish_presentation_state(xid, presentation)?;
            }
            WmCommand::RequestFullscreen { window, enabled } => {
                let xid = xid(window)?;
                if enabled {
                    self.conn
                        .configure_window(
                            xid,
                            &ConfigureWindowAux::new()
                                .x(0)
                                .y(0)
                                .width(self.width)
                                .height(self.height),
                        )
                        .map_err(operation_error)?;
                    self.set_state(xid, &[self.atoms.net_wm_state_fullscreen])?;
                } else {
                    let snapshot = self.manager.window(window).ok_or_else(|| {
                        X11BackendError::Operation("fullscreen target disappeared".into())
                    })?;
                    let geometry = snapshot.geometry;
                    self.conn
                        .configure_window(
                            xid,
                            &ConfigureWindowAux::new()
                                .x(geometry.x)
                                .y(geometry.y)
                                .width(geometry.width)
                                .height(geometry.height),
                        )
                        .map_err(operation_error)?;
                    self.publish_presentation_state(xid, snapshot.presentation)?;
                }
            }
            WmCommand::RequestClose { window } => self.request_close(xid(window)?)?,
        }
        self.conn.flush().map_err(operation_error)
    }

    fn request_close(&self, window: u32) -> Result<(), X11BackendError> {
        if self.supports_protocol(window, self.atoms.wm_delete_window)? {
            let event = ClientMessageEvent::new(
                32,
                window,
                self.atoms.wm_protocols,
                [self.atoms.wm_delete_window, CURRENT_TIME, 0, 0, 0],
            );
            self.conn
                .send_event(false, window, EventMask::NO_EVENT, event)
                .map_err(operation_error)?;
        } else {
            self.conn.kill_client(window).map_err(operation_error)?;
        }
        Ok(())
    }

    fn send_take_focus_if_supported(&self, window: u32) -> Result<(), X11BackendError> {
        if self.supports_protocol(window, self.atoms.wm_take_focus)? {
            let event = ClientMessageEvent::new(
                32,
                window,
                self.atoms.wm_protocols,
                [self.atoms.wm_take_focus, CURRENT_TIME, 0, 0, 0],
            );
            self.conn
                .send_event(false, window, EventMask::NO_EVENT, event)
                .map_err(operation_error)?;
        }
        Ok(())
    }

    fn supports_protocol(&self, window: u32, protocol: Atom) -> Result<bool, X11BackendError> {
        let reply = self
            .conn
            .get_property(
                false,
                window,
                self.atoms.wm_protocols,
                AtomEnum::ATOM,
                0,
                64,
            )
            .map_err(operation_error)?
            .reply()
            .map_err(operation_error)?;
        Ok(reply
            .value32()
            .is_some_and(|mut atoms| atoms.any(|atom| atom == protocol)))
    }

    fn read_metadata(&self, window: u32) -> Result<WindowMetadata, X11BackendError> {
        let net_title = self
            .conn
            .get_property(
                false,
                window,
                self.atoms.net_wm_name,
                self.atoms.utf8_string,
                0,
                4096,
            )
            .map_err(operation_error)?
            .reply()
            .map_err(operation_error)?;
        let title = if !net_title.value.is_empty() {
            String::from_utf8_lossy(&net_title.value).into_owned()
        } else {
            let legacy = self
                .conn
                .get_property(false, window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 4096)
                .map_err(operation_error)?
                .reply()
                .map_err(operation_error)?;
            String::from_utf8_lossy(&legacy.value).into_owned()
        };
        let application_id = WmClass::get(&self.conn, window)
            .map_err(operation_error)?
            .reply()
            .map_err(operation_error)?
            .and_then(|class| {
                let preferred = if class.class().is_empty() {
                    class.instance()
                } else {
                    class.class()
                };
                (!preferred.is_empty()).then(|| String::from_utf8_lossy(preferred).into_owned())
            });
        Ok(WindowMetadata {
            title,
            application_id,
        })
    }

    fn read_constraints(&self, window: u32) -> Result<SizeConstraints, X11BackendError> {
        let hints = WmSizeHints::get_normal_hints(&self.conn, window)
            .map_err(operation_error)?
            .reply()
            .map_err(operation_error)?;
        let Some(hints) = hints else {
            return Ok(SizeConstraints::default());
        };
        let (min_width, min_height) = hints
            .min_size
            .map(|(width, height)| (positive_dimension(width), positive_dimension(height)))
            .unwrap_or((1, 1));
        let max_width = hints
            .max_size
            .and_then(|(width, _)| optional_dimension(width));
        let max_height = hints
            .max_size
            .and_then(|(_, height)| optional_dimension(height));
        let constraints = SizeConstraints {
            min_width,
            min_height,
            max_width,
            max_height,
        };
        constraints.validate().map_err(operation_error)?;
        Ok(constraints)
    }

    fn publish_client_list(&self) -> Result<(), X11BackendError> {
        let clients: Vec<u32> = self
            .managed
            .iter()
            .copied()
            .map(xid)
            .collect::<Result<_, _>>()?;
        self.conn
            .change_property32(
                PropMode::REPLACE,
                self.root,
                self.atoms.net_client_list,
                AtomEnum::WINDOW,
                &clients,
            )
            .map_err(operation_error)?;
        self.conn
            .change_property32(
                PropMode::REPLACE,
                self.root,
                self.atoms.net_client_list_stacking,
                AtomEnum::WINDOW,
                &clients,
            )
            .map_err(operation_error)?;
        Ok(())
    }

    fn publish_active(&self, id: Option<WindowId>) -> Result<(), X11BackendError> {
        let value = match id {
            Some(id) => xid(id)?,
            None => 0,
        };
        self.conn
            .change_property32(
                PropMode::REPLACE,
                self.root,
                self.atoms.net_active_window,
                AtomEnum::WINDOW,
                &[value],
            )
            .map_err(operation_error)?;
        Ok(())
    }

    fn publish_presentation_state(
        &self,
        window: u32,
        state: PresentationState,
    ) -> Result<(), X11BackendError> {
        match state {
            PresentationState::Normal => self.set_state(window, &[]),
            PresentationState::Maximized => self.set_state(
                window,
                &[
                    self.atoms.net_wm_state_maximized_vert,
                    self.atoms.net_wm_state_maximized_horz,
                ],
            ),
            PresentationState::Fullscreen => {
                self.set_state(window, &[self.atoms.net_wm_state_fullscreen])
            }
        }
    }

    fn set_state(&self, window: u32, states: &[Atom]) -> Result<(), X11BackendError> {
        self.conn
            .change_property32(
                PropMode::REPLACE,
                window,
                self.atoms.net_wm_state,
                AtomEnum::ATOM,
                states,
            )
            .map_err(operation_error)?;
        Ok(())
    }

    fn known_id(&self, window: u32) -> Option<WindowId> {
        let id = window_id(window).ok()?;
        self.managed.contains(&id).then_some(id)
    }

    fn apply(&mut self, event: WmEvent) -> Result<(), X11BackendError> {
        self.manager
            .apply_event(event)
            .map(|_| ())
            .map_err(operation_error)
    }
}

fn requested_state(action: u32, current: bool) -> bool {
    match action {
        0 => false,
        1 => true,
        2 => !current,
        _ => current,
    }
}

fn positive_dimension(value: i32) -> u32 {
    u32::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .unwrap_or(1)
}

fn optional_dimension(value: i32) -> Option<u32> {
    u32::try_from(value).ok().filter(|value| *value > 0)
}

pub(crate) fn window_id(window: u32) -> Result<WindowId, X11BackendError> {
    WindowId::new(u64::from(window)).map_err(operation_error)
}

fn xid(id: WindowId) -> Result<u32, X11BackendError> {
    u32::try_from(id.get()).map_err(operation_error)
}

/// Opens a display without claiming the WM role. Used by preflight/output
/// inspection so configuration checks never disturb the active window manager.
pub(crate) fn inspect_output(display: Option<&str>) -> Result<OutputInfo, X11BackendError> {
    let (conn, screen_num) =
        x11rb::connect(display).map_err(|error| X11BackendError::Unavailable(error.to_string()))?;
    let screen =
        conn.setup().roots.get(screen_num).ok_or_else(|| {
            X11BackendError::Unavailable("selected X11 screen does not exist".into())
        })?;
    Ok(OutputInfo {
        id: OutputId::new(format!("x11-screen-{screen_num}"))
            .map_err(|error| X11BackendError::Operation(error.to_owned()))?,
        name: format!("X11 Screen {screen_num}"),
        width: u32::from(screen.width_in_pixels),
        height: u32::from(screen.height_in_pixels),
        scale_milli: 1000,
        primary: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ewmh_state_actions_follow_remove_add_toggle_semantics() {
        assert!(!requested_state(0, true));
        assert!(requested_state(1, false));
        assert!(!requested_state(2, true));
        assert!(requested_state(2, false));
    }

    #[test]
    fn xid_round_trip_uses_the_backend_neutral_window_id() {
        let id = window_id(42).unwrap();
        assert_eq!(xid(id).unwrap(), 42);
    }
}
