use nexxus_tiling::{
    DEFAULT_TILE_FIT_SHORTCUT, LayoutError, LayoutSpec, NormalizedRect, OutputArea, OutputId, Point,
    SnapDetector, SnapIntent, SnapTarget, TILE_FIT_ACTION_ID, TileTarget, TilingEngine, TilingError,
};
use nexxus_wm::{
    Geometry, SizeConstraints, WindowId, WindowManager, WindowMetadata, WindowPlacement, WmEvent,
};
use nexxus_workspaces::WorkspaceManager;

fn window_id(value: u64) -> WindowId {
    WindowId::new(value).unwrap()
}

fn output(value: u64, x: i32, width: u32, height: u32) -> OutputArea {
    OutputArea::new(
        OutputId::new(value).unwrap(),
        Geometry::new(x, 0, width, height).unwrap(),
    )
}

fn create_window(
    wm: &mut WindowManager,
    id: WindowId,
    geometry: Geometry,
    constraints: SizeConstraints,
) {
    wm.apply_event(WmEvent::WindowCreated {
        id,
        geometry,
        constraints,
        metadata: WindowMetadata::default(),
    })
    .unwrap();
}

fn setup_single_window() -> (WorkspaceManager, WindowManager, WindowId) {
    let mut workspaces = WorkspaceManager::with_single_fixed("WORK").unwrap();
    let mut wm = WindowManager::new();
    let window = window_id(1);
    create_window(
        &mut wm,
        window,
        Geometry::new(120, 80, 900, 700).unwrap(),
        SizeConstraints::default(),
    );
    workspaces.assign_new_window(window, None).unwrap();
    (workspaces, wm, window)
}

#[test]
fn approved_tile_fit_action_and_shortcut_are_stable() {
    assert_eq!(TILE_FIT_ACTION_ID, "nexxus.tiling.tile-fit");
    assert_eq!(DEFAULT_TILE_FIT_SHORTCUT, "Super+T");
}

#[test]
fn tile_fit_preserves_and_restores_floating_geometry() {
    let (workspaces, mut wm, window) = setup_single_window();
    let original = wm.window(window).unwrap().geometry;
    let mut engine = TilingEngine::new();

    let plan = engine
        .tile_fit_active(&workspaces, &mut wm, output(1, 0, 1_920, 1_080), window)
        .unwrap();

    assert_eq!(wm.window(window).unwrap().placement, WindowPlacement::Tiled);
    assert_eq!(wm.window(window).unwrap().floating_geometry, original);
    assert_eq!(plan.geometry, Geometry::new(0, 0, 960, 1_080).unwrap());

    wm.apply_event(WmEvent::WindowGeometryChanged {
        id: window,
        geometry: plan.geometry,
    })
    .unwrap();

    let restored = engine.untile(&mut wm, window).unwrap();
    assert_eq!(restored.geometry, original);
    assert_eq!(wm.window(window).unwrap().placement, WindowPlacement::Floating);
    assert_eq!(wm.window(window).unwrap().geometry, original);
}

#[test]
fn manual_move_release_returns_tiled_window_to_floating() {
    let (workspaces, mut wm, window) = setup_single_window();
    let original = wm.window(window).unwrap().geometry;
    let mut engine = TilingEngine::new();

    let plan = engine
        .tile_fit_active(&workspaces, &mut wm, output(1, 0, 1_920, 1_080), window)
        .unwrap();
    wm.apply_event(WmEvent::WindowGeometryChanged {
        id: window,
        geometry: plan.geometry,
    })
    .unwrap();

    let released = engine
        .release_for_manual_operation(&mut wm, window)
        .unwrap()
        .unwrap();
    assert_eq!(released.geometry, original);
    assert_eq!(wm.window(window).unwrap().placement, WindowPlacement::Floating);
}

#[test]
fn different_workspaces_can_use_distinct_layouts() {
    let mut workspaces = WorkspaceManager::with_single_fixed("WEB").unwrap();
    let second = workspaces.create_fixed("MAIL").unwrap();
    let first = workspaces.active_id();

    let mut wm = WindowManager::new();
    let first_window = window_id(1);
    let second_window = window_id(2);
    create_window(
        &mut wm,
        first_window,
        Geometry::new(50, 50, 700, 600).unwrap(),
        SizeConstraints::default(),
    );
    create_window(
        &mut wm,
        second_window,
        Geometry::new(80, 80, 700, 600).unwrap(),
        SizeConstraints::default(),
    );
    workspaces.assign_new_window(first_window, None).unwrap();
    workspaces.activate(second).unwrap();
    workspaces.assign_new_window(second_window, None).unwrap();

    let mut engine = TilingEngine::new();
    engine.set_layout(first, LayoutSpec::balanced_columns(2).unwrap());
    engine.set_layout(second, LayoutSpec::balanced_columns(3).unwrap());

    let first_plan = engine
        .tile_fit(
            &workspaces,
            &mut wm,
            first,
            output(1, 0, 1_800, 900),
            first_window,
        )
        .unwrap();
    let second_plan = engine
        .tile_fit(
            &workspaces,
            &mut wm,
            second,
            output(1, 0, 1_800, 900),
            second_window,
        )
        .unwrap();

    assert_eq!(first_plan.geometry.width, 900);
    assert_eq!(second_plan.geometry.width, 600);
}

#[test]
fn assignments_are_independent_between_outputs() {
    let mut workspaces = WorkspaceManager::with_single_fixed("WORK").unwrap();
    let mut wm = WindowManager::new();
    let first = window_id(1);
    let second = window_id(2);
    for window in [first, second] {
        create_window(
            &mut wm,
            window,
            Geometry::new(100, 100, 600, 500).unwrap(),
            SizeConstraints::default(),
        );
        workspaces.assign_new_window(window, None).unwrap();
    }

    let mut engine = TilingEngine::new();
    let first_plan = engine
        .tile_fit_active(&workspaces, &mut wm, output(1, 0, 1_920, 1_080), first)
        .unwrap();
    let second_plan = engine
        .tile_fit_active(
            &workspaces,
            &mut wm,
            output(2, 1_920, 1_280, 1_024),
            second,
        )
        .unwrap();

    assert_eq!(first_plan.target, TileTarget::LayoutSlot(0));
    assert_eq!(second_plan.target, TileTarget::LayoutSlot(0));
    assert_eq!(second_plan.geometry.x, 1_920);
    assert_eq!(second_plan.geometry.width, 640);
}

#[test]
fn max_size_is_centered_inside_the_slot() {
    let (workspaces, mut wm, window) = setup_single_window();
    wm.apply_event(WmEvent::WindowDestroyed { id: window }).unwrap();

    create_window(
        &mut wm,
        window,
        Geometry::new(120, 80, 700, 500).unwrap(),
        SizeConstraints {
            min_width: 100,
            min_height: 100,
            max_width: Some(500),
            max_height: Some(400),
        },
    );

    let mut engine = TilingEngine::new();
    let plan = engine
        .tile_fit_active(&workspaces, &mut wm, output(1, 0, 2_000, 1_000), window)
        .unwrap();

    assert_eq!(plan.geometry, Geometry::new(250, 300, 500, 400).unwrap());
}

#[test]
fn impossible_minimum_size_is_rejected_before_placement_changes() {
    let mut workspaces = WorkspaceManager::with_single_fixed("WORK").unwrap();
    let mut wm = WindowManager::new();
    let window = window_id(1);
    create_window(
        &mut wm,
        window,
        Geometry::new(10, 10, 900, 700).unwrap(),
        SizeConstraints {
            min_width: 1_200,
            min_height: 200,
            max_width: None,
            max_height: None,
        },
    );
    workspaces.assign_new_window(window, None).unwrap();

    let mut engine = TilingEngine::new();
    let error = engine
        .tile_fit_active(&workspaces, &mut wm, output(1, 0, 1_920, 1_080), window)
        .unwrap_err();

    assert!(matches!(
        error,
        TilingError::Layout(LayoutError::MinimumSizeDoesNotFit)
    ));
    assert_eq!(wm.window(window).unwrap().placement, WindowPlacement::Floating);
}

#[test]
fn occupied_slots_are_selected_deterministically_and_then_exhausted() {
    let mut workspaces = WorkspaceManager::with_single_fixed("WORK").unwrap();
    let mut wm = WindowManager::new();
    let mut engine = TilingEngine::new();

    for raw in 1..=3 {
        let window = window_id(raw);
        create_window(
            &mut wm,
            window,
            Geometry::new(10, 10, 500, 400).unwrap(),
            SizeConstraints::default(),
        );
        workspaces.assign_new_window(window, None).unwrap();
    }

    let first = engine
        .tile_fit_active(
            &workspaces,
            &mut wm,
            output(1, 0, 1_600, 900),
            window_id(1),
        )
        .unwrap();
    let second = engine
        .tile_fit_active(
            &workspaces,
            &mut wm,
            output(1, 0, 1_600, 900),
            window_id(2),
        )
        .unwrap();
    let third = engine
        .tile_fit_active(
            &workspaces,
            &mut wm,
            output(1, 0, 1_600, 900),
            window_id(3),
        )
        .unwrap_err();

    assert_eq!(first.target, TileTarget::LayoutSlot(0));
    assert_eq!(second.target, TileTarget::LayoutSlot(1));
    assert!(matches!(third, TilingError::NoAvailableSlot { .. }));
}

#[test]
fn snap_detector_produces_direct_and_overlay_hooks() {
    let area = output(1, 0, 1_920, 1_080);
    let detector = SnapDetector::default();

    assert!(matches!(
        detector.detect(area, Point { x: 0, y: 500 }),
        Some(SnapIntent::Tile {
            target: SnapTarget::LeftHalf,
            ..
        })
    ));
    assert_eq!(
        detector.detect(area, Point { x: 960, y: 0 }),
        Some(SnapIntent::ShowLayoutChoices)
    );
}

#[test]
fn snap_applies_geometry_without_erasing_floating_restore_point() {
    let (workspaces, mut wm, window) = setup_single_window();
    let original = wm.window(window).unwrap().geometry;
    let mut engine = TilingEngine::new();
    let area = output(1, -1_280, 1_280, 1_024);
    let intent = SnapIntent::Tile {
        target: SnapTarget::RightHalf,
        slot: NormalizedRect::new(5_000, 0, 5_000, 10_000).unwrap(),
    };

    let plan = engine
        .apply_snap(intent, &workspaces, &mut wm, area, window)
        .unwrap()
        .unwrap();

    assert_eq!(plan.geometry, Geometry::new(-640, 0, 640, 1_024).unwrap());
    assert_eq!(wm.window(window).unwrap().floating_geometry, original);
}
