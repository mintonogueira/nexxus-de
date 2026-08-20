use std::fs;

use nexxus_app_menu::{
    APPLICATION_MENU_PLUGIN_ID, ApplicationMenuPanelPlugin, ApplicationMenuState, MenuSection,
};
use nexxus_panel::{PanelPlugin, PluginApiVersion, PluginRegistry};
use nexxus_xdg_application_index::{ApplicationIndexConfig, ApplicationRoot, LaunchContext, scan};
use tempfile::tempdir;

fn fixture_snapshot() -> nexxus_xdg_application_index::IndexSnapshot {
    let dir = tempdir().expect("tempdir");
    let apps = dir.path().join("applications");
    fs::create_dir_all(&apps).expect("applications directory");
    fs::write(
        apps.join("org.example.Editor.desktop"),
        "[Desktop Entry]\nType=Application\nName=Editor\nExec=editor --safe\nIcon=editor\nCategories=Utility;\nKeywords=text;notes;\n",
    )
    .expect("desktop entry");
    fs::write(
        apps.join("org.example.Browser.desktop"),
        "[Desktop Entry]\nType=Application\nName=Browser\nExec=browser %u\nIcon=browser\nCategories=Network;\nKeywords=web;internet;\n",
    )
    .expect("desktop entry");

    let config = ApplicationIndexConfig {
        roots: vec![ApplicationRoot::custom(apps, "test")],
        locales: Vec::new(),
        current_desktops: Vec::new(),
        max_desktop_file_bytes: 1024 * 1024,
    };
    scan(&config).expect("scan")
}

#[test]
fn search_favorites_recents_and_shell_free_launch_share_one_index() {
    let snapshot = fixture_snapshot();
    let mut menu = ApplicationMenuState::default();

    menu.set_section(MenuSection::All);
    assert_eq!(menu.visible_entries(&snapshot).len(), 2);

    menu.set_query("web");
    let results = menu.visible_entries(&snapshot);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].desktop_id, "org.example.Browser.desktop");

    menu.toggle_favorite("org.example.Browser.desktop");
    menu.set_section(MenuSection::Favorites);
    let favorites = menu.visible_entries(&snapshot);
    assert_eq!(favorites.len(), 1);
    assert!(favorites[0].favorite);

    let command = menu
        .launch_command(
            &snapshot,
            "org.example.Browser.desktop",
            &LaunchContext {
                files: Vec::new(),
                urls: vec!["https://example.invalid".to_owned()],
            },
        )
        .expect("launch command");
    assert_eq!(command.program, "browser");
    assert_eq!(command.arguments, vec!["https://example.invalid"]);

    menu.set_section(MenuSection::Recent);
    assert_eq!(menu.visible_entries(&snapshot).len(), 1);
}

#[test]
fn application_menu_obeys_panel_plugin_api() {
    let plugin = ApplicationMenuPanelPlugin::new();
    let metadata = plugin.metadata();
    assert_eq!(metadata.plugin_id, APPLICATION_MENU_PLUGIN_ID);
    assert_eq!(metadata.api, PluginApiVersion::CURRENT);

    let mut registry = PluginRegistry::new();
    registry
        .load("application-menu-0", Box::new(plugin))
        .expect("load plugin");
    assert!(registry.is_loaded("application-menu-0"));
    registry
        .unload("application-menu-0")
        .expect("unload plugin");
    assert!(!registry.is_loaded("application-menu-0"));
}

#[test]
fn super_open_semantics_are_idempotent_at_state_level() {
    let mut menu = ApplicationMenuState::default();
    assert!(!menu.is_open());
    menu.open();
    menu.open();
    assert!(menu.is_open());
    menu.toggle();
    assert!(!menu.is_open());
}
