//! Window Chrome próprio do Nexxus para SSD X11 inicial.
//!
//! O módulo concentra política CSD/SSD, geometria/hit-testing, pintura pela
//! Nexxus UI, consumo dos Visual Assets e adapter X11. Wayland decorations,
//! Settings e minimizar globalmente permanecem fora desta etapa.

#![forbid(unsafe_code)]

pub mod geometry;
pub mod integration;
pub mod policy;
pub mod render;
pub mod x11;

pub use geometry::{
    ChromeButton, ChromeMetrics, FrameExtents, HitTarget, ResizeEdge, TitlebarLayout,
    resized_geometry,
};
pub use integration::{
    ChromeIntegrationError, X11CommandSink, close_window, maximize_restore,
    release_for_manual_operation, tile_fit,
};
pub use policy::{DecorationDecision, DecorationHints, WindowType, decide_decoration};
pub use render::{AssetSource, ChromePainter, ChromeRenderError, ChromeVisualState};
pub use x11::{ChromeX11Error, X11ChromeAdapter};

use nexxus_wm::WindowId;

/// Hooks cuja implementação pertence ao host da sessão. O adapter X11 chama
/// estes pontos sem duplicar a inteligência do Tiling Engine.
pub trait ChromeHooks {
    /// Encaixa a janela através do contrato real da Etapa 06.
    fn tile_fit(&mut self, window: WindowId) -> Result<(), String>;

    /// Libera tiled antes de move/resize manual, preservando a liberdade floating.
    fn release_for_manual_operation(&mut self, window: WindowId) -> Result<(), String>;
}

/// Hook neutro útil para hosts que ainda não conectaram o Tiling Engine e para
/// testes de operações que não dependem de tiling.
#[derive(Default)]
pub struct NoopChromeHooks;

impl ChromeHooks for NoopChromeHooks {
    fn tile_fit(&mut self, _window: WindowId) -> Result<(), String> {
        Err("tile-fit hook is not connected".into())
    }

    fn release_for_manual_operation(&mut self, _window: WindowId) -> Result<(), String> {
        Ok(())
    }
}
