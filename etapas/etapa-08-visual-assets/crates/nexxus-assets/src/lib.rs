//! Semantic catalog and policy helpers for Nexxus visual assets.
//! Stage 08 owns naming/fallback/recoloring, not rendering or platform backends.
#![forbid(unsafe_code)]

mod catalog;
pub use catalog::{ICONS, WALLPAPERS};

pub const SYSTEM_ASSET_ROOT: &str = "/usr/share/nexxus/assets";
pub const SYMBOLIC_PALETTE_TOKEN: &str = "#FFFFFF";
pub const DEFAULT_FONT_FAMILY: &str = "Hack";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IconContext {
    Actions,
    Places,
    Devices,
    Status,
    Categories,
    MimeTypes,
}

/// Metadata for one Nexxus-owned symbolic icon.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IconSpec {
    pub name: &'static str,
    pub context: IconContext,
    pub relative_path: &'static str,
    pub tintable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WallpaperSpec {
    pub name: &'static str,
    pub relative_path: &'static str,
    pub category: &'static str,
    pub width: u32,
    pub height: u32,
}

/// Coarse category used only when a `.desktop` entry has no usable icon.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppCategory {
    AudioVideo,
    Development,
    Education,
    Game,
    Graphics,
    Network,
    Office,
    Settings,
    System,
    Utility,
    Other,
}

/// External icons are deliberately separated so callers cannot accidentally
/// tint branded application artwork through the Nexxus symbolic path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationIcon<'a> {
    External(&'a str),
    Builtin(&'static IconSpec),
}

pub fn icon(name: &str) -> Option<&'static IconSpec> {
    ICONS.iter().find(|item| item.name == name)
}

pub fn wallpaper(name: &str) -> Option<&'static WallpaperSpec> {
    WALLPAPERS.iter().find(|item| item.name == name)
}

/// Preserves a declared external icon and falls back only when it is absent.
pub fn resolve_application_icon<'a>(
    declared: Option<&'a str>,
    category: AppCategory,
) -> ApplicationIcon<'a> {
    if let Some(name) = declared.map(str::trim).filter(|name| !name.is_empty()) {
        return ApplicationIcon::External(name);
    }
    let fallback = match category {
        AppCategory::AudioVideo => "applications-multimedia",
        AppCategory::Development => "applications-development",
        AppCategory::Education | AppCategory::Other => "application-x-generic",
        AppCategory::Game => "applications-other",
        AppCategory::Graphics => "applications-graphics",
        AppCategory::Network => "applications-internet",
        AppCategory::Office => "applications-office",
        AppCategory::Settings => "preferences-system",
        AppCategory::System => "applications-system",
        AppCategory::Utility => "applications-utilities",
    };
    ApplicationIcon::Builtin(icon(fallback).expect("built-in fallback must exist"))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecolorError {
    InvalidUtf8,
    PaletteTokenMissing,
}

impl std::fmt::Display for RecolorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidUtf8 => "symbolic SVG is not valid UTF-8",
            Self::PaletteTokenMissing => "symbolic SVG does not contain the Nexxus palette token",
        })
    }
}
impl std::error::Error for RecolorError {}

/// Recolors only the explicit token used by project-owned symbolic SVGs.
pub fn recolor_symbolic_svg(svg: &[u8], rgb: [u8; 3]) -> Result<Vec<u8>, RecolorError> {
    let source = std::str::from_utf8(svg).map_err(|_| RecolorError::InvalidUtf8)?;
    if !source.contains(SYMBOLIC_PALETTE_TOKEN) {
        return Err(RecolorError::PaletteTokenMissing);
    }
    let color = format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
    Ok(source.replace(SYMBOLIC_PALETTE_TOKEN, &color).into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_external_desktop_icon() {
        assert_eq!(
            resolve_application_icon(Some("firefox"), AppCategory::Network),
            ApplicationIcon::External("firefox")
        );
    }

    #[test]
    fn category_fallback_is_deterministic() {
        let ApplicationIcon::Builtin(spec) =
            resolve_application_icon(Some(" "), AppCategory::Office)
        else {
            panic!("expected fallback")
        };
        assert_eq!(spec.name, "applications-office");
    }

    #[test]
    fn recolors_palette_token() {
        assert_eq!(
            recolor_symbolic_svg(br##"<path stroke="#FFFFFF"/>"##, [0x36, 0xF5, 0x7B]).unwrap(),
            br##"<path stroke="#36F57B"/>"##
        );
    }

    #[test]
    fn icon_names_are_unique() {
        for (i, left) in ICONS.iter().enumerate() {
            assert!(
                ICONS
                    .iter()
                    .skip(i + 1)
                    .all(|right| left.name != right.name),
                "duplicate icon: {}",
                left.name
            );
        }
    }
}
