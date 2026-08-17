//! Retained widget tree with deterministic layout, hit-testing, focus and input.

use crate::accessibility::{AccessibilityNode, AccessibilityRole, AccessibilityTree};
use crate::geometry::{Insets, LogicalPoint, LogicalRect};
use crate::input::{Key, PointerButton, UiEvent, WidgetId};
use crate::layout::{Axis, FlexItem, layout_flex};
use crate::render::{DisplayList, DrawCommand, TextStyle};
use crate::theme::Theme;

const LIST_ROW_HEIGHT: f32 = 28.0;

/// Built-in widgets required by the UI Core. Concrete desktop components are
/// intentionally left to their own later stages.
#[derive(Clone, Debug, PartialEq)]
pub enum WidgetKind {
    Container { axis: Axis, gap: f32 },
    Label { text: String },
    Button { label: String, pressed: bool },
    Toggle { label: String, value: bool, pressed: bool },
    Checkbox { label: String, checked: bool, pressed: bool },
    TextField { text: String, placeholder: String, cursor: usize },
    List { items: Vec<String>, selected: Option<usize>, offset: f32 },
    Scroll { offset_x: f32, offset_y: f32 },
    Menu { items: Vec<String>, selected: Option<usize>, open: bool },
    Popup { open: bool },
    Tabs { labels: Vec<String>, active: usize },
    Spacer,
}

impl WidgetKind {
    fn focusable(&self) -> bool {
        matches!(self, Self::Button { .. } | Self::Toggle { .. } | Self::Checkbox { .. } | Self::TextField { .. } | Self::List { .. } | Self::Menu { .. } | Self::Tabs { .. })
    }

    fn intrinsic_main(&self, axis: Axis, theme: &Theme) -> f32 {
        match (axis, self) {
            (Axis::Vertical, Self::Label { .. }) => theme.typography.body_size * theme.typography.line_height + theme.metrics.padding,
            (Axis::Vertical, Self::Button { .. } | Self::Toggle { .. } | Self::Checkbox { .. } | Self::TextField { .. } | Self::Tabs { .. }) => theme.metrics.control_height,
            (Axis::Vertical, Self::Menu { items, .. } | Self::List { items, .. }) => (items.len().clamp(1, 8) as f32 * LIST_ROW_HEIGHT).max(theme.metrics.control_height),
            (Axis::Vertical, Self::Spacer | Self::Container { .. } | Self::Scroll { .. } | Self::Popup { .. }) => 0.0,
            (Axis::Horizontal, _) => 0.0,
        }
    }
}

/// Retained node. `flex_grow` only affects placement inside a Container.
#[derive(Clone, Debug, PartialEq)]
pub struct UiNode {
    pub id: WidgetId,
    pub kind: WidgetKind,
    pub rect: LogicalRect,
    pub visible: bool,
    pub enabled: bool,
    pub flex_grow: f32,
    pub children: Vec<WidgetId>,
    parent: Option<WidgetId>,
    hovered: bool,
}

impl UiNode {
    fn new(id: WidgetId, kind: WidgetKind) -> Self {
        Self {
            id,
            kind,
            rect: LogicalRect::default(),
            visible: true,
            enabled: true,
            flex_grow: 1.0,
            children: Vec::new(),
            parent: None,
            hovered: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiMessage {
    Clicked(WidgetId),
    Toggled { id: WidgetId, value: bool },
    Checked { id: WidgetId, checked: bool },
    TextChanged { id: WidgetId, text: String },
    Submitted { id: WidgetId, text: String },
    ListSelectionChanged { id: WidgetId, selected: usize },
    ScrollChanged { id: WidgetId, x: f32, y: f32 },
    MenuItemActivated { id: WidgetId, index: usize },
    TabChanged { id: WidgetId, active: usize },
    PopupDismissed(WidgetId),
    FocusChanged(Option<WidgetId>),
}

/// Arena-like retained tree. IDs never get reused in Stage 07, which keeps
/// event routing deterministic and avoids stale-ID aliasing.
#[derive(Clone, Debug, Default)]
pub struct UiTree {
    nodes: Vec<UiNode>,
    root: Option<WidgetId>,
    focused: Option<WidgetId>,
    pointer_capture: Option<WidgetId>,
}

impl UiTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, kind: WidgetKind) -> WidgetId {
        let id = WidgetId(self.nodes.len() as u64);
        self.nodes.push(UiNode::new(id, kind));
        if self.root.is_none() {
            self.root = Some(id);
        }
        id
    }

    pub fn set_root(&mut self, id: WidgetId) -> bool {
        if self.node(id).is_none() {
            return false;
        }
        self.root = Some(id);
        true
    }

    pub fn root(&self) -> Option<WidgetId> {
        self.root
    }

    pub fn focused(&self) -> Option<WidgetId> {
        self.focused
    }

    pub fn node(&self, id: WidgetId) -> Option<&UiNode> {
        self.nodes.get(id.0 as usize)
    }

    pub fn node_mut(&mut self, id: WidgetId) -> Option<&mut UiNode> {
        self.nodes.get_mut(id.0 as usize)
    }

    /// Links an existing child while rejecting cycles and multiple parents.
    pub fn add_child(&mut self, parent: WidgetId, child: WidgetId) -> bool {
        if parent == child || self.node(parent).is_none() || self.node(child).is_none() {
            return false;
        }
        if self.node(child).and_then(|node| node.parent).is_some() || self.is_descendant(child, parent) {
            return false;
        }
        if let Some(child_node) = self.node_mut(child) {
            child_node.parent = Some(parent);
        }
        if let Some(parent_node) = self.node_mut(parent) {
            parent_node.children.push(child);
            true
        } else {
            false
        }
    }

    fn is_descendant(&self, candidate_parent: WidgetId, candidate_child: WidgetId) -> bool {
        let mut current = Some(candidate_child);
        while let Some(id) = current {
            if id == candidate_parent {
                return true;
            }
            current = self.node(id).and_then(|node| node.parent);
        }
        false
    }

    /// Performs deterministic retained layout. Containers use a simple flex
    /// primitive; complex component-specific layouts belong to later stages.
    pub fn layout(&mut self, bounds: LogicalRect, theme: &Theme) {
        if let Some(root) = self.root {
            self.layout_node(root, bounds, theme);
        }
    }

    fn layout_node(&mut self, id: WidgetId, bounds: LogicalRect, theme: &Theme) {
        let Some(snapshot) = self.node(id).cloned() else { return; };
        if let Some(node) = self.node_mut(id) {
            node.rect = bounds;
        }
        if snapshot.children.is_empty() {
            return;
        }

        match snapshot.kind {
            WidgetKind::Container { axis, gap } => {
                let items: Vec<_> = snapshot.children.iter().filter_map(|child_id| {
                    self.node(*child_id).map(|child| FlexItem {
                        id: *child_id,
                        basis: child.kind.intrinsic_main(axis, theme),
                        grow: child.flex_grow.max(0.0),
                        min: 0.0,
                        max: f32::INFINITY,
                    })
                }).collect();
                for (child, rect) in layout_flex(bounds, axis, gap, &items) {
                    self.layout_node(child, rect, theme);
                }
            }
            WidgetKind::Scroll { offset_x, offset_y } => {
                let content = LogicalRect::new(bounds.x - offset_x, bounds.y - offset_y, bounds.width, bounds.height);
                for child in snapshot.children {
                    self.layout_node(child, content, theme);
                }
            }
            _ => {
                for child in snapshot.children {
                    self.layout_node(child, bounds, theme);
                }
            }
        }
    }

    /// Returns the deepest visible node under the pointer. Children are tested
    /// in reverse paint order so popups/overlays naturally win hit-testing.
    pub fn hit_test(&self, point: LogicalPoint) -> Option<WidgetId> {
        self.root.and_then(|root| self.hit_test_node(root, point))
    }

    fn hit_test_node(&self, id: WidgetId, point: LogicalPoint) -> Option<WidgetId> {
        let node = self.node(id)?;
        if !node.visible || !node.rect.contains(point) {
            return None;
        }
        for child in node.children.iter().rev() {
            if let Some(hit) = self.hit_test_node(*child, point) {
                return Some(hit);
            }
        }
        Some(id)
    }

    fn nearest_interactive(&self, mut id: WidgetId) -> Option<WidgetId> {
        loop {
            let node = self.node(id)?;
            if node.enabled && node.kind.focusable() {
                return Some(id);
            }
            id = node.parent?;
        }
    }

    fn set_focus(&mut self, focus: Option<WidgetId>, messages: &mut Vec<UiMessage>) {
        let next = focus.filter(|id| self.node(*id).is_some_and(|node| node.visible && node.enabled && node.kind.focusable()));
        if self.focused != next {
            self.focused = next;
            messages.push(UiMessage::FocusChanged(next));
        }
    }

    fn move_focus(&mut self, reverse: bool, messages: &mut Vec<UiMessage>) {
        let focusable: Vec<_> = self.nodes.iter().filter(|node| node.visible && node.enabled && node.kind.focusable()).map(|node| node.id).collect();
        if focusable.is_empty() {
            self.set_focus(None, messages);
            return;
        }
        let current = self.focused.and_then(|id| focusable.iter().position(|item| *item == id));
        let index = match (current, reverse) {
            (Some(0), true) | (None, true) => focusable.len() - 1,
            (Some(index), true) => index - 1,
            (Some(index), false) => (index + 1) % focusable.len(),
            (None, false) => 0,
        };
        self.set_focus(Some(focusable[index]), messages);
    }

    /// Dispatches a normalized event and returns semantic messages for the
    /// owning component. Platform-specific keycodes/buttons never enter here.
    pub fn handle_event(&mut self, event: &UiEvent) -> Vec<UiMessage> {
        let mut messages = Vec::new();
        match event {
            UiEvent::PointerMove { position } => {
                for node in &mut self.nodes { node.hovered = false; }
                if let Some(hit) = self.hit_test(*position) {
                    if let Some(node) = self.node_mut(hit) { node.hovered = true; }
                }
            }
            UiEvent::PointerDown { position, button: PointerButton::Primary } => {
                let target = self.hit_test(*position).and_then(|id| self.nearest_interactive(id));
                self.pointer_capture = target;
                self.set_focus(target, &mut messages);
                if let Some(id) = target {
                    self.set_pressed(id, true);
                }
            }
            UiEvent::PointerUp { position, button: PointerButton::Primary } => {
                let captured = self.pointer_capture.take();
                let target = self.hit_test(*position).and_then(|id| self.nearest_interactive(id));
                if let Some(id) = captured {
                    self.set_pressed(id, false);
                    if target == Some(id) {
                        self.activate_at(id, *position, &mut messages);
                    }
                }
            }
            UiEvent::Scroll { position, delta_x, delta_y } => {
                if let Some(hit) = self.hit_test(*position) {
                    self.apply_scroll(hit, *delta_x, *delta_y, &mut messages);
                }
            }
            UiEvent::KeyDown { key: Key::Tab, modifiers } => self.move_focus(modifiers.shift, &mut messages),
            UiEvent::KeyDown { key, .. } => {
                if let Some(id) = self.focused {
                    self.handle_focused_key(id, *key, &mut messages);
                }
            }
            UiEvent::TextInput(text) => {
                if let Some(id) = self.focused {
                    self.insert_text(id, text, &mut messages);
                }
            }
            UiEvent::PointerDown { .. } | UiEvent::PointerUp { .. } | UiEvent::KeyUp { .. } => {}
        }
        messages
    }

    fn set_pressed(&mut self, id: WidgetId, pressed: bool) {
        let Some(node) = self.node_mut(id) else { return; };
        match &mut node.kind {
            WidgetKind::Button { pressed: state, .. } | WidgetKind::Toggle { pressed: state, .. } | WidgetKind::Checkbox { pressed: state, .. } => *state = pressed,
            _ => {}
        }
    }

    fn activate_at(&mut self, id: WidgetId, position: LogicalPoint, messages: &mut Vec<UiMessage>) {
        let Some(node) = self.node_mut(id) else { return; };
        match &mut node.kind {
            WidgetKind::Button { .. } => messages.push(UiMessage::Clicked(id)),
            WidgetKind::Toggle { value, .. } => { *value = !*value; messages.push(UiMessage::Toggled { id, value: *value }); }
            WidgetKind::Checkbox { checked, .. } => { *checked = !*checked; messages.push(UiMessage::Checked { id, checked: *checked }); }
            WidgetKind::List { items, selected, offset } => {
                let relative = (position.y - node.rect.y + *offset).max(0.0);
                let index = (relative / LIST_ROW_HEIGHT) as usize;
                if index < items.len() { *selected = Some(index); messages.push(UiMessage::ListSelectionChanged { id, selected: index }); }
            }
            WidgetKind::Menu { items, selected, open } if *open => {
                let index = ((position.y - node.rect.y).max(0.0) / LIST_ROW_HEIGHT) as usize;
                if index < items.len() { *selected = Some(index); *open = false; messages.push(UiMessage::MenuItemActivated { id, index }); }
            }
            WidgetKind::Tabs { labels, active } if !labels.is_empty() => {
                let segment = node.rect.width / labels.len() as f32;
                if segment > 0.0 {
                    let index = (((position.x - node.rect.x).max(0.0) / segment) as usize).min(labels.len() - 1);
                    *active = index;
                    messages.push(UiMessage::TabChanged { id, active: index });
                }
            }
            _ => {}
        }
    }

    fn apply_scroll(&mut self, mut id: WidgetId, delta_x: f32, delta_y: f32, messages: &mut Vec<UiMessage>) {
        loop {
            let parent = self.node(id).and_then(|node| node.parent);
            if let Some(node) = self.node_mut(id) {
                match &mut node.kind {
                    WidgetKind::Scroll { offset_x, offset_y } => {
                        *offset_x = (*offset_x + delta_x).max(0.0);
                        *offset_y = (*offset_y + delta_y).max(0.0);
                        messages.push(UiMessage::ScrollChanged { id, x: *offset_x, y: *offset_y });
                        return;
                    }
                    WidgetKind::List { offset, .. } => {
                        *offset = (*offset + delta_y).max(0.0);
                        messages.push(UiMessage::ScrollChanged { id, x: 0.0, y: *offset });
                        return;
                    }
                    _ => {}
                }
            }
            let Some(next) = parent else { return; };
            id = next;
        }
    }

    fn handle_focused_key(&mut self, id: WidgetId, key: Key, messages: &mut Vec<UiMessage>) {
        let Some(node) = self.node_mut(id) else { return; };
        match (&mut node.kind, key) {
            (WidgetKind::Button { .. }, Key::Enter | Key::Space) => messages.push(UiMessage::Clicked(id)),
            (WidgetKind::Toggle { value, .. }, Key::Enter | Key::Space) => { *value = !*value; messages.push(UiMessage::Toggled { id, value: *value }); }
            (WidgetKind::Checkbox { checked, .. }, Key::Enter | Key::Space) => { *checked = !*checked; messages.push(UiMessage::Checked { id, checked: *checked }); }
            (WidgetKind::TextField { text, cursor, .. }, Key::ArrowLeft) => *cursor = previous_boundary(text, *cursor),
            (WidgetKind::TextField { text, cursor, .. }, Key::ArrowRight) => *cursor = next_boundary(text, *cursor),
            (WidgetKind::TextField { cursor, .. }, Key::Home) => *cursor = 0,
            (WidgetKind::TextField { text, cursor, .. }, Key::End) => *cursor = text.len(),
            (WidgetKind::TextField { text, cursor, .. }, Key::Backspace) => {
                let previous = previous_boundary(text, *cursor);
                if previous < *cursor { text.replace_range(previous..*cursor, ""); *cursor = previous; messages.push(UiMessage::TextChanged { id, text: text.clone() }); }
            }
            (WidgetKind::TextField { text, cursor, .. }, Key::Delete) => {
                let next = next_boundary(text, *cursor);
                if next > *cursor { text.replace_range(*cursor..next, ""); messages.push(UiMessage::TextChanged { id, text: text.clone() }); }
            }
            (WidgetKind::TextField { text, .. }, Key::Enter) => messages.push(UiMessage::Submitted { id, text: text.clone() }),
            (WidgetKind::List { items, selected, .. }, Key::ArrowDown) if !items.is_empty() => {
                let index = selected.map_or(0, |value| (value + 1).min(items.len() - 1)); *selected = Some(index); messages.push(UiMessage::ListSelectionChanged { id, selected: index });
            }
            (WidgetKind::List { items, selected, .. }, Key::ArrowUp) if !items.is_empty() => {
                let index = selected.map_or(0, |value| value.saturating_sub(1)); *selected = Some(index); messages.push(UiMessage::ListSelectionChanged { id, selected: index });
            }
            (WidgetKind::Tabs { labels, active }, Key::ArrowRight) if !labels.is_empty() => { *active = (*active + 1) % labels.len(); messages.push(UiMessage::TabChanged { id, active: *active }); }
            (WidgetKind::Tabs { labels, active }, Key::ArrowLeft) if !labels.is_empty() => { *active = if *active == 0 { labels.len() - 1 } else { *active - 1 }; messages.push(UiMessage::TabChanged { id, active: *active }); }
            (WidgetKind::Popup { open }, Key::Escape) if *open => { *open = false; messages.push(UiMessage::PopupDismissed(id)); }
            _ => {}
        }
    }

    fn insert_text(&mut self, id: WidgetId, inserted: &str, messages: &mut Vec<UiMessage>) {
        let Some(node) = self.node_mut(id) else { return; };
        if let WidgetKind::TextField { text, cursor, .. } = &mut node.kind {
            let safe_cursor = (*cursor).min(text.len());
            let safe_cursor = if text.is_char_boundary(safe_cursor) { safe_cursor } else { previous_boundary(text, safe_cursor) };
            text.insert_str(safe_cursor, inserted);
            *cursor = safe_cursor + inserted.len();
            messages.push(UiMessage::TextChanged { id, text: text.clone() });
        }
    }

    /// Paints the retained tree into renderer-neutral commands. There are no
    /// animation/fade/shadow commands by design.
    pub fn paint(&self, theme: &Theme) -> DisplayList {
        let mut list = DisplayList::new();
        list.push(DrawCommand::Clear(theme.palette.background));
        if let Some(root) = self.root {
            self.paint_node(root, theme, &mut list);
        }
        list
    }

    fn text_style(theme: &Theme, muted: bool) -> TextStyle {
        TextStyle::new(
            theme.typography.family.clone(),
            theme.typography.body_size,
            theme.typography.body_size * theme.typography.line_height,
            if muted { theme.palette.text_muted } else { theme.palette.text },
        )
    }

    fn paint_node(&self, id: WidgetId, theme: &Theme, list: &mut DisplayList) {
        let Some(node) = self.node(id) else { return; };
        if !node.visible { return; }
        let focused = self.focused == Some(id);
        let padding = Insets::uniform(theme.metrics.padding);
        let content = padding.shrink(node.rect);
        let normal_text = Self::text_style(theme, false);
        match &node.kind {
            WidgetKind::Container { .. } | WidgetKind::Spacer => {}
            WidgetKind::Label { text } => list.push(DrawCommand::Text { rect: node.rect, text: text.clone(), style: normal_text }),
            WidgetKind::Button { label, pressed } => {
                let background = if *pressed { theme.palette.accent } else if node.hovered { theme.palette.surface_alt } else { theme.palette.surface };
                list.push(DrawCommand::FillRect { rect: node.rect, color: background });
                list.push(DrawCommand::StrokeRect { rect: node.rect, color: if focused { theme.palette.accent } else { theme.palette.border }, width: if focused { theme.metrics.focus_width } else { theme.metrics.border_width } });
                list.push(DrawCommand::Text { rect: content, text: label.clone(), style: normal_text });
            }
            WidgetKind::Toggle { label, value, .. } => {
                list.push(DrawCommand::FillRect { rect: node.rect, color: theme.palette.surface });
                let indicator = LogicalRect::new(node.rect.x + theme.metrics.padding, node.rect.y + (node.rect.height - 16.0) / 2.0, 30.0, 16.0);
                list.push(DrawCommand::FillRect { rect: indicator, color: if *value { theme.palette.accent } else { theme.palette.surface_alt } });
                list.push(DrawCommand::StrokeRect { rect: node.rect, color: if focused { theme.palette.accent } else { theme.palette.border }, width: theme.metrics.border_width });
                let text_rect = LogicalRect::new(content.x + 36.0, content.y, (content.width - 36.0).max(0.0), content.height);
                list.push(DrawCommand::Text { rect: text_rect, text: label.clone(), style: normal_text });
            }
            WidgetKind::Checkbox { label, checked, .. } => {
                let square = LogicalRect::new(node.rect.x + theme.metrics.padding, node.rect.y + (node.rect.height - 16.0) / 2.0, 16.0, 16.0);
                list.push(DrawCommand::FillRect { rect: square, color: if *checked { theme.palette.accent } else { theme.palette.surface } });
                list.push(DrawCommand::StrokeRect { rect: square, color: if focused { theme.palette.accent } else { theme.palette.border }, width: theme.metrics.border_width });
                let text_rect = LogicalRect::new(content.x + 22.0, content.y, (content.width - 22.0).max(0.0), content.height);
                list.push(DrawCommand::Text { rect: text_rect, text: label.clone(), style: normal_text });
            }
            WidgetKind::TextField { text, placeholder, .. } => {
                list.push(DrawCommand::FillRect { rect: node.rect, color: theme.palette.surface });
                list.push(DrawCommand::StrokeRect { rect: node.rect, color: if focused { theme.palette.accent } else { theme.palette.border }, width: if focused { theme.metrics.focus_width } else { theme.metrics.border_width } });
                let (value, muted) = if text.is_empty() { (placeholder, true) } else { (text, false) };
                list.push(DrawCommand::Text { rect: content, text: value.clone(), style: Self::text_style(theme, muted) });
            }
            WidgetKind::List { items, selected, offset } => {
                list.push(DrawCommand::FillRect { rect: node.rect, color: theme.palette.surface });
                list.push(DrawCommand::PushClip(node.rect));
                for (index, item) in items.iter().enumerate() {
                    let y = node.rect.y + index as f32 * LIST_ROW_HEIGHT - *offset;
                    let row = LogicalRect::new(node.rect.x, y, node.rect.width, LIST_ROW_HEIGHT);
                    if *selected == Some(index) { list.push(DrawCommand::FillRect { rect: row, color: theme.palette.selection }); }
                    list.push(DrawCommand::Text { rect: padding.shrink(row), text: item.clone(), style: normal_text.clone() });
                }
                list.push(DrawCommand::PopClip);
                list.push(DrawCommand::StrokeRect { rect: node.rect, color: if focused { theme.palette.accent } else { theme.palette.border }, width: theme.metrics.border_width });
            }
            WidgetKind::Scroll { .. } => {
                list.push(DrawCommand::PushClip(node.rect));
                for child in &node.children { self.paint_node(*child, theme, list); }
                list.push(DrawCommand::PopClip);
                return;
            }
            WidgetKind::Menu { items, selected, open } => if *open {
                list.push(DrawCommand::FillRect { rect: node.rect, color: theme.palette.surface });
                for (index, item) in items.iter().enumerate() {
                    let row = LogicalRect::new(node.rect.x, node.rect.y + index as f32 * LIST_ROW_HEIGHT, node.rect.width, LIST_ROW_HEIGHT);
                    if *selected == Some(index) { list.push(DrawCommand::FillRect { rect: row, color: theme.palette.selection }); }
                    list.push(DrawCommand::Text { rect: padding.shrink(row), text: item.clone(), style: normal_text.clone() });
                }
                list.push(DrawCommand::StrokeRect { rect: node.rect, color: theme.palette.border, width: theme.metrics.border_width });
            },
            WidgetKind::Popup { open } => if *open {
                list.push(DrawCommand::FillRect { rect: node.rect, color: theme.palette.surface });
                list.push(DrawCommand::StrokeRect { rect: node.rect, color: theme.palette.border, width: theme.metrics.border_width });
            },
            WidgetKind::Tabs { labels, active } => {
                if !labels.is_empty() {
                    let width = node.rect.width / labels.len() as f32;
                    for (index, label) in labels.iter().enumerate() {
                        let rect = LogicalRect::new(node.rect.x + width * index as f32, node.rect.y, width, node.rect.height);
                        list.push(DrawCommand::FillRect { rect, color: if *active == index { theme.palette.selection } else { theme.palette.surface } });
                        list.push(DrawCommand::Text { rect: padding.shrink(rect), text: label.clone(), style: normal_text.clone() });
                    }
                    list.push(DrawCommand::StrokeRect { rect: node.rect, color: if focused { theme.palette.accent } else { theme.palette.border }, width: theme.metrics.border_width });
                }
            }
        }

        for child in &node.children {
            self.paint_node(*child, theme, list);
        }
    }

    /// Materializes semantic metadata without binding the UI Core to AT-SPI.
    pub fn accessibility_tree(&self) -> AccessibilityTree {
        let mut tree = AccessibilityTree { root: self.root, nodes: Vec::new() };
        if let Some(root) = self.root {
            self.accessibility_node(root, &mut tree.nodes);
        }
        tree
    }

    fn accessibility_node(&self, id: WidgetId, output: &mut Vec<AccessibilityNode>) {
        let Some(node) = self.node(id) else { return; };
        if !node.visible { return; }
        let (role, label, value, checked, selected) = match &node.kind {
            WidgetKind::Container { .. } | WidgetKind::Spacer => (AccessibilityRole::Group, None, None, None, None),
            WidgetKind::Label { text } => (AccessibilityRole::Label, Some(text.clone()), None, None, None),
            WidgetKind::Button { label, .. } => (AccessibilityRole::Button, Some(label.clone()), None, None, None),
            WidgetKind::Toggle { label, value, .. } => (AccessibilityRole::ToggleButton, Some(label.clone()), None, Some(*value), None),
            WidgetKind::Checkbox { label, checked, .. } => (AccessibilityRole::CheckBox, Some(label.clone()), None, Some(*checked), None),
            WidgetKind::TextField { text, placeholder, .. } => (AccessibilityRole::TextField, Some(placeholder.clone()), Some(text.clone()), None, None),
            WidgetKind::List { selected, .. } => (AccessibilityRole::List, None, selected.map(|value| value.to_string()), None, selected.map(|_| true)),
            WidgetKind::Scroll { .. } => (AccessibilityRole::ScrollArea, None, None, None, None),
            WidgetKind::Menu { selected, .. } => (AccessibilityRole::Menu, None, selected.map(|value| value.to_string()), None, selected.map(|_| true)),
            WidgetKind::Popup { .. } => (AccessibilityRole::Dialog, None, None, None, None),
            WidgetKind::Tabs { active, .. } => (AccessibilityRole::TabList, None, Some(active.to_string()), None, Some(true)),
        };
        output.push(AccessibilityNode {
            id,
            role,
            label,
            value,
            bounds: node.rect,
            focused: self.focused == Some(id),
            disabled: !node.enabled,
            checked,
            selected,
            children: node.children.clone(),
        });
        for child in &node.children { self.accessibility_node(*child, output); }
    }
}

fn previous_boundary(text: &str, cursor: usize) -> usize {
    let cursor = cursor.min(text.len());
    text[..cursor].char_indices().next_back().map_or(0, |(index, _)| index)
}

fn next_boundary(text: &str, cursor: usize) -> usize {
    let cursor = cursor.min(text.len());
    if cursor >= text.len() { return text.len(); }
    let mut index = cursor;
    while index < text.len() && !text.is_char_boundary(index) { index += 1; }
    text[index..].chars().next().map_or(text.len(), |character| index + character.len_utf8())
}
