use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use nexxus_xdg_application_index::{
    ApplicationIndexConfig, ApplicationIndexEvent, ApplicationIndexService, ApplicationRoot,
    ApplicationSource, IconReference, MainCategory, scan,
};
use tempfile::TempDir;

fn config(roots: Vec<ApplicationRoot>) -> ApplicationIndexConfig {
    ApplicationIndexConfig {
        roots,
        locales: vec!["pt_BR".to_owned(), "pt".to_owned()],
        current_desktops: vec!["NEXXUS".to_owned()],
        max_desktop_file_bytes: 2 * 1024 * 1024,
    }
}

fn write_entry(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

#[test]
fn indexes_valid_entries_and_keeps_invalid_entries_as_diagnostics() {
    let temp = TempDir::new().unwrap();
    let apps = temp.path().join("applications");
    fs::create_dir_all(&apps).unwrap();
    write_entry(
        &apps.join("org.example.Editor.desktop"),
        "[Desktop Entry]\nType=Application\nName=Editor\nName[pt_BR]=Editor PT\nExec=editor %F\nCategories=Development;Utility;\nKeywords=code;texto;\n",
    );
    write_entry(
        &apps.join("broken.desktop"),
        "[Desktop Entry]\nType=Application\nName=Broken\nExec=broken %Z\n",
    );

    let snapshot = scan(&config(vec![ApplicationRoot::custom(&apps, "fixture")])).unwrap();
    let editor = snapshot.by_id("org.example.Editor.desktop").unwrap();
    assert_eq!(editor.name, "Editor PT");
    assert_eq!(
        editor.main_categories,
        [MainCategory::Development, MainCategory::Utility]
    );
    assert!(matches!(editor.icon, IconReference::NexxusFallback { .. }));
    assert!(snapshot.by_id("broken.desktop").is_none());
    assert_eq!(snapshot.diagnostics().len(), 1);
}

#[test]
fn higher_precedence_hidden_entry_masks_system_copy() {
    let temp = TempDir::new().unwrap();
    let user = temp.path().join("user/applications");
    let system = temp.path().join("system/applications");
    fs::create_dir_all(&user).unwrap();
    fs::create_dir_all(&system).unwrap();
    write_entry(
        &user.join("org.example.App.desktop"),
        "[Desktop Entry]\nType=Application\nName=Masked\nHidden=true\nExec=masked\n",
    );
    write_entry(
        &system.join("org.example.App.desktop"),
        "[Desktop Entry]\nType=Application\nName=System App\nExec=system-app\n",
    );

    let snapshot = scan(&config(vec![
        ApplicationRoot::custom(&user, "user"),
        ApplicationRoot::custom(&system, "system"),
    ]))
    .unwrap();
    assert!(snapshot.by_id("org.example.App.desktop").is_none());
}

#[test]
fn nodisplay_entry_exists_but_is_excluded_from_visible_views() {
    let temp = TempDir::new().unwrap();
    let apps = temp.path().join("applications");
    fs::create_dir_all(&apps).unwrap();
    write_entry(
        &apps.join("helper.desktop"),
        "[Desktop Entry]\nType=Application\nName=Helper\nExec=helper\nNoDisplay=true\nCategories=Utility;\n",
    );
    let snapshot = scan(&config(vec![ApplicationRoot::custom(&apps, "fixture")])).unwrap();
    assert!(snapshot.by_id("helper.desktop").is_some());
    assert_eq!(snapshot.visible_entries().count(), 0);
}

#[test]
fn flatpak_export_is_indexed_without_invoking_flatpak_cli() {
    let temp = TempDir::new().unwrap();
    let apps = temp.path().join("flatpak/exports/share/applications");
    fs::create_dir_all(&apps).unwrap();
    write_entry(
        &apps.join("org.example.Flat.desktop"),
        "[Desktop Entry]\nType=Application\nName=Flat App\nExec=/usr/bin/flatpak run org.example.Flat\nIcon=org.example.Flat\nCategories=Network;\nX-Flatpak=org.example.Flat\n",
    );
    let snapshot = scan(&config(vec![ApplicationRoot {
        path: apps,
        source: ApplicationSource::UserFlatpak,
    }]))
    .unwrap();
    let entry = snapshot.by_id("org.example.Flat.desktop").unwrap();
    assert_eq!(entry.source, ApplicationSource::UserFlatpak);
    assert_eq!(
        entry.icon,
        IconReference::ExternalName("org.example.Flat".into())
    );
}

#[test]
fn filesystem_changes_are_published_without_restarting_service() {
    let temp = TempDir::new().unwrap();
    let apps = temp.path().join("applications");
    fs::create_dir_all(&apps).unwrap();
    let service = ApplicationIndexService::start(config(vec![ApplicationRoot::custom(
        &apps,
        "live-fixture",
    )]))
    .unwrap();
    let events = service.subscribe();

    write_entry(
        &apps.join("live.desktop"),
        "[Desktop Entry]\nType=Application\nName=Live App\nExec=live-app\nCategories=Utility;\n",
    );

    let deadline = Instant::now() + Duration::from_secs(8);
    let mut observed = false;
    while Instant::now() < deadline {
        match events.recv_timeout(Duration::from_millis(500)) {
            Ok(ApplicationIndexEvent::Changed(delta))
                if delta.added.iter().any(|id| id.as_str() == "live.desktop") =>
            {
                observed = true;
                break;
            }
            Ok(_) | Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(error) => panic!("watch channel closed unexpectedly: {error}"),
        }
    }
    assert!(observed, "live .desktop creation was not observed");
    assert!(service.snapshot().by_id("live.desktop").is_some());
}
