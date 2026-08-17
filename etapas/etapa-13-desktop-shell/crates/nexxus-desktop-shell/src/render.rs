//! Desktop layout and backend-neutral painting through `nexxus-ui`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use image::ImageError;
use nexxus_assets::{SYSTEM_ASSET_ROOT, icon as builtin_icon, wallpaper as builtin_wallpaper};
use nexxus_ui::{
    DisplayList, DrawCommand, Frame, ImageData, LogicalRect, LogicalSize, RenderError, Renderer,
    ScaleFactor, SoftwareRenderer, TextStyle, Theme,
};
use nexxus_xdg_application_index::IconReference;
use thiserror::Error;

use crate::menu::MenuAction;
use crate::model::{DesktopShell, DesktopShellError};

const LAUNCHER_WIDTH: f32 = 88.0;
const LAUNCHER_HEIGHT: f32 = 76.0;
const ICON_SIZE: f32 = 42.0;
const MENU_WIDTH: f32 = 260.0;
const MENU_ROW_HEIGHT: f32 = 30.0;

#[derive(Clone, Debug, PartialEq)]
pub struct LauncherHitBox {
    pub desktop_id: String,
    pub rect: LogicalRect,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FolderHitBox {
    pub path: PathBuf,
    pub rect: LogicalRect,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuHitBox {
    pub action: MenuAction,
    pub rect: LogicalRect,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesktopLayout {
    pub launchers: Vec<LauncherHitBox>,
    pub folders: Vec<FolderHitBox>,
    pub menu_entries: Vec<MenuHitBox>,
}

#[derive(Clone, Debug)]
pub struct AssetSource {
    root: PathBuf,
    icon_roots: Vec<PathBuf>,
}

impl AssetSource {
    pub fn system() -> Self {
        Self::new(PathBuf::from(SYSTEM_ASSET_ROOT))
    }

    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            icon_roots: xdg_icon_roots(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn builtin_wallpaper_path(&self, name: &str) -> Option<PathBuf> {
        let spec = builtin_wallpaper(name)?;
        first_existing([
            self.root.join("wallpapers").join(spec.relative_path),
            self.root.join(spec.relative_path),
        ])
    }

    fn builtin_icon_path(&self, relative_path: &str) -> Option<PathBuf> {
        first_existing([
            self.root.join("icons").join(relative_path),
            self.root.join(relative_path),
        ])
    }

    fn external_icon_path(&self, name: &str) -> Option<PathBuf> {
        let path = Path::new(name);
        if path.is_absolute() && path.is_file() {
            return Some(path.to_path_buf());
        }
        const SUBDIRS: &[&str] = &[
            "hicolor/scalable/apps",
            "hicolor/128x128/apps",
            "hicolor/64x64/apps",
            "hicolor/48x48/apps",
            "hicolor/32x32/apps",
            "hicolor/24x24/apps",
            "hicolor/16x16/apps",
        ];
        const EXTENSIONS: &[&str] = &["svg", "png", "jpg", "jpeg", "webp"];
        for root in &self.icon_roots {
            for subdir in SUBDIRS {
                for extension in EXTENSIONS {
                    let candidate = root
                        .join(subdir)
                        .join(format!("{name}.{extension}"));
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }
        }
        for extension in EXTENSIONS {
            let candidate = PathBuf::from("/usr/share/pixmaps").join(format!("{name}.{extension}"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }
}

pub struct DesktopPainter {
    theme: Theme,
    assets: AssetSource,
    renderer: SoftwareRenderer,
}

impl DesktopPainter {
    pub fn new(theme: Theme, assets: AssetSource) -> Result<Self, DesktopRenderError> {
        theme.validate()?;
        Ok(Self {
            theme,
            assets,
            renderer: SoftwareRenderer::new(),
        })
    }

    pub fn render(
        &mut self,
        shell: &DesktopShell,
        logical_size: LogicalSize,
        scale: ScaleFactor,
    ) -> Result<(Frame, DesktopLayout), DesktopRenderError> {
        let mut list = DisplayList::new();
        list.push(DrawCommand::Clear(self.theme.palette.background));
        self.paint_wallpaper(&mut list, shell, logical_size)?;

        let mut layout = DesktopLayout::default();
        self.paint_launchers(&mut list, shell, &mut layout)?;
        self.paint_folders(&mut list, shell, &mut layout)?;
        self.paint_menu(&mut list, shell, &mut layout);

        let frame = self.renderer.render(&list, logical_size, scale)?;
        Ok((frame, layout))
    }

    fn paint_wallpaper(
        &self,
        list: &mut DisplayList,
        shell: &DesktopShell,
        size: LogicalSize,
    ) -> Result<(), DesktopRenderError> {
        let requested = match &shell.config().wallpaper {
            crate::config::WallpaperSelection::Builtin { name } => {
                self.assets.builtin_wallpaper_path(name)
            }
            crate::config::WallpaperSelection::File { path } => Some(path.clone()),
        };
        let fallback = self.assets.builtin_wallpaper_path(crate::config::DEFAULT_WALLPAPER);
        if let Some(path) = requested.or(fallback) {
            if let Ok(asset) = load_graphic(&path) {
                push_graphic(
                    list,
                    LogicalRect::new(0.0, 0.0, size.width, size.height),
                    asset,
                );
            }
        }
        Ok(())
    }

    fn paint_launchers(
        &self,
        list: &mut DisplayList,
        shell: &DesktopShell,
        layout: &mut DesktopLayout,
    ) -> Result<(), DesktopRenderError> {
        for (placement, record) in shell.visible_launchers() {
            let rect = LogicalRect::new(placement.x, placement.y, LAUNCHER_WIDTH, LAUNCHER_HEIGHT);
            let icon_rect = LogicalRect::new(
                rect.x + (LAUNCHER_WIDTH - ICON_SIZE) / 2.0,
                rect.y + 2.0,
                ICON_SIZE,
                ICON_SIZE,
            );
            self.paint_icon(list, &record.icon, icon_rect)?;
            list.push(DrawCommand::Text {
                rect: LogicalRect::new(rect.x, rect.y + 48.0, rect.width, 24.0),
                text: record.name.clone(),
                style: self.desktop_label_style(),
            });
            layout.launchers.push(LauncherHitBox {
                desktop_id: record.id.as_str().to_owned(),
                rect,
            });
        }
        Ok(())
    }

    fn paint_folders(
        &self,
        list: &mut DisplayList,
        shell: &DesktopShell,
        layout: &mut DesktopLayout,
    ) -> Result<(), DesktopRenderError> {
        let existing = layout.launchers.len();
        let primary = shell
            .monitors()
            .iter()
            .find(|monitor| monitor.primary)
            .or_else(|| shell.monitors().first())
            .ok_or(DesktopShellError::NoMonitor)?;
        let rows = ((primary.rect.height - 48.0) / 88.0).floor().max(1.0) as usize;
        let folder_icon = builtin_icon("folder").expect("Stage 08 folder icon must exist");
        for (offset, path) in shell.desktop_folders()?.into_iter().enumerate() {
            let index = existing + offset;
            let column = index / rows;
            let row = index % rows;
            let rect = LogicalRect::new(
                primary.rect.x + 24.0 + column as f32 * 104.0,
                primary.rect.y + 24.0 + row as f32 * 88.0,
                LAUNCHER_WIDTH,
                LAUNCHER_HEIGHT,
            );
            if let Some(icon_path) = self.assets.builtin_icon_path(folder_icon.relative_path) {
                if let Ok(asset) = load_graphic(&icon_path) {
                    push_graphic(
                        list,
                        LogicalRect::new(
                            rect.x + (LAUNCHER_WIDTH - ICON_SIZE) / 2.0,
                            rect.y + 2.0,
                            ICON_SIZE,
                            ICON_SIZE,
                        ),
                        asset,
                    );
                }
            }
            let label = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("Folder")
                .to_owned();
            list.push(DrawCommand::Text {
                rect: LogicalRect::new(rect.x, rect.y + 48.0, rect.width, 24.0),
                text: label,
                style: self.desktop_label_style(),
            });
            layout.folders.push(FolderHitBox { path, rect });
        }
        Ok(())
    }

    fn paint_menu(&self, list: &mut DisplayList, shell: &DesktopShell, layout: &mut DesktopLayout) {
        let Some(menu) = shell.menu() else {
            return;
        };
        let entries = shell.menu_entries();
        if entries.is_empty() {
            return;
        }
        let monitor = shell.monitors()[menu.monitor_index].rect;
        let menu_height = entries.len() as f32 * MENU_ROW_HEIGHT + 2.0;
        let x = menu.anchor.x.min((monitor.x + monitor.width - MENU_WIDTH).max(monitor.x));
        let y = menu.anchor.y.min((monitor.y + monitor.height - menu_height).max(monitor.y));
        let menu_rect = LogicalRect::new(x, y, MENU_WIDTH, menu_height);
        list.push(DrawCommand::FillRect {
            rect: menu_rect,
            color: self.theme.palette.surface,
        });
        list.push(DrawCommand::StrokeRect {
            rect: menu_rect,
            color: self.theme.palette.border,
            width: self.theme.metrics.border_width,
        });
        for (index, entry) in entries.into_iter().enumerate() {
            let row = LogicalRect::new(
                x + 1.0,
                y + 1.0 + index as f32 * MENU_ROW_HEIGHT,
                MENU_WIDTH - 2.0,
                MENU_ROW_HEIGHT,
            );
            list.push(DrawCommand::Text {
                rect: LogicalRect::new(row.x + 10.0, row.y + 5.0, row.width - 20.0, row.height - 8.0),
                text: entry.label,
                style: self.menu_text_style(),
            });
            layout.menu_entries.push(MenuHitBox {
                action: entry.action,
                rect: row,
            });
        }
    }

    fn paint_icon(
        &self,
        list: &mut DisplayList,
        reference: &IconReference,
        rect: LogicalRect,
    ) -> Result<(), DesktopRenderError> {
        let path = match reference {
            IconReference::ExternalPath(path) => Some(path.clone()),
            IconReference::ExternalName(name) => self.assets.external_icon_path(name),
            IconReference::NexxusFallback { relative_path, .. } => {
                self.assets.builtin_icon_path(relative_path)
            }
        }
        .or_else(|| {
            builtin_icon("application-x-generic")
                .and_then(|icon| self.assets.builtin_icon_path(icon.relative_path))
        });
        if let Some(path) = path {
            if let Ok(asset) = load_graphic(&path) {
                push_graphic(list, rect, asset);
            }
        }
        Ok(())
    }

    fn desktop_label_style(&self) -> TextStyle {
        TextStyle::new(
            self.theme.typography.family.clone(),
            self.theme.typography.small_size,
            self.theme.typography.small_size * self.theme.typography.line_height,
            self.theme.palette.text,
        )
    }

    fn menu_text_style(&self) -> TextStyle {
        TextStyle::new(
            self.theme.typography.family.clone(),
            self.theme.typography.body_size,
            self.theme.typography.body_size * self.theme.typography.line_height,
            self.theme.palette.text,
        )
    }
}

enum GraphicAsset {
    Svg(Vec<u8>),
    Raster(ImageData),
}

fn push_graphic(list: &mut DisplayList, rect: LogicalRect, asset: GraphicAsset) {
    match asset {
        GraphicAsset::Svg(bytes) => list.push(DrawCommand::Svg { rect, bytes }),
        GraphicAsset::Raster(image) => list.push(DrawCommand::Image { rect, image }),
    }
}

fn load_graphic(path: &Path) -> Result<GraphicAsset, DesktopRenderError> {
    let bytes = fs::read(path)?;
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
    {
        return Ok(GraphicAsset::Svg(bytes));
    }
    let decoded = image::load_from_memory(&bytes)?.to_rgba8();
    let (width, height) = decoded.dimensions();
    Ok(GraphicAsset::Raster(ImageData::new(
        width,
        height,
        decoded.into_raw(),
    )?))
}

fn first_existing<const N: usize>(paths: [PathBuf; N]) -> Option<PathBuf> {
    paths.into_iter().find(|path| path.is_file())
}

fn xdg_icon_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
        roots.push(
            env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
                .unwrap_or_else(|| home.join(".local/share"))
                .join("icons"),
        );
        roots.push(home.join(".icons"));
    }
    let data_dirs = env::var_os("XDG_DATA_DIRS")
        .map(|raw| env::split_paths(&raw).collect::<Vec<_>>())
        .unwrap_or_else(|| vec![PathBuf::from("/usr/local/share"), PathBuf::from("/usr/share")]);
    roots.extend(data_dirs.into_iter().map(|path| path.join("icons")));
    roots
}

#[derive(Debug, Error)]
pub enum DesktopRenderError {
    #[error(transparent)]
    Render(#[from] RenderError),
    #[error(transparent)]
    Theme(#[from] nexxus_ui::ThemeError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Image(#[from] ImageError),
    #[error(transparent)]
    Shell(#[from] DesktopShellError),
}
