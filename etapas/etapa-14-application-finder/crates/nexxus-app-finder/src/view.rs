//! Nexxus UI composition for the compact Finder surface.

use nexxus_ui::{
    Axis, DisplayList, DrawCommand, Key, LogicalPoint, LogicalRect, Modifiers, TextStyle, Theme,
    UiEvent, UiMessage, UiTree, WidgetId, WidgetKind,
};

use crate::{FinderIconResolver, FinderMatch};

const RESULT_ROW_HEIGHT: f32 = 36.0;
const RESULT_ICON_SIZE: f32 = 22.0;
const RESULT_ICON_LEFT: f32 = 7.0;
const RESULT_TEXT_LEFT: f32 = 38.0;

/// Retained UI owned by Stage 14. The query field uses reusable Nexxus UI
/// widgets, while component-specific result rows are painted here so they can
/// carry application icons without modifying Stage 07.
#[derive(Clone, Debug)]
pub struct FinderView {
    tree: UiTree,
    query: WidgetId,
    result_area: WidgetId,
    results: Vec<FinderMatch>,
    selected: Option<usize>,
    icons: FinderIconResolver,
}

impl FinderView {
    pub fn new() -> Self {
        Self::with_icon_resolver(FinderIconResolver::system())
    }

    pub fn with_icon_resolver(icons: FinderIconResolver) -> Self {
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
        let result_area = tree.insert(WidgetKind::Spacer);
        if let Some(node) = tree.node_mut(query) {
            node.flex_grow = 0.0;
        }
        if let Some(node) = tree.node_mut(result_area) {
            node.flex_grow = 1.0;
        }
        let _ = tree.add_child(root, query);
        let _ = tree.add_child(root, result_area);
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
            result_area,
            results: Vec::new(),
            selected: None,
            icons,
        }
    }

    pub fn query_id(&self) -> WidgetId {
        self.query
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
        self.results = matches.to_vec();
        self.set_selection(selected);
    }

    pub fn set_selection(&mut self, selected: Option<usize>) {
        self.selected = selected.filter(|index| *index < self.results.len());
    }

    /// Returns the result under a primary-button event using the exact geometry
    /// that Stage 14 paints, avoiding coupling to the private List row metric in
    /// Nexxus UI Core.
    pub fn result_at(&self, point: LogicalPoint) -> Option<usize> {
        let rect = self.result_area_rect()?;
        if !rect.contains(point) || self.results.is_empty() {
            return None;
        }
        let (start, visible) = self.visible_window(rect);
        let row = ((point.y - rect.y) / RESULT_ROW_HEIGHT).floor() as usize;
        if row >= visible {
            return None;
        }
        let index = start + row;
        (index < self.results.len()).then_some(index)
    }

    pub fn handle_event(&mut self, event: &UiEvent) -> Vec<UiMessage> {
        self.tree.handle_event(event)
    }

    pub fn layout(&mut self, bounds: LogicalRect, theme: &Theme) {
        self.tree.layout(bounds, theme);
    }

    /// Paints the reusable query control first, then the Finder-owned result
    /// rows with official/fallback icons and deterministic selection geometry.
    pub fn paint(&self, theme: &Theme) -> DisplayList {
        let mut list = self.tree.paint(theme);
        let Some(rect) = self.result_area_rect() else {
            return list;
        };

        list.push(DrawCommand::FillRect {
            rect,
            color: theme.palette.surface,
        });
        list.push(DrawCommand::PushClip(rect));

        if self.results.is_empty() {
            list.push(DrawCommand::Text {
                rect: LogicalRect::new(
                    rect.x + theme.metrics.padding,
                    rect.y + theme.metrics.padding,
                    (rect.width - 2.0 * theme.metrics.padding).max(0.0),
                    theme.typography.body_size * theme.typography.line_height,
                ),
                text: "Nenhuma aplicação encontrada".to_owned(),
                style: TextStyle::new(
                    theme.typography.family.clone(),
                    theme.typography.body_size,
                    theme.typography.body_size * theme.typography.line_height,
                    theme.palette.text_muted,
                ),
            });
        } else {
            let (start, visible) = self.visible_window(rect);
            for offset in 0..visible {
                let index = start + offset;
                let Some(result) = self.results.get(index) else {
                    break;
                };
                let row = LogicalRect::new(
                    rect.x,
                    rect.y + offset as f32 * RESULT_ROW_HEIGHT,
                    rect.width,
                    RESULT_ROW_HEIGHT,
                );
                if self.selected == Some(index) {
                    list.push(DrawCommand::FillRect {
                        rect: row,
                        color: theme.palette.selection,
                    });
                }
                let icon_rect = LogicalRect::new(
                    row.x + RESULT_ICON_LEFT,
                    row.y + (RESULT_ROW_HEIGHT - RESULT_ICON_SIZE) / 2.0,
                    RESULT_ICON_SIZE,
                    RESULT_ICON_SIZE,
                );
                self.icons.paint(&mut list, &result.icon, icon_rect);
                list.push(DrawCommand::Text {
                    rect: LogicalRect::new(
                        row.x + RESULT_TEXT_LEFT,
                        row.y
                            + (RESULT_ROW_HEIGHT
                                - theme.typography.body_size * theme.typography.line_height)
                                / 2.0,
                        (row.width - RESULT_TEXT_LEFT - theme.metrics.padding).max(0.0),
                        theme.typography.body_size * theme.typography.line_height,
                    ),
                    text: result.name.clone(),
                    style: TextStyle::new(
                        theme.typography.family.clone(),
                        theme.typography.body_size,
                        theme.typography.body_size * theme.typography.line_height,
                        theme.palette.text,
                    ),
                });
            }
        }

        list.push(DrawCommand::PopClip);
        list.push(DrawCommand::StrokeRect {
            rect,
            color: theme.palette.border,
            width: theme.metrics.border_width,
        });
        list
    }

    fn result_area_rect(&self) -> Option<LogicalRect> {
        self.tree.node(self.result_area).map(|node| node.rect)
    }

    fn visible_window(&self, rect: LogicalRect) -> (usize, usize) {
        let capacity = ((rect.height / RESULT_ROW_HEIGHT).floor() as usize).max(1);
        let selected = self
            .selected
            .unwrap_or(0)
            .min(self.results.len().saturating_sub(1));
        let start = if selected >= capacity {
            selected + 1 - capacity
        } else {
            0
        };
        let visible = capacity.min(self.results.len().saturating_sub(start));
        (start, visible)
    }
}

impl Default for FinderView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexxus_xdg_application_index::IconReference;

    fn result(id: &str, name: &str) -> FinderMatch {
        FinderMatch {
            desktop_id: id.to_owned(),
            name: name.to_owned(),
            icon: IconReference::ExternalName("missing-test-icon".to_owned()),
            score: 0,
            field: crate::MatchField::Name,
        }
    }

    #[test]
    fn pointer_geometry_selects_the_same_rows_that_are_painted() {
        let mut view = FinderView::with_icon_resolver(FinderIconResolver::new(
            std::path::PathBuf::from("/nonexistent"),
            Vec::new(),
        ));
        view.set_results(
            &[result("a.desktop", "Alpha"), result("b.desktop", "Beta")],
            Some(0),
        );
        view.layout(LogicalRect::new(0.0, 0.0, 560.0, 360.0), &Theme::default());
        let rect = view.result_area_rect().unwrap();
        assert_eq!(
            view.result_at(LogicalPoint::new(rect.x + 10.0, rect.y + 40.0)),
            Some(1)
        );
    }

    #[test]
    fn selected_result_is_kept_inside_visible_window() {
        let mut view = FinderView::default();
        let results: Vec<_> = (0..20)
            .map(|index| result(&format!("{index}.desktop"), &format!("App {index}")))
            .collect();
        view.set_results(&results, Some(19));
        view.layout(LogicalRect::new(0.0, 0.0, 560.0, 240.0), &Theme::default());
        let rect = view.result_area_rect().unwrap();
        let (start, visible) = view.visible_window(rect);
        assert!(start <= 19);
        assert!(19 < start + visible);
    }
}
