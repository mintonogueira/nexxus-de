//! Stable logical commands and routing domains for shortcut dispatch.

use nexxus_tiling::{TILE_FIT_ACTION_ID, TilingAction};
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CommandId(String);

impl CommandId {
    pub fn new(value: impl Into<String>) -> Result<Self, CommandIdError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 128
            || value
                .bytes()
                .any(|byte| !(byte.is_ascii_alphanumeric() || b".-_".contains(&byte)))
        {
            return Err(CommandIdError(value));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CommandId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
#[error("invalid shortcut command identifier '{0}'")]
pub struct CommandIdError(String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WmAction {
    CycleCurrentWorkspace,
    CloseFocused,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceAction {
    CycleMruForward,
    CycleMruBackward,
    Previous,
    Next,
    MoveFocusedPrevious,
    MoveFocusedNext,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LauncherAction {
    ApplicationFinder,
    NexxusTerminal,
    Bashtop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShellAction {
    ApplicationMenu,
    DesktopMenu,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionAction {
    Lock,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureAction {
    DefaultMode,
    AlternateMode,
    ShiftMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaAction {
    VolumeDown,
    VolumeUp,
    VolumeMute,
    PlayPause,
    Previous,
    Next,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrightnessAction {
    Down,
    Up,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandTarget {
    Wm(WmAction),
    Workspaces(WorkspaceAction),
    Tiling(TilingAction),
    Launcher(LauncherAction),
    Shell(ShellAction),
    Session(SessionAction),
    Capture(CaptureAction),
    Media(MediaAction),
    Brightness(BrightnessAction),
}

/// Registry metadata intentionally contains only stable routing information.
/// User-facing descriptions belong to later UI/Settings stages.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandDescriptor {
    id: CommandId,
    target: CommandTarget,
}

impl CommandDescriptor {
    pub fn new(id: CommandId, target: CommandTarget) -> Self {
        Self { id, target }
    }

    pub fn id(&self) -> &CommandId {
        &self.id
    }

    pub fn target(&self) -> CommandTarget {
        self.target
    }

    pub(crate) fn builtins() -> Vec<Self> {
        vec![
            builtin(
                "nexxus.shell.application-menu",
                CommandTarget::Shell(ShellAction::ApplicationMenu),
            ),
            builtin(
                "nexxus.launcher.application-finder",
                CommandTarget::Launcher(LauncherAction::ApplicationFinder),
            ),
            builtin(
                TILE_FIT_ACTION_ID,
                CommandTarget::Tiling(TilingAction::TileFit),
            ),
            builtin(
                "nexxus.launcher.terminal",
                CommandTarget::Launcher(LauncherAction::NexxusTerminal),
            ),
            builtin(
                "nexxus.wm.cycle-current-workspace",
                CommandTarget::Wm(WmAction::CycleCurrentWorkspace),
            ),
            builtin(
                "nexxus.workspaces.cycle-mru-forward",
                CommandTarget::Workspaces(WorkspaceAction::CycleMruForward),
            ),
            builtin(
                "nexxus.workspaces.cycle-mru-backward",
                CommandTarget::Workspaces(WorkspaceAction::CycleMruBackward),
            ),
            builtin(
                "nexxus.session.lock",
                CommandTarget::Session(SessionAction::Lock),
            ),
            builtin(
                "nexxus.shell.desktop-menu",
                CommandTarget::Shell(ShellAction::DesktopMenu),
            ),
            builtin(
                "nexxus.launcher.bashtop",
                CommandTarget::Launcher(LauncherAction::Bashtop),
            ),
            builtin(
                "nexxus.wm.close-focused",
                CommandTarget::Wm(WmAction::CloseFocused),
            ),
            builtin(
                "nexxus.workspaces.previous",
                CommandTarget::Workspaces(WorkspaceAction::Previous),
            ),
            builtin(
                "nexxus.workspaces.next",
                CommandTarget::Workspaces(WorkspaceAction::Next),
            ),
            builtin(
                "nexxus.workspaces.move-focused-previous",
                CommandTarget::Workspaces(WorkspaceAction::MoveFocusedPrevious),
            ),
            builtin(
                "nexxus.workspaces.move-focused-next",
                CommandTarget::Workspaces(WorkspaceAction::MoveFocusedNext),
            ),
            builtin(
                "nexxus.capture.default",
                CommandTarget::Capture(CaptureAction::DefaultMode),
            ),
            builtin(
                "nexxus.capture.alternate",
                CommandTarget::Capture(CaptureAction::AlternateMode),
            ),
            builtin(
                "nexxus.capture.shift",
                CommandTarget::Capture(CaptureAction::ShiftMode),
            ),
            builtin(
                "nexxus.media.volume-down",
                CommandTarget::Media(MediaAction::VolumeDown),
            ),
            builtin(
                "nexxus.media.volume-up",
                CommandTarget::Media(MediaAction::VolumeUp),
            ),
            builtin(
                "nexxus.media.volume-mute",
                CommandTarget::Media(MediaAction::VolumeMute),
            ),
            builtin(
                "nexxus.media.play-pause",
                CommandTarget::Media(MediaAction::PlayPause),
            ),
            builtin(
                "nexxus.media.previous",
                CommandTarget::Media(MediaAction::Previous),
            ),
            builtin("nexxus.media.next", CommandTarget::Media(MediaAction::Next)),
            builtin(
                "nexxus.brightness.down",
                CommandTarget::Brightness(BrightnessAction::Down),
            ),
            builtin(
                "nexxus.brightness.up",
                CommandTarget::Brightness(BrightnessAction::Up),
            ),
        ]
    }
}

fn builtin(id: &str, target: CommandTarget) -> CommandDescriptor {
    CommandDescriptor::new(
        CommandId::new(id).expect("static built-in command identifier is valid"),
        target,
    )
}
