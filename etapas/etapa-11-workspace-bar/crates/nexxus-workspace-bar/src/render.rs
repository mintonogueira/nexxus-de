//! Composição opaca da Workspace Bar usando Nexxus UI e Visual Assets.

use std::fs;
use std::path::{Path, PathBuf};

use nexxus_assets::{SYSTEM_ASSET_ROOT, icon, recolor_symbolic_svg};
use nexxus_ui::{
    Color, DisplayList, DrawCommand, LogicalRect, RenderError, Renderer, ScaleFactor,
    SoftwareRenderer, TextStyle, Theme,
};
use thiserror::Error;

use crate::{
    InteractionState, WorkspaceBarLayout, WorkspaceBarMetrics, WorkspaceBarModel,
    WorkspaceBarTarget,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorkspaceBarVisualState {
    pub interaction: InteractionState,
}

/// Origem substituível dos assets para runtime e testes.
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

    fn symbolic(&self, name: &str, color: Color) -> Result<Vec<u8>, WorkspaceBarRenderError> {
        let spec =
            icon(name).ok_or_else(|| WorkspaceBarRenderError::MissingIcon(name.to_owned()))?;
        let path = self.root.join("icons").join(spec.relative_path);
        let bytes = fs::read(&path)
            .map_err(|source| WorkspaceBarRenderError::ReadAsset { path, source })?;
        if !spec.tintable {
            return Ok(bytes);
        }
        recolor_symbolic_svg(&bytes, [color.r, color.g, color.b])
            .map_err(|error| WorkspaceBarRenderError::Recolor(error.to_string()))
    }
}

#[derive(Debug, Error)]
pub enum WorkspaceBarRenderError {
    #[error("required Nexxus icon is missing: {0}")]
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

/// Painter sem animações, sombras, blur ou transparência estrutural.
pub struct WorkspaceBarPainter {
    theme: Theme,
    metrics: WorkspaceBarMetrics,
    assets: AssetSource,
    renderer: SoftwareRenderer,
}

impl WorkspaceBarPainter {
    pub fn new(theme: Theme, metrics: WorkspaceBarMetrics, assets: AssetSource) -> Self {
        Self {
            theme,
            metrics,
            assets,
            renderer: SoftwareRenderer::new(),
        }
    }

    pub fn display_list(
        &self,
        model: &WorkspaceBarModel,
        layout: &WorkspaceBarLayout,
        visual: WorkspaceBarVisualState,
    ) -> Result<DisplayList, WorkspaceBarRenderError> {
        let palette = self.theme.palette;
        let mut list = DisplayList::new();
        list.push(DrawCommand::Clear(palette.surface));
        list.push(DrawCommand::StrokeRect {
            rect: LogicalRect::new(0.0, 0.0, layout.window.width, layout.window.height),
            color: palette.border,
            width: self.metrics.border_width,
        });

        for button in &layout.workspaces {
            let Some(entry) = model.entries().iter().find(|entry| entry.id == button.id) else {
                continue;
            };
            let target = WorkspaceBarTarget::Workspace(entry.id);
            let hovered = visual.interaction.hovered == Some(target);
            let pressed = visual.interaction.pressed == Some(target);
            let background = if entry.active {
                palette.accent
            } else if pressed {
                palette.selection
            } else if hovered {
                palette.surface_alt
            } else {
                palette.surface
            };
            list.push(DrawCommand::FillRect {
                rect: button.rect,
                color: background,
            });
            list.push(DrawCommand::StrokeRect {
                rect: button.rect,
                color: if entry.active {
                    palette.accent
                } else {
                    palette.border
                },
                width: self.metrics.border_width,
            });
            let text_color = if entry.active {
                palette.accent_text
            } else {
                palette.text
            };
            let style = TextStyle::new(
                self.theme.typography.family.clone(),
                self.theme.typography.body_size,
                self.theme.typography.body_size * self.theme.typography.line_height,
                text_color,
            );
            list.push(DrawCommand::PushClip(button.rect));
            let text_rect = LogicalRect::new(
                button.rect.x + self.metrics.padding * 2.0,
                button.rect.y,
                (button.rect.width - self.metrics.padding * 4.0).max(0.0),
                button.rect.height,
            );
            list.push(DrawCommand::Text {
                rect: text_rect,
                text: entry.name.clone(),
                style,
            });
            list.push(DrawCommand::PopClip);
        }

        let settings_target = WorkspaceBarTarget::Settings;
        let settings_background = if visual.interaction.pressed == Some(settings_target) {
            palette.selection
        } else if visual.interaction.hovered == Some(settings_target) {
            palette.surface_alt
        } else {
            palette.surface
        };
        list.push(DrawCommand::FillRect {
            rect: layout.settings,
            color: settings_background,
        });
        list.push(DrawCommand::StrokeRect {
            rect: layout.settings,
            color: palette.border,
            width: self.metrics.border_width,
        });
        let svg = self
            .assets
            .symbolic("preferences-workspaces", palette.text)?;
        let icon_size = self
            .metrics
            .icon_size
            .min(layout.settings.width)
            .min(layout.settings.height);
        let icon_rect = LogicalRect::new(
            layout.settings.x + (layout.settings.width - icon_size) * 0.5,
            layout.settings.y + (layout.settings.height - icon_size) * 0.5,
            icon_size,
            icon_size,
        );
        list.push(DrawCommand::Svg {
            rect: icon_rect,
            bytes: svg,
        });
        Ok(list)
    }

    pub fn render(
        &mut self,
        model: &WorkspaceBarModel,
        layout: &WorkspaceBarLayout,
        visual: WorkspaceBarVisualState,
        scale: ScaleFactor,
    ) -> Result<nexxus_ui::Frame, WorkspaceBarRenderError> {
        let list = self.display_list(model, layout, visual)?;
        Ok(self.renderer.render(&list, layout.window.size(), scale)?)
    }
}
