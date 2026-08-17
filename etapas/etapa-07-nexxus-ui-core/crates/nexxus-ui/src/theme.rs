//! Opaque dark theme primitives and consistent UI metrics.

use std::error::Error;
use std::fmt;

/// Straight-alpha RGBA color used by the Nexxus display list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn is_opaque(self) -> bool {
        self.a == 255
    }
}

/// Semantic palette. Structural surfaces are intentionally opaque.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Palette {
    pub background: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub border: Color,
    pub text: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_text: Color,
    pub danger: Color,
    pub selection: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            background: Color::rgb(12, 15, 16),
            surface: Color::rgb(20, 24, 25),
            surface_alt: Color::rgb(29, 34, 35),
            border: Color::rgb(58, 66, 67),
            text: Color::rgb(238, 242, 242),
            text_muted: Color::rgb(164, 174, 174),
            accent: Color::rgb(39, 190, 98),
            accent_text: Color::rgb(6, 17, 10),
            danger: Color::rgb(220, 76, 76),
            selection: Color::rgb(35, 110, 64),
        }
    }
}

/// Typography contract. Stage 08 may package assets, while this stage defines
/// Hack as the semantic default and permits user-selected families later.
#[derive(Clone, Debug, PartialEq)]
pub struct Typography {
    pub family: String,
    pub body_size: f32,
    pub small_size: f32,
    pub title_size: f32,
    pub line_height: f32,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            family: "Hack".to_owned(),
            body_size: 13.0,
            small_size: 11.0,
            title_size: 15.0,
            line_height: 1.35,
        }
    }
}

/// Shared metrics prevent every future component from inventing dimensions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMetrics {
    pub control_height: f32,
    pub compact_height: f32,
    pub padding: f32,
    pub gap: f32,
    pub border_width: f32,
    pub focus_width: f32,
}

impl Default for UiMetrics {
    fn default() -> Self {
        Self {
            control_height: 32.0,
            compact_height: 26.0,
            padding: 8.0,
            gap: 6.0,
            border_width: 1.0,
            focus_width: 2.0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Theme {
    pub palette: Palette,
    pub typography: Typography,
    pub metrics: UiMetrics,
}

impl Theme {
    /// Enforces the no-translucent-surface identity rule without forbidding
    /// alpha where technically necessary for glyph/SVG antialiasing.
    pub fn validate(&self) -> Result<(), ThemeError> {
        let surfaces = [
            ("background", self.palette.background),
            ("surface", self.palette.surface),
            ("surface_alt", self.palette.surface_alt),
        ];
        for (name, color) in surfaces {
            if !color.is_opaque() {
                return Err(ThemeError::TranslucentSurface(name));
            }
        }
        if self.typography.family.trim().is_empty() {
            return Err(ThemeError::EmptyFontFamily);
        }
        if self.typography.body_size <= 0.0 || self.typography.line_height <= 0.0 {
            return Err(ThemeError::InvalidTypographyMetrics);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThemeError {
    TranslucentSurface(&'static str),
    EmptyFontFamily,
    InvalidTypographyMetrics,
}

impl fmt::Display for ThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TranslucentSurface(name) => write!(f, "structural surface {name} must be opaque"),
            Self::EmptyFontFamily => f.write_str("font family cannot be empty"),
            Self::InvalidTypographyMetrics => f.write_str("typography metrics must be positive"),
        }
    }
}

impl Error for ThemeError {}
