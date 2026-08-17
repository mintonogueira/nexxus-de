//! Deterministic scanning of XDG application directories in precedence order.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use freedesktop_desktop_entry::DesktopEntry;
use thiserror::Error;

use crate::model::{IndexDiagnostic, IndexDiagnosticKind};
use crate::{
    ApplicationIndexConfig, ApplicationRecord, DesktopId, ExecTemplate, MainCategory,
    resolve_icon_reference,
};

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("application index configuration contains no roots")]
    NoRoots,
}

/// Builds one immutable generation. Missing application roots are normal and do
/// not fail startup because optional Flatpak/user directories may not exist yet.
pub fn scan(config: &ApplicationIndexConfig) -> Result<crate::IndexSnapshot, ScanError> {
    if config.roots.is_empty() {
        return Err(ScanError::NoRoots);
    }

    let mut entries = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let mut seen_ids = BTreeSet::new();

    for root in &config.roots {
        if !root.path.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        collect_desktop_files(&root.path, &mut files, &mut diagnostics);
        files.sort();

        for path in files {
            let Some(id) = desktop_id(&root.path, &path) else {
                continue;
            };
            if !seen_ids.insert(id.clone()) {
                diagnostics.push(IndexDiagnostic {
                    path,
                    kind: IndexDiagnosticKind::DuplicateDesktopId,
                    message: format!(
                        "desktop file ID {id} is already claimed by a higher-precedence entry"
                    ),
                });
                continue;
            }

            let metadata = match fs::metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) => {
                    diagnostics.push(io_diagnostic(path, error));
                    continue;
                }
            };
            if metadata.len() > config.max_desktop_file_bytes {
                diagnostics.push(IndexDiagnostic {
                    path,
                    kind: IndexDiagnosticKind::TooLarge,
                    message: format!(
                        "desktop entry is {} bytes; configured limit is {} bytes",
                        metadata.len(),
                        config.max_desktop_file_bytes
                    ),
                });
                continue;
            }

            let desktop = match DesktopEntry::from_path(path.clone(), Some(&config.locales)) {
                Ok(desktop) => desktop,
                Err(error) => {
                    diagnostics.push(IndexDiagnostic {
                        path,
                        kind: IndexDiagnosticKind::InvalidDesktopEntry,
                        message: error.to_string(),
                    });
                    continue;
                }
            };

            if desktop.hidden() {
                continue;
            }
            if desktop.type_() != Some("Application") {
                continue;
            }

            let Some(name) = desktop
                .name(&config.locales)
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
            else {
                diagnostics.push(IndexDiagnostic {
                    path,
                    kind: IndexDiagnosticKind::MissingName,
                    message: "Application entry has no usable Name".to_owned(),
                });
                continue;
            };

            let dbus_activatable = desktop.dbus_activatable();
            let exec = match desktop.exec() {
                Some(raw) => match ExecTemplate::parse(raw) {
                    Ok(template) => Some(template),
                    Err(error) => {
                        diagnostics.push(IndexDiagnostic {
                            path: path.clone(),
                            kind: IndexDiagnosticKind::InvalidExec,
                            message: error.to_string(),
                        });
                        if dbus_activatable {
                            None
                        } else {
                            continue;
                        }
                    }
                },
                None if dbus_activatable => None,
                None => {
                    diagnostics.push(IndexDiagnostic {
                        path,
                        kind: IndexDiagnosticKind::MissingExec,
                        message: "Application requires Exec when DBusActivatable is false"
                            .to_owned(),
                    });
                    continue;
                }
            };

            let categories = string_list(desktop.categories());
            let mut main_categories: Vec<MainCategory> = categories
                .iter()
                .filter_map(|category| MainCategory::from_xdg(category))
                .collect();
            main_categories.sort();
            main_categories.dedup();
            let fallback_category = main_categories
                .first()
                .copied()
                .unwrap_or(MainCategory::Other);

            let keywords = desktop
                .keywords(&config.locales)
                .unwrap_or_default()
                .into_iter()
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
                .collect();

            let record = ApplicationRecord {
                id: DesktopId::new(id),
                desktop_file: path,
                source: root.source.clone(),
                name,
                exec,
                dbus_activatable,
                icon: resolve_icon_reference(desktop.icon(), fallback_category),
                categories,
                main_categories,
                keywords,
                no_display: desktop.no_display(),
                visible_in_current_desktop: visible_in_current_desktop(
                    desktop.only_show_in().as_deref(),
                    desktop.not_show_in().as_deref(),
                    &config.current_desktops,
                ),
            };
            entries.insert(record.id.clone(), record);
        }
    }

    Ok(crate::IndexSnapshot::from_parts(1, entries, diagnostics))
}

fn string_list(values: Option<Vec<&str>>) -> Vec<String> {
    values
        .unwrap_or_default()
        .into_iter()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

fn desktop_id(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    let components: Vec<String> = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();
    (!components.is_empty()).then(|| components.join("-"))
}

/// Collects desktop files recursively without following directory symlinks,
/// which avoids cycles while still accepting symlinks that resolve to files.
fn collect_desktop_files(
    directory: &Path,
    files: &mut Vec<PathBuf>,
    diagnostics: &mut Vec<IndexDiagnostic>,
) {
    let read = match fs::read_dir(directory) {
        Ok(read) => read,
        Err(error) => {
            diagnostics.push(io_diagnostic(directory.to_path_buf(), error));
            return;
        }
    };

    let mut children: Vec<_> = read.filter_map(Result::ok).collect();
    children.sort_by_key(|entry| entry.file_name());
    for entry in children {
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(error) => {
                diagnostics.push(io_diagnostic(path, error));
                continue;
            }
        };
        if file_type.is_dir() {
            collect_desktop_files(&path, files, diagnostics);
            continue;
        }
        if file_type.is_symlink() && !fs::metadata(&path).is_ok_and(|meta| meta.is_file()) {
            continue;
        }
        if path
            .extension()
            .is_some_and(|extension| extension == "desktop")
        {
            files.push(path);
        }
    }
}

fn io_diagnostic(path: PathBuf, error: std::io::Error) -> IndexDiagnostic {
    IndexDiagnostic {
        path,
        kind: IndexDiagnosticKind::Io,
        message: error.to_string(),
    }
}

fn visible_in_current_desktop(
    only_show_in: Option<&[&str]>,
    not_show_in: Option<&[&str]>,
    current_desktops: &[String],
) -> bool {
    for desktop in current_desktops {
        if only_show_in.is_some_and(|values| values.contains(&desktop.as_str())) {
            return true;
        }
        if not_show_in.is_some_and(|values| values.contains(&desktop.as_str())) {
            return false;
        }
    }
    only_show_in.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_desktop_id_replaces_separator_with_dash() {
        assert_eq!(
            desktop_id(
                Path::new("/data/applications"),
                Path::new("/data/applications/foo/bar.desktop")
            ),
            Some("foo-bar.desktop".to_owned())
        );
    }

    #[test]
    fn only_show_in_defaults_to_hidden_without_match() {
        assert!(!visible_in_current_desktop(
            Some(&["GNOME"]),
            None,
            &["NEXXUS".to_owned()]
        ));
    }
}
