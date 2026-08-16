//! Backend-neutral window identifiers, geometry, metadata and state.

use std::num::NonZeroU64;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WindowId(NonZeroU64);

impl WindowId {
    /// Creates an internal identifier. Zero is reserved as an invalid sentinel.
    pub fn new(value: u64) -> Result<Self, WindowIdError> {
        NonZeroU64::new(value).map(Self).ok_or(WindowIdError)
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
#[error("window id cannot be zero")]
pub struct WindowIdError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Geometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Geometry {
    /// Creates geometry and rejects zero-sized surfaces.
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Result<Self, GeometryError> {
        if width == 0 || height == 0 {
            return Err(GeometryError::ZeroSize);
        }
        Ok(Self { x, y, width, height })
    }

    /// Applies size constraints while preserving the requested position.
    pub fn constrained(self, constraints: SizeConstraints) -> Result<Self, GeometryError> {
        constraints.validate()?;
        Ok(Self {
            width: constraints.clamp_width(self.width),
            height: constraints.clamp_height(self.height),
            ..self
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SizeConstraints {
    pub min_width: u32,
    pub min_height: u32,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

impl Default for SizeConstraints {
    fn default() -> Self {
        Self { min_width: 1, min_height: 1, max_width: None, max_height: None }
    }
}

impl SizeConstraints {
    /// Rejects contradictory bounds before they influence window state.
    pub fn validate(self) -> Result<(), GeometryError> {
        if self.min_width == 0 || self.min_height == 0 {
            return Err(GeometryError::ZeroMinimum);
        }
        if self.max_width.is_some_and(|maximum| maximum < self.min_width)
            || self.max_height.is_some_and(|maximum| maximum < self.min_height)
        {
            return Err(GeometryError::InvertedBounds);
        }
        Ok(())
    }

    fn clamp_width(self, value: u32) -> u32 {
        let value = value.max(self.min_width);
        self.max_width.map_or(value, |maximum| value.min(maximum))
    }

    fn clamp_height(self, value: u32) -> u32 {
        let value = value.max(self.min_height);
        self.max_height.map_or(value, |maximum| value.min(maximum))
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum GeometryError {
    #[error("window geometry cannot have zero width or height")]
    ZeroSize,
    #[error("minimum window size must be at least one logical pixel")]
    ZeroMinimum,
    #[error("maximum window size cannot be smaller than its minimum")]
    InvertedBounds,
}

/// Backend-neutral application identity. X11 WM_CLASS and Wayland app_id are
/// normalized into this field by their future adapter stages.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WindowMetadata {
    pub title: String,
    pub application_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowPlacement {
    Floating,
    Tiled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationState {
    Normal,
    Maximized,
    Fullscreen,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RestoreSnapshot {
    pub geometry: Geometry,
    pub placement: WindowPlacement,
    pub presentation: PresentationState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Window {
    pub id: WindowId,
    pub geometry: Geometry,
    pub constraints: SizeConstraints,
    pub visible: bool,
    pub mapped: bool,
    pub active: bool,
    pub placement: WindowPlacement,
    pub presentation: PresentationState,
    pub metadata: WindowMetadata,
    pub floating_geometry: Geometry,
    restore_stack: Vec<RestoreSnapshot>,
}

impl Window {
    /// Builds a new normal floating window with validated geometry constraints.
    pub fn new(id: WindowId, geometry: Geometry, constraints: SizeConstraints, metadata: WindowMetadata) -> Result<Self, GeometryError> {
        let geometry = geometry.constrained(constraints)?;
        Ok(Self {
            id,
            geometry,
            constraints,
            visible: true,
            mapped: true,
            active: false,
            placement: WindowPlacement::Floating,
            presentation: PresentationState::Normal,
            metadata,
            floating_geometry: geometry,
            restore_stack: Vec::new(),
        })
    }

    pub(crate) fn push_restore_snapshot(&mut self) {
        self.restore_stack.push(RestoreSnapshot { geometry: self.geometry, placement: self.placement, presentation: self.presentation });
    }

    pub(crate) fn pop_restore_snapshot(&mut self) -> Option<RestoreSnapshot> {
        self.restore_stack.pop()
    }

    pub(crate) fn update_geometry(&mut self, geometry: Geometry) -> Result<(), GeometryError> {
        self.geometry = geometry.constrained(self.constraints)?;
        if self.presentation == PresentationState::Normal && self.placement == WindowPlacement::Floating {
            self.floating_geometry = self.geometry;
        }
        Ok(())
    }
}
