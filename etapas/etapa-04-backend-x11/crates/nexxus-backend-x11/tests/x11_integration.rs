//! Real-X-server integration tests. CI provides Xvfb through DISPLAY.

#![forbid(unsafe_code)]

use nexxus_backend_api::BackendKind;
use nexxus_backend_x11::{X11BackendModule, X11Service};
use nexxus_session::{BackendModule, SessionRuntime};
use nexxus_wm::{PresentationState, WindowId};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{
    AtomEnum, ConnectionExt as _, CreateWindowAux, EventMask, PropMode, WindowClass,
};
use x11rb::wrapper::ConnectionExt as _;

fn wait_until(mut predicate: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if predicate() {
            return;
        }
        std::thread::sleep(Duration::from_millis(15));
    }
    panic!("timed out waiting for X11 state transition");
}

fn private_dir() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("nexxus-x11-stage4-{}-{nonce}", std::process::id()));
    fs::create_dir(&path).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    path
}

fn intern<C: Connection>(connection: &C, name: &[u8]) -> u32 {
    connection
        .intern_atom(false, name)
        .unwrap()
        .reply()
        .unwrap()
        .atom
}

/// Waits for the ICCCM close protocol rather than destroying the client from
/// the WM side. This verifies that polite application shutdown is preferred.
fn wait_for_delete_message<C: Connection>(
    connection: &C,
    window: u32,
    wm_protocols: u32,
    wm_delete_window: u32,
) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        while let Some(event) = connection.poll_for_event().unwrap() {
            if let Event::ClientMessage(event) = event {
                let data = event.data.as_data32();
                if event.window == window
                    && event.type_ == wm_protocols
                    && data[0] == wm_delete_window
                {
                    return;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(15));
    }
    panic!("timed out waiting for WM_DELETE_WINDOW");
}

#[test]
fn session_runtime_accepts_the_concrete_x11_module_contract() {
    let dir = private_dir();
    let runtime = SessionRuntime::prepare(
        BackendKind::X11,
        &dir,
        Some(BackendModule {
            kind: BackendKind::X11,
            module: Box::new(X11BackendModule::new(None)),
        }),
    )
    .unwrap();
    assert_eq!(runtime.status().backend, BackendKind::X11);
    drop(runtime);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn manages_a_real_x11_window_with_ewmh_icccm_and_no_reparenting() {
    let mut service = X11Service::start(None).unwrap();
    let controller = service.controller();
    let (client, screen_num) = x11rb::connect(None).unwrap();
    let screen = &client.setup().roots[screen_num];
    let window = client.generate_id().unwrap();
    let wm_protocols = intern(&client, b"WM_PROTOCOLS");
    let wm_delete_window = intern(&client, b"WM_DELETE_WINDOW");
    let net_client_list = intern(&client, b"_NET_CLIENT_LIST");
    let net_supporting_wm_check = intern(&client, b"_NET_SUPPORTING_WM_CHECK");

    client
        .create_window(
            COPY_DEPTH_FROM_PARENT,
            window,
            screen.root,
            20,
            30,
            320,
            200,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new().event_mask(EventMask::STRUCTURE_NOTIFY),
        )
        .unwrap();
    client
        .change_property32(
            PropMode::REPLACE,
            window,
            wm_protocols,
            AtomEnum::ATOM,
            &[wm_delete_window],
        )
        .unwrap();
    client.map_window(window).unwrap();
    client.flush().unwrap();

    let id = WindowId::new(u64::from(window)).unwrap();
    wait_until(|| {
        controller
            .windows()
            .unwrap()
            .iter()
            .any(|candidate| candidate.id == id)
    });

    // Stage 04 must not create frame windows: CSD/SSD remains untouched until
    // the dedicated Window Chrome stage. A managed client stays a root child.
    let tree = client.query_tree(window).unwrap().reply().unwrap();
    assert_eq!(tree.parent, screen.root);

    // The backend publishes the EWMH identity and client list expected by
    // panels, pagers and ordinary X11 applications.
    let supporting = client
        .get_property(
            false,
            screen.root,
            net_supporting_wm_check,
            AtomEnum::WINDOW,
            0,
            1,
        )
        .unwrap()
        .reply()
        .unwrap();
    assert!(supporting.value32().is_some_and(|mut value| value.next().is_some()));
    wait_until(|| {
        client
            .get_property(
                false,
                screen.root,
                net_client_list,
                AtomEnum::WINDOW,
                0,
                1024,
            )
            .unwrap()
            .reply()
            .unwrap()
            .value32()
            .is_some_and(|mut clients| clients.any(|client_window| client_window == window))
    });

    controller.focus(id).unwrap();
    controller.move_window(id, 80, 90).unwrap();
    controller.resize_window(id, 480, 300).unwrap();
    wait_until(|| {
        controller
            .windows()
            .unwrap()
            .iter()
            .find(|candidate| candidate.id == id)
            .is_some_and(|candidate| {
                candidate.geometry.x == 80
                    && candidate.geometry.y == 90
                    && candidate.geometry.width == 480
                    && candidate.geometry.height == 300
            })
    });

    controller.maximize(id).unwrap();
    wait_until(|| {
        controller
            .windows()
            .unwrap()
            .iter()
            .find(|candidate| candidate.id == id)
            .is_some_and(|candidate| candidate.presentation == PresentationState::Maximized)
    });
    controller.restore(id).unwrap();
    wait_until(|| {
        controller
            .windows()
            .unwrap()
            .iter()
            .find(|candidate| candidate.id == id)
            .is_some_and(|candidate| {
                candidate.presentation == PresentationState::Normal
                    && candidate.geometry.width == 480
            })
    });

    controller.fullscreen(id, true).unwrap();
    wait_until(|| {
        controller
            .windows()
            .unwrap()
            .iter()
            .find(|candidate| candidate.id == id)
            .is_some_and(|candidate| candidate.presentation == PresentationState::Fullscreen)
    });
    controller.fullscreen(id, false).unwrap();
    wait_until(|| {
        controller
            .windows()
            .unwrap()
            .iter()
            .find(|candidate| candidate.id == id)
            .is_some_and(|candidate| candidate.presentation == PresentationState::Normal)
    });

    controller.close(id).unwrap();
    wait_for_delete_message(&client, window, wm_protocols, wm_delete_window);

    // The test client performs its own destruction after acknowledging the
    // polite close request; the backend must then remove it deterministically.
    client.destroy_window(window).unwrap();
    client.flush().unwrap();
    wait_until(|| {
        !controller
            .windows()
            .unwrap()
            .iter()
            .any(|candidate| candidate.id == id)
    });
    service.stop().unwrap();
}
