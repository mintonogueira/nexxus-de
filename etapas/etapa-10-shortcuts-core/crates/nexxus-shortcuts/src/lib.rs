//! Backend-neutral global shortcut infrastructure for Nexxus.
//!
//! This crate owns shortcut representation, command/binding registries,
//! conflict validation, capture, dispatch, versioned persistence and the
//! initial X11 passive-grab adapter. Product modules remain responsible for
//! executing their own actions.

#![forbid(unsafe_code)]

mod command;
mod input;
mod model;
mod registry;
mod x11;

pub use command::{
    BrightnessAction, CaptureAction, CommandDescriptor, CommandId, CommandIdError, CommandTarget,
    LauncherAction, MediaAction, SessionAction, ShellAction, WmAction, WorkspaceAction,
};
pub use input::{InputEvent, InputKey, KeyState, ShortcutCapture, ShortcutRecognizer};
pub use model::{Key, KeyChord, Modifier, Trigger, TriggerParseError};
pub use registry::{
    BindingConflict, DispatchError, PersistedBinding, ShortcutConfig, ShortcutDispatchSink,
    ShortcutError, ShortcutRegistry, SHORTCUT_CONFIG_SCHEMA,
};
pub use x11::{GrabSpec, X11GrabError, X11ShortcutGrabs};
