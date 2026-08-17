//! Logical and physical geometry shared by widgets and renderers.

/// Point in logical desktop-independent pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LogicalPoint {
    pub x: f32,
    pub y: f32,
}

impl LogicalPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Size in logical desktop-independent pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LogicalSize {
    pub width: f32,
    pub height: f32,
}

impl LogicalSize {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// Rectangle in logical desktop-independent pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LogicalRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LogicalRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Returns whether a logical point falls inside the half-open rectangle.
    pub fn contains(self, point: LogicalPoint) -> bool {
        point.x >= self.x
            && point.y >= self.y
            && point.x < self.x + self.width
            && point.y < self.y + self.height
    }

    /// Intersects two rectangles while never returning a negative size.
    pub fn intersection(self, other: Self) -> Option<Self> {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);
        (right > left && bottom > top).then(|| Self::new(left, top, right - left, bottom - top))
    }

    pub const fn size(self) -> LogicalSize {
        LogicalSize::new(self.width, self.height)
    }
}

/// Insets used by widget padding and content areas.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Insets {
    pub const fn uniform(value: f32) -> Self {
        Self { left: value, top: value, right: value, bottom: value }
    }

    pub fn shrink(self, rect: LogicalRect) -> LogicalRect {
        LogicalRect::new(
            rect.x + self.left,
            rect.y + self.top,
            (rect.width - self.left - self.right).max(0.0),
            (rect.height - self.top - self.bottom).max(0.0),
        )
    }
}

/// Min/max constraints applied by reusable layout code.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Constraints {
    pub min: LogicalSize,
    pub max: LogicalSize,
}

impl Constraints {
    pub const fn new(min: LogicalSize, max: LogicalSize) -> Self {
        Self { min, max }
    }

    pub fn clamp(self, size: LogicalSize) -> LogicalSize {
        LogicalSize::new(
            size.width.max(self.min.width).min(self.max.width),
            size.height.max(self.min.height).min(self.max.height),
        )
    }
}

/// Positive scale factor used to map logical units to physical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaleFactor(f32);

impl ScaleFactor {
    /// Validates a scale. Zero, negative and non-finite values are rejected.
    pub fn new(value: f32) -> Option<Self> {
        (value.is_finite() && value > 0.0).then_some(Self(value))
    }

    pub const fn get(self) -> f32 {
        self.0
    }

    pub fn physical_size(self, logical: LogicalSize) -> PhysicalSize {
        PhysicalSize {
            width: (logical.width.max(0.0) * self.0).round() as u32,
            height: (logical.height.max(0.0) * self.0).round() as u32,
        }
    }

    /// Converts rectangle edges independently. Shared logical edges therefore
    /// round to the same physical pixel, avoiding fractional-scale gaps.
    pub fn physical_rect(self, logical: LogicalRect) -> PhysicalRect {
        let left = (logical.x * self.0).round() as i32;
        let top = (logical.y * self.0).round() as i32;
        let right = ((logical.x + logical.width) * self.0).round() as i32;
        let bottom = ((logical.y + logical.height) * self.0).round() as i32;
        PhysicalRect::new(left, top, (right - left).max(0) as u32, (bottom - top).max(0) as u32)
    }
}

impl Default for ScaleFactor {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Physical framebuffer size.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PhysicalSize {
    pub width: u32,
    pub height: u32,
}

/// Physical integer rectangle.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PhysicalRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl PhysicalRect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn intersection(self, other: Self) -> Option<Self> {
        let left = i64::from(self.x).max(i64::from(other.x));
        let top = i64::from(self.y).max(i64::from(other.y));
        let right = (i64::from(self.x) + i64::from(self.width)).min(i64::from(other.x) + i64::from(other.width));
        let bottom = (i64::from(self.y) + i64::from(self.height)).min(i64::from(other.y) + i64::from(other.height));
        (right > left && bottom > top).then(|| Self::new(left as i32, top as i32, (right - left) as u32, (bottom - top) as u32))
    }
}
