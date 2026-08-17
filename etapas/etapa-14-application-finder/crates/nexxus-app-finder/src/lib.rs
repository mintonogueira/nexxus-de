//! Compact, backend-neutral Application Finder for the Nexxus desktop.
//!
//! Stage 14 owns ranking, finder state, Nexxus UI composition and application
//! activation. XDG discovery/parsing remains exclusively in Stage 12.
#![forbid(unsafe_code)]

mod controller;
mod launch;
mod runtime;
mod search;
mod view;

pub use controller::{FinderAction, FinderController, FinderState};
pub use launch::{LaunchError, LaunchPlan, execute_launch_plan, plan_application_launch};
pub use runtime::{FinderRuntime, FinderRuntimeError};
pub use search::{
    CommentProvider, FinderCorpus, FinderMatch, MatchField, NoCommentProvider, SearchDocument,
};
pub use view::FinderView;

/// Stable shortcut command already registered by Stage 10.
pub const APPLICATION_FINDER_COMMAND_ID: &str = "nexxus.launcher.application-finder";
