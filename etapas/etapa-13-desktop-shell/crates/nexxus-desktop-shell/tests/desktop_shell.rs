use std::fs;
use std::thread;
use std::time::{Duration, Instant};

use nexxus_desktop_shell::{
    DesktopConfigStore, DesktopPainter, DesktopShell, DesktopShellAction, DesktopShellRuntime,
    LaunchPlan, MenuAction, MonitorGeometry,
};
use nexxus_shortcuts::{CommandTarget, ShellAction};
use nexxus_ui::{LogicalPoint, LogicalRect, LogicalSize, ScaleFactor, Theme};
use nexxus_xdg_application_index::{ApplicationIndexConfig, ApplicationRoot, scan};
use tempfile::TempDir;

fn write_app(root: &TempDir, file: &str, name: &str, exec: &str, category: &str) {
    fs::write(
        root.path().join(file),
        format!(
            "[Desktop Entry]\nType=Application\nName={name}\nExec={exec}\nCategories={category};\n"
        ),
    )
    .unwrap();
}

fn index_config(root: &TempDir) -> ApplicationIndexConfig {
    ApplicationIndexConfig {
        roots: vec![ApplicationRoot::custom(root.path(), "stage13-test")],
        locales: Vec::new(),
        current_desktops: Vec::new(),
        max_desktop_file_bytes: 2 * 1024 * 1024,
    }
}

fn monitors() -> Vec<MonitorGeometry> {
    let scale = ScaleFactor::new(1.0).unwrap();
    vec![
        MonitorGeometry {
            rect: LogicalRect::new(0.0, 0.0, 1280.0, 720.0),
            scale,
            primary: true,
        },
        MonitorGeometry {
            rect: LogicalRect::new(1280.0, 0.0, 1280.0, 720.0),
            scale,
            primary: false,
        },
    ]
}

fn build_shell(apps: &TempDir, state: &TempDir, desktop: &TempDir) -> DesktopShell {
    let snapshot = scan(&index_config(apps)).unwrap();
    DesktopShell::new(
        DesktopConfigStore::new(state.path().join("desktop.toml")),
        snapshot,
        monitors(),
        desktop.path().to_path_buf(),
    )
    .unwrap()
}

#[test]
fn ctrl_escape_target_opens_one_primary_menu() {
    let apps = TempDir::new().unwrap();
    let state = TempDir::new().unwrap();
    let desktop = TempDir::new().unwrap();
    write_app(&apps, "demo.desktop", "Demo", "demo", "Utility");
    let mut shell = build_shell(&apps, &state, &desktop);

    assert!(
        shell
            .handle_shortcut_target(CommandTarget::Shell(ShellAction::DesktopMenu))
            .unwrap()
    );
    let menu = shell.menu().unwrap();
    assert_eq!(menu.monitor_index, 0);
    assert!(
        shell
            .menu_entries()
            .iter()
            .any(|entry| entry.label == "Applications")
    );
}

#[test]
fn right_click_location_selects_only_that_monitor_for_menu() {
    let apps = TempDir::new().unwrap();
    let state = TempDir::new().unwrap();
    let desktop = TempDir::new().unwrap();
    let mut shell = build_shell(&apps, &state, &desktop);
    shell
        .open_context_menu(LogicalPoint::new(1500.0, 100.0))
        .unwrap();
    assert_eq!(shell.menu().unwrap().monitor_index, 1);
}

#[test]
fn pinned_launcher_persists_and_uses_shell_free_exec_plan() {
    let apps = TempDir::new().unwrap();
    let state = TempDir::new().unwrap();
    let desktop = TempDir::new().unwrap();
    write_app(
        &apps,
        "demo.desktop",
        "Demo",
        "printf %%s a;touch-pwned",
        "Utility",
    );
    let mut shell = build_shell(&apps, &state, &desktop);
    shell.pin_launcher("demo.desktop", None).unwrap();
    let action = shell.launch_action("demo.desktop").unwrap();
    assert_eq!(
        action,
        DesktopShellAction::Launch(LaunchPlan::Exec {
            program: "printf".to_owned(),
            arguments: vec!["%s".to_owned(), "a;touch-pwned".to_owned()],
        })
    );

    let restored = build_shell(&apps, &state, &desktop);
    assert_eq!(restored.config().launchers.len(), 1);
    assert_eq!(restored.config().launchers[0].desktop_id, "demo.desktop");
}

#[test]
fn create_folder_action_changes_only_desktop_directory() {
    let apps = TempDir::new().unwrap();
    let state = TempDir::new().unwrap();
    let desktop = TempDir::new().unwrap();
    let mut shell = build_shell(&apps, &state, &desktop);
    shell.open_context_menu_from_shortcut().unwrap();
    let action = shell
        .activate_menu_action(MenuAction::CreateFolder)
        .unwrap()
        .unwrap();
    let DesktopShellAction::FolderCreated(path) = action else {
        panic!("unexpected action")
    };
    assert!(path.is_dir());
    assert_eq!(path.parent(), Some(desktop.path()));
}

#[test]
fn painter_uses_stage08_assets_without_gtk_or_qt() {
    let apps = TempDir::new().unwrap();
    let state = TempDir::new().unwrap();
    let desktop = TempDir::new().unwrap();
    write_app(&apps, "demo.desktop", "Demo", "demo", "Utility");
    let mut shell = build_shell(&apps, &state, &desktop);
    shell.pin_launcher("demo.desktop", None).unwrap();
    let assets = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../etapa-08-visual-assets/assets");
    let mut painter = DesktopPainter::new(
        Theme::default(),
        nexxus_desktop_shell::AssetSource::new(assets),
    )
    .unwrap();
    let (frame, layout) = painter
        .render(
            &shell,
            LogicalSize::new(1280.0, 720.0),
            ScaleFactor::new(1.0).unwrap(),
        )
        .unwrap();
    assert_eq!(frame.size.width, 1280);
    assert_eq!(layout.launchers.len(), 1);
}

#[test]
fn live_index_change_reaches_desktop_runtime() {
    let apps = TempDir::new().unwrap();
    let state = TempDir::new().unwrap();
    let desktop = TempDir::new().unwrap();
    write_app(&apps, "one.desktop", "One", "one", "Utility");
    let mut runtime = DesktopShellRuntime::start(
        DesktopConfigStore::new(state.path().join("desktop.toml")),
        index_config(&apps),
        monitors(),
        desktop.path().to_path_buf(),
    )
    .unwrap();
    assert!(runtime.shell().snapshot().by_id("two.desktop").is_none());

    write_app(&apps, "two.desktop", "Two", "two", "Development");
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        let _ = runtime.poll_index_updates();
        if runtime.shell().snapshot().by_id("two.desktop").is_some() {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("desktop runtime did not observe Stage 12 index change");
}
