use std::path::PathBuf;

use nexxus_ui::{ScaleFactor, Theme};
use nexxus_workspaces::WorkspaceManager;
use nexxus_workspace_bar::{AssetSource, x11::X11WorkspaceBar};

#[test]
fn creates_workspace_bar_surface_on_x11() {
    if std::env::var_os("DISPLAY").is_none() {
        return;
    }
    let manager = WorkspaceManager::with_single_fixed("WEB").unwrap();
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../etapa-08-visual-assets/assets");
    let bar = X11WorkspaceBar::connect(
        None,
        &manager,
        ScaleFactor::new(1.0).unwrap(),
        Theme::default(),
        AssetSource::new(assets),
    )
    .unwrap();
    assert_ne!(bar.window(), 0);
    assert!(bar.layout().window.width > 0.0);
}
