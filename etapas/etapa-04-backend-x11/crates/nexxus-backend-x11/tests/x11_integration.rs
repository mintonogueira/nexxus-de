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
use x11rb::protocol::xproto::{ConnectionExt as _, CreateWindowAux, EventMask, WindowClass};

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
fn maps_moves_resizes_maximizes_fullscreens_and_restores_a_real_x11_window() {
    let mut service = X11Service::start(None).unwrap();
    let controller = service.controller();
    let (client, screen_num) = x11rb::connect(None).unwrap();
    let screen = &client.setup().roots[screen_num];
    let window = client.generate_id().unwrap();
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
