//! Development harness for visually inspecting the Stage 07 widget primitives.

use std::error::Error;
use std::path::PathBuf;

use nexxus_ui::{Axis, LogicalPoint, LogicalRect, LogicalSize, Modifiers, PointerButton, Renderer, ScaleFactor, SoftwareRenderer, Theme, UiEvent, UiTree, WidgetKind};

fn main() -> Result<(), Box<dyn Error>> {
    let theme = Theme::default();
    theme.validate()?;

    let mut ui = UiTree::new();
    let root = ui.insert(WidgetKind::Container { axis: Axis::Vertical, gap: theme.metrics.gap });
    let title = ui.insert(WidgetKind::Label { text: "NEXXUS UI CORE".into() });
    let button = ui.insert(WidgetKind::Button { label: "Button".into(), pressed: false });
    let toggle = ui.insert(WidgetKind::Toggle { label: "Tiling enabled".into(), value: true, pressed: false });
    let checkbox = ui.insert(WidgetKind::Checkbox { label: "Remember choice".into(), checked: true, pressed: false });
    let field = ui.insert(WidgetKind::TextField { text: "Hack / Unicode: Olá Nexxus".into(), placeholder: "Type here".into(), cursor: 27 });
    let tabs = ui.insert(WidgetKind::Tabs { labels: vec!["GENERAL".into(), "DISPLAY".into(), "INPUT".into()], active: 0 });
    let list = ui.insert(WidgetKind::List { items: vec!["WEB".into(), "MAIL".into(), "FILES".into(), "DEV".into()], selected: Some(0), offset: 0.0 });

    for child in [title, button, toggle, checkbox, field, tabs, list] {
        assert!(ui.add_child(root, child), "demo tree must remain acyclic");
    }

    // Drive the same normalized event API a real X11/Wayland adapter uses.
    ui.layout(LogicalRect::new(0.0, 0.0, 640.0, 420.0), &theme);
    let button_center = ui.node(button).map(|node| LogicalPoint::new(node.rect.x + 12.0, node.rect.y + 12.0)).expect("demo button exists");
    let _ = ui.handle_event(&UiEvent::PointerMove { position: button_center });
    let _ = ui.handle_event(&UiEvent::PointerDown { position: button_center, button: PointerButton::Primary });
    let _ = ui.handle_event(&UiEvent::PointerUp { position: button_center, button: PointerButton::Primary });
    let _ = ui.handle_event(&UiEvent::KeyDown { key: nexxus_ui::Key::Tab, modifiers: Modifiers::default() });

    let display_list = ui.paint(&theme);
    let mut renderer = SoftwareRenderer::new();
    let scale = ScaleFactor::new(1.0).expect("unit scale is valid");
    let frame = renderer.render(&display_list, LogicalSize::new(640.0, 420.0), scale)?;

    let output = std::env::args_os().nth(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("nexxus-ui-demo.ppm"));
    frame.save_ppm(&output)?;
    println!("Nexxus UI demo: {}x{} RGBA -> {}", frame.size.width, frame.size.height, output.display());
    Ok(())
}
