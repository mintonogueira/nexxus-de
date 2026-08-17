//! Adapter X11 da Workspace Bar.
//!
//! A superfície é `override_redirect`: não entra na lista de janelas de
//! aplicações do Backend X11 e não cria associação workspace↔monitor. RandR é
//! usado somente para posicionar a barra no monitor primário.

use std::convert::TryFrom;

use nexxus_ui::{LogicalPoint, LogicalRect, ScaleFactor, Theme};
use nexxus_workspaces::{WorkspaceError, WorkspaceEvent, WorkspaceManager};
use thiserror::Error;
use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::randr::ConnectionExt as _;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ConfigureWindowAux, ConnectionExt as _, CreateGCAux, CreateWindowAux,
    EventMask, ImageFormat, PropMode, WindowClass,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use crate::{
    AssetSource, InteractionState, MonitorGeometry, WorkspaceBarAction, WorkspaceBarLayout,
    WorkspaceBarMetrics, WorkspaceBarModel, WorkspaceBarPainter, WorkspaceBarRenderError,
    WorkspaceBarVisualState,
};

#[derive(Debug, Error)]
pub enum WorkspaceBarX11Error {
    #[error("cannot connect Workspace Bar to X11: {0}")]
    Unavailable(String),
    #[error("X11 Workspace Bar operation failed: {0}")]
    Operation(String),
    #[error("workspace topology has no usable monitor")]
    NoMonitor,
    #[error("Workspace Bar dimension exceeds X11 protocol limits")]
    DimensionOverflow,
    #[error(transparent)]
    Render(#[from] WorkspaceBarRenderError),
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
}

struct Atoms {
    net_wm_window_type: Atom,
    net_wm_window_type_dock: Atom,
}

impl Atoms {
    fn load(conn: &RustConnection) -> Result<Self, WorkspaceBarX11Error> {
        Ok(Self {
            net_wm_window_type: intern(conn, b"_NET_WM_WINDOW_TYPE")?,
            net_wm_window_type_dock: intern(conn, b"_NET_WM_WINDOW_TYPE_DOCK")?,
        })
    }
}

/// Serviço X11 autocontido da barra. A ativação de workspace continua sendo
/// executada pelo Workspace Manager; o adapter apenas traduz clique em ação.
pub struct X11WorkspaceBar {
    conn: RustConnection,
    root: u32,
    root_depth: u8,
    window: u32,
    gc: u32,
    scale: ScaleFactor,
    monitor: MonitorGeometry,
    metrics: WorkspaceBarMetrics,
    model: WorkspaceBarModel,
    layout: WorkspaceBarLayout,
    painter: WorkspaceBarPainter,
    interaction: InteractionState,
}

impl X11WorkspaceBar {
    pub fn connect(
        display: Option<&str>,
        manager: &WorkspaceManager,
        scale: ScaleFactor,
        theme: Theme,
        assets: AssetSource,
    ) -> Result<Self, WorkspaceBarX11Error> {
        let (conn, screen_num) = x11rb::connect(display)
            .map_err(|error| WorkspaceBarX11Error::Unavailable(error.to_string()))?;
        let screen = conn.setup().roots.get(screen_num).ok_or_else(|| {
            WorkspaceBarX11Error::Unavailable("selected X11 screen does not exist".into())
        })?;
        let root = screen.root;
        let root_depth = screen.root_depth;
        let black_pixel = screen.black_pixel;
        let fallback_width = screen.width_in_pixels;
        let fallback_height = screen.height_in_pixels;
        let monitor = discover_primary_monitor(&conn, root, fallback_width, fallback_height, scale)?;
        let metrics = WorkspaceBarMetrics::default();
        let model = WorkspaceBarModel::from_manager(manager);
        let layout = build_layout(monitor, &model, metrics)?;
        let painter = WorkspaceBarPainter::new(theme, metrics, assets);
        let window = conn.generate_id().map_err(operation_error)?;
        let gc = conn.generate_id().map_err(operation_error)?;
        let physical = scale.physical_rect(layout.window);
        let width = u16::try_from(physical.width.max(1)).map_err(|_| WorkspaceBarX11Error::DimensionOverflow)?;
        let height = u16::try_from(physical.height.max(1)).map_err(|_| WorkspaceBarX11Error::DimensionOverflow)?;
        let x = i16::try_from(physical.x).map_err(|_| WorkspaceBarX11Error::DimensionOverflow)?;
        let y = i16::try_from(physical.y).map_err(|_| WorkspaceBarX11Error::DimensionOverflow)?;
        let aux = CreateWindowAux::new()
            .override_redirect(1)
            .background_pixel(black_pixel)
            .border_pixel(black_pixel)
            .event_mask(
                EventMask::BUTTON_PRESS
                    | EventMask::BUTTON_RELEASE
                    | EventMask::POINTER_MOTION
                    | EventMask::EXPOSURE,
            );
        conn.create_window(
            COPY_DEPTH_FROM_PARENT,
            window,
            root,
            x,
            y,
            width,
            height,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &aux,
        )
        .map_err(operation_error)?;
        conn.create_gc(gc, window, &CreateGCAux::new()).map_err(operation_error)?;
        let atoms = Atoms::load(&conn)?;
        conn.change_property32(
            PropMode::REPLACE,
            window,
            atoms.net_wm_window_type,
            AtomEnum::ATOM,
            &[atoms.net_wm_window_type_dock],
        )
        .map_err(operation_error)?;
        conn.map_window(window).map_err(operation_error)?;
        conn.flush().map_err(operation_error)?;

        let mut bar = Self {
            conn,
            root,
            root_depth,
            window,
            gc,
            scale,
            monitor,
            metrics,
            model,
            layout,
            painter,
            interaction: InteractionState::default(),
        };
        bar.redraw()?;
        Ok(bar)
    }

    pub fn window(&self) -> u32 {
        self.window
    }

    pub fn layout(&self) -> &WorkspaceBarLayout {
        &self.layout
    }

    /// Recria o snapshot visual quando o consumidor prefere sincronização por
    /// estado completo em vez de fan-out incremental de eventos.
    pub fn sync_from_manager(&mut self, manager: &WorkspaceManager) -> Result<(), WorkspaceBarX11Error> {
        self.model = WorkspaceBarModel::from_manager(manager);
        self.relayout_and_redraw()
    }

    /// Consome cópias de eventos já distribuídos pelo coordenador. A barra não
    /// chama `drain_events()` por conta própria para não roubar eventos de outros
    /// consumidores do Workspace Manager.
    pub fn apply_workspace_events<'a>(
        &mut self,
        events: impl IntoIterator<Item = &'a WorkspaceEvent>,
    ) -> Result<(), WorkspaceBarX11Error> {
        let mut changed = false;
        for event in events {
            changed |= self.model.apply_event(event);
        }
        if changed {
            self.relayout_and_redraw()?;
        }
        Ok(())
    }

    /// Reconsulta RandR. Pode ser chamado pelo runtime quando receber mudança de
    /// topologia; apenas o monitor primário influencia a posição da barra.
    pub fn refresh_monitor_topology(&mut self) -> Result<(), WorkspaceBarX11Error> {
        let screen = &self.conn.setup().roots[0];
        self.monitor = discover_primary_monitor(
            &self.conn,
            self.root,
            screen.width_in_pixels,
            screen.height_in_pixels,
            self.scale,
        )?;
        self.relayout_and_redraw()
    }

    /// Processa input não bloqueante e devolve ações sem iniciar Settings dentro
    /// desta etapa. Expose redesenha imediatamente, sem animação.
    pub fn poll_actions(&mut self) -> Result<Vec<WorkspaceBarAction>, WorkspaceBarX11Error> {
        let mut actions = Vec::new();
        while let Some(event) = self.conn.poll_for_event().map_err(operation_error)? {
            match event {
                Event::MotionNotify(event) if event.event == self.window => {
                    let point = self.logical_point(event.event_x, event.event_y);
                    if self.interaction.pointer_move(&self.layout, point) {
                        self.redraw()?;
                    }
                }
                Event::ButtonPress(event) if event.event == self.window && event.detail == 1 => {
                    let point = self.logical_point(event.event_x, event.event_y);
                    if self.interaction.pointer_press(&self.layout, point) {
                        self.redraw()?;
                    }
                }
                Event::ButtonRelease(event) if event.event == self.window && event.detail == 1 => {
                    let point = self.logical_point(event.event_x, event.event_y);
                    if let Some(action) = self.interaction.pointer_release(&self.layout, point) {
                        actions.push(action);
                    }
                    self.redraw()?;
                }
                Event::Expose(event) if event.window == self.window => self.redraw()?,
                _ => {}
            }
        }
        self.conn.flush().map_err(operation_error)?;
        Ok(actions)
    }

    /// Executa apenas a parte pertencente à Workspace Bar. O pedido de Settings
    /// é devolvido ao coordenador porque o módulo de configurações é outra etapa.
    pub fn dispatch_action(
        &mut self,
        manager: &mut WorkspaceManager,
        action: WorkspaceBarAction,
    ) -> Result<Option<WorkspaceBarAction>, WorkspaceBarX11Error> {
        match action {
            WorkspaceBarAction::Activate(id) => {
                let previous = manager.active_id();
                manager.activate(id)?;
                if previous != id {
                    self.model.apply_event(&WorkspaceEvent::Activated { previous, current: id });
                    self.redraw()?;
                }
                Ok(None)
            }
            WorkspaceBarAction::OpenWorkspaceSettings => Ok(Some(action)),
        }
    }

    fn logical_point(&self, x: i16, y: i16) -> LogicalPoint {
        LogicalPoint::new(x as f32 / self.scale.get(), y as f32 / self.scale.get())
    }

    fn relayout_and_redraw(&mut self) -> Result<(), WorkspaceBarX11Error> {
        self.layout = build_layout(self.monitor, &self.model, self.metrics)?;
        let physical = self.scale.physical_rect(self.layout.window);
        self.conn
            .configure_window(
                self.window,
                &ConfigureWindowAux::new()
                    .x(physical.x)
                    .y(physical.y)
                    .width(physical.width.max(1))
                    .height(physical.height.max(1)),
            )
            .map_err(operation_error)?;
        self.redraw()
    }

    fn redraw(&mut self) -> Result<(), WorkspaceBarX11Error> {
        let frame = self.painter.render(
            &self.model,
            &self.layout,
            WorkspaceBarVisualState { interaction: self.interaction },
            self.scale,
        )?;
        upload_frame(&self.conn, self.root_depth, self.window, self.gc, &frame)?;
        Ok(())
    }
}

impl Drop for X11WorkspaceBar {
    fn drop(&mut self) {
        let _ = self.conn.destroy_window(self.window);
        let _ = self.conn.free_gc(self.gc);
        let _ = self.conn.flush();
    }
}

fn build_layout(
    monitor: MonitorGeometry,
    model: &WorkspaceBarModel,
    metrics: WorkspaceBarMetrics,
) -> Result<WorkspaceBarLayout, WorkspaceBarX11Error> {
    let labels: Vec<_> = model.entries().iter().map(|entry| (entry.id, entry.name.as_str())).collect();
    WorkspaceBarLayout::build(&[monitor], &labels, metrics).ok_or(WorkspaceBarX11Error::NoMonitor)
}

fn discover_primary_monitor(
    conn: &RustConnection,
    root: u32,
    fallback_width: u16,
    fallback_height: u16,
    scale: ScaleFactor,
) -> Result<MonitorGeometry, WorkspaceBarX11Error> {
    let reply = conn.randr_get_monitors(root, true).map_err(operation_error)?.reply();
    if let Ok(reply) = reply {
        if let Some(monitor) = reply.monitors.iter().find(|monitor| monitor.primary).or_else(|| reply.monitors.first()) {
            return Ok(MonitorGeometry {
                rect: LogicalRect::new(
                    monitor.x as f32 / scale.get(),
                    monitor.y as f32 / scale.get(),
                    monitor.width as f32 / scale.get(),
                    monitor.height as f32 / scale.get(),
                ),
                scale,
                primary: true,
            });
        }
    }
    Ok(MonitorGeometry {
        rect: LogicalRect::new(
            0.0,
            0.0,
            fallback_width as f32 / scale.get(),
            fallback_height as f32 / scale.get(),
        ),
        scale,
        primary: true,
    })
}

fn upload_frame(
    conn: &RustConnection,
    root_depth: u8,
    drawable: u32,
    gc: u32,
    frame: &nexxus_ui::Frame,
) -> Result<(), WorkspaceBarX11Error> {
    let width = u16::try_from(frame.size.width).map_err(|_| WorkspaceBarX11Error::DimensionOverflow)?;
    let height = u16::try_from(frame.size.height).map_err(|_| WorkspaceBarX11Error::DimensionOverflow)?;
    let format = conn.setup().pixmap_formats.iter().find(|format| format.depth == root_depth);
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
        conn.clear_area(false, drawable, 0, 0, width, height).map_err(operation_error)?;
    }
    Ok(())
}

fn intern(conn: &RustConnection, name: &[u8]) -> Result<Atom, WorkspaceBarX11Error> {
    conn.intern_atom(false, name)
        .map_err(operation_error)?
        .reply()
        .map(|reply| reply.atom)
        .map_err(operation_error)
}

fn operation_error(error: impl std::fmt::Display) -> WorkspaceBarX11Error {
    WorkspaceBarX11Error::Operation(error.to_string())
}
