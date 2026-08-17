//! Initial X11 presentation adapter for the Desktop Shell.
//!
//! A single screen-sized `override_redirect` surface is marked as
//! `_NET_WM_WINDOW_TYPE_DESKTOP` and kept below normal application windows. One
//! surface spans the X11 screen; RandR monitor rectangles are used only for
//! per-monitor menu placement, avoiding duplicated primary menus.

use std::convert::TryFrom;
use std::path::PathBuf;

use nexxus_shortcuts::{CommandTarget, ShellAction};
use nexxus_ui::{LogicalPoint, LogicalSize, ScaleFactor, Theme};
use nexxus_xdg_application_index::ApplicationIndexConfig;
use thiserror::Error;
use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::randr::ConnectionExt as _;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ConfigureWindowAux, ConnectionExt as _, CreateGCAux, CreateWindowAux,
    EventMask, ImageFormat, PropMode, StackMode, WindowClass,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use crate::config::DesktopConfigStore;
use crate::desktop_dir::resolve_desktop_dir;
use crate::menu::MenuAction;
use crate::model::{
    DesktopShellAction, DesktopShellError, DesktopShellRuntime, MonitorGeometry, RuntimeError,
};
use crate::render::{AssetSource, DesktopLayout, DesktopPainter, DesktopRenderError};

#[derive(Debug, Error)]
pub enum X11DesktopShellError {
    #[error("cannot connect Desktop Shell to X11: {0}")]
    Unavailable(String),
    #[error("X11 Desktop Shell operation failed: {0}")]
    Operation(String),
    #[error("Desktop Shell dimension exceeds X11 protocol limits")]
    DimensionOverflow,
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    Shell(#[from] DesktopShellError),
    #[error(transparent)]
    Render(#[from] DesktopRenderError),
}

struct Atoms {
    net_wm_window_type: Atom,
    net_wm_window_type_desktop: Atom,
}

impl Atoms {
    fn load(conn: &RustConnection) -> Result<Self, X11DesktopShellError> {
        Ok(Self {
            net_wm_window_type: intern(conn, b"_NET_WM_WINDOW_TYPE")?,
            net_wm_window_type_desktop: intern(conn, b"_NET_WM_WINDOW_TYPE_DESKTOP")?,
        })
    }
}

/// X11 surface plus Stage 12 live-index runtime. Normal application launches
/// are returned as semantic actions so Session Runtime remains the process
/// orchestration boundary.
pub struct X11DesktopShell {
    conn: RustConnection,
    root: u32,
    root_depth: u8,
    root_size: LogicalSize,
    window: u32,
    gc: u32,
    scale: ScaleFactor,
    runtime: DesktopShellRuntime,
    painter: DesktopPainter,
    layout: DesktopLayout,
    last_pointer: Option<LogicalPoint>,
}

impl X11DesktopShell {
    pub fn connect(
        display: Option<&str>,
        store: DesktopConfigStore,
        index_config: ApplicationIndexConfig,
        desktop_dir: PathBuf,
        scale: ScaleFactor,
        theme: Theme,
        assets: AssetSource,
    ) -> Result<Self, X11DesktopShellError> {
        let (conn, screen_num) = x11rb::connect(display)
            .map_err(|error| X11DesktopShellError::Unavailable(error.to_string()))?;
        let screen = conn.setup().roots.get(screen_num).ok_or_else(|| {
            X11DesktopShellError::Unavailable("selected X11 screen does not exist".into())
        })?;
        let root = screen.root;
        let root_depth = screen.root_depth;
        let root_size = LogicalSize::new(
            screen.width_in_pixels as f32 / scale.get(),
            screen.height_in_pixels as f32 / scale.get(),
        );
        let monitors = discover_monitors(
            &conn,
            root,
            screen.width_in_pixels,
            screen.height_in_pixels,
            scale,
        )?;
        let runtime = DesktopShellRuntime::start(store, index_config, monitors, desktop_dir)?;
        let painter = DesktopPainter::new(theme, assets)?;
        let window = conn.generate_id().map_err(operation_error)?;
        let gc = conn.generate_id().map_err(operation_error)?;
        let width = screen.width_in_pixels.max(1);
        let height = screen.height_in_pixels.max(1);
        let aux = CreateWindowAux::new()
            .override_redirect(1)
            .background_pixel(screen.black_pixel)
            .border_pixel(screen.black_pixel)
            .event_mask(
                EventMask::BUTTON_PRESS
                    | EventMask::BUTTON_RELEASE
                    | EventMask::POINTER_MOTION
                    | EventMask::EXPOSURE
                    | EventMask::STRUCTURE_NOTIFY,
            );
        conn.create_window(
            COPY_DEPTH_FROM_PARENT,
            window,
            root,
            0,
            0,
            width,
            height,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &aux,
        )
        .map_err(operation_error)?;
        conn.create_gc(gc, window, &CreateGCAux::new())
            .map_err(operation_error)?;
        let atoms = Atoms::load(&conn)?;
        conn.change_property32(
            PropMode::REPLACE,
            window,
            atoms.net_wm_window_type,
            AtomEnum::ATOM,
            &[atoms.net_wm_window_type_desktop],
        )
        .map_err(operation_error)?;
        conn.map_window(window).map_err(operation_error)?;
        conn.configure_window(
            window,
            &ConfigureWindowAux::new().stack_mode(StackMode::BELOW),
        )
        .map_err(operation_error)?;
        conn.flush().map_err(operation_error)?;

        let mut shell = Self {
            conn,
            root,
            root_depth,
            root_size,
            window,
            gc,
            scale,
            runtime,
            painter,
            layout: DesktopLayout::default(),
            last_pointer: None,
        };
        shell.redraw()?;
        Ok(shell)
    }

    pub fn connect_from_environment(
        display: Option<&str>,
        scale: ScaleFactor,
        theme: Theme,
        assets: AssetSource,
    ) -> Result<Self, X11DesktopShellError> {
        Self::connect(
            display,
            DesktopConfigStore::from_environment()?,
            ApplicationIndexConfig::from_environment().map_err(|error| {
                X11DesktopShellError::Unavailable(format!("XDG application index config: {error}"))
            })?,
            resolve_desktop_dir()?,
            scale,
            theme,
            assets,
        )
    }

    pub fn window(&self) -> u32 {
        self.window
    }

    pub fn layout(&self) -> &DesktopLayout {
        &self.layout
    }

    pub fn runtime(&self) -> &DesktopShellRuntime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut DesktopShellRuntime {
        &mut self.runtime
    }

    /// Processes live index changes and X11 mouse/expose events without
    /// blocking. Semantic requests for later modules are returned to the host.
    pub fn poll_actions(&mut self) -> Result<Vec<DesktopShellAction>, X11DesktopShellError> {
        let index_events = self.runtime.poll_index_updates();
        if index_events
            .iter()
            .any(|event| matches!(event, nexxus_xdg_application_index::ApplicationIndexEvent::Changed(_)))
        {
            self.redraw()?;
        }

        let mut actions = Vec::new();
        while let Some(event) = self.conn.poll_for_event().map_err(operation_error)? {
            match event {
                Event::MotionNotify(event) if event.event == self.window => {
                    self.last_pointer = Some(self.logical_point(event.event_x, event.event_y));
                }
                Event::ButtonPress(event) if event.event == self.window && event.detail == 3 => {
                    let point = self.logical_point(event.event_x, event.event_y);
                    self.last_pointer = Some(point);
                    self.runtime.shell_mut().open_context_menu(point)?;
                    self.redraw()?;
                }
                Event::ButtonRelease(event) if event.event == self.window && event.detail == 1 => {
                    let point = self.logical_point(event.event_x, event.event_y);
                    self.last_pointer = Some(point);
                    if let Some(action) = self.activate_at(point)? {
                        actions.push(action);
                    }
                    self.redraw()?;
                }
                Event::Expose(event) if event.window == self.window => self.redraw()?,
                Event::ConfigureNotify(event) if event.window == self.window => {
                    let next = LogicalSize::new(
                        event.width as f32 / self.scale.get(),
                        event.height as f32 / self.scale.get(),
                    );
                    if next != self.root_size {
                        self.root_size = next;
                        self.refresh_monitor_topology()?;
                    }
                }
                _ => {}
            }
        }
        self.conn.flush().map_err(operation_error)?;
        Ok(actions)
    }

    /// Routes only the Desktop Menu target from Stage 10. Other targets remain
    /// owned by their respective modules.
    pub fn handle_shortcut_target(
        &mut self,
        target: CommandTarget,
    ) -> Result<bool, X11DesktopShellError> {
        if target != CommandTarget::Shell(ShellAction::DesktopMenu) {
            return Ok(false);
        }
        if let Some(point) = self.last_pointer {
            self.runtime.shell_mut().open_context_menu(point)?;
        } else {
            self.runtime
                .shell_mut()
                .open_context_menu_from_shortcut()?;
        }
        self.redraw()?;
        Ok(true)
    }

    pub fn refresh_monitor_topology(&mut self) -> Result<(), X11DesktopShellError> {
        let screen = &self.conn.setup().roots[0];
        let monitors = discover_monitors(
            &self.conn,
            self.root,
            screen.width_in_pixels,
            screen.height_in_pixels,
            self.scale,
        )?;
        self.runtime.shell_mut().replace_monitors(monitors)?;
        self.redraw()
    }

    fn activate_at(
        &mut self,
        point: LogicalPoint,
    ) -> Result<Option<DesktopShellAction>, X11DesktopShellError> {
        if let Some(hit) = self
            .layout
            .menu_entries
            .iter()
            .find(|hit| hit.rect.contains(point))
            .cloned()
        {
            return self
                .runtime
                .shell_mut()
                .activate_menu_action(hit.action)
                .map_err(Into::into);
        }
        if self.runtime.shell().menu().is_some() {
            self.runtime.shell_mut().close_menu();
            return Ok(None);
        }
        if let Some(hit) = self
            .layout
            .launchers
            .iter()
            .find(|hit| hit.rect.contains(point))
        {
            return self
                .runtime
                .shell()
                .launch_action(&hit.desktop_id)
                .map(Some)
                .map_err(Into::into);
        }
        if let Some(hit) = self.layout.folders.iter().find(|hit| hit.rect.contains(point)) {
            return Ok(Some(DesktopShellAction::OpenFileManager {
                path: hit.path.clone(),
            }));
        }
        Ok(None)
    }

    fn logical_point(&self, x: i16, y: i16) -> LogicalPoint {
        LogicalPoint::new(x as f32 / self.scale.get(), y as f32 / self.scale.get())
    }

    fn redraw(&mut self) -> Result<(), X11DesktopShellError> {
        let (frame, layout) = self.painter.render(
            self.runtime.shell(),
            self.root_size,
            self.scale,
        )?;
        upload_frame(&self.conn, self.root_depth, self.window, self.gc, &frame)?;
        self.layout = layout;
        Ok(())
    }
}

impl Drop for X11DesktopShell {
    fn drop(&mut self) {
        let _ = self.conn.destroy_window(self.window);
        let _ = self.conn.free_gc(self.gc);
        let _ = self.conn.flush();
    }
}

fn discover_monitors(
    conn: &RustConnection,
    root: u32,
    fallback_width: u16,
    fallback_height: u16,
    scale: ScaleFactor,
) -> Result<Vec<MonitorGeometry>, X11DesktopShellError> {
    if let Ok(reply) = conn
        .randr_get_monitors(root, true)
        .map_err(operation_error)?
        .reply()
    {
        if !reply.monitors.is_empty() {
            return Ok(reply
                .monitors
                .iter()
                .map(|monitor| MonitorGeometry {
                    rect: nexxus_ui::LogicalRect::new(
                        monitor.x as f32 / scale.get(),
                        monitor.y as f32 / scale.get(),
                        monitor.width as f32 / scale.get(),
                        monitor.height as f32 / scale.get(),
                    ),
                    scale,
                    primary: monitor.primary,
                })
                .collect());
        }
    }
    Ok(vec![MonitorGeometry {
        rect: nexxus_ui::LogicalRect::new(
            0.0,
            0.0,
            fallback_width as f32 / scale.get(),
            fallback_height as f32 / scale.get(),
        ),
        scale,
        primary: true,
    }])
}

fn upload_frame(
    conn: &RustConnection,
    root_depth: u8,
    drawable: u32,
    gc: u32,
    frame: &nexxus_ui::Frame,
) -> Result<(), X11DesktopShellError> {
    let width = u16::try_from(frame.size.width)
        .map_err(|_| X11DesktopShellError::DimensionOverflow)?;
    let height = u16::try_from(frame.size.height)
        .map_err(|_| X11DesktopShellError::DimensionOverflow)?;
    let format = conn
        .setup()
        .pixmap_formats
        .iter()
        .find(|format| format.depth == root_depth);
    if format.is_some_and(|format| format.bits_per_pixel == 32) {
        let mut bgrx = Vec::with_capacity(frame.pixels.len());
        for rgba in frame.pixels.chunks_exact(4) {
            bgrx.extend_from_slice(&[rgba[2], rgba[1], rgba[0], 0]);
        }
        conn.put_image(
            ImageFormat::Z_PIXMAP,
            drawable,
            gc,
            width,
            height,
            0,
            0,
            0,
            root_depth,
            &bgrx,
        )
        .map_err(operation_error)?;
    } else {
        conn.clear_area(false, drawable, 0, 0, width, height)
            .map_err(operation_error)?;
    }
    Ok(())
}

fn intern(conn: &RustConnection, name: &[u8]) -> Result<Atom, X11DesktopShellError> {
    conn.intern_atom(false, name)
        .map_err(operation_error)?
        .reply()
        .map(|reply| reply.atom)
        .map_err(operation_error)
}

fn operation_error(error: impl std::fmt::Display) -> X11DesktopShellError {
    X11DesktopShellError::Operation(error.to_string())
}
