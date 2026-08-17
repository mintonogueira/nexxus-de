//! Pointer-edge snap detection and UI-neutral preview intents.

use crate::{NormalizedRect, OutputArea};
use thiserror::Error;

/// Logical pointer coordinates in the same global coordinate space as outputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// Direct snap destinations whose geometry is fully known without a visual
/// overlay. Top-center intentionally requests the later layout chooser UI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapTarget {
    LeftHalf,
    RightHalf,
    TopLeftQuarter,
    TopRightQuarter,
    BottomLeftQuarter,
    BottomRightQuarter,
}

/// Result of edge hit-testing. The engine can immediately tile direct targets or
/// emit `ShowLayoutChoices` for a future overlay without implementing that UI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapIntent {
    Tile {
        target: SnapTarget,
        slot: NormalizedRect,
    },
    ShowLayoutChoices,
}

/// Stateless snap hit-testing configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapDetector {
    edge_threshold: u32,
}

impl Default for SnapDetector {
    fn default() -> Self {
        Self { edge_threshold: 24 }
    }
}

impl SnapDetector {
    /// Creates a detector with an explicit logical-pixel edge threshold.
    pub fn new(edge_threshold: u32) -> Result<Self, SnapDetectorError> {
        if edge_threshold == 0 {
            return Err(SnapDetectorError);
        }
        Ok(Self { edge_threshold })
    }

    pub const fn edge_threshold(self) -> u32 {
        self.edge_threshold
    }

    /// Detects one snap intent only when the pointer is inside the usable area.
    ///
    /// Corners take precedence over straight edges. Top-center emits a layout
    /// chooser hook because the definitive visual overlay belongs to a later UI
    /// stage.
    pub fn detect(self, area: OutputArea, pointer: Point) -> Option<SnapIntent> {
        let geometry = area.geometry;
        let left = i64::from(geometry.x);
        let top = i64::from(geometry.y);
        let right = left + i64::from(geometry.width) - 1;
        let bottom = top + i64::from(geometry.height) - 1;
        let x = i64::from(pointer.x);
        let y = i64::from(pointer.y);

        if x < left || x > right || y < top || y > bottom {
            return None;
        }

        let horizontal_threshold =
            i64::from(self.edge_threshold.min(geometry.width.saturating_sub(1).max(1)));
        let vertical_threshold =
            i64::from(self.edge_threshold.min(geometry.height.saturating_sub(1).max(1)));

        let near_left = x - left < horizontal_threshold;
        let near_right = right - x < horizontal_threshold;
        let near_top = y - top < vertical_threshold;
        let near_bottom = bottom - y < vertical_threshold;

        if near_top && near_left {
            return Some(tile_intent(SnapTarget::TopLeftQuarter));
        }
        if near_top && near_right {
            return Some(tile_intent(SnapTarget::TopRightQuarter));
        }
        if near_bottom && near_left {
            return Some(tile_intent(SnapTarget::BottomLeftQuarter));
        }
        if near_bottom && near_right {
            return Some(tile_intent(SnapTarget::BottomRightQuarter));
        }
        if near_left {
            return Some(tile_intent(SnapTarget::LeftHalf));
        }
        if near_right {
            return Some(tile_intent(SnapTarget::RightHalf));
        }
        if near_top {
            return Some(SnapIntent::ShowLayoutChoices);
        }

        None
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
#[error("snap edge threshold must be greater than zero")]
pub struct SnapDetectorError;

fn tile_intent(target: SnapTarget) -> SnapIntent {
    let slot = match target {
        SnapTarget::LeftHalf => NormalizedRect::new(0, 0, 5_000, 10_000),
        SnapTarget::RightHalf => NormalizedRect::new(5_000, 0, 5_000, 10_000),
        SnapTarget::TopLeftQuarter => NormalizedRect::new(0, 0, 5_000, 5_000),
        SnapTarget::TopRightQuarter => NormalizedRect::new(5_000, 0, 5_000, 5_000),
        SnapTarget::BottomLeftQuarter => NormalizedRect::new(0, 5_000, 5_000, 5_000),
        SnapTarget::BottomRightQuarter => NormalizedRect::new(5_000, 5_000, 5_000, 5_000),
    }
    .expect("static normalized snap rectangles are valid");

    SnapIntent::Tile { target, slot }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OutputId;
    use nexxus_wm::Geometry;

    fn area() -> OutputArea {
        OutputArea::new(
            OutputId::new(1).unwrap(),
            Geometry::new(100, 50, 1_000, 800).unwrap(),
        )
    }

    #[test]
    fn top_center_requests_layout_choices() {
        assert_eq!(
            SnapDetector::default().detect(area(), Point { x: 600, y: 50 }),
            Some(SnapIntent::ShowLayoutChoices)
        );
    }

    #[test]
    fn left_edge_requests_half_snap() {
        assert!(matches!(
            SnapDetector::default().detect(area(), Point { x: 100, y: 450 }),
            Some(SnapIntent::Tile {
                target: SnapTarget::LeftHalf,
                ..
            })
        ));
    }

    #[test]
    fn point_outside_work_area_is_ignored() {
        assert_eq!(
            SnapDetector::default().detect(area(), Point { x: 99, y: 450 }),
            None
        );
    }
}
