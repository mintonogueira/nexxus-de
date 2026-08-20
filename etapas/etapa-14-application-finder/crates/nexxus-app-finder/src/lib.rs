//! Compact, backend-neutral Application Finder for the Nexxus desktop.
//!
//! Stage 14 owns ranking, finder state, Nexxus UI composition and application
//! activation. XDG discovery/parsing remains exclusively in Stage 12.
#![forbid(unsafe_code)]

mod controller;
mod icon;
mod launch;
mod runtime;
mod search;
mod view;
mod window;

pub use controller::{FinderAction, FinderController, FinderState};
pub use icon::FinderIconResolver;
pub use launch::{LaunchError, LaunchPlan, execute_launch_plan, plan_application_launch};
pub use runtime::{FinderRuntime, FinderRuntimeError};
pub use search::{
    CommentProvider, FinderCorpus, FinderMatch, MatchField, NoCommentProvider, SearchDocument,
};
pub use view::FinderView;
pub use window::{
    FINDER_MINIMUM_SIZE, FINDER_PREFERRED_SIZE, FinderWindowRequest, FinderWindowSpec,
};

/// Stable shortcut command already registered by Stage 10.
pub const APPLICATION_FINDER_COMMAND_ID: &str = "nexxus.launcher.application-finder";
