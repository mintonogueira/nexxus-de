//! Composição visual da titlebar usando exclusivamente Nexxus UI + Visual Assets.

use std::fs;
use std::path::{Path, PathBuf};

use nexxus_assets::{SYSTEM_ASSET_ROOT, icon, recolor_symbolic_svg};
use nexxus_ui::{
    Color, DisplayList, DrawCommand, LogicalRect, LogicalSize, RenderError, Renderer, ScaleFactor,
    SoftwareRenderer, TextStyle, Theme,
};
use thiserror::Error;

use crate::geometry::{ChromeButton, ChromeMetrics, TitlebarLayout};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ChromeVisualState {
    pub active: bool,
    pub maximized: bool,
    pub hovered: Option<ChromeButton>,
    pub pressed: Option<ChromeButton>,
}

/// Origem configurável dos assets. Runtime usa `/usr/share/nexxus/assets`;
/// testes podem apontar para a árvore versionada da Etapa 08.
#[derive(Clone, Debug)]
pub struct AssetSource {
    root: PathBuf,
}

impl Default for AssetSource {
    fn default() -> Self {
        Self::new(SYSTEM_ASSET_ROOT)
    }
}

impl AssetSource {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn symbolic(&self, name: &str, color: Color) -> Result<Vec<u8>, ChromeRenderError> {
        let spec = icon(name).ok_or_else(|| ChromeRenderError::MissingIcon(name.to_owned()))?;
        let path = self.root.join(spec.relative_path);
        let bytes = fs::read(&path).map_err(|source| ChromeRenderError::ReadAsset {
            path: path.clone(),
            source,
        })?;
        if !spec.tintable {
            return Ok(bytes);
        }
        recolor_symbolic_svg(&bytes, [color.r, color.g, color.b])
            .map_err(|error| ChromeRenderError::Recolor(error.to_string()))
    }
}

#[derive(Debug, Error)]
pub enum ChromeRenderError {
    #[error("required Nexxus icon is missing from the Etapa 08 catalog: {0}")]
    MissingIcon(String),
    #[error("cannot read asset {path}: {source}")]
    ReadAsset {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot recolor symbolic icon: {0}")]
    Recolor(String),
    #[error(transparent)]
    Render(#[from] RenderError),
}

/// Painter stateless: cada frame nasce do estado atual, sem transições ou
/// animações, conforme a identidade visual normativa do Nexxus.
pub struct ChromePainter {
    theme: Theme,
    metrics: ChromeMetrics,
    assets: AssetSource,
    renderer: SoftwareRenderer,
}

impl ChromePainter {
    pub fn new(theme: Theme, metrics: ChromeMetrics, assets: AssetSource) -> Self {
        Self {
            theme,
            metrics,
            assets,
            renderer: SoftwareRenderer::new(),
        }
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn display_list(
        &self,
        width: f32,
        title: &str,
        state: ChromeVisualState,
    ) -> Result<DisplayList, ChromeRenderError> {
        let layout = TitlebarLayout::new(width, self.metrics);
        let palette = self.theme.palette;
        let mut list = DisplayList::new();
        let surface = if state.active {
            palette.surface_alt
        } else {
            palette.surface
        };
        list.push(DrawCommand::Clear(surface));
        list.push(DrawCommand::StrokeRect {
            rect: layout.bounds,
            color: if state.active {
                palette.accent
            } else {
                palette.border
            },
            width: self.metrics.border_width,
        });

        let text_style = TextStyle::new(
            self.theme.typography.family.clone(),
            self.theme.typography.body_size,
            self.theme.typography.line_height,
            if state.active {
                palette.text
            } else {
                palette.text_muted
            },
        );
        list.push(DrawCommand::Text {
            rect: layout.title,
            text: title.to_owned(),
            style: text_style,
        });

        self.push_button(
            &mut list,
            layout.tile_fit,
            ChromeButton::TileFit,
            "window-tile",
            state,
        )?;
        let maximize_icon = if state.maximized {
            "window-restore"
        } else {
            "window-maximize"
        };
        self.push_button(
            &mut list,
            layout.maximize_restore,
            ChromeButton::MaximizeRestore,
            maximize_icon,
            state,
        )?;
        self.push_button(
            &mut list,
            layout.close,
            ChromeButton::Close,
            "window-close",
            state,
        )?;
        Ok(list)
    }

    pub fn render(
        &mut self,
        width: f32,
        title: &str,
        state: ChromeVisualState,
        scale: ScaleFactor,
    ) -> Result<nexxus_ui::Frame, ChromeRenderError> {
        let list = self.display_list(width, title, state)?;
        Ok(self.renderer.render(
            &list,
            LogicalSize::new(width.max(1.0), self.metrics.titlebar_height),
            scale,
        )?)
    }

    fn push_button(
        &self,
        list: &mut DisplayList,
        rect: LogicalRect,
        button: ChromeButton,
        icon_name: &str,
        state: ChromeVisualState,
    ) -> Result<(), ChromeRenderError> {
        let palette = self.theme.palette;
        let background = if state.pressed == Some(button) {
            if button == ChromeButton::Close {
                palette.danger
            } else {
                palette.selection
            }
        } else if state.hovered == Some(button) {
            if button == ChromeButton::Close {
                palette.danger
            } else {
                palette.surface_alt
            }
        } else if state.active {
            palette.surface_alt
        } else {
            palette.surface
        };
        list.push(DrawCommand::FillRect {
            rect,
            color: background,
        });

        let color = if button == ChromeButton::Close && state.hovered == Some(button) {
            Color::rgb(255, 255, 255)
        } else if state.active {
            palette.text
        } else {
            palette.text_muted
        };
        let svg = self.assets.symbolic(icon_name, color)?;
        let icon_size = self.metrics.icon_size.min(rect.width).min(rect.height);
        let icon_rect = LogicalRect::new(
            rect.x + (rect.width - icon_size) * 0.5,
            rect.y + (rect.height - icon_size) * 0.5,
            icon_size,
            icon_size,
        );
        list.push(DrawCommand::Svg {
            rect: icon_rect,
            bytes: svg,
        });
        Ok(())
    }
}
