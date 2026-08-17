//! Normalized layout model and deterministic geometry solver.

use nexxus_wm::{Geometry, GeometryError, SizeConstraints};
use std::num::NonZeroU64;
use thiserror::Error;

/// Fixed-point scale used by layout slots. Ten thousand units represent 100%.
pub const NORMALIZED_SCALE: u32 = 10_000;

/// A rectangle expressed in fixed-point coordinates inside an output work area.
///
/// The representation avoids floating-point drift and makes geometry calculations
/// deterministic across backends and architectures.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NormalizedRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl NormalizedRect {
    /// Creates a normalized slot and rejects empty or out-of-bounds rectangles.
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Result<Self, LayoutError> {
        let right = u32::from(x) + u32::from(width);
        let bottom = u32::from(y) + u32::from(height);
        if width == 0 || height == 0 {
            return Err(LayoutError::EmptySlot);
        }
        if right > NORMALIZED_SCALE || bottom > NORMALIZED_SCALE {
            return Err(LayoutError::SlotOutsideNormalizedArea);
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }

    /// Returns a slot covering the complete work area.
    pub fn full() -> Self {
        Self {
            x: 0,
            y: 0,
            width: NORMALIZED_SCALE as u16,
            height: NORMALIZED_SCALE as u16,
        }
    }
}

/// Ordered set of non-overlapping tile slots for one workspace layout.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutSpec {
    slots: Vec<NormalizedRect>,
}

impl LayoutSpec {
    /// Creates a layout and rejects overlap because tile slots represent mutually
    /// exclusive destinations in the assisted tiling model.
    pub fn new(slots: Vec<NormalizedRect>) -> Result<Self, LayoutError> {
        if slots.is_empty() {
            return Err(LayoutError::EmptyLayout);
        }

        for (index, first) in slots.iter().enumerate() {
            for second in slots.iter().skip(index + 1) {
                if rectangles_overlap(*first, *second) {
                    return Err(LayoutError::OverlappingSlots);
                }
            }
        }

        Ok(Self { slots })
    }

    /// Generates equal-width columns with exact normalized coverage.
    ///
    /// Integer boundaries are derived from cumulative fractions so rounding
    /// cannot create gaps between adjacent columns.
    pub fn balanced_columns(columns: u16) -> Result<Self, LayoutError> {
        if columns == 0 {
            return Err(LayoutError::InvalidColumnCount);
        }
        if u32::from(columns) > NORMALIZED_SCALE {
            return Err(LayoutError::InvalidColumnCount);
        }

        let mut slots = Vec::with_capacity(usize::from(columns));
        for column in 0..columns {
            let left = NORMALIZED_SCALE * u32::from(column) / u32::from(columns);
            let right = NORMALIZED_SCALE * u32::from(column + 1) / u32::from(columns);
            slots.push(NormalizedRect::new(
                left as u16,
                0,
                (right - left) as u16,
                NORMALIZED_SCALE as u16,
            )?);
        }
        Self::new(slots)
    }

    pub fn slots(&self) -> &[NormalizedRect] {
        &self.slots
    }

    pub fn slot(&self, index: usize) -> Option<NormalizedRect> {
        self.slots.get(index).copied()
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }
}

/// Backend-neutral physical output identifier.
///
/// Workspaces are not bound to this identifier. It is used only while solving
/// physical geometry so one logical workspace can span multiple outputs.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OutputId(NonZeroU64);

impl OutputId {
    pub fn new(value: u64) -> Result<Self, OutputIdError> {
        NonZeroU64::new(value).map(Self).ok_or(OutputIdError)
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
#[error("output id cannot be zero")]
pub struct OutputIdError;

/// Usable output rectangle after panels, docks or other reserved regions have
/// already been removed by the backend/session integration layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputArea {
    pub output: OutputId,
    pub geometry: Geometry,
}

impl OutputArea {
    pub const fn new(output: OutputId, geometry: Geometry) -> Self {
        Self { output, geometry }
    }
}

/// Errors emitted before any window state is mutated.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum LayoutError {
    #[error("a layout must contain at least one slot")]
    EmptyLayout,
    #[error("a layout slot cannot have zero width or height")]
    EmptySlot,
    #[error("a layout slot exceeds the normalized work area")]
    SlotOutsideNormalizedArea,
    #[error("layout slots cannot overlap")]
    OverlappingSlots,
    #[error("balanced column count must be between 1 and 10000")]
    InvalidColumnCount,
    #[error("normalized slot collapsed to zero pixels in this work area")]
    ZeroPixelSlot,
    #[error("window minimum size does not fit inside the selected slot")]
    MinimumSizeDoesNotFit,
    #[error("calculated window coordinates exceed the supported i32 range")]
    CoordinateOverflow,
    #[error(transparent)]
    Geometry(#[from] GeometryError),
}

/// Converts one normalized layout slot to concrete pixels and applies window
/// min/max constraints without letting the result escape the slot.
pub(crate) fn fit_slot(
    area: OutputArea,
    slot: NormalizedRect,
    constraints: SizeConstraints,
) -> Result<Geometry, LayoutError> {
    constraints.validate()?;

    let area_geometry = area.geometry;
    let left_offset = scaled_edge(area_geometry.width, slot.x);
    let right_offset = scaled_edge(
        area_geometry.width,
        u16::try_from(u32::from(slot.x) + u32::from(slot.width))
            .map_err(|_| LayoutError::SlotOutsideNormalizedArea)?,
    );
    let top_offset = scaled_edge(area_geometry.height, slot.y);
    let bottom_offset = scaled_edge(
        area_geometry.height,
        u16::try_from(u32::from(slot.y) + u32::from(slot.height))
            .map_err(|_| LayoutError::SlotOutsideNormalizedArea)?,
    );

    let cell_width = right_offset
        .checked_sub(left_offset)
        .ok_or(LayoutError::ZeroPixelSlot)?;
    let cell_height = bottom_offset
        .checked_sub(top_offset)
        .ok_or(LayoutError::ZeroPixelSlot)?;
    if cell_width == 0 || cell_height == 0 {
        return Err(LayoutError::ZeroPixelSlot);
    }

    let cell_width = u32::try_from(cell_width).map_err(|_| LayoutError::CoordinateOverflow)?;
    let cell_height = u32::try_from(cell_height).map_err(|_| LayoutError::CoordinateOverflow)?;

    if constraints.min_width > cell_width || constraints.min_height > cell_height {
        return Err(LayoutError::MinimumSizeDoesNotFit);
    }

    let width = constraints
        .max_width
        .map_or(cell_width, |maximum| cell_width.min(maximum))
        .max(constraints.min_width);
    let height = constraints
        .max_height
        .map_or(cell_height, |maximum| cell_height.min(maximum))
        .max(constraints.min_height);

    let x = i64::from(area_geometry.x)
        + i64::try_from(left_offset).map_err(|_| LayoutError::CoordinateOverflow)?
        + i64::from((cell_width - width) / 2);
    let y = i64::from(area_geometry.y)
        + i64::try_from(top_offset).map_err(|_| LayoutError::CoordinateOverflow)?
        + i64::from((cell_height - height) / 2);

    let x = i32::try_from(x).map_err(|_| LayoutError::CoordinateOverflow)?;
    let y = i32::try_from(y).map_err(|_| LayoutError::CoordinateOverflow)?;

    Geometry::new(x, y, width, height).map_err(LayoutError::from)
}

fn scaled_edge(total: u32, normalized: u16) -> u64 {
    u64::from(total) * u64::from(normalized) / u64::from(NORMALIZED_SCALE)
}

fn rectangles_overlap(first: NormalizedRect, second: NormalizedRect) -> bool {
    let first_right = u32::from(first.x) + u32::from(first.width);
    let second_right = u32::from(second.x) + u32::from(second.width);
    let first_bottom = u32::from(first.y) + u32::from(first.height);
    let second_bottom = u32::from(second.y) + u32::from(second.height);

    u32::from(first.x) < second_right
        && u32::from(second.x) < first_right
        && u32::from(first.y) < second_bottom
        && u32::from(second.y) < first_bottom
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balanced_columns_cover_the_full_normalized_width() {
        let layout = LayoutSpec::balanced_columns(3).unwrap();
        assert_eq!(layout.len(), 3);
        assert_eq!(layout.slots()[0].x, 0);
        let last = layout.slots()[2];
        assert_eq!(
            u32::from(last.x) + u32::from(last.width),
            NORMALIZED_SCALE
        );
    }

    #[test]
    fn overlapping_slots_are_rejected() {
        let first = NormalizedRect::new(0, 0, 6_000, 10_000).unwrap();
        let second = NormalizedRect::new(5_000, 0, 5_000, 10_000).unwrap();
        assert_eq!(
            LayoutSpec::new(vec![first, second]),
            Err(LayoutError::OverlappingSlots)
        );
    }
}
