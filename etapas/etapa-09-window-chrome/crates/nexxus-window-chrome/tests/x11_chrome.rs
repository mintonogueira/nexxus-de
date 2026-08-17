use std::collections::BTreeSet;
use std::thread;
use std::time::Duration;

use nexxus_backend_x11::X11Service;
use nexxus_ui::{ScaleFactor, Theme};
use nexxus_window_chrome::{AssetSource, NoopChromeHooks, X11ChromeAdapter, maximize_restore};
use nexxus_wm::{PresentationState, WindowId};
use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ConnectionExt as _, CreateWindowAux, EventMask, PropMode, WindowClass,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

fn create_client(conn: &RustConnection, screen_num: usize, csd: bool, x: i16) -> u32 {
    let screen = &conn.setup().roots[screen_num];
    let window = conn.generate_id().unwrap();
    conn.create_window(
        COPY_DEPTH_FROM_PARENT,
        window,
        screen.root,
        x,
        120,
        420,
        280,
        0,
        WindowClass::INPUT_OUTPUT,
        0,
        &CreateWindowAux::new().event_mask(EventMask::STRUCTURE_NOTIFY),
    )
    .unwrap();
    if csd {
        let gtk = conn
            .intern_atom(false, b"_GTK_FRAME_EXTENTS")
            .unwrap()
            .reply()
            .unwrap()
            .atom;
        conn.change_property32(
            PropMode::REPLACE,
            window,
            gtk,
            AtomEnum::CARDINAL,
            &[1, 1, 24, 1],
        )
        .unwrap();
    }
    conn.map_window(window).unwrap();
    conn.flush().unwrap();
    window
}

fn wait_for_windows(controller: &nexxus_backend_x11::X11Controller, ids: &[u32]) {
    for _ in 0..100 {
        let present: BTreeSet<u64> = controller
            .windows()
            .unwrap()
            .into_iter()
            .map(|window| window.id.get())
            .collect();
        if ids.iter().all(|id| present.contains(&u64::from(*id))) {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("X11 backend did not observe test clients in time");
}

fn stage08_icons() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../etapa-08-visual-assets/assets/icons")
}

fn window_snapshot(
    controller: &nexxus_backend_x11::X11Controller,
    id: WindowId,
) -> nexxus_wm::Window {
    controller
        .windows()
        .unwrap()
        .into_iter()
        .find(|window| window.id == id)
        .expect("test window must remain registered")
}

#[test]
fn x11_adapter_decorates_ssd_skips_csd_and_preserves_restore_geometry() {
    let mut backend = X11Service::start(None).unwrap();
    let controller = backend.controller();
    let (clients, client_screen) = x11rb::connect(None).unwrap();
    let ssd = create_client(&clients, client_screen, false, 160);
    let csd = create_client(&clients, client_screen, true, 680);
    wait_for_windows(&controller, &[ssd, csd]);

    let assets = AssetSource::new(stage08_icons());
    let mut chrome = X11ChromeAdapter::connect(
        None,
        controller.clone(),
        ScaleFactor::default(),
        Theme::default(),
        assets,
    )
    .unwrap();
    chrome.sync().unwrap();
    chrome.poll(&mut NoopChromeHooks).unwrap();

    let decorated: BTreeSet<u64> = chrome.decorated_windows().map(|id| id.get()).collect();
    assert!(decorated.contains(&u64::from(ssd)));
    assert!(!decorated.contains(&u64::from(csd)));

    let ssd_id = WindowId::new(u64::from(ssd)).unwrap();
    let initial = window_snapshot(&controller, ssd_id).geometry;
    maximize_restore(&controller, ssd_id).unwrap();
    assert_eq!(
        window_snapshot(&controller, ssd_id).presentation,
        PresentationState::Maximized
    );
    maximize_restore(&controller, ssd_id).unwrap();
    let restored = window_snapshot(&controller, ssd_id);
    assert_eq!(restored.presentation, PresentationState::Normal);
    assert_eq!(restored.geometry, initial);

    let (probe, _) = x11rb::connect(None).unwrap();
    let frame_atom = probe
        .intern_atom(false, b"_NET_FRAME_EXTENTS")
        .unwrap()
        .reply()
        .unwrap()
        .atom;
    let ssd_extents: Vec<u32> = probe
        .get_property(false, ssd, frame_atom, AtomEnum::CARDINAL, 0, 4)
        .unwrap()
        .reply()
        .unwrap()
        .value32()
        .unwrap()
        .collect();
    assert_eq!(ssd_extents, vec![2, 2, 32, 2]);

    let csd_reply = probe
        .get_property(false, csd, frame_atom, AtomEnum::CARDINAL, 0, 4)
        .unwrap()
        .reply()
        .unwrap();
    assert_eq!(csd_reply.type_, 0);

    drop(chrome);
    drop(probe);
    drop(clients);
    let _ = backend.stop();
}
