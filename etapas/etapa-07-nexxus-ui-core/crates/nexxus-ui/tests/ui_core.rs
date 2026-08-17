use nexxus_ui::{AccessibilityRole, Axis, Color, DisplayList, DrawCommand, FlexItem, Key, LogicalPoint, LogicalRect, LogicalSize, Modifiers, PointerButton, Renderer, ScaleFactor, SoftwareRenderer, Theme, UiEvent, UiMessage, UiTree, WidgetId, WidgetKind, layout_flex};

#[test]
fn fractional_scale_preserves_shared_edges() {
    let scale = ScaleFactor::new(1.25).expect("valid scale");
    let left = scale.physical_rect(LogicalRect::new(0.0, 0.0, 33.3, 20.0));
    let right = scale.physical_rect(LogicalRect::new(33.3, 0.0, 66.7, 20.0));
    assert_eq!(left.x + left.width as i32, right.x);
    assert_eq!(right.x + right.width as i32, 125);
}

#[test]
fn flex_layout_is_deterministic_and_fills_available_space() {
    let items = [FlexItem::flexible(WidgetId(0)), FlexItem::flexible(WidgetId(1)), FlexItem::flexible(WidgetId(2))];
    let result = layout_flex(LogicalRect::new(0.0, 0.0, 100.0, 20.0), Axis::Horizontal, 0.0, &items);
    assert_eq!(result.len(), 3);
    let last = result.last().expect("last flex item").1;
    assert!(((last.x + last.width) - 100.0).abs() < 0.001);
}

#[test]
fn button_pointer_activation_and_focus_are_semantic() {
    let theme = Theme::default();
    let mut ui = UiTree::new();
    let root = ui.insert(WidgetKind::Container { axis: Axis::Vertical, gap: 0.0 });
    let button = ui.insert(WidgetKind::Button { label: "Apply".into(), pressed: false });
    assert!(ui.add_child(root, button));
    ui.layout(LogicalRect::new(0.0, 0.0, 200.0, 40.0), &theme);
    let point = LogicalPoint::new(10.0, 10.0);

    let down = ui.handle_event(&UiEvent::PointerDown { position: point, button: PointerButton::Primary });
    assert!(down.contains(&UiMessage::FocusChanged(Some(button))));
    let up = ui.handle_event(&UiEvent::PointerUp { position: point, button: PointerButton::Primary });
    assert!(up.contains(&UiMessage::Clicked(button)));
}

#[test]
fn text_field_edits_utf8_on_character_boundaries() {
    let theme = Theme::default();
    let mut ui = UiTree::new();
    let field = ui.insert(WidgetKind::TextField { text: "Aé".into(), placeholder: String::new(), cursor: "Aé".len() });
    ui.layout(LogicalRect::new(0.0, 0.0, 200.0, 32.0), &theme);
    let point = LogicalPoint::new(4.0, 4.0);
    let _ = ui.handle_event(&UiEvent::PointerDown { position: point, button: PointerButton::Primary });
    let messages = ui.handle_event(&UiEvent::KeyDown { key: Key::Backspace, modifiers: Modifiers::default() });
    assert!(messages.contains(&UiMessage::TextChanged { id: field, text: "A".into() }));
    match &ui.node(field).expect("field exists").kind {
        WidgetKind::TextField { text, cursor, .. } => { assert_eq!(text, "A"); assert_eq!(*cursor, 1); }
        _ => panic!("wrong widget kind"),
    }
}

#[test]
fn accessibility_tree_exposes_widget_semantics() {
    let theme = Theme::default();
    let mut ui = UiTree::new();
    let root = ui.insert(WidgetKind::Container { axis: Axis::Vertical, gap: 0.0 });
    let checkbox = ui.insert(WidgetKind::Checkbox { label: "Secure".into(), checked: true, pressed: false });
    assert!(ui.add_child(root, checkbox));
    ui.layout(LogicalRect::new(0.0, 0.0, 120.0, 32.0), &theme);
    let tree = ui.accessibility_tree();
    let node = tree.nodes.iter().find(|node| node.id == checkbox).expect("checkbox semantics");
    assert_eq!(node.role, AccessibilityRole::CheckBox);
    assert_eq!(node.checked, Some(true));
    assert_eq!(node.label.as_deref(), Some("Secure"));
}

#[test]
fn theme_rejects_translucent_structural_surface() {
    let mut theme = Theme::default();
    theme.palette.surface = Color::rgba(10, 10, 10, 200);
    assert!(theme.validate().is_err());
}

#[test]
fn software_renderer_draws_svg_into_backend_neutral_frame() {
    let mut list = DisplayList::new();
    list.push(DrawCommand::Clear(Color::rgb(0, 0, 0)));
    list.push(DrawCommand::Svg {
        rect: LogicalRect::new(0.0, 0.0, 10.0, 10.0),
        bytes: br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10" fill="#00ff00"/></svg>"#.to_vec(),
    });
    let mut renderer = SoftwareRenderer::new();
    let frame = renderer.render(&list, LogicalSize::new(10.0, 10.0), ScaleFactor::default()).expect("SVG renders");
    let center = 5usize * frame.stride + 5usize * 4;
    assert!(frame.pixels[center + 1] > 200);
    assert_eq!(frame.pixels[center + 3], 255);
}

#[test]
fn unmatched_pop_clip_fails_safely() {
    let mut list = DisplayList::new();
    list.push(DrawCommand::PopClip);
    let mut renderer = SoftwareRenderer::new();
    assert!(renderer.render(&list, LogicalSize::new(10.0, 10.0), ScaleFactor::default()).is_err());
}
