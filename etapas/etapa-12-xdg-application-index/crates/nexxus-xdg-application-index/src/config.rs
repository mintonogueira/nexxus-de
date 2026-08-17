//! Resolution of XDG and Flatpak application roots from the process environment.

use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};

use thiserror::Error;

/// Origin of one application directory. The value is metadata, not package-manager truth.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationSource {
    UserXdg,
    SystemXdg,
    UserFlatpak,
    SystemFlatpak,
    Custom(String),
}

/// One `applications/` directory in precedence order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationRoot {
    pub path: PathBuf,
    pub source: ApplicationSource,
}

impl ApplicationRoot {
    /// Builds a custom root, primarily for integration tests and embedding.
    pub fn custom(path: impl Into<PathBuf>, label: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            source: ApplicationSource::Custom(label.into()),
        }
    }
}

/// Runtime configuration for scanning and live refresh.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationIndexConfig {
    pub roots: Vec<ApplicationRoot>,
    pub locales: Vec<String>,
    pub current_desktops: Vec<String>,
    pub max_desktop_file_bytes: u64,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("HOME is unavailable and XDG_DATA_HOME is not a usable absolute path")]
    HomeUnavailable,
}

impl ApplicationIndexConfig {
    /// Resolves XDG precedence and explicitly supplements Flatpak exports when
    /// they are not already represented by the configured XDG data directories.
    pub fn from_environment() -> Result<Self, ConfigError> {
        let home = env::var_os("HOME").map(PathBuf::from);
        let data_home = env_absolute_path("XDG_DATA_HOME")
            .or_else(|| home.as_ref().map(|value| value.join(".local/share")))
            .ok_or(ConfigError::HomeUnavailable)?;

        let data_dirs = env_absolute_path_list("XDG_DATA_DIRS").unwrap_or_else(|| {
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share"),
            ]
        });

        let mut roots = Vec::new();
        let mut seen = BTreeSet::new();
        push_unique_root(
            &mut roots,
            &mut seen,
            data_home.join("applications"),
            ApplicationSource::UserXdg,
        );

        if let Some(home) = &home {
            push_unique_root(
                &mut roots,
                &mut seen,
                home.join(".local/share/flatpak/exports/share/applications"),
                ApplicationSource::UserFlatpak,
            );
        }

        for data_dir in data_dirs {
            let source = if data_dir == Path::new("/var/lib/flatpak/exports/share") {
                ApplicationSource::SystemFlatpak
            } else if data_dir
                .to_string_lossy()
                .contains("/flatpak/exports/share")
            {
                ApplicationSource::UserFlatpak
            } else {
                ApplicationSource::SystemXdg
            };
            push_unique_root(&mut roots, &mut seen, data_dir.join("applications"), source);
        }

        push_unique_root(
            &mut roots,
            &mut seen,
            PathBuf::from("/var/lib/flatpak/exports/share/applications"),
            ApplicationSource::SystemFlatpak,
        );

        Ok(Self {
            roots,
            locales: locale_candidates(),
            current_desktops: env::var("XDG_CURRENT_DESKTOP")
                .ok()
                .map(|value| {
                    value
                        .split(':')
                        .filter(|item| !item.is_empty())
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default(),
            // A desktop entry is metadata; bounding reads prevents an accidental
            // or hostile multi-gigabyte file from becoming an indexing DoS.
            max_desktop_file_bytes: 2 * 1024 * 1024,
        })
    }
}

fn push_unique_root(
    roots: &mut Vec<ApplicationRoot>,
    seen: &mut BTreeSet<PathBuf>,
    path: PathBuf,
    source: ApplicationSource,
) {
    if seen.insert(path.clone()) {
        roots.push(ApplicationRoot { path, source });
    }
}

fn env_absolute_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}

fn env_absolute_path_list(name: &str) -> Option<Vec<PathBuf>> {
    let raw = env::var_os(name)?;
    let values: Vec<PathBuf> = env::split_paths(&raw)
        .filter(|path| path.is_absolute())
        .collect();
    (!values.is_empty()).then_some(values)
}

/// Produces the locale fallback order required for localized Desktop Entry keys.
fn locale_candidates() -> Vec<String> {
    let raw = env::var("LC_MESSAGES")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| env::var("LANG").ok().filter(|value| !value.is_empty()));

    let Some(raw) = raw else {
        return Vec::new();
    };
    if raw == "C" || raw == "POSIX" {
        return Vec::new();
    }

    let without_encoding = match raw.split_once('.') {
        Some((prefix, suffix)) => match suffix.split_once('@') {
            Some((_, modifier)) => format!("{prefix}@{modifier}"),
            None => prefix.to_owned(),
        },
        None => raw,
    };

    let (base, modifier) = match without_encoding.split_once('@') {
        Some((base, modifier)) => (base, Some(modifier)),
        None => (without_encoding.as_str(), None),
    };
    let (language, country) = match base.split_once('_') {
        Some((language, country)) => (language, Some(country)),
        None => (base, None),
    };

    let mut values = Vec::new();
    match (country, modifier) {
        (Some(country), Some(modifier)) => {
            values.push(format!("{language}_{country}@{modifier}"));
            values.push(format!("{language}_{country}"));
            values.push(format!("{language}@{modifier}"));
        }
        (Some(country), None) => values.push(format!("{language}_{country}")),
        (None, Some(modifier)) => values.push(format!("{language}@{modifier}")),
        (None, None) => {}
    }
    values.push(language.to_owned());
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_root_keeps_explicit_label() {
        let root = ApplicationRoot::custom("/tmp/apps", "fixture");
        assert_eq!(root.path, PathBuf::from("/tmp/apps"));
        assert_eq!(root.source, ApplicationSource::Custom("fixture".into()));
    }
}
