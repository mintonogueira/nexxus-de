//! Finder state machine, keyboard/mouse navigation and shortcut integration.

use nexxus_shortcuts::{CommandTarget, LauncherAction};
use nexxus_ui::{Key, PointerButton, UiEvent, UiMessage};

use crate::{FinderCorpus, FinderMatch, FinderView};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FinderAction {
    None,
    Opened,
    Closed,
    Launch(String),
}

/// Observable state kept independent from platform window objects.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FinderState {
    pub visible: bool,
    pub query: String,
    pub selected: Option<usize>,
    pub results: Vec<FinderMatch>,
    pub source_generation: u64,
}

pub struct FinderController {
    corpus: FinderCorpus,
    state: FinderState,
    view: FinderView,
}

impl FinderController {
    pub fn new(corpus: FinderCorpus) -> Self {
        let mut controller = Self {
            corpus,
            state: FinderState::default(),
            view: FinderView::new(),
        };
        controller.refresh_results();
        controller
    }

    pub fn state(&self) -> &FinderState {
        &self.state
    }

    pub fn view(&self) -> &FinderView {
        &self.view
    }

    pub fn view_mut(&mut self) -> &mut FinderView {
        &mut self.view
    }

    /// Consumes only the logical target registered by Stage 10; no keycode or
    /// backend-specific shortcut detail leaks into the Finder.
    pub fn handle_shortcut_target(&mut self, target: CommandTarget) -> FinderAction {
        if target == CommandTarget::Launcher(LauncherAction::ApplicationFinder) {
            self.open()
        } else {
            FinderAction::None
        }
    }

    pub fn open(&mut self) -> FinderAction {
        self.state.visible = true;
        self.state.query.clear();
        self.view.set_query("");
        self.refresh_results();
        FinderAction::Opened
    }

    pub fn close(&mut self) -> FinderAction {
        if !self.state.visible {
            return FinderAction::None;
        }
        self.state.visible = false;
        FinderAction::Closed
    }

    /// Replaces the source corpus after Stage 12 publishes a new generation and
    /// keeps the current query active without requiring logout/reopen.
    pub fn replace_corpus(&mut self, corpus: FinderCorpus) {
        if corpus.generation() == self.corpus.generation() {
            return;
        }
        self.corpus = corpus;
        self.refresh_results();
    }

    /// Handles Finder-level navigation before delegating text editing and
    /// pointer hit-testing to Nexxus UI Core.
    pub fn handle_event(&mut self, event: &UiEvent) -> FinderAction {
        if !self.state.visible {
            return FinderAction::None;
        }

        match event {
            UiEvent::KeyDown {
                key: Key::Escape, ..
            } => return self.close(),
            UiEvent::KeyDown {
                key: Key::ArrowDown,
                ..
            } => {
                self.move_selection(1);
                return FinderAction::None;
            }
            UiEvent::KeyDown {
                key: Key::ArrowUp, ..
            } => {
                self.move_selection(-1);
                return FinderAction::None;
            }
            _ => {}
        }

        let pointer_release = matches!(
            event,
            UiEvent::PointerUp {
                button: PointerButton::Primary,
                ..
            }
        );
        let messages = self.view.handle_event(event);
        let mut launch_from_pointer = false;
        for message in messages {
            match message {
                UiMessage::TextChanged { id, text } if id == self.view.query_id() => {
                    self.state.query = text;
                    self.refresh_results();
                }
                UiMessage::Submitted { id, .. } if id == self.view.query_id() => {
                    return self.launch_selected();
                }
                UiMessage::ListSelectionChanged { id, selected }
                    if id == self.view.results_id() =>
                {
                    self.state.selected = Some(selected);
                    launch_from_pointer |= pointer_release;
                }
                _ => {}
            }
        }

        if launch_from_pointer {
            self.launch_selected()
        } else {
            FinderAction::None
        }
    }

    fn refresh_results(&mut self) {
        self.state.results = self.corpus.search(&self.state.query);
        self.state.source_generation = self.corpus.generation();
        self.state.selected = (!self.state.results.is_empty()).then_some(0);
        self.view
            .set_results(&self.state.results, self.state.selected);
    }

    fn move_selection(&mut self, delta: i8) {
        let len = self.state.results.len();
        if len == 0 {
            self.state.selected = None;
            self.view.set_results(&self.state.results, None);
            return;
        }

        let current = self.state.selected.unwrap_or(0);
        let next = if delta < 0 {
            current.saturating_sub(1)
        } else {
            (current + 1).min(len - 1)
        };
        self.state.selected = Some(next);
        self.view
            .set_results(&self.state.results, self.state.selected);
    }

    fn launch_selected(&self) -> FinderAction {
        self.state
            .selected
            .and_then(|index| self.state.results.get(index))
            .map(|result| FinderAction::Launch(result.desktop_id.clone()))
            .unwrap_or(FinderAction::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexxus_xdg_application_index::IconReference;

    fn corpus() -> FinderCorpus {
        FinderCorpus::from_documents(
            1,
            vec![
                crate::SearchDocument {
                    desktop_id: "a.desktop".to_owned(),
                    name: "Alpha".to_owned(),
                    icon: IconReference::ExternalName("a".to_owned()),
                    keywords: vec![],
                    comment: None,
                    categories: vec![],
                },
                crate::SearchDocument {
                    desktop_id: "b.desktop".to_owned(),
                    name: "Beta".to_owned(),
                    icon: IconReference::ExternalName("b".to_owned()),
                    keywords: vec![],
                    comment: None,
                    categories: vec![],
                },
            ],
        )
    }

    #[test]
    fn shortcut_opens_and_escape_closes() {
        let mut controller = FinderController::new(corpus());
        assert_eq!(
            controller
                .handle_shortcut_target(CommandTarget::Launcher(LauncherAction::ApplicationFinder)),
            FinderAction::Opened
        );
        assert!(controller.state().visible);
        assert_eq!(
            controller.handle_event(&UiEvent::KeyDown {
                key: Key::Escape,
                modifiers: Default::default(),
            }),
            FinderAction::Closed
        );
    }

    #[test]
    fn incremental_text_filters_results_and_enter_launches_selected() {
        let mut controller = FinderController::new(corpus());
        controller.open();
        assert_eq!(
            controller.handle_event(&UiEvent::TextInput("bet".to_owned())),
            FinderAction::None
        );
        assert_eq!(controller.state().results.len(), 1);
        assert_eq!(
            controller.handle_event(&UiEvent::KeyDown {
                key: Key::Enter,
                modifiers: Default::default(),
            }),
            FinderAction::Launch("b.desktop".to_owned())
        );
    }

    #[test]
    fn arrow_navigation_changes_selected_result() {
        let mut controller = FinderController::new(corpus());
        controller.open();
        controller.handle_event(&UiEvent::KeyDown {
            key: Key::ArrowDown,
            modifiers: Default::default(),
        });
        assert_eq!(controller.state().selected, Some(1));
    }

    #[test]
    fn new_generation_reapplies_query() {
        let mut controller = FinderController::new(corpus());
        controller.open();
        controller.handle_event(&UiEvent::TextInput("alp".to_owned()));

        let replacement = FinderCorpus::from_documents(
            2,
            vec![crate::SearchDocument {
                desktop_id: "c.desktop".to_owned(),
                name: "Alpine".to_owned(),
                icon: IconReference::ExternalName("c".to_owned()),
                keywords: vec![],
                comment: None,
                categories: vec![],
            }],
        );
        controller.replace_corpus(replacement);
        assert_eq!(controller.state().source_generation, 2);
        assert_eq!(controller.state().results[0].name, "Alpine");
    }
}
