//! Geometria lógica do chrome e hit targets escaláveis para mouse.

use nexxus_ui::{LogicalPoint, LogicalRect, ScaleFactor};
use nexxus_wm::Geometry;

/// Métricas próprias da decoração. Valores são unidades lógicas; a conversão
/// para pixel físico ocorre somente na borda X11.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChromeMetrics {
    pub titlebar_height: f32,
    pub border_width: f32,
    pub resize_grab: f32,
    pub button_width: f32,
    pub icon_size: f32,
    pub title_padding: f32,
}

impl Default for ChromeMetrics {
    fn default() -> Self {
        Self {
            titlebar_height: 32.0,
            border_width: 2.0,
            resize_grab: 8.0,
            button_width: 36.0,
            icon_size: 16.0,
            title_padding: 10.0,
        }
    }
}

/// Extensões físicas publicadas em `_NET_FRAME_EXTENTS`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameExtents {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

impl ChromeMetrics {
    /// Converte as dimensões lógicas para pixels sem permitir hit target zero.
    pub fn frame_extents(self, scale: ScaleFactor) -> FrameExtents {
        let physical = |logical: f32| -> u32 {
            (logical.max(1.0) * scale.get()).round().max(1.0) as u32
        };
        FrameExtents {
            left: physical(self.border_width),
            right: physical(self.border_width),
            top: physical(self.titlebar_height),
            bottom: physical(self.border_width),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChromeButton {
    TileFit,
    MaximizeRestore,
    Close,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResizeEdge {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HitTarget {
    Titlebar,
    Button(ChromeButton),
    Resize(ResizeEdge),
    None,
}

/// Layout lógico da barra de título. As bordas físicas são criadas pelo adapter
/// X11, enquanto esta estrutura concentra pintura e hit-testing da titlebar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TitlebarLayout {
    pub bounds: LogicalRect,
    pub tile_fit: LogicalRect,
    pub maximize_restore: LogicalRect,
    pub close: LogicalRect,
    pub title: LogicalRect,
}

impl TitlebarLayout {
    pub fn new(width: f32, metrics: ChromeMetrics) -> Self {
        let height = metrics.titlebar_height;
        let button = metrics.button_width;
        let close = LogicalRect::new((width - button).max(0.0), 0.0, button.min(width), height);
        let maximize_restore = LogicalRect::new((width - button * 2.0).max(0.0), 0.0, button.min(width), height);
        let tile_fit = LogicalRect::new((width - button * 3.0).max(0.0), 0.0, button.min(width), height);
        let title_right = (width - button * 3.0 - metrics.title_padding).max(metrics.title_padding);
        let title = LogicalRect::new(
            metrics.title_padding,
            0.0,
            (title_right - metrics.title_padding).max(0.0),
            height,
        );
        Self {
            bounds: LogicalRect::new(0.0, 0.0, width.max(0.0), height),
            tile_fit,
            maximize_restore,
            close,
            title,
        }
    }

    /// Botões possuem precedência sobre a zona de arraste da titlebar.
    pub fn hit_test(self, point: LogicalPoint) -> HitTarget {
        if self.close.contains(point) {
            HitTarget::Button(ChromeButton::Close)
        } else if self.maximize_restore.contains(point) {
            HitTarget::Button(ChromeButton::MaximizeRestore)
        } else if self.tile_fit.contains(point) {
            HitTarget::Button(ChromeButton::TileFit)
        } else if self.bounds.contains(point) {
            HitTarget::Titlebar
        } else {
            HitTarget::None
        }
    }
}

/// Calcula um resize mantendo a borda oposta fixa e respeitando os mínimos.
/// As constraints finais continuam sendo aplicadas pelo WM Core/backend.
pub fn resized_geometry(
    initial: Geometry,
    edge: ResizeEdge,
    dx: i32,
    dy: i32,
    min_width: u32,
    min_height: u32,
) -> Geometry {
    let left = i64::from(initial.x);
    let top = i64::from(initial.y);
    let right = left + i64::from(initial.width);
    let bottom = top + i64::from(initial.height);

    let (mut new_left, mut new_top, mut new_right, mut new_bottom) = (left, top, right, bottom);
    match edge {
        ResizeEdge::Left => new_left += i64::from(dx),
        ResizeEdge::Right => new_right += i64::from(dx),
        ResizeEdge::Top => new_top += i64::from(dy),
        ResizeEdge::Bottom => new_bottom += i64::from(dy),
        ResizeEdge::TopLeft => {
            new_left += i64::from(dx);
            new_top += i64::from(dy);
        }
        ResizeEdge::TopRight => {
            new_right += i64::from(dx);
            new_top += i64::from(dy);
        }
        ResizeEdge::BottomLeft => {
            new_left += i64::from(dx);
            new_bottom += i64::from(dy);
        }
        ResizeEdge::BottomRight => {
            new_right += i64::from(dx);
            new_bottom += i64::from(dy);
        }
    }

    if new_right - new_left < i64::from(min_width) {
        if matches!(edge, ResizeEdge::Left | ResizeEdge::TopLeft | ResizeEdge::BottomLeft) {
            new_left = new_right - i64::from(min_width);
        } else {
            new_right = new_left + i64::from(min_width);
        }
    }
    if new_bottom - new_top < i64::from(min_height) {
        if matches!(edge, ResizeEdge::Top | ResizeEdge::TopLeft | ResizeEdge::TopRight) {
            new_top = new_bottom - i64::from(min_height);
        } else {
            new_bottom = new_top + i64::from(min_height);
        }
    }

    Geometry::new(
        saturating_i32(new_left),
        saturating_i32(new_top),
        (new_right - new_left).max(1).min(i64::from(u32::MAX)) as u32,
        (new_bottom - new_top).max(1).min(i64::from(u32::MAX)) as u32,
    )
    .expect("resize helper always produces non-zero geometry")
}

fn saturating_i32(value: i64) -> i32 {
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}
