//! Backend-neutral tiling and snap engine for Nexxus.
//!
//! The engine computes deterministic geometries for workspace layouts, preserves
//! floating geometry through `nexxus-wm`, exposes the `tile-fit` action consumed
//! by the future Shortcuts Core and produces snap intents for later UI overlays.
//! It intentionally contains no X11 or Wayland implementation details.

#![forbid(unsafe_code)]

mod engine;
mod layout;
mod snap;

pub use engine::{
    Assignment, DEFAULT_TILE_FIT_SHORTCUT, TILE_FIT_ACTION_ID, TilePlan, TileTarget, TilingAction,
    TilingEngine, TilingError, TilingEvent, UntilePlan,
};
pub use layout::{
    LayoutError, LayoutSpec, NORMALIZED_SCALE, NormalizedRect, OutputArea, OutputId, OutputIdError,
};
pub use snap::{Point, SnapDetector, SnapDetectorError, SnapIntent, SnapTarget};
