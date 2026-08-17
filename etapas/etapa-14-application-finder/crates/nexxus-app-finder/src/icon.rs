//! Resolution and painting of application icons for the Finder result rows.
//!
//! `.desktop` parsing remains in Stage 12. This module only resolves the
//! already-normalized `IconReference` into a local graphic and emits Nexxus UI
//! display commands. PNG and SVG cover the normal modern icon-theme paths;
//! legacy XPM-only artwork falls back to the Nexxus generic application icon.

use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use nexxus_ui::{DisplayList, DrawCommand, ImageData, LogicalRect};
use nexxus_xdg_application_index::IconReference;

const SYSTEM_ASSET_ROOT: &str = "/usr/share/nexxus/assets";
const GENERIC_FALLBACK: &str = "mimetypes/application-x-generic.svg";
const MAX_ICON_FILE_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Clone, Debug)]
enum GraphicAsset {
    Svg(Vec<u8>),
    Raster(ImageData),
}

/// Resolves official XDG icon names and Nexxus fallback references without
/// introducing a second application index or desktop-entry parser.
#[derive(Clone, Debug)]
pub struct FinderIconResolver {
    asset_root: PathBuf,
    icon_roots: Vec<PathBuf>,
    cache: RefCell<HashMap<PathBuf, Option<GraphicAsset>>>,
}

impl FinderIconResolver {
    /// Uses the canonical installed Nexxus asset root and XDG icon roots.
    pub fn system() -> Self {
        Self::new(PathBuf::from(SYSTEM_ASSET_ROOT), xdg_icon_roots())
    }

    /// Constructor also used by tests and future embedders with staged assets.
    pub fn new(asset_root: PathBuf, icon_roots: Vec<PathBuf>) -> Self {
        Self {
            asset_root,
            icon_roots,
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn asset_root(&self) -> &Path {
        &self.asset_root
    }

    /// Resolves the official icon first. If the official reference cannot be
    /// represented by the current renderer, a generic Nexxus fallback is used.
    pub fn resolve_path(&self, reference: &IconReference) -> Option<PathBuf> {
        let official = match reference {
            IconReference::ExternalPath(path) => path.is_file().then(|| path.clone()),
            IconReference::ExternalName(name) => self.external_icon_path(name),
            IconReference::NexxusFallback { relative_path, .. } => {
                self.builtin_icon_path(relative_path)
            }
        };
        official.or_else(|| self.builtin_icon_path(GENERIC_FALLBACK))
    }

    /// Appends one icon to the component display list. Corrupt/unreadable
    /// artwork never aborts the Finder; the generic fallback is attempted once.
    pub fn paint(&self, list: &mut DisplayList, reference: &IconReference, rect: LogicalRect) {
        let Some(path) = self.resolve_path(reference) else {
            return;
        };
        if let Some(asset) = self.load_cached(&path) {
            push_graphic(list, rect, asset);
            return;
        }

        let Some(fallback) = self.builtin_icon_path(GENERIC_FALLBACK) else {
            return;
        };
        if fallback != path {
            if let Some(asset) = self.load_cached(&fallback) {
                push_graphic(list, rect, asset);
            }
        }
    }

    fn builtin_icon_path(&self, relative_path: &str) -> Option<PathBuf> {
        first_existing([
            self.asset_root.join("icons").join(relative_path),
            self.asset_root.join(relative_path),
        ])
    }

    fn external_icon_path(&self, name: &str) -> Option<PathBuf> {
        let direct = Path::new(name);
        if direct.is_absolute() && direct.is_file() {
            return Some(direct.to_path_buf());
        }

        // Stage 14 currently has no user-selectable application icon theme.
        // The Freedesktop lookup contract always ends in hicolor, so this
        // deterministic baseline preserves application-provided artwork while
        // keeping theme policy out of the Finder module.
        const SUBDIRS: &[&str] = &[
            "hicolor/scalable/apps",
            "hicolor/symbolic/apps",
            "hicolor/256x256/apps",
            "hicolor/128x128/apps",
            "hicolor/64x64/apps",
            "hicolor/48x48/apps",
            "hicolor/32x32/apps",
            "hicolor/24x24/apps",
            "hicolor/16x16/apps",
        ];
        for root in &self.icon_roots {
            for subdir in SUBDIRS {
                if let Some(candidate) = named_candidate(root.join(subdir), name) {
                    return Some(candidate);
                }
            }
        }

        // `/usr/share/pixmaps` remains a widely used unthemed compatibility
        // location and is checked only after the hicolor roots.
        for root in pixmap_roots() {
            if let Some(candidate) = named_candidate(root, name) {
                return Some(candidate);
            }
        }
        None
    }

    fn load_cached(&self, path: &Path) -> Option<GraphicAsset> {
        if let Some(cached) = self.cache.borrow().get(path) {
            return cached.clone();
        }
        let loaded = load_graphic(path);
        self.cache
            .borrow_mut()
            .insert(path.to_path_buf(), loaded.clone());
        loaded
    }
}

impl Default for FinderIconResolver {
    fn default() -> Self {
        Self::system()
    }
}

fn named_candidate(directory: PathBuf, name: &str) -> Option<PathBuf> {
    let declared = Path::new(name);
    if declared.extension().is_some() {
        let candidate = directory.join(name);
        return supported_graphic(&candidate).then_some(candidate);
    }
    for extension in ["png", "svg"] {
        let candidate = directory.join(format!("{name}.{extension}"));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn supported_graphic(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("png") || extension.eq_ignore_ascii_case("svg")
            })
}

fn load_graphic(path: &Path) -> Option<GraphicAsset> {
    let metadata = fs::metadata(path).ok()?;
    if metadata.len() > MAX_ICON_FILE_BYTES {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
    {
        return Some(GraphicAsset::Svg(bytes));
    }

    let decoded = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let (width, height) = decoded.dimensions();
    let image = ImageData::new(width, height, decoded.into_raw()).ok()?;
    Some(GraphicAsset::Raster(image))
}

fn push_graphic(list: &mut DisplayList, rect: LogicalRect, asset: GraphicAsset) {
    match asset {
        GraphicAsset::Svg(bytes) => list.push(DrawCommand::Svg { rect, bytes }),
        GraphicAsset::Raster(image) => list.push(DrawCommand::Image { rect, image }),
    }
}

fn first_existing<const N: usize>(paths: [PathBuf; N]) -> Option<PathBuf> {
    paths.into_iter().find(|path| supported_graphic(path))
}

fn xdg_icon_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
        roots.push(
            env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
                .unwrap_or_else(|| home.join(".local/share"))
                .join("icons"),
        );
        // Legacy user icon location retained for desktop compatibility.
        roots.push(home.join(".icons"));
    }
    let data_dirs = data_dirs();
    roots.extend(data_dirs.into_iter().map(|path| path.join("icons")));
    roots
}

fn pixmap_roots() -> Vec<PathBuf> {
    data_dirs()
        .into_iter()
        .map(|path| path.join("pixmaps"))
        .collect()
}

fn data_dirs() -> Vec<PathBuf> {
    env::var_os("XDG_DATA_DIRS")
        .map(|raw| env::split_paths(&raw).collect::<Vec<_>>())
        .filter(|paths| !paths.is_empty())
        .unwrap_or_else(|| {
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share"),
            ]
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after UNIX epoch")
            .as_nanos();
        env::temp_dir().join(format!(
            "nexxus-app-finder-{label}-{}-{nonce}",
            std::process::id()
        ))
    }

    fn write_svg(path: &Path) {
        fs::create_dir_all(path.parent().expect("test icon parent")).unwrap();
        fs::write(
            path,
            br#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"></svg>"#,
        )
        .unwrap();
    }

    #[test]
    fn resolves_nexxus_relative_fallback_from_staged_asset_root() {
        let root = temp_root("fallback");
        let icon = root.join("icons/mimetypes/application-x-generic.svg");
        write_svg(&icon);
        let resolver = FinderIconResolver::new(root.clone(), Vec::new());
        let reference = IconReference::NexxusFallback {
            name: "application-x-generic".to_owned(),
            relative_path: "mimetypes/application-x-generic.svg".to_owned(),
        };
        assert_eq!(resolver.resolve_path(&reference), Some(icon));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resolves_named_official_icon_from_hicolor_before_generic_fallback() {
        let root = temp_root("official");
        let icon_root = root.join("xdg-icons");
        let official = icon_root.join("hicolor/48x48/apps/org.example.App.svg");
        write_svg(&official);
        let resolver = FinderIconResolver::new(root.join("assets"), vec![icon_root]);
        assert_eq!(
            resolver.resolve_path(&IconReference::ExternalName("org.example.App".to_owned())),
            Some(official)
        );
        fs::remove_dir_all(root).unwrap();
    }
}
