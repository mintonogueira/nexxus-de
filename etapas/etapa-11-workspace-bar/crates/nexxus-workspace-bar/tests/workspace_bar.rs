use std::path::PathBuf;

use nexxus_ui::{LogicalPoint, LogicalRect, ScaleFactor, Theme};
use nexxus_workspaces::{WorkspaceEvent, WorkspaceManager};
use nexxus_workspace_bar::{
    AssetSource, InteractionState, MonitorGeometry, WorkspaceBarAction, WorkspaceBarLayout,
    WorkspaceBarMetrics, WorkspaceBarModel, WorkspaceBarPainter, WorkspaceBarTarget,
    WorkspaceBarVisualState,
};

#[test]
fn snapshot_preserves_manager_order_and_active_workspace() {
    let mut manager = WorkspaceManager::with_single_fixed("WEB").unwrap();
    let mail = manager.create_fixed("MAIL").unwrap();
    manager.create_dynamic("TEMP").unwrap();
    manager.activate(mail).unwrap();
    let model = WorkspaceBarModel::from_manager(&manager);
    let names: Vec<_> = model.entries().iter().map(|entry| entry.name.as_str()).collect();
    assert_eq!(names, ["WEB", "MAIL", "TEMP"]);
    assert_eq!(model.active_id(), Some(mail));
}

#[test]
fn incremental_events_update_visible_state_only() {
    let mut manager = WorkspaceManager::with_single_fixed("WEB").unwrap();
    let mut model = WorkspaceBarModel::from_manager(&manager);
    let mail = manager.create_fixed("MAIL").unwrap();
    assert!(model.apply_event(&WorkspaceEvent::Created {
        workspace: mail,
        name: "MAIL".into(),
        kind: nexxus_workspaces::WorkspaceKind::Fixed,
    }));
    assert!(model.apply_event(&WorkspaceEvent::Renamed {
        workspace: mail,
        old_name: "MAIL".into(),
        new_name: "CORREIO".into(),
    }));
    let previous = manager.active_id();
    assert!(model.apply_event(&WorkspaceEvent::Activated { previous, current: mail }));
    assert_eq!(model.active_id(), Some(mail));
    assert_eq!(model.entries()[1].name, "CORREIO");
}

#[test]
fn layout_is_centered_only_on_primary_monitor() {
    let scale = ScaleFactor::new(1.0).unwrap();
    let secondary = MonitorGeometry {
        rect: LogicalRect::new(0.0, 0.0, 1280.0, 1024.0),
        scale,
        primary: false,
    };
    let primary = MonitorGeometry {
        rect: LogicalRect::new(1280.0, 0.0, 1920.0, 1080.0),
        scale,
        primary: true,
    };
    let id = nexxus_workspaces::WorkspaceId::new(1).unwrap();
    let layout = WorkspaceBarLayout::build(
        &[secondary, primary],
        &[(id, "WEB")],
        WorkspaceBarMetrics::default(),
    )
    .unwrap();
    assert!(layout.window.x >= 1280.0);
    assert!(layout.window.x + layout.window.width <= 3200.0);
    assert_eq!(layout.window.y, 8.0);
}

#[test]
fn pointer_release_emits_workspace_or_settings_action() {
    let scale = ScaleFactor::new(1.0).unwrap();
    let monitor = MonitorGeometry {
        rect: LogicalRect::new(0.0, 0.0, 1920.0, 1080.0),
        scale,
        primary: true,
    };
    let id = nexxus_workspaces::WorkspaceId::new(1).unwrap();
    let layout = WorkspaceBarLayout::build(&[monitor], &[(id, "WEB")], WorkspaceBarMetrics::default()).unwrap();
    let mut state = InteractionState::default();
    let button = layout.workspaces[0].rect;
    let point = LogicalPoint::new(button.x + 2.0, button.y + 2.0);
    state.pointer_press(&layout, point);
    assert_eq!(state.pointer_release(&layout, point), Some(WorkspaceBarAction::Activate(id)));

    let settings = LogicalPoint::new(layout.settings.x + 2.0, layout.settings.y + 2.0);
    state.pointer_press(&layout, settings);
    assert_eq!(state.hovered, None);
    assert_eq!(state.pointer_release(&layout, settings), Some(WorkspaceBarAction::OpenWorkspaceSettings));
    assert_eq!(InteractionState::hit_test(&layout, settings), Some(WorkspaceBarTarget::Settings));
}

#[test]
fn painter_renders_hidpi_frame_with_workspace_asset() {
    let manager = WorkspaceManager::with_single_fixed("WEB").unwrap();
    let model = WorkspaceBarModel::from_manager(&manager);
    let scale = ScaleFactor::new(1.25).unwrap();
    let monitor = MonitorGeometry {
        rect: LogicalRect::new(0.0, 0.0, 1536.0, 864.0),
        scale,
        primary: true,
    };
    let labels: Vec<_> = model.entries().iter().map(|entry| (entry.id, entry.name.as_str())).collect();
    let metrics = WorkspaceBarMetrics::default();
    let layout = WorkspaceBarLayout::build(&[monitor], &labels, metrics).unwrap();
    let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../etapa-08-visual-assets/assets");
    let mut painter = WorkspaceBarPainter::new(Theme::default(), metrics, AssetSource::new(asset_root));
    let frame = painter
        .render(&model, &layout, WorkspaceBarVisualState::default(), scale)
        .unwrap();
    assert_eq!(frame.size, scale.physical_size(layout.window.size()));
    assert_eq!(frame.pixels.len(), frame.size.width as usize * frame.size.height as usize * 4);
}
