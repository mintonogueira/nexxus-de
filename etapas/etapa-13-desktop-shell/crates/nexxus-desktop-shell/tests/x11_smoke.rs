use std::fs;

use nexxus_desktop_shell::{AssetSource, DesktopConfigStore, x11::X11DesktopShell};
use nexxus_ui::{ScaleFactor, Theme};
use nexxus_xdg_application_index::{ApplicationIndexConfig, ApplicationRoot};
use tempfile::TempDir;

#[test]
fn creates_desktop_surface_on_x11() {
    if std::env::var_os("DISPLAY").is_none() {
        return;
    }
    let apps = TempDir::new().unwrap();
    fs::write(
        apps.path().join("demo.desktop"),
        "[Desktop Entry]\nType=Application\nName=Demo\nExec=demo\nCategories=Utility;\n",
    )
    .unwrap();
    let state = TempDir::new().unwrap();
    let desktop = TempDir::new().unwrap();
    let index_config = ApplicationIndexConfig {
        roots: vec![ApplicationRoot::custom(apps.path(), "x11-smoke")],
        locales: Vec::new(),
        current_desktops: Vec::new(),
        max_desktop_file_bytes: 2 * 1024 * 1024,
    };
    let assets = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../etapa-08-visual-assets/assets");
    let shell = X11DesktopShell::connect(
        None,
        DesktopConfigStore::new(state.path().join("desktop.toml")),
        index_config,
        desktop.path().to_path_buf(),
        ScaleFactor::new(1.0).unwrap(),
        Theme::default(),
        AssetSource::new(assets),
    )
    .unwrap();
    assert_ne!(shell.window(), 0);
}
