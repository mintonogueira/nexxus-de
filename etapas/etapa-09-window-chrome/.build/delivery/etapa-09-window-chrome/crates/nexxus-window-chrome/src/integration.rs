//! Contratos de integração com WM Core, Tiling Engine e Backend X11 existentes.

use nexxus_backend_api::BackendError;
use nexxus_backend_x11::X11Controller;
use nexxus_tiling::{OutputArea, TilePlan, TilingEngine, TilingError, UntilePlan};
use nexxus_wm::{BackendCommandSink, PresentationState, WindowId, WindowManager, WmCommand};
use nexxus_workspaces::WorkspaceManager;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChromeIntegrationError {
    #[error("X11 window operation failed: {0}")]
    X11(String),
    #[error(transparent)]
    Tiling(#[from] TilingError),
    #[error("window '{0:?}' disappeared while routing chrome action")]
    UnknownWindow(WindowId),
}

/// Adapta o controller concreto da Etapa 04 ao `BackendCommandSink` definido
/// pela Etapa 02, sem alterar o backend anterior nem vazar X11 para o tiling.
pub struct X11CommandSink {
    controller: X11Controller,
}

impl X11CommandSink {
    pub fn new(controller: X11Controller) -> Self {
        Self { controller }
    }
}

impl BackendCommandSink for X11CommandSink {
    fn submit(&mut self, command: &WmCommand) -> Result<(), BackendError> {
        let result = match *command {
            WmCommand::RequestFocus { window } => self.controller.focus(window),
            WmCommand::RequestMove { window, x, y } => self.controller.move_window(window, x, y),
            WmCommand::RequestResize {
                window,
                width,
                height,
            } => self.controller.resize_window(window, width, height),
            WmCommand::RequestMaximize { window } => self.controller.maximize(window),
            WmCommand::RequestRestore { window } => self.controller.restore(window),
            WmCommand::RequestFullscreen { window, enabled } => {
                self.controller.fullscreen(window, enabled)
            }
            WmCommand::RequestClose { window } => self.controller.close(window),
        };
        result.map_err(|error| BackendError::Operation(error.to_string()))
    }
}

/// Integra o botão tile-fit ao motor real da Etapa 06. O caller continua dono
/// das instâncias canônicas de WM/workspaces/tiling; o chrome não cria cópias.
pub fn tile_fit<S: BackendCommandSink>(
    engine: &mut TilingEngine,
    workspaces: &WorkspaceManager,
    wm: &mut WindowManager,
    sink: &mut S,
    area: OutputArea,
    window: WindowId,
) -> Result<TilePlan, ChromeIntegrationError> {
    let plan = engine.tile_fit_active(workspaces, wm, area, window)?;
    engine.dispatch_tile_plan(wm, sink, &plan)?;
    Ok(plan)
}

/// Antes de move/resize manual, libera uma janela tiled para preservar a regra
/// de que o tiling é ferramenta pontual e nunca uma trava.
pub fn release_for_manual_operation<S: BackendCommandSink>(
    engine: &mut TilingEngine,
    wm: &mut WindowManager,
    sink: &mut S,
    window: WindowId,
) -> Result<Option<UntilePlan>, ChromeIntegrationError> {
    let plan = engine.release_for_manual_operation(wm, window)?;
    if let Some(plan) = plan.as_ref() {
        engine.dispatch_untile_plan(wm, sink, plan)?;
    }
    Ok(plan)
}

/// Maximizar/restaurar e fechar usam diretamente a inteligência já entregue
/// pelo WM Core através do controller concreto da Etapa 04.
pub fn maximize_restore(
    controller: &X11Controller,
    window: WindowId,
) -> Result<(), ChromeIntegrationError> {
    let current = controller
        .windows()
        .map_err(|error| ChromeIntegrationError::X11(error.to_string()))?
        .into_iter()
        .find(|candidate| candidate.id == window)
        .ok_or(ChromeIntegrationError::UnknownWindow(window))?;
    let result = if current.presentation == PresentationState::Maximized {
        controller.restore(window)
    } else {
        controller.maximize(window)
    };
    result.map_err(|error| ChromeIntegrationError::X11(error.to_string()))
}

pub fn close_window(
    controller: &X11Controller,
    window: WindowId,
) -> Result<(), ChromeIntegrationError> {
    controller
        .close(window)
        .map_err(|error| ChromeIntegrationError::X11(error.to_string()))
}
