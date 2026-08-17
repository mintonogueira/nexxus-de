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

    pub fn handle_ui_event(
        &mut self,
        event: &UiEvent,
    ) -> Result<FinderAction, FinderRuntimeError> {
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
