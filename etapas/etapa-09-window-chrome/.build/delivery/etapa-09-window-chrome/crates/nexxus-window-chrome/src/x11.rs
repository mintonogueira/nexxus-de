//! Adapter X11 inicial para SSD sem reparenting do cliente.
//!
//! As janelas de decoração são `override_redirect`, portanto o Backend X11 da
//! Etapa 04 continua proprietário exclusivo das janelas de aplicação. O chrome
//! usa uma conexão própria apenas para superfícies/hit targets da moldura e
//! delega operações de janela ao `X11Controller` já existente.

use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryFrom;

use nexxus_backend_x11::X11Controller;
use nexxus_ui::{ScaleFactor, Theme};
use nexxus_wm::{Geometry, PresentationState, Window, WindowId, WindowPlacement};
use thiserror::Error;
use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ChangeWindowAttributesAux, ConfigureWindowAux, ConnectionExt as _, CreateGCAux,
    CreateWindowAux, EventMask, ImageFormat, PropMode, WindowClass,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use crate::ChromeHooks;
use crate::geometry::{
    ChromeButton, ChromeMetrics, HitTarget, ResizeEdge, TitlebarLayout, resized_geometry,
};
use crate::integration::{ChromeIntegrationError, close_window, maximize_restore};
use crate::policy::{DecorationDecision, DecorationHints, WindowType, decide_decoration};
use crate::render::{AssetSource, ChromePainter, ChromeRenderError, ChromeVisualState};

const MWM_HINTS_DECORATIONS: u32 = 1 << 1;

#[derive(Debug, Error)]
pub enum ChromeX11Error {
    #[error("cannot connect Window Chrome to X11: {0}")]
    Unavailable(String),
    #[error("X11 Window Chrome operation failed: {0}")]
    Operation(String),
    #[error(transparent)]
    Render(#[from] ChromeRenderError),
    #[error(transparent)]
    Integration(#[from] ChromeIntegrationError),
    #[error("WindowId {0} cannot be represented by an X11 XID")]
    InvalidWindowId(u64),
    #[error("X11 decoration dimension exceeds protocol limits")]
    DimensionOverflow,
    #[error("Window Chrome hook failed: {0}")]
    Hook(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SurfaceKind {
    Titlebar,
    LeftBorder,
    RightBorder,
    BottomBorder,
    LeftGrab,
    RightGrab,
    BottomGrab,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SurfaceOwner {
    window: WindowId,
    kind: SurfaceKind,
}

#[derive(Clone, Copy, Debug)]
struct DecorationWindows {
    titlebar: u32,
    title_gc: u32,
    left_border: u32,
    right_border: u32,
    bottom_border: u32,
    left_grab: u32,
    right_grab: u32,
    bottom_grab: u32,
}

impl DecorationWindows {
    fn all(self) -> [u32; 7] {
        [
            self.titlebar,
            self.left_border,
            self.right_border,
            self.bottom_border,
            self.left_grab,
            self.right_grab,
            self.bottom_grab,
        ]
    }
}

#[derive(Clone, Copy, Debug)]
enum DragKind {
    Move,
    Resize(ResizeEdge),
}

#[derive(Clone, Copy, Debug)]
struct DragState {
    window: WindowId,
    kind: DragKind,
    root_x: i32,
    root_y: i32,
    initial: Geometry,
    min_width: u32,
    min_height: u32,
}

#[derive(Clone, Copy, Debug)]
struct PressedButton {
    window: WindowId,
    button: ChromeButton,
}

struct Atoms {
    net_frame_extents: Atom,
    net_wm_window_type: Atom,
    net_wm_window_type_normal: Atom,
    net_wm_window_type_dialog: Atom,
    net_wm_window_type_utility: Atom,
    net_wm_window_type_desktop: Atom,
    net_wm_window_type_dock: Atom,
    net_wm_window_type_toolbar: Atom,
    net_wm_window_type_menu: Atom,
    net_wm_window_type_splash: Atom,
    net_wm_window_type_dropdown_menu: Atom,
    net_wm_window_type_popup_menu: Atom,
    net_wm_window_type_tooltip: Atom,
    net_wm_window_type_notification: Atom,
    net_wm_window_type_combo: Atom,
    net_wm_window_type_dnd: Atom,
    gtk_frame_extents: Atom,
    motif_wm_hints: Atom,
}

impl Atoms {
    fn load(conn: &RustConnection) -> Result<Self, ChromeX11Error> {
        Ok(Self {
            net_frame_extents: intern(conn, b"_NET_FRAME_EXTENTS")?,
            net_wm_window_type: intern(conn, b"_NET_WM_WINDOW_TYPE")?,
            net_wm_window_type_normal: intern(conn, b"_NET_WM_WINDOW_TYPE_NORMAL")?,
            net_wm_window_type_dialog: intern(conn, b"_NET_WM_WINDOW_TYPE_DIALOG")?,
            net_wm_window_type_utility: intern(conn, b"_NET_WM_WINDOW_TYPE_UTILITY")?,
            net_wm_window_type_desktop: intern(conn, b"_NET_WM_WINDOW_TYPE_DESKTOP")?,
            net_wm_window_type_dock: intern(conn, b"_NET_WM_WINDOW_TYPE_DOCK")?,
            net_wm_window_type_toolbar: intern(conn, b"_NET_WM_WINDOW_TYPE_TOOLBAR")?,
            net_wm_window_type_menu: intern(conn, b"_NET_WM_WINDOW_TYPE_MENU")?,
            net_wm_window_type_splash: intern(conn, b"_NET_WM_WINDOW_TYPE_SPLASH")?,
            net_wm_window_type_dropdown_menu: intern(conn, b"_NET_WM_WINDOW_TYPE_DROPDOWN_MENU")?,
            net_wm_window_type_popup_menu: intern(conn, b"_NET_WM_WINDOW_TYPE_POPUP_MENU")?,
            net_wm_window_type_tooltip: intern(conn, b"_NET_WM_WINDOW_TYPE_TOOLTIP")?,
            net_wm_window_type_notification: intern(conn, b"_NET_WM_WINDOW_TYPE_NOTIFICATION")?,
            net_wm_window_type_combo: intern(conn, b"_NET_WM_WINDOW_TYPE_COMBO")?,
            net_wm_window_type_dnd: intern(conn, b"_NET_WM_WINDOW_TYPE_DND")?,
            gtk_frame_extents: intern(conn, b"_GTK_FRAME_EXTENTS")?,
            motif_wm_hints: intern(conn, b"_MOTIF_WM_HINTS")?,
        })
    }

    fn classify_window_type(&self, atom: Atom) -> WindowType {
        if atom == self.net_wm_window_type_normal {
            WindowType::Normal
        } else if atom == self.net_wm_window_type_dialog {
            WindowType::Dialog
        } else if atom == self.net_wm_window_type_utility {
            WindowType::Utility
        } else if atom == self.net_wm_window_type_desktop {
            WindowType::Desktop
        } else if atom == self.net_wm_window_type_dock {
            WindowType::Dock
        } else if atom == self.net_wm_window_type_toolbar {
            WindowType::Toolbar
        } else if atom == self.net_wm_window_type_menu {
            WindowType::Menu
        } else if atom == self.net_wm_window_type_splash {
            WindowType::Splash
        } else if atom == self.net_wm_window_type_dropdown_menu {
            WindowType::DropdownMenu
        } else if atom == self.net_wm_window_type_popup_menu {
            WindowType::PopupMenu
        } else if atom == self.net_wm_window_type_tooltip {
            WindowType::Tooltip
        } else if atom == self.net_wm_window_type_notification {
            WindowType::Notification
        } else if atom == self.net_wm_window_type_combo {
            WindowType::Combo
        } else if atom == self.net_wm_window_type_dnd {
            WindowType::DragAndDrop
        } else {
            WindowType::Unknown
        }
    }
}

/// Serviço de Window Chrome X11. `sync()` reconcilia superfícies com o snapshot
/// do backend; `poll()` processa input sem bloquear o worker da Etapa 04.
pub struct X11ChromeAdapter {
    conn: RustConnection,
    root: u32,
    root_depth: u8,
    controller: X11Controller,
    atoms: Atoms,
    scale: ScaleFactor,
    metrics: ChromeMetrics,
    painter: ChromePainter,
    decorations: BTreeMap<WindowId, DecorationWindows>,
    owners: BTreeMap<u32, SurfaceOwner>,
    hovered: BTreeMap<WindowId, Option<ChromeButton>>,
    pressed: Option<PressedButton>,
    drag: Option<DragState>,
}

impl X11ChromeAdapter {
    pub fn connect(
        display: Option<&str>,
        controller: X11Controller,
        scale: ScaleFactor,
        theme: Theme,
        assets: AssetSource,
    ) -> Result<Self, ChromeX11Error> {
        let (conn, screen_num) = x11rb::connect(display)
            .map_err(|error| ChromeX11Error::Unavailable(error.to_string()))?;
        let screen = conn.setup().roots.get(screen_num).ok_or_else(|| {
            ChromeX11Error::Unavailable("selected X11 screen does not exist".into())
        })?;
        let root = screen.root;
        let root_depth = screen.root_depth;
        let atoms = Atoms::load(&conn)?;
        let metrics = ChromeMetrics::default();
        Ok(Self {
            conn,
            root,
            root_depth,
            controller,
            atoms,
            scale,
            metrics,
            painter: ChromePainter::new(theme, metrics, assets),
            decorations: BTreeMap::new(),
            owners: BTreeMap::new(),
            hovered: BTreeMap::new(),
            pressed: None,
            drag: None,
        })
    }

    pub fn decorated_windows(&self) -> impl Iterator<Item = WindowId> + '_ {
        self.decorations.keys().copied()
    }

    /// Reavalia CSD/SSD e posiciona as superfícies a partir do estado canônico
    /// exposto pelo Backend X11. Fullscreen nunca recebe chrome sobreposto.
    pub fn sync(&mut self) -> Result<(), ChromeX11Error> {
        let windows = self
            .controller
            .windows()
            .map_err(|error| ChromeX11Error::Operation(error.to_string()))?;
        let present: BTreeSet<_> = windows.iter().map(|window| window.id).collect();
        let stale: Vec<_> = self
            .decorations
            .keys()
            .copied()
            .filter(|id| !present.contains(id))
            .collect();
        for id in stale {
            self.remove_decoration(id)?;
        }

        for window in windows {
            if !window.mapped
                || !window.visible
                || window.presentation == PresentationState::Fullscreen
            {
                self.remove_decoration(window.id)?;
                continue;
            }
            let hints = self.read_decoration_hints(window.id)?;
            if decide_decoration(hints) != DecorationDecision::ServerSide {
                self.remove_decoration(window.id)?;
                self.delete_frame_extents(window.id)?;
                continue;
            }
            self.ensure_decoration(&window)?;
            self.sync_decoration(&window)?;
        }
        self.conn.flush().map_err(operation_error)
    }

    /// Processa todos os eventos já enfileirados. Hooks de tiling são chamados
    /// sincronicamente para que uma operação manual possa liberar tiled antes
    /// do primeiro delta de move/resize.
    pub fn poll(&mut self, hooks: &mut impl ChromeHooks) -> Result<(), ChromeX11Error> {
        while let Some(event) = self.conn.poll_for_event().map_err(operation_error)? {
            self.handle_event(event, hooks)?;
        }
        self.conn.flush().map_err(operation_error)
    }

    fn handle_event(
        &mut self,
        event: Event,
        hooks: &mut impl ChromeHooks,
    ) -> Result<(), ChromeX11Error> {
        match event {
            Event::ButtonPress(event) if event.detail == 1 => {
                self.pointer_press(
                    event.event,
                    i32::from(event.event_x),
                    i32::from(event.event_y),
                    i32::from(event.root_x),
                    i32::from(event.root_y),
                    hooks,
                )?;
            }
            Event::ButtonRelease(event) if event.detail == 1 => {
                self.pointer_release(
                    event.event,
                    i32::from(event.event_x),
                    i32::from(event.event_y),
                    hooks,
                )?;
            }
            Event::MotionNotify(event) => {
                self.pointer_motion(
                    event.event,
                    i32::from(event.event_x),
                    i32::from(event.event_y),
                    i32::from(event.root_x),
                    i32::from(event.root_y),
                )?;
            }
            Event::Expose(event) => {
                if let Some(owner) = self.owners.get(&event.window).copied() {
                    if owner.kind == SurfaceKind::Titlebar {
                        self.redraw(owner.window)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn pointer_press(
        &mut self,
        surface: u32,
        event_x: i32,
        event_y: i32,
        root_x: i32,
        root_y: i32,
        hooks: &mut impl ChromeHooks,
    ) -> Result<(), ChromeX11Error> {
        let Some(owner) = self.owners.get(&surface).copied() else {
            return Ok(());
        };
        let window = self.snapshot(owner.window)?;
        let target = self.hit_target(owner, &window, event_x, event_y);
        match target {
            HitTarget::Button(button) => {
                self.pressed = Some(PressedButton {
                    window: owner.window,
                    button,
                });
                self.hovered.insert(owner.window, Some(button));
                self.redraw(owner.window)?;
            }
            HitTarget::Titlebar => {
                if window.placement == WindowPlacement::Tiled {
                    hooks
                        .release_for_manual_operation(owner.window)
                        .map_err(ChromeX11Error::Hook)?;
                }
                self.drag = Some(DragState {
                    window: owner.window,
                    kind: DragKind::Move,
                    root_x,
                    root_y,
                    initial: window.geometry,
                    min_width: window.constraints.min_width,
                    min_height: window.constraints.min_height,
                });
            }
            HitTarget::Resize(edge) => {
                if window.placement == WindowPlacement::Tiled {
                    hooks
                        .release_for_manual_operation(owner.window)
                        .map_err(ChromeX11Error::Hook)?;
                }
                self.drag = Some(DragState {
                    window: owner.window,
                    kind: DragKind::Resize(edge),
                    root_x,
                    root_y,
                    initial: window.geometry,
                    min_width: window.constraints.min_width,
                    min_height: window.constraints.min_height,
                });
            }
            HitTarget::None => {}
        }
        Ok(())
    }

    fn pointer_motion(
        &mut self,
        surface: u32,
        event_x: i32,
        event_y: i32,
        root_x: i32,
        root_y: i32,
    ) -> Result<(), ChromeX11Error> {
        if let Some(drag) = self.drag {
            let dx = root_x.saturating_sub(drag.root_x);
            let dy = root_y.saturating_sub(drag.root_y);
            match drag.kind {
                DragKind::Move => self
                    .controller
                    .move_window(
                        drag.window,
                        drag.initial.x.saturating_add(dx),
                        drag.initial.y.saturating_add(dy),
                    )
                    .map_err(|error| ChromeX11Error::Operation(error.to_string()))?,
                DragKind::Resize(edge) => {
                    let geometry = resized_geometry(
                        drag.initial,
                        edge,
                        dx,
                        dy,
                        drag.min_width,
                        drag.min_height,
                    );
                    if geometry.x != drag.initial.x || geometry.y != drag.initial.y {
                        self.controller
                            .move_window(drag.window, geometry.x, geometry.y)
                            .map_err(|error| ChromeX11Error::Operation(error.to_string()))?;
                    }
                    self.controller
                        .resize_window(drag.window, geometry.width, geometry.height)
                        .map_err(|error| ChromeX11Error::Operation(error.to_string()))?;
                }
            }
            return Ok(());
        }

        if let Some(owner) = self.owners.get(&surface).copied() {
            if owner.kind == SurfaceKind::Titlebar {
                let window = self.snapshot(owner.window)?;
                let hovered = match self.hit_target(owner, &window, event_x, event_y) {
                    HitTarget::Button(button) => Some(button),
                    _ => None,
                };
                if self.hovered.get(&owner.window).copied().flatten() != hovered {
                    self.hovered.insert(owner.window, hovered);
                    self.redraw(owner.window)?;
                }
            }
        }
        Ok(())
    }

    fn pointer_release(
        &mut self,
        surface: u32,
        event_x: i32,
        event_y: i32,
        hooks: &mut impl ChromeHooks,
    ) -> Result<(), ChromeX11Error> {
        self.drag = None;
        let pressed = self.pressed.take();
        let Some(owner) = self.owners.get(&surface).copied() else {
            return Ok(());
        };
        if let Some(pressed) = pressed {
            let window = self.snapshot(owner.window)?;
            let released = self.hit_target(owner, &window, event_x, event_y);
            if pressed.window == owner.window && released == HitTarget::Button(pressed.button) {
                match pressed.button {
                    ChromeButton::TileFit => {
                        hooks.tile_fit(owner.window).map_err(ChromeX11Error::Hook)?
                    }
                    ChromeButton::MaximizeRestore => {
                        maximize_restore(&self.controller, owner.window)?
                    }
                    ChromeButton::Close => close_window(&self.controller, owner.window)?,
                }
            }
            self.redraw(owner.window)?;
        }
        Ok(())
    }

    fn hit_target(&self, owner: SurfaceOwner, window: &Window, x: i32, y: i32) -> HitTarget {
        let grab = (self.metrics.resize_grab * self.scale.get())
            .round()
            .max(1.0) as i32;
        match owner.kind {
            SurfaceKind::Titlebar => {
                let width = self.titlebar_logical_width(window.geometry);
                let local = nexxus_ui::LogicalPoint::new(
                    x as f32 / self.scale.get(),
                    y as f32 / self.scale.get(),
                );
                let button_target = TitlebarLayout::new(width, self.metrics).hit_test(local);
                if matches!(button_target, HitTarget::Button(_)) {
                    return button_target;
                }
                let physical_width = (width * self.scale.get()).round() as i32;
                if y < grab {
                    if x < grab {
                        HitTarget::Resize(ResizeEdge::TopLeft)
                    } else if x >= physical_width.saturating_sub(grab) {
                        HitTarget::Resize(ResizeEdge::TopRight)
                    } else {
                        HitTarget::Resize(ResizeEdge::Top)
                    }
                } else {
                    button_target
                }
            }
            SurfaceKind::LeftBorder | SurfaceKind::LeftGrab => HitTarget::Resize(ResizeEdge::Left),
            SurfaceKind::RightBorder | SurfaceKind::RightGrab => {
                HitTarget::Resize(ResizeEdge::Right)
            }
            SurfaceKind::BottomBorder => HitTarget::Resize(ResizeEdge::Bottom),
            SurfaceKind::BottomGrab => {
                let width = i32::try_from(window.geometry.width).unwrap_or(i32::MAX);
                if x < grab {
                    HitTarget::Resize(ResizeEdge::BottomLeft)
                } else if x >= width.saturating_sub(grab) {
                    HitTarget::Resize(ResizeEdge::BottomRight)
                } else {
                    HitTarget::Resize(ResizeEdge::Bottom)
                }
            }
        }
    }

    fn ensure_decoration(&mut self, window: &Window) -> Result<(), ChromeX11Error> {
        if self.decorations.contains_key(&window.id) {
            return Ok(());
        }
        let screen = &self.conn.setup().roots[0];
        let titlebar = self.create_surface(WindowClass::INPUT_OUTPUT, screen.black_pixel, true)?;
        let title_gc = self.conn.generate_id().map_err(operation_error)?;
        self.conn
            .create_gc(title_gc, titlebar, &CreateGCAux::new())
            .map_err(operation_error)?;
        let left_border =
            self.create_surface(WindowClass::INPUT_OUTPUT, screen.black_pixel, true)?;
        let right_border =
            self.create_surface(WindowClass::INPUT_OUTPUT, screen.black_pixel, true)?;
        let bottom_border =
            self.create_surface(WindowClass::INPUT_OUTPUT, screen.black_pixel, true)?;
        let left_grab = self.create_surface(WindowClass::INPUT_ONLY, 0, false)?;
        let right_grab = self.create_surface(WindowClass::INPUT_ONLY, 0, false)?;
        let bottom_grab = self.create_surface(WindowClass::INPUT_ONLY, 0, false)?;
        let decoration = DecorationWindows {
            titlebar,
            title_gc,
            left_border,
            right_border,
            bottom_border,
            left_grab,
            right_grab,
            bottom_grab,
        };
        for (xid, kind) in [
            (titlebar, SurfaceKind::Titlebar),
            (left_border, SurfaceKind::LeftBorder),
            (right_border, SurfaceKind::RightBorder),
            (bottom_border, SurfaceKind::BottomBorder),
            (left_grab, SurfaceKind::LeftGrab),
            (right_grab, SurfaceKind::RightGrab),
            (bottom_grab, SurfaceKind::BottomGrab),
        ] {
            self.owners.insert(
                xid,
                SurfaceOwner {
                    window: window.id,
                    kind,
                },
            );
            self.conn.map_window(xid).map_err(operation_error)?;
        }
        self.decorations.insert(window.id, decoration);
        self.hovered.insert(window.id, None);
        self.publish_frame_extents(window.id)?;
        Ok(())
    }

    fn create_surface(
        &self,
        class: WindowClass,
        background: u32,
        visual_surface: bool,
    ) -> Result<u32, ChromeX11Error> {
        let xid = self.conn.generate_id().map_err(operation_error)?;
        let mut aux = CreateWindowAux::new().override_redirect(1).event_mask(
            EventMask::BUTTON_PRESS
                | EventMask::BUTTON_RELEASE
                | EventMask::POINTER_MOTION
                | EventMask::EXPOSURE,
        );
        if visual_surface {
            aux = aux.background_pixel(background).border_pixel(background);
        }
        self.conn
            .create_window(
                if class == WindowClass::INPUT_ONLY {
                    0
                } else {
                    COPY_DEPTH_FROM_PARENT
                },
                xid,
                self.root,
                0,
                0,
                1,
                1,
                0,
                class,
                0,
                &aux,
            )
            .map_err(operation_error)?;
        Ok(xid)
    }

    fn sync_decoration(&mut self, window: &Window) -> Result<(), ChromeX11Error> {
        let decoration = self.decorations[&window.id];
        let extents = self.metrics.frame_extents(self.scale);
        let grab = (self.metrics.resize_grab * self.scale.get())
            .round()
            .max(1.0) as u32;
        let x = window.geometry.x;
        let y = window.geometry.y;
        let width = window.geometry.width;
        let height = window.geometry.height;
        let title_width = width
            .saturating_add(extents.left)
            .saturating_add(extents.right);

        self.configure(
            decoration.titlebar,
            x.saturating_sub(extents.left as i32),
            y.saturating_sub(extents.top as i32),
            title_width,
            extents.top,
        )?;
        self.configure(
            decoration.left_border,
            x.saturating_sub(extents.left as i32),
            y,
            extents.left,
            height,
        )?;
        self.configure(
            decoration.right_border,
            x.saturating_add(width as i32),
            y,
            extents.right,
            height,
        )?;
        self.configure(
            decoration.bottom_border,
            x.saturating_sub(extents.left as i32),
            y.saturating_add(height as i32),
            title_width,
            extents.bottom,
        )?;
        self.configure(
            decoration.left_grab,
            x.saturating_sub(grab as i32),
            y,
            grab,
            height.saturating_add(grab),
        )?;
        self.configure(
            decoration.right_grab,
            x.saturating_add(width as i32),
            y,
            grab,
            height.saturating_add(grab),
        )?;
        self.configure(
            decoration.bottom_grab,
            x,
            y.saturating_add(height as i32),
            width,
            grab,
        )?;
        self.redraw_with(window)?;
        Ok(())
    }

    fn configure(
        &self,
        xid: u32,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Result<(), ChromeX11Error> {
        self.conn
            .configure_window(
                xid,
                &ConfigureWindowAux::new()
                    .x(x)
                    .y(y)
                    .width(width.max(1))
                    .height(height.max(1)),
            )
            .map_err(operation_error)?;
        Ok(())
    }

    fn redraw(&mut self, id: WindowId) -> Result<(), ChromeX11Error> {
        let window = self.snapshot(id)?;
        self.redraw_with(&window)
    }

    fn redraw_with(&mut self, window: &Window) -> Result<(), ChromeX11Error> {
        let decoration = match self.decorations.get(&window.id).copied() {
            Some(value) => value,
            None => return Ok(()),
        };
        let state = ChromeVisualState {
            active: window.active,
            maximized: window.presentation == PresentationState::Maximized,
            hovered: self.hovered.get(&window.id).copied().flatten(),
            pressed: self
                .pressed
                .filter(|pressed| pressed.window == window.id)
                .map(|pressed| pressed.button),
        };
        let width = self.titlebar_logical_width(window.geometry);
        let frame = self
            .painter
            .render(width, &window.metadata.title, state, self.scale)?;
        self.upload_frame(decoration.titlebar, decoration.title_gc, &frame)?;
        Ok(())
    }

    fn upload_frame(
        &self,
        drawable: u32,
        gc: u32,
        frame: &nexxus_ui::Frame,
    ) -> Result<(), ChromeX11Error> {
        let width =
            u16::try_from(frame.size.width).map_err(|_| ChromeX11Error::DimensionOverflow)?;
        let height =
            u16::try_from(frame.size.height).map_err(|_| ChromeX11Error::DimensionOverflow)?;
        let format = self
            .conn
            .setup()
            .pixmap_formats
            .iter()
            .find(|format| format.depth == self.root_depth);
        if format.is_some_and(|format| format.bits_per_pixel == 32) {
            let mut bgrx = Vec::with_capacity(frame.pixels.len());
            for rgba in frame.pixels.chunks_exact(4) {
                bgrx.extend_from_slice(&[rgba[2], rgba[1], rgba[0], 0]);
            }
            self.conn
                .put_image(
                    ImageFormat::Z_PIXMAP,
                    drawable,
                    gc,
                    width,
                    height,
                    0,
                    0,
                    0,
                    self.root_depth,
                    &bgrx,
                )
                .map_err(operation_error)?;
        } else {
            // Servidores incomuns continuam funcionais: a superfície opaca
            // permanece utilizável, embora sem o frame rasterizado neste ciclo.
            self.conn
                .clear_area(false, drawable, 0, 0, width, height)
                .map_err(operation_error)?;
        }
        Ok(())
    }

    fn snapshot(&self, id: WindowId) -> Result<Window, ChromeX11Error> {
        self.controller
            .windows()
            .map_err(|error| ChromeX11Error::Operation(error.to_string()))?
            .into_iter()
            .find(|window| window.id == id)
            .ok_or_else(|| ChromeX11Error::Operation(format!("window {id:?} disappeared")))
    }

    fn titlebar_logical_width(&self, geometry: Geometry) -> f32 {
        let extents = self.metrics.frame_extents(self.scale);
        geometry
            .width
            .saturating_add(extents.left)
            .saturating_add(extents.right) as f32
            / self.scale.get()
    }

    fn read_decoration_hints(&self, id: WindowId) -> Result<DecorationHints, ChromeX11Error> {
        let xid = xid(id)?;
        let attrs = self
            .conn
            .get_window_attributes(xid)
            .map_err(operation_error)?
            .reply()
            .map_err(operation_error)?;
        let gtk = self
            .conn
            .get_property(
                false,
                xid,
                self.atoms.gtk_frame_extents,
                AtomEnum::ANY,
                0,
                16,
            )
            .map_err(operation_error)?
            .reply()
            .map_err(operation_error)?;
        let motif = self
            .conn
            .get_property(
                false,
                xid,
                self.atoms.motif_wm_hints,
                self.atoms.motif_wm_hints,
                0,
                5,
            )
            .map_err(operation_error)?
            .reply()
            .map_err(operation_error)?;
        let motif_decorations_disabled = motif.value32().is_some_and(|values| {
            let values: Vec<_> = values.collect();
            values.len() >= 3 && values[0] & MWM_HINTS_DECORATIONS != 0 && values[2] == 0
        });
        let types = self
            .conn
            .get_property(
                false,
                xid,
                self.atoms.net_wm_window_type,
                AtomEnum::ATOM,
                0,
                16,
            )
            .map_err(operation_error)?
            .reply()
            .map_err(operation_error)?;
        let window_type = types
            .value32()
            .and_then(|mut values| values.next())
            .map(|atom| self.atoms.classify_window_type(atom))
            .unwrap_or(WindowType::Normal);
        Ok(DecorationHints {
            override_redirect: attrs.override_redirect,
            gtk_frame_extents: gtk.type_ != 0,
            motif_decorations_disabled,
            window_type,
        })
    }

    fn publish_frame_extents(&self, id: WindowId) -> Result<(), ChromeX11Error> {
        let extents = self.metrics.frame_extents(self.scale);
        self.conn
            .change_property32(
                PropMode::REPLACE,
                xid(id)?,
                self.atoms.net_frame_extents,
                AtomEnum::CARDINAL,
                &[extents.left, extents.right, extents.top, extents.bottom],
            )
            .map_err(operation_error)?;
        Ok(())
    }

    fn delete_frame_extents(&self, id: WindowId) -> Result<(), ChromeX11Error> {
        self.conn
            .delete_property(xid(id)?, self.atoms.net_frame_extents)
            .map_err(operation_error)?;
        Ok(())
    }

    fn remove_decoration(&mut self, id: WindowId) -> Result<(), ChromeX11Error> {
        let Some(decoration) = self.decorations.remove(&id) else {
            return Ok(());
        };
        for xid in decoration.all() {
            self.owners.remove(&xid);
            self.conn.destroy_window(xid).map_err(operation_error)?;
        }
        self.conn
            .free_gc(decoration.title_gc)
            .map_err(operation_error)?;
        self.hovered.remove(&id);
        if self.pressed.is_some_and(|pressed| pressed.window == id) {
            self.pressed = None;
        }
        if self.drag.is_some_and(|drag| drag.window == id) {
            self.drag = None;
        }
        Ok(())
    }
}

impl Drop for X11ChromeAdapter {
    fn drop(&mut self) {
        let ids: Vec<_> = self.decorations.keys().copied().collect();
        for id in ids {
            let _ = self.remove_decoration(id);
        }
        let _ = self.conn.flush();
    }
}

fn intern(conn: &RustConnection, name: &[u8]) -> Result<Atom, ChromeX11Error> {
    conn.intern_atom(false, name)
        .map_err(operation_error)?
        .reply()
        .map(|reply| reply.atom)
        .map_err(operation_error)
}

fn xid(id: WindowId) -> Result<u32, ChromeX11Error> {
    u32::try_from(id.get()).map_err(|_| ChromeX11Error::InvalidWindowId(id.get()))
}

fn operation_error(error: impl std::fmt::Display) -> ChromeX11Error {
    ChromeX11Error::Operation(error.to_string())
}
