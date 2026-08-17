//! Nexxus Desktop Shell: wallpaper, desktop launchers/folders and the desktop
//! context menu for the initial X11 backend.
//!
//! The crate consumes Stage 07 UI, Stage 08 assets, Stage 10 shortcut routing
//! and Stage 12 application indexing without reimplementing those modules.

#![forbid(unsafe_code)]

mod config;
mod desktop_dir;
mod launch;
mod menu;
mod model;
mod render;
pub mod x11;

pub use config::{
    DEFAULT_WALLPAPER, DESKTOP_CONFIG_SCHEMA, DesktopConfig, DesktopConfigError,
    DesktopConfigStore, LauncherPlacement, WallpaperSelection, default_config_path,
};
pub use desktop_dir::{
    DesktopDirectoryError, create_unique_folder, list_desktop_folders, resolve_desktop_dir,
    resolve_desktop_dir_from,
};
pub use launch::{LaunchError, LaunchPlan, execute_launch_plan, plan_application_launch};
pub use menu::{MenuAction, MenuEntry, MenuPage, category_label};
pub use model::{
    ContextMenuState, DesktopShell, DesktopShellAction, DesktopShellError, DesktopShellRuntime,
    MonitorGeometry, RuntimeError,
};
pub use render::{
    AssetSource, DesktopLayout, DesktopPainter, DesktopRenderError, FolderHitBox, LauncherHitBox,
    MenuHitBox,
};
