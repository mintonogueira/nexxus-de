//! Integration of Stage 12 live snapshots, Stage 10 shortcut target and Finder UI.

use std::sync::mpsc::{self, Receiver};

use nexxus_shortcuts::CommandTarget;
use nexxus_ui::UiEvent;
use nexxus_xdg_application_index::{
    ApplicationIndexConfig, ApplicationIndexEvent, ApplicationIndexService, ConfigError,
    LaunchContext, ServiceError,
};
use thiserror::Error;

use crate::{
    FinderAction, FinderController, FinderCorpus, LaunchError, NoCommentProvider,
    execute_launch_plan, plan_application_launch,
};

#[derive(Debug, Error)]
pub enum FinderRuntimeError {
    #[error("XDG application index configuration failed: {0}")]
    Config(#[from] ConfigError),
    #[error("XDG application index service failed: {0}")]
    Index(#[from] ServiceError),
    #[error("application launch failed: {0}")]
    Launch(#[from] LaunchError),
}

/// Runtime facade suitable for the Session Runtime to embed. It owns no
/// X11/Wayland object and therefore remains backend-neutral.
pub struct FinderRuntime {
    index: ApplicationIndexService,
    index_events: Receiver<ApplicationIndexEvent>,
    controller: FinderController,
}

impl FinderRuntime {
    pub fn start(config: ApplicationIndexConfig) -> Result<Self, FinderRuntimeError> {
        let index = ApplicationIndexService::start(config)?;
        let index_events = index.subscribe();
        let corpus = FinderCorpus::from_snapshot(&index.snapshot(), &NoCommentProvider);
        Ok(Self {
            index,
            index_events,
            controller: FinderController::new(corpus),
        })
    }

    pub fn start_from_environment() -> Result<Self, FinderRuntimeError> {
        Self::start(ApplicationIndexConfig::from_environment()?)
    }

    pub fn controller(&self) -> &FinderController {
        &self.controller
    }

    pub fn controller_mut(&mut self) -> &mut FinderController {
        &mut self.controller
    }

    pub fn handle_shortcut_target(&mut self, target: CommandTarget) -> FinderAction {
        self.refresh_index();
        self.controller.handle_shortcut_target(target)
    }

    pub fn handle_ui_event(&mut self, event: &UiEvent) -> Result<FinderAction, FinderRuntimeError> {
        self.refresh_index();
        let action = self.controller.handle_event(event);
        if let FinderAction::Launch(desktop_id) = &action {
            self.launch(desktop_id)?;
            self.controller.close();
        }
        Ok(action)
    }

    /// Drains all pending Stage 12 events and replaces the corpus only after an
    /// authoritative changed generation is available.
    pub fn refresh_index(&mut self) {
        let mut changed = false;
        loop {
            match self.index_events.try_recv() {
                Ok(ApplicationIndexEvent::Changed(_)) => changed = true,
                Ok(ApplicationIndexEvent::WatchError(_))
                | Ok(ApplicationIndexEvent::RescanError(_)) => {}
                Err(mpsc::TryRecvError::Empty | mpsc::TryRecvError::Disconnected) => break,
            }
        }
        if changed {
            let snapshot = self.index.snapshot();
            self.controller
                .replace_corpus(FinderCorpus::from_snapshot(&snapshot, &NoCommentProvider));
        }
    }

    fn launch(&self, desktop_id: &str) -> Result<(), FinderRuntimeError> {
        let snapshot = self.index.snapshot();
        let Some(record) = snapshot.by_id(desktop_id) else {
            return Ok(());
        };
        let plan = plan_application_launch(record, &LaunchContext::default())?;
        let _child = execute_launch_plan(&plan)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use nexxus_shortcuts::{CommandTarget, LauncherAction};
    use nexxus_ui::{Key, Modifiers};
    use nexxus_xdg_application_index::ApplicationRoot;

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after UNIX epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "nexxus-app-finder-runtime-{label}-{}-{nonce}",
            std::process::id()
        ))
    }

    fn wait_for_file(path: &Path) -> bool {
        for _ in 0..100 {
            if path.is_file() {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        false
    }

    #[test]
    fn shortcut_search_enter_launches_the_selected_desktop_entry() {
        let root = temp_root("launch");
        fs::create_dir_all(&root).unwrap();
        let marker = root.join("launched.marker");
        fs::write(
            root.join("org.example.FinderAlpha.desktop"),
            format!(
                "[Desktop Entry]\nType=Application\nName=Finder Alpha\nExec=/usr/bin/touch {}\nCategories=Utility;\n",
                marker.display()
            ),
        )
        .unwrap();

        let config = ApplicationIndexConfig {
            roots: vec![ApplicationRoot::custom(&root, "finder-runtime-test")],
            locales: Vec::new(),
            current_desktops: Vec::new(),
            max_desktop_file_bytes: 2 * 1024 * 1024,
        };
        let mut runtime = FinderRuntime::start(config).unwrap();
        assert_eq!(
            runtime
                .handle_shortcut_target(CommandTarget::Launcher(LauncherAction::ApplicationFinder)),
            FinderAction::Opened
        );
        assert_eq!(
            runtime
                .handle_ui_event(&UiEvent::TextInput("alpha".to_owned()))
                .unwrap(),
            FinderAction::None
        );
        assert_eq!(
            runtime
                .handle_ui_event(&UiEvent::KeyDown {
                    key: Key::Enter,
                    modifiers: Modifiers::default(),
                })
                .unwrap(),
            FinderAction::Launch("org.example.FinderAlpha.desktop".to_owned())
        );
        assert!(
            wait_for_file(&marker),
            "selected application was not launched"
        );
        assert!(!runtime.controller().state().visible);

        drop(runtime);
        fs::remove_dir_all(root).unwrap();
    }
}
