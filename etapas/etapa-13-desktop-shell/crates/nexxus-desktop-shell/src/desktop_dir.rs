//! Minimal XDG desktop-directory resolution and folder creation.
//!
//! Stage 13 owns the desktop surface action "create folder" but deliberately
//! does not implement a full file manager. The resolver reads only the standard
//! user-dirs assignment needed to locate the user's Desktop directory.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DesktopDirectoryError {
    #[error("HOME is unavailable")]
    HomeUnavailable,
    #[error("desktop directory I/O failed: {0}")]
    Io(#[from] io::Error),
}

pub fn resolve_desktop_dir() -> Result<PathBuf, DesktopDirectoryError> {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or(DesktopDirectoryError::HomeUnavailable)?;
    let config_home = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| home.join(".config"));
    Ok(resolve_desktop_dir_from(&home, &config_home))
}

pub fn resolve_desktop_dir_from(home: &Path, config_home: &Path) -> PathBuf {
    let user_dirs = config_home.join("user-dirs.dirs");
    let Ok(contents) = fs::read_to_string(user_dirs) else {
        return home.join("Desktop");
    };
    for raw_line in contents.lines() {
        let line = raw_line.trim();
        let Some(value) = line.strip_prefix("XDG_DESKTOP_DIR=") else {
            continue;
        };
        let value = value.trim();
        if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
            continue;
        }
        let value = &value[1..value.len() - 1];
        if let Some(relative) = value.strip_prefix("$HOME/") {
            return home.join(relative);
        }
        if value == "$HOME" {
            return home.to_path_buf();
        }
        let path = PathBuf::from(value);
        if path.is_absolute() {
            return path;
        }
    }
    home.join("Desktop")
}

/// Creates a collision-free folder without invoking a shell. The conventional
/// first name is "New Folder" and numeric suffixes are added deterministically.
pub fn create_unique_folder(desktop_dir: &Path) -> Result<PathBuf, DesktopDirectoryError> {
    fs::create_dir_all(desktop_dir)?;
    for index in 1u32..=10_000 {
        let name = if index == 1 {
            "New Folder".to_owned()
        } else {
            format!("New Folder ({index})")
        };
        let candidate = desktop_dir.join(name);
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "unable to allocate a unique desktop folder name",
    )
    .into())
}

/// Lists only immediate directories. This is sufficient for desktop folder
/// icons and intentionally avoids implementing file-manager enumeration rules.
pub fn list_desktop_folders(desktop_dir: &Path) -> Result<Vec<PathBuf>, DesktopDirectoryError> {
    if !desktop_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut folders = Vec::new();
    for entry in fs::read_dir(desktop_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            folders.push(entry.path());
        }
    }
    folders.sort_by(|left, right| left.file_name().cmp(&right.file_name()));
    Ok(folders)
}
