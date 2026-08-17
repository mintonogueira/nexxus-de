//! Application icon references and deterministic Nexxus fallback selection.

use std::path::{Path, PathBuf};

use nexxus_assets::{ApplicationIcon, resolve_application_icon};

use crate::MainCategory;

/// An icon reference safe for later UI consumers to resolve/render.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IconReference {
    ExternalPath(PathBuf),
    ExternalName(String),
    NexxusFallback {
        name: String,
        relative_path: String,
    },
}

impl IconReference {
    /// Returns the exact icon string required by `%i` expansion when available.
    pub fn exec_icon_value(&self) -> Option<&str> {
        match self {
            Self::ExternalPath(path) => path.to_str(),
            Self::ExternalName(name) | Self::NexxusFallback { name, .. } => Some(name),
        }
    }
}

/// Preserves official application artwork. Nexxus fallbacks are selected only
/// when `Icon=` is absent/empty, matching the visual contract from Stage 08.
pub fn resolve_icon_reference(
    declared: Option<&str>,
    category: MainCategory,
) -> IconReference {
    if let Some(value) = declared.map(str::trim).filter(|value| !value.is_empty()) {
        let path = Path::new(value);
        return if path.is_absolute() {
            IconReference::ExternalPath(path.to_path_buf())
        } else {
            IconReference::ExternalName(value.to_owned())
        };
    }

    match resolve_application_icon(None, category.asset_category()) {
        ApplicationIcon::Builtin(spec) => IconReference::NexxusFallback {
            name: spec.name.to_owned(),
            relative_path: spec.relative_path.to_owned(),
        },
        ApplicationIcon::External(_) => unreachable!("missing icons always use a Nexxus fallback"),
    }
}
