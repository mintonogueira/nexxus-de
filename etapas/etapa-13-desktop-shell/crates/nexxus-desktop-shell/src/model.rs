//! Desktop-shell state, actions and live application-index integration.

use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;

use nexxus_shortcuts::{CommandTarget, ShellAction};
use nexxus_ui::{LogicalPoint, LogicalRect, ScaleFactor};
use nexxus_xdg_application_index::{
    ApplicationIndexConfig, ApplicationIndexEvent, ApplicationIndexService, IndexSnapshot,
    LaunchContext, ServiceError,
};
use thiserror::Error;

use crate::config::{
    DesktopConfig, DesktopConfigError, DesktopConfigStore, LauncherPlacement, WallpaperSelection,
};
use crate::desktop_dir::{
    DesktopDirectoryError, create_unique_folder, list_desktop_folders, resolve_desktop_dir,
};
use crate::launch::{LaunchError, LaunchPlan, plan_application_launch};
use crate::menu::{MenuAction, MenuEntry, MenuPage, entries as menu_entries};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonitorGeometry {
    pub rect: LogicalRect,
    pub scale: ScaleFactor,
    pub primary: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContextMenuState {
    pub page: MenuPage,
    pub anchor: LogicalPoint,
    pub monitor_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesktopShellAction {
    Launch(LaunchPlan),
    OpenTerminal,
    OpenFileManager { path: PathBuf },
    OpenDesktopSettings,
    FolderCreated(PathBuf),
    LauncherPinned(String),
}

#[derive(Debug, Error)]
pub enum DesktopShellError {
    #[error("desktop shell requires at least one monitor")]
    NoMonitor,
    #[error("application '{0}' is unavailable in the current XDG index")]
    UnknownApplication(String),
    #[error("application '{0}' is hidden from desktop presentation")]
    HiddenApplication(String),
    #[error(transparent)]
    Config(#[from] DesktopConfigError),
    #[error(transparent)]
    DesktopDirectory(#[from] DesktopDirectoryError),
    #[error(transparent)]
    Launch(#[from] LaunchError),
}

/// Authoritative model for the Stage 13 desktop surface. It stores only
/// desktop-owned state and consumes immutable snapshots from Stage 12.
pub struct DesktopShell {
    store: DesktopConfigStore,
    config: DesktopConfig,
    snapshot: IndexSnapshot,
    monitors: Vec<MonitorGeometry>,
    desktop_dir: PathBuf,
    menu: Option<ContextMenuState>,
}

impl DesktopShell {
    pub fn new(
        store: DesktopConfigStore,
        snapshot: IndexSnapshot,
        monitors: Vec<MonitorGeometry>,
        desktop_dir: PathBuf,
    ) -> Result<Self, DesktopShellError> {
        if monitors.is_empty() {
            return Err(DesktopShellError::NoMonitor);
        }
        let config = store.load_or_default()?;
        Ok(Self {
            store,
            config,
            snapshot,
            monitors,
            desktop_dir,
            menu: None,
        })
    }

    pub fn from_environment(
        snapshot: IndexSnapshot,
        monitors: Vec<MonitorGeometry>,
    ) -> Result<Self, DesktopShellError> {
        Self::new(
            DesktopConfigStore::from_environment()?,
            snapshot,
            monitors,
            resolve_desktop_dir()?,
        )
    }

    pub fn config(&self) -> &DesktopConfig {
        &self.config
    }

    pub fn snapshot(&self) -> &IndexSnapshot {
        &self.snapshot
    }

    pub fn monitors(&self) -> &[MonitorGeometry] {
        &self.monitors
    }

    pub fn desktop_dir(&self) -> &Path {
        &self.desktop_dir
    }

    pub fn menu(&self) -> Option<&ContextMenuState> {
        self.menu.as_ref()
    }

    pub fn menu_entries(&self) -> Vec<MenuEntry> {
        self.menu
            .as_ref()
            .map(|state| menu_entries(&state.page, &self.snapshot))
            .unwrap_or_default()
    }

    pub fn set_wallpaper(
        &mut self,
        wallpaper: WallpaperSelection,
    ) -> Result<(), DesktopShellError> {
        self.config.wallpaper = wallpaper;
        self.store.save(&self.config)?;
        Ok(())
    }

    /// A pinned launcher is admitted only when the current common XDG index
    /// exposes the application as visible. Persisted stale IDs are retained but
    /// naturally disappear from presentation until the application returns.
    pub fn pin_launcher(
        &mut self,
        desktop_id: &str,
        position: Option<LogicalPoint>,
    ) -> Result<(), DesktopShellError> {
        let record = self
            .snapshot
            .by_id(desktop_id)
            .ok_or_else(|| DesktopShellError::UnknownApplication(desktop_id.to_owned()))?;
        if !record.is_visible() {
            return Err(DesktopShellError::HiddenApplication(desktop_id.to_owned()));
        }
        if self
            .config
            .launchers
            .iter()
            .any(|launcher| launcher.desktop_id == desktop_id)
        {
            return Ok(());
        }
        let point = position.unwrap_or_else(|| self.next_launcher_position());
        self.config
            .launchers
            .push(LauncherPlacement::new(desktop_id, point.x, point.y)?);
        self.store.save(&self.config)?;
        Ok(())
    }

    pub fn move_launcher(
        &mut self,
        desktop_id: &str,
        point: LogicalPoint,
    ) -> Result<(), DesktopShellError> {
        let Some(launcher) = self
            .config
            .launchers
            .iter_mut()
            .find(|launcher| launcher.desktop_id == desktop_id)
        else {
            return Err(DesktopShellError::UnknownApplication(desktop_id.to_owned()));
        };
        if !point.x.is_finite() || !point.y.is_finite() {
            return Err(DesktopConfigError::InvalidCoordinate.into());
        }
        launcher.x = point.x;
        launcher.y = point.y;
        self.store.save(&self.config)?;
        Ok(())
    }

    pub fn visible_launchers(
        &self,
    ) -> impl Iterator<
        Item = (
            &LauncherPlacement,
            &nexxus_xdg_application_index::ApplicationRecord,
        ),
    > {
        self.config.launchers.iter().filter_map(|launcher| {
            self.snapshot
                .by_id(&launcher.desktop_id)
                .filter(|record| record.is_visible())
                .map(|record| (launcher, record))
        })
    }

    pub fn desktop_folders(&self) -> Result<Vec<PathBuf>, DesktopShellError> {
        Ok(list_desktop_folders(&self.desktop_dir)?)
    }

    pub fn open_context_menu(&mut self, point: LogicalPoint) -> Result<(), DesktopShellError> {
        let monitor_index = self
            .monitors
            .iter()
            .position(|monitor| monitor.rect.contains(point))
            .or_else(|| self.primary_monitor_index())
            .ok_or(DesktopShellError::NoMonitor)?;
        self.menu = Some(ContextMenuState {
            page: MenuPage::Root,
            anchor: clamp_point(point, self.monitors[monitor_index].rect),
            monitor_index,
        });
        Ok(())
    }

    pub fn open_context_menu_from_shortcut(&mut self) -> Result<(), DesktopShellError> {
        let index = self
            .primary_monitor_index()
            .ok_or(DesktopShellError::NoMonitor)?;
        let rect = self.monitors[index].rect;
        self.menu = Some(ContextMenuState {
            page: MenuPage::Root,
            anchor: LogicalPoint::new(rect.x + 24.0, rect.y + 24.0),
            monitor_index: index,
        });
        Ok(())
    }

    /// Consumes the semantic dispatch target from Stage 10. No second shortcut
    /// registry or X11 grab is created by the Desktop Shell.
    pub fn handle_shortcut_target(
        &mut self,
        target: CommandTarget,
    ) -> Result<bool, DesktopShellError> {
        if target == CommandTarget::Shell(ShellAction::DesktopMenu) {
            self.open_context_menu_from_shortcut()?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn close_menu(&mut self) {
        self.menu = None;
    }

    pub fn activate_menu_action(
        &mut self,
        action: MenuAction,
    ) -> Result<Option<DesktopShellAction>, DesktopShellError> {
        match action {
            MenuAction::OpenApplications => self.set_menu_page(MenuPage::Applications),
            MenuAction::OpenCategory(category) => self.set_menu_page(MenuPage::Category(category)),
            MenuAction::OpenCreateLauncher => self.set_menu_page(MenuPage::CreateLauncher),
            MenuAction::Back => self.set_menu_page(MenuPage::Root),
            MenuAction::LaunchApplication(id) => {
                let action = self.launch_action(&id)?;
                self.close_menu();
                return Ok(Some(action));
            }
            MenuAction::OpenTerminal => {
                self.close_menu();
                return Ok(Some(DesktopShellAction::OpenTerminal));
            }
            MenuAction::OpenFileManager => {
                self.close_menu();
                return Ok(Some(DesktopShellAction::OpenFileManager {
                    path: self.desktop_dir.clone(),
                }));
            }
            MenuAction::CreateFolder => {
                let path = create_unique_folder(&self.desktop_dir)?;
                self.close_menu();
                return Ok(Some(DesktopShellAction::FolderCreated(path)));
            }
            MenuAction::PinLauncher(id) => {
                self.pin_launcher(&id, None)?;
                self.close_menu();
                return Ok(Some(DesktopShellAction::LauncherPinned(id)));
            }
            MenuAction::OpenDesktopSettings => {
                self.close_menu();
                return Ok(Some(DesktopShellAction::OpenDesktopSettings));
            }
        }
        Ok(None)
    }

    pub fn launch_action(&self, desktop_id: &str) -> Result<DesktopShellAction, DesktopShellError> {
        let record = self
            .snapshot
            .by_id(desktop_id)
            .ok_or_else(|| DesktopShellError::UnknownApplication(desktop_id.to_owned()))?;
        if !record.is_visible() {
            return Err(DesktopShellError::HiddenApplication(desktop_id.to_owned()));
        }
        Ok(DesktopShellAction::Launch(plan_application_launch(
            record,
            &LaunchContext::default(),
        )?))
    }

    pub fn apply_index_snapshot(&mut self, snapshot: IndexSnapshot) {
        self.snapshot = snapshot;
    }

    pub fn replace_monitors(
        &mut self,
        monitors: Vec<MonitorGeometry>,
    ) -> Result<(), DesktopShellError> {
        if monitors.is_empty() {
            return Err(DesktopShellError::NoMonitor);
        }
        self.monitors = monitors;
        if self
            .menu
            .as_ref()
            .is_some_and(|menu| menu.monitor_index >= self.monitors.len())
        {
            self.menu = None;
        }
        Ok(())
    }

    fn set_menu_page(&mut self, page: MenuPage) {
        if let Some(menu) = &mut self.menu {
            menu.page = page;
        }
    }

    fn primary_monitor_index(&self) -> Option<usize> {
        self.monitors
            .iter()
            .position(|monitor| monitor.primary)
            .or((!self.monitors.is_empty()).then_some(0))
    }

    fn next_launcher_position(&self) -> LogicalPoint {
        let index = self.config.launchers.len() as f32;
        let monitor = self
            .primary_monitor_index()
            .map(|index| self.monitors[index].rect)
            .unwrap_or_default();
        let cell_height = 88.0;
        let cell_width = 104.0;
        let usable_rows = ((monitor.height - 48.0) / cell_height).floor().max(1.0);
        let column = (index / usable_rows).floor();
        let row = index % usable_rows;
        LogicalPoint::new(
            monitor.x + 24.0 + column * cell_width,
            monitor.y + 24.0 + row * cell_height,
        )
    }
}

fn clamp_point(point: LogicalPoint, rect: LogicalRect) -> LogicalPoint {
    LogicalPoint::new(
        point.x.max(rect.x).min(rect.x + rect.width - 1.0),
        point.y.max(rect.y).min(rect.y + rect.height - 1.0),
    )
}

/// Runtime wrapper that owns the Stage 12 live service and applies changed
/// generations to the desktop model without restarting the session.
pub struct DesktopShellRuntime {
    shell: DesktopShell,
    index_service: ApplicationIndexService,
    index_events: Receiver<ApplicationIndexEvent>,
}

impl DesktopShellRuntime {
    pub fn start(
        store: DesktopConfigStore,
        index_config: ApplicationIndexConfig,
        monitors: Vec<MonitorGeometry>,
        desktop_dir: PathBuf,
    ) -> Result<Self, RuntimeError> {
        let index_service = ApplicationIndexService::start(index_config)?;
        let index_events = index_service.subscribe();
        let snapshot = index_service.snapshot();
        let shell = DesktopShell::new(store, snapshot, monitors, desktop_dir)?;
        Ok(Self {
            shell,
            index_service,
            index_events,
        })
    }

    pub fn shell(&self) -> &DesktopShell {
        &self.shell
    }

    pub fn shell_mut(&mut self) -> &mut DesktopShell {
        &mut self.shell
    }

    /// Drains watcher notifications. Only successful `Changed` events replace
    /// the model snapshot; watcher/rescan failures preserve the last valid view.
    pub fn poll_index_updates(&mut self) -> Vec<ApplicationIndexEvent> {
        let mut observed = Vec::new();
        while let Ok(event) = self.index_events.try_recv() {
            if matches!(event, ApplicationIndexEvent::Changed(_)) {
                self.shell
                    .apply_index_snapshot(self.index_service.snapshot());
            }
            observed.push(event);
        }
        observed
    }
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error(transparent)]
    Index(#[from] ServiceError),
    #[error(transparent)]
    Shell(#[from] DesktopShellError),
}
