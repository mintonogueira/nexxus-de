//! Small deterministic flex layout primitive used by container widgets.

use crate::geometry::LogicalRect;
use crate::input::WidgetId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexItem {
    pub id: WidgetId,
    pub basis: f32,
    pub grow: f32,
    pub min: f32,
    pub max: f32,
}

impl FlexItem {
    pub fn flexible(id: WidgetId) -> Self {
        Self { id, basis: 0.0, grow: 1.0, min: 0.0, max: f32::INFINITY }
    }
}

/// Lays out items along one axis. The function is deliberately deterministic:
/// all calculations happen in logical space; pixel rounding belongs to the
/// renderer's scale conversion so sibling edges remain consistent.
pub fn layout_flex(bounds: LogicalRect, axis: Axis, gap: f32, items: &[FlexItem]) -> Vec<(WidgetId, LogicalRect)> {
    if items.is_empty() {
        return Vec::new();
    }

    let main = match axis { Axis::Horizontal => bounds.width, Axis::Vertical => bounds.height }.max(0.0);
    let total_gap = gap.max(0.0) * items.len().saturating_sub(1) as f32;
    let available = (main - total_gap).max(0.0);
    let mut sizes: Vec<f32> = items.iter().map(|item| item.basis.max(item.min).min(item.max).max(0.0)).collect();
    let used: f32 = sizes.iter().sum();

    if used < available {
        let extra = available - used;
        let total_grow: f32 = items.iter().map(|item| item.grow.max(0.0)).sum();
        if total_grow > 0.0 {
            for (size, item) in sizes.iter_mut().zip(items) {
                let share = extra * item.grow.max(0.0) / total_grow;
                *size = (*size + share).min(item.max);
            }
        }
    } else if used > available && used > 0.0 {
        let ratio = available / used;
        for (size, item) in sizes.iter_mut().zip(items) {
            *size = (*size * ratio).max(item.min).min(item.max);
        }
    }

    // Residual space from max/min constraints stays at the end rather than
    // violating constraints or silently changing the requested gap.
    let mut cursor = match axis { Axis::Horizontal => bounds.x, Axis::Vertical => bounds.y };
    let mut result = Vec::with_capacity(items.len());
    for (index, (item, size)) in items.iter().zip(sizes).enumerate() {
        let rect = match axis {
            Axis::Horizontal => LogicalRect::new(cursor, bounds.y, size, bounds.height),
            Axis::Vertical => LogicalRect::new(bounds.x, cursor, bounds.width, size),
        };
        result.push((item.id, rect));
        cursor += size;
        if index + 1 < items.len() {
            cursor += gap.max(0.0);
        }
    }
    result
}
