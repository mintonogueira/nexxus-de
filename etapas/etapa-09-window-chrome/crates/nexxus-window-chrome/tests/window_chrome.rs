use nexxus_backend_api::BackendError;
use nexxus_tiling::{OutputArea, OutputId, TilingEngine};
use nexxus_ui::{LogicalPoint, ScaleFactor, Theme};
use nexxus_window_chrome::{
    AssetSource, ChromeButton, ChromeMetrics, ChromePainter, DecorationDecision, DecorationHints,
    HitTarget, ResizeEdge, TitlebarLayout, WindowType, decide_decoration,
    release_for_manual_operation, resized_geometry, tile_fit,
};
use nexxus_wm::{
    BackendCommandSink, Geometry, SizeConstraints, WindowId, WindowManager, WindowMetadata, WmCommand,
    WmEvent, WindowPlacement,
};
use nexxus_workspaces::WorkspaceManager;

struct RecordingSink(Vec<WmCommand>);

impl BackendCommandSink for RecordingSink {
    fn submit(&mut self, command: &WmCommand) -> Result<(), BackendError> {
        self.0.push(command.clone());
        Ok(())
    }
}

fn id(value: u64) -> WindowId {
    WindowId::new(value).unwrap()
}

#[test]
fn csd_signals_prevent_double_decoration() {
    assert_eq!(
        decide_decoration(DecorationHints {
            gtk_frame_extents: true,
            ..DecorationHints::default()
        }),
        DecorationDecision::ClientSide
    );
    assert_eq!(
        decide_decoration(DecorationHints {
            motif_decorations_disabled: true,
            ..DecorationHints::default()
        }),
        DecorationDecision::ClientSide
    );
    assert_eq!(
        decide_decoration(DecorationHints {
            window_type: WindowType::Dock,
            ..DecorationHints::default()
        }),
        DecorationDecision::None
    );
    assert_eq!(
        decide_decoration(DecorationHints::default()),
        DecorationDecision::ServerSide
    );
}

#[test]
fn titlebar_buttons_have_priority_over_drag_region() {
    let layout = TitlebarLayout::new(600.0, ChromeMetrics::default());
    assert_eq!(
        layout.hit_test(LogicalPoint::new(590.0, 16.0)),
        HitTarget::Button(ChromeButton::Close)
    );
    assert_eq!(
        layout.hit_test(LogicalPoint::new(550.0, 16.0)),
        HitTarget::Button(ChromeButton::MaximizeRestore)
    );
    assert_eq!(
        layout.hit_test(LogicalPoint::new(510.0, 16.0)),
        HitTarget::Button(ChromeButton::TileFit)
    );
    assert_eq!(layout.hit_test(LogicalPoint::new(200.0, 16.0)), HitTarget::Titlebar);
}

#[test]
fn resize_from_left_preserves_opposite_edge_and_minimum() {
    let original = Geometry::new(100, 100, 500, 300).unwrap();
    let resized = resized_geometry(original, ResizeEdge::Left, 480, 0, 120, 80);
    assert_eq!(resized.width, 120);
    assert_eq!(resized.x + resized.width as i32, 600);
}

#[test]
fn frame_extents_scale_to_physical_pixels() {
    let scale = ScaleFactor::new(1.5).unwrap();
    let extents = ChromeMetrics::default().frame_extents(scale);
    assert_eq!(extents.top, 48);
    assert!(extents.left >= 3);
}

#[test]
fn painter_consumes_stage08_window_icons_without_animation_state() {
    let root = std::path::PathBuf::from("../etapa-08-visual-assets/assets");
    let mut painter = ChromePainter::new(Theme::default(), ChromeMetrics::default(), AssetSource::new(root));
    let frame = painter
        .render(
            640.0,
            "Nexxus Window",
            nexxus_window_chrome::ChromeVisualState {
                active: true,
                maximized: false,
                hovered: Some(ChromeButton::TileFit),
                pressed: None,
            },
            ScaleFactor::default(),
        )
        .unwrap();
    assert_eq!(frame.size.height, 32);
    assert_eq!(frame.size.width, 640);
}

#[test]
fn tile_fit_and_manual_release_use_stage06_engine_and_do_not_trap_window() {
    let window = id(7);
    let initial = Geometry::new(100, 120, 700, 500).unwrap();
    let mut wm = WindowManager::new();
    wm.apply_event(WmEvent::WindowCreated {
        id: window,
        geometry: initial,
        constraints: SizeConstraints::default(),
        metadata: WindowMetadata::default(),
    })
    .unwrap();
    let mut workspaces = WorkspaceManager::with_single_fixed("MAIN").unwrap();
    workspaces.assign_new_window(window, None).unwrap();
    let mut engine = TilingEngine::new();
    let area = OutputArea::new(OutputId::new(1).unwrap(), Geometry::new(0, 0, 1920, 1080).unwrap());
    let mut sink = RecordingSink(Vec::new());

    let plan = tile_fit(&mut engine, &workspaces, &mut wm, &mut sink, area, window).unwrap();
    assert_eq!(wm.window(window).unwrap().placement, WindowPlacement::Tiled);
    assert_eq!(sink.0.as_slice(), plan.commands());

    sink.0.clear();
    let release = release_for_manual_operation(&mut engine, &mut wm, &mut sink, window)
        .unwrap()
        .expect("tiled window must be released");
    assert_eq!(wm.window(window).unwrap().placement, WindowPlacement::Floating);
    assert_eq!(wm.window(window).unwrap().geometry, initial);
    assert_eq!(sink.0.as_slice(), release.commands());
}
