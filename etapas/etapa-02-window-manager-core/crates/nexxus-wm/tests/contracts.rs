use nexxus_wm::{BackendCommandSink, Geometry, SizeConstraints, WindowId, WindowManager, WindowMetadata, WmCommand, WmEvent};
use nexxus_backend_api::BackendError;

struct RecordingSink {
    commands: Vec<WmCommand>,
}

impl BackendCommandSink for RecordingSink {
    fn submit(&mut self, command: &WmCommand) -> Result<(), BackendError> {
        self.commands.push(command.clone());
        Ok(())
    }
}

fn id(value: u64) -> WindowId {
    WindowId::new(value).expect("test id must be non-zero")
}

#[test]
fn backend_contract_uses_only_logical_identifiers_and_commands() {
    let mut wm = WindowManager::new();
    wm.apply_event(WmEvent::WindowCreated {
        id: id(7),
        geometry: Geometry::new(40, 50, 900, 700).unwrap(),
        constraints: SizeConstraints::default(),
        metadata: WindowMetadata {
            title: "Editor".into(),
            application_id: Some("org.example.Editor".into()),
        },
    })
    .unwrap();

    let command = wm.request_focus(id(7)).unwrap();
    let mut sink = RecordingSink { commands: Vec::new() };
    wm.dispatch(&mut sink, &command).unwrap();

    assert_eq!(sink.commands, vec![WmCommand::RequestFocus { window: id(7) }]);
}

#[test]
fn destroyed_window_during_pending_operation_does_not_reappear() {
    let mut wm = WindowManager::new();
    wm.apply_event(WmEvent::WindowCreated {
        id: id(9),
        geometry: Geometry::new(0, 0, 640, 480).unwrap(),
        constraints: SizeConstraints::default(),
        metadata: WindowMetadata::default(),
    })
    .unwrap();

    let pending = wm.request_resize(id(9), 800, 600).unwrap();
    wm.apply_event(WmEvent::WindowDestroyed { id: id(9) }).unwrap();
    assert!(wm.window(id(9)).is_none());

    let outcome = wm
        .apply_event(WmEvent::WindowGeometryChanged {
            id: id(9),
            geometry: Geometry::new(0, 0, 800, 600).unwrap(),
        })
        .unwrap();
    assert_eq!(format!("{outcome:?}"), "IgnoredStale");
    assert_eq!(pending, WmCommand::RequestResize { window: id(9), width: 800, height: 600 });
    assert!(wm.window(id(9)).is_none());
}
