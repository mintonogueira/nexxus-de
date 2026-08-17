//! Backend-neutral UI foundation for the Nexxus desktop environment.
//!
//! The crate owns logical geometry, theming, a retained display list, software
//! rendering, input/focus routing, basic widgets and accessibility metadata.
//! Protocol-specific X11/Wayland presentation remains outside this crate.

#![forbid(unsafe_code)]

pub mod accessibility;
pub mod geometry;
pub mod input;
pub mod layout;
pub mod render;
pub mod theme;
pub mod widgets;

pub use accessibility::{AccessibilityNode, AccessibilityRole, AccessibilityTree};
pub use geometry::{Constraints, Insets, LogicalPoint, LogicalRect, LogicalSize, PhysicalRect, PhysicalSize, ScaleFactor};
pub use input::{Key, Modifiers, PointerButton, UiEvent, WidgetId};
pub use layout::{Axis, FlexItem, layout_flex};
pub use render::{DisplayList, DrawCommand, Frame, ImageData, RenderError, Renderer, SoftwareRenderer, TextMeasurer, TextStyle};
pub use theme::{Color, Palette, Theme, ThemeError, Typography, UiMetrics};
pub use widgets::{UiMessage, UiNode, UiTree, WidgetKind};
