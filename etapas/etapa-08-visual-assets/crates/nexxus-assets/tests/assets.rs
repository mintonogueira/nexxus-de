//! Filesystem-level contract tests for the Stage 08 source tree.

use std::fs;
use std::path::{Path, PathBuf};

use nexxus_assets::{ICONS, SYMBOLIC_PALETTE_TOKEN, WALLPAPERS};

fn stage_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("stage root must exist")
}

/// Every catalog entry must resolve to a real project-owned SVG source.
#[test]
fn catalog_paths_exist() {
    let root = stage_root();
    for icon in ICONS {
        let path = root.join("assets/icons").join(icon.relative_path);
        assert!(path.is_file(), "missing icon: {}", path.display());
    }
    for wallpaper in WALLPAPERS {
        let path = root.join("assets/wallpapers").join(wallpaper.relative_path);
        assert!(path.is_file(), "missing wallpaper: {}", path.display());
    }
}

/// Symbolic icons keep the canonical 24-unit viewBox and palette token.
#[test]
fn symbolic_sources_follow_contract() {
    let root = stage_root();
    for icon in ICONS {
        let path = root.join("assets/icons").join(icon.relative_path);
        let text = fs::read_to_string(&path).expect("icon must be UTF-8");
        assert!(
            text.contains(r#"viewBox="0 0 24 24""#),
            "invalid icon viewBox: {}",
            path.display()
        );
        assert!(
            text.contains(SYMBOLIC_PALETTE_TOKEN),
            "palette token missing: {}",
            path.display()
        );
        assert!(
            !text.contains("<script") && !text.contains("<image") && !text.contains("<filter"),
            "unsafe or non-symbolic SVG feature: {}",
            path.display()
        );
    }
}

/// Wallpaper dimensions are explicit and no external resources are embedded.
#[test]
fn wallpapers_are_local_opaque_vectors() {
    let root = stage_root();
    assert_eq!(WALLPAPERS.len(), 10);
    for wallpaper in WALLPAPERS {
        let path = root.join("assets/wallpapers").join(wallpaper.relative_path);
        let text = fs::read_to_string(&path).expect("wallpaper must be UTF-8");
        assert!(text.contains(r#"viewBox="0 0 1920 1080""#));
        assert!(!text.contains("<script"));
        assert!(!text.contains("<image"));
        // xmlns="http://www.w3.org/2000/svg" is the required SVG namespace;
        // only resource-bearing href attributes are forbidden here.
        assert!(!text.contains(" href=") && !text.contains("xlink:href"));
        assert!(!text.contains("opacity=") && !text.contains("<filter"));
    }
}
