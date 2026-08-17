//! Nexxus UI composition for the compact Finder surface.

use nexxus_ui::{
    Axis, DisplayList, Key, LogicalRect, Modifiers, Theme, UiEvent, UiMessage, UiTree, WidgetId,
    WidgetKind,
};

use crate::FinderMatch;

/// Retained UI owned by Stage 14. Window-system presentation stays in the
/// session/backend adapter, preserving backend neutrality.
#[derive(Clone, Debug)]
pub struct FinderView {
    tree: UiTree,
    query: WidgetId,
    results: WidgetId,
}

impl FinderView {
    pub fn new() -> Self {
        let mut tree = UiTree::new();
        let root = tree.insert(WidgetKind::Container {
            axis: Axis::Vertical,
            gap: 8.0,
        });
        let query = tree.insert(WidgetKind::TextField {
            text: String::new(),
            placeholder: "Buscar aplicações".to_owned(),
            cursor: 0,
        });
        let results = tree.insert(WidgetKind::List {
            items: Vec::new(),
            selected: None,
            offset: 0.0,
        });
        let _ = tree.add_child(root, query);
        let _ = tree.add_child(root, results);
        let _ = tree.set_root(root);

        // UiTree focuses the first focusable widget on Tab. Keeping the query
        // focused allows typing immediately when the Finder is shown.
        let _ = tree.handle_event(&UiEvent::KeyDown {
            key: Key::Tab,
            modifiers: Modifiers::default(),
        });

        Self {
            tree,
            query,
            results,
        }
    }

    pub fn query_id(&self) -> WidgetId {
        self.query
    }

    pub fn results_id(&self) -> WidgetId {
        self.results
    }

    pub fn set_query(&mut self, value: &str) {
        if let Some(node) = self.tree.node_mut(self.query) {
            if let WidgetKind::TextField { text, cursor, .. } = &mut node.kind {
                *text = value.to_owned();
                *cursor = text.len();
            }
        }
    }

    pub fn set_results(&mut self, matches: &[FinderMatch], selected: Option<usize>) {
        if let Some(node) = self.tree.node_mut(self.results) {
            if let WidgetKind::List {
                items,
                selected: current,
                offset,
            } = &mut node.kind
            {
                *items = matches.iter().map(|item| item.name.clone()).collect();
                *current = selected.filter(|index| *index < items.len());
                *offset = 0.0;
            }
        }
    }

    pub fn handle_event(&mut self, event: &UiEvent) -> Vec<UiMessage> {
        self.tree.handle_event(event)
    }

    pub fn layout(&mut self, bounds: LogicalRect, theme: &Theme) {
        self.tree.layout(bounds, theme);
    }

    pub fn paint(&self, theme: &Theme) -> DisplayList {
        self.tree.paint(theme)
    }
}

impl Default for FinderView {
    fn default() -> Self {
        Self::new()
    }
}
