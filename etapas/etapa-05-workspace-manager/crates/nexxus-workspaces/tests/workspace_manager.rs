//! Contract tests for fixed/dynamic lifecycle, MRU, placement and persistence.

use nexxus_wm::WindowId;
use nexxus_workspaces::{
    DynamicPolicy, PlacementRule, WorkspaceId, WorkspaceKind, WorkspaceManager,
};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn wid(value: u64) -> WindowId {
    WindowId::new(value).expect("test window ids are non-zero")
}

fn temp_path(name: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock must be after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "nexxus-workspaces-{}-{nonce}-{name}.toml",
        std::process::id()
    ))
}

#[test]
fn fixed_dynamic_creation_activation_and_mru_are_deterministic() {
    let mut manager = WorkspaceManager::with_single_fixed("WEB").expect("valid manager");
    let mail = manager.create_fixed("MAIL").expect("create MAIL");
    let scratch = manager
        .create_dynamic("SCRATCH")
        .expect("create dynamic workspace");

    manager.activate(mail).expect("activate MAIL");
    manager.activate(scratch).expect("activate SCRATCH");

    let mru: Vec<_> = manager.mru_order().collect();
    assert_eq!(mru[0], scratch);
    assert_eq!(mru[1], mail);
    assert_eq!(manager.previous_mru(), Some(mail));
    assert_eq!(manager.workspace(scratch).expect("scratch exists").kind, WorkspaceKind::Dynamic);
}

#[test]
fn placement_rule_is_initial_only_and_never_locks_manual_movement() {
    let mut manager = WorkspaceManager::with_single_fixed("WEB").expect("valid manager");
    let mail = manager.create_fixed("MAIL").expect("create MAIL");
    manager
        .set_placement_rules(vec![PlacementRule {
            application_id: "org.mozilla.Thunderbird".into(),
            workspace: mail,
        }])
        .expect("valid placement rule");

    let window = wid(10);
    assert_eq!(
        manager
            .assign_new_window(window, Some("org.mozilla.Thunderbird"))
            .expect("initial placement"),
        mail
    );

    let web = WorkspaceId::new(1).expect("known workspace id");
    manager
        .move_window(window, web)
        .expect("manual move must override rule");
    assert_eq!(manager.workspace_of(window), Some(web));
}

#[test]
fn empty_inactive_dynamic_workspace_is_pruned_without_losing_windows() {
    let mut manager = WorkspaceManager::with_single_fixed("MAIN").expect("valid manager");
    manager.set_dynamic_policy(DynamicPolicy::RemoveEmptyInactive);
    let transient = manager
        .create_dynamic("TEMP")
        .expect("create dynamic workspace");
    let window = wid(20);

    manager.activate(transient).expect("activate TEMP");
    manager
        .assign_new_window(window, None)
        .expect("assign to active TEMP");
    let main = WorkspaceId::new(1).expect("known workspace id");
    manager.activate(main).expect("activate MAIN");
    manager
        .move_window(window, main)
        .expect("move out of TEMP");

    assert!(manager.workspace(transient).is_none());
    assert_eq!(manager.workspace_of(window), Some(main));
    assert_eq!(manager.active_id(), main);
}

#[test]
fn removing_active_workspace_rehomes_resident_windows_before_removal() {
    let mut manager = WorkspaceManager::with_single_fixed("ONE").expect("valid manager");
    let two = manager.create_fixed("TWO").expect("create TWO");
    let window = wid(30);
    manager.activate(two).expect("activate TWO");
    manager
        .assign_new_window(window, None)
        .expect("assign window to TWO");

    manager.remove(two).expect("remove active TWO safely");

    let one = WorkspaceId::new(1).expect("known workspace id");
    assert_eq!(manager.active_id(), one);
    assert_eq!(manager.workspace_of(window), Some(one));
    assert!(manager.active_windows().any(|candidate| candidate == window));
}

#[test]
fn persisted_configuration_roundtrips_without_runtime_window_membership() {
    let path = temp_path("roundtrip");
    let mut manager = WorkspaceManager::with_single_fixed("DEV").expect("valid manager");
    let chat = manager.create_fixed("CHAT").expect("create CHAT");
    manager.activate(chat).expect("activate CHAT");
    manager
        .assign_new_window(wid(40), None)
        .expect("track runtime window");
    manager.save(&path).expect("save workspace config");

    let loaded = WorkspaceManager::load(&path).expect("load workspace config");
    assert_eq!(loaded.active_id(), chat);
    assert_eq!(loaded.workspaces().count(), 2);
    assert_eq!(loaded.active_windows().count(), 0);

    let _ = fs::remove_file(path);
}
