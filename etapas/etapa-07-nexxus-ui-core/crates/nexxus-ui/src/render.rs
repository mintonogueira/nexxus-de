//! Retained display list and backend-neutral software renderer.
//!
//! The renderer produces a plain RGBA frame. X11 and future Wayland adapters
//! can upload/present that frame without leaking protocol types into `nexxus-ui`.

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use cosmic_text::{Attrs, Buffer, Color as CosmicColor, Family, FontSystem, Metrics, Shaping, SwashCache, Weight};

use crate::geometry::{LogicalRect, LogicalSize, PhysicalRect, PhysicalSize, ScaleFactor};
use crate::theme::Color;

/// Text attributes stored in display commands.
#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    pub family: String,
    pub size: f32,
    pub line_height: f32,
    pub color: Color,
    pub bold: bool,
}

impl TextStyle {
    pub fn new(family: impl Into<String>, size: f32, line_height: f32, color: Color) -> Self {
        Self { family: family.into(), size, line_height, color, bold: false }
    }
}

/// Immutable RGBA image. Pixels are straight-alpha, row-major RGBA8.
#[derive(Clone, Debug, PartialEq)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl ImageData {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, RenderError> {
        let expected = usize::try_from(width)
            .ok()
            .and_then(|w| usize::try_from(height).ok().and_then(|h| w.checked_mul(h)))
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(RenderError::SizeOverflow)?;
        if pixels.len() != expected {
            return Err(RenderError::InvalidImageData { expected, actual: pixels.len() });
        }
        Ok(Self { width, height, pixels })
    }
}

/// Renderer-independent drawing operation.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawCommand {
    Clear(Color),
    FillRect { rect: LogicalRect, color: Color },
    StrokeRect { rect: LogicalRect, color: Color, width: f32 },
    Text { rect: LogicalRect, text: String, style: TextStyle },
    Image { rect: LogicalRect, image: ImageData },
    Svg { rect: LogicalRect, bytes: Vec<u8> },
    PushClip(LogicalRect),
    PopClip,
}

/// Ordered retained display list. Widgets generate commands; presentation
/// targets consume the resulting frame independently.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DisplayList {
    commands: Vec<DrawCommand>,
}

impl DisplayList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, command: DrawCommand) {
        self.commands.push(command);
    }

    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

/// Finished straight-alpha RGBA8 frame suitable for platform presentation.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub size: PhysicalSize,
    pub stride: usize,
    pub pixels: Vec<u8>,
}

impl Frame {
    fn new(size: PhysicalSize) -> Result<Self, RenderError> {
        let width = usize::try_from(size.width).map_err(|_| RenderError::SizeOverflow)?;
        let height = usize::try_from(size.height).map_err(|_| RenderError::SizeOverflow)?;
        let stride = width.checked_mul(4).ok_or(RenderError::SizeOverflow)?;
        let len = stride.checked_mul(height).ok_or(RenderError::SizeOverflow)?;
        Ok(Self { size, stride, pixels: vec![0; len] })
    }

    /// Writes a portable PPM snapshot for the Stage 07 demo without adding an
    /// image-encoding dependency to the Nexxus UI API.
    pub fn save_ppm(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let mut file = File::create(path)?;
        write!(file, "P6\n{} {}\n255\n", self.size.width, self.size.height)?;
        for pixel in self.pixels.chunks_exact(4) {
            file.write_all(&pixel[..3])?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RenderError {
    SizeOverflow,
    InvalidImageData { expected: usize, actual: usize },
    ClipStackUnderflow,
    Svg(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SizeOverflow => f.write_str("frame/image dimensions overflow addressable memory"),
            Self::InvalidImageData { expected, actual } => write!(f, "invalid RGBA image length: expected {expected}, got {actual}"),
            Self::ClipStackUnderflow => f.write_str("display list contains PopClip without matching PushClip"),
            Self::Svg(message) => write!(f, "SVG parse/render error: {message}"),
        }
    }
}

impl Error for RenderError {}

/// Abstract renderer contract consumed by UI surfaces and future backends.
pub trait Renderer {
    fn render(&mut self, list: &DisplayList, logical_size: LogicalSize, scale: ScaleFactor) -> Result<Frame, RenderError>;
}

/// Measurement contract used by layout without coupling widgets to a concrete
/// shaping implementation.
pub trait TextMeasurer {
    fn measure_text(&mut self, text: &str, style: &TextStyle, max_width: Option<f32>) -> LogicalSize;
}

/// CPU renderer chosen as the first portable target. It gives X11 a simple
/// framebuffer contract while remaining directly reusable by a Wayland target.
pub struct SoftwareRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
}

impl fmt::Debug for SoftwareRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SoftwareRenderer").finish_non_exhaustive()
    }
}

impl Default for SoftwareRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl SoftwareRenderer {
    pub fn new() -> Self {
        Self { font_system: FontSystem::new(), swash_cache: SwashCache::new() }
    }

    fn frame_bounds(frame: &Frame) -> PhysicalRect {
        PhysicalRect::new(0, 0, frame.size.width, frame.size.height)
    }

    fn current_clip(frame: &Frame, clips: &[PhysicalRect]) -> Option<PhysicalRect> {
        let mut clip = Self::frame_bounds(frame);
        for item in clips {
            clip = clip.intersection(*item)?;
        }
        Some(clip)
    }

    /// Straight-alpha source-over blending used for antialiased text, images
    /// and SVG. Structural Nexxus surfaces remain opaque by theme validation.
    fn blend_pixel(frame: &mut Frame, x: i32, y: i32, color: Color, clip: PhysicalRect) {
        if x < clip.x || y < clip.y || x >= clip.x + clip.width as i32 || y >= clip.y + clip.height as i32 {
            return;
        }
        if x < 0 || y < 0 || x >= frame.size.width as i32 || y >= frame.size.height as i32 {
            return;
        }
        let index = y as usize * frame.stride + x as usize * 4;
        let sa = u32::from(color.a);
        let inv = 255 - sa;
        for (offset, src) in [color.r, color.g, color.b].into_iter().enumerate() {
            let dst = u32::from(frame.pixels[index + offset]);
            frame.pixels[index + offset] = ((u32::from(src) * sa + dst * inv + 127) / 255) as u8;
        }
        let da = u32::from(frame.pixels[index + 3]);
        frame.pixels[index + 3] = (sa + (da * inv + 127) / 255).min(255) as u8;
    }

    fn fill_rect(frame: &mut Frame, rect: PhysicalRect, color: Color, clip: PhysicalRect) {
        let Some(rect) = rect.intersection(clip).and_then(|r| r.intersection(Self::frame_bounds(frame))) else { return; };
        for y in rect.y..rect.y + rect.height as i32 {
            for x in rect.x..rect.x + rect.width as i32 {
                Self::blend_pixel(frame, x, y, color, clip);
            }
        }
    }

    fn stroke_rect(frame: &mut Frame, rect: PhysicalRect, color: Color, width: u32, clip: PhysicalRect) {
        let width = width.max(1).min(rect.width.max(1)).min(rect.height.max(1));
        Self::fill_rect(frame, PhysicalRect::new(rect.x, rect.y, rect.width, width), color, clip);
        Self::fill_rect(frame, PhysicalRect::new(rect.x, rect.y + rect.height as i32 - width as i32, rect.width, width), color, clip);
        Self::fill_rect(frame, PhysicalRect::new(rect.x, rect.y, width, rect.height), color, clip);
        Self::fill_rect(frame, PhysicalRect::new(rect.x + rect.width as i32 - width as i32, rect.y, width, rect.height), color, clip);
    }

    fn render_text(&mut self, frame: &mut Frame, rect: PhysicalRect, text: &str, style: &TextStyle, scale: ScaleFactor, clip: PhysicalRect) {
        if rect.width == 0 || rect.height == 0 || text.is_empty() {
            return;
        }
        let size = (style.size * scale.get()).max(1.0);
        let line_height = (style.line_height * scale.get()).max(size);
        let metrics = Metrics::new(size, line_height);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        let mut borrowed = buffer.borrow_with(&mut self.font_system);
        borrowed.set_size(Some(rect.width as f32), Some(rect.height as f32));
        let weight = if style.bold { Weight::BOLD } else { Weight::NORMAL };
        let attrs = Attrs::new().family(Family::Name(style.family.as_str())).weight(weight);
        borrowed.set_text(text, &attrs, Shaping::Advanced, None);
        borrowed.shape_until_scroll(true);
        let color = CosmicColor::rgba(style.color.r, style.color.g, style.color.b, style.color.a);
        let mut pixels = Vec::new();
        borrowed.draw(&mut self.swash_cache, color, |x, y, w, h, pixel| {
            pixels.push((x, y, w, h, pixel.as_rgba_tuple()));
        });
        drop(borrowed);
        for (x, y, width, height, rgba) in pixels {
            let color = Color::rgba(rgba.0, rgba.1, rgba.2, rgba.3);
            Self::fill_rect(frame, PhysicalRect::new(rect.x + x, rect.y + y, width, height), color, clip);
        }
    }

    fn render_image(frame: &mut Frame, rect: PhysicalRect, image: &ImageData, clip: PhysicalRect) {
        if rect.width == 0 || rect.height == 0 || image.width == 0 || image.height == 0 {
            return;
        }
        for dy in 0..rect.height {
            let sy = (u64::from(dy) * u64::from(image.height) / u64::from(rect.height)) as u32;
            for dx in 0..rect.width {
                let sx = (u64::from(dx) * u64::from(image.width) / u64::from(rect.width)) as u32;
                let source = (sy as usize * image.width as usize + sx as usize) * 4;
                let color = Color::rgba(image.pixels[source], image.pixels[source + 1], image.pixels[source + 2], image.pixels[source + 3]);
                Self::blend_pixel(frame, rect.x + dx as i32, rect.y + dy as i32, color, clip);
            }
        }
    }

    fn render_svg(frame: &mut Frame, rect: PhysicalRect, bytes: &[u8], clip: PhysicalRect) -> Result<(), RenderError> {
        if rect.width == 0 || rect.height == 0 {
            return Ok(());
        }
        let options = resvg::usvg::Options::default();
        let tree = resvg::usvg::Tree::from_data(bytes, &options).map_err(|error| RenderError::Svg(error.to_string()))?;
        let mut pixmap = resvg::tiny_skia::Pixmap::new(rect.width, rect.height).ok_or(RenderError::SizeOverflow)?;
        let sx = rect.width as f32 / tree.size().width();
        let sy = rect.height as f32 / tree.size().height();
        let transform = resvg::tiny_skia::Transform::from_scale(sx, sy);
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        // tiny-skia stores premultiplied RGBA. Convert to straight alpha before
        // compositing into the public Frame contract.
        for (index, pixel) in pixmap.data().chunks_exact(4).enumerate() {
            let alpha = pixel[3];
            if alpha == 0 {
                continue;
            }
            let unpremultiply = |channel: u8| ((u32::from(channel) * 255 + u32::from(alpha) / 2) / u32::from(alpha)).min(255) as u8;
            let color = Color::rgba(unpremultiply(pixel[0]), unpremultiply(pixel[1]), unpremultiply(pixel[2]), alpha);
            let x = (index % rect.width as usize) as i32;
            let y = (index / rect.width as usize) as i32;
            Self::blend_pixel(frame, rect.x + x, rect.y + y, color, clip);
        }
        Ok(())
    }
}

impl TextMeasurer for SoftwareRenderer {
    fn measure_text(&mut self, text: &str, style: &TextStyle, max_width: Option<f32>) -> LogicalSize {
        if text.is_empty() {
            return LogicalSize::new(0.0, style.line_height.max(style.size));
        }
        let metrics = Metrics::new(style.size.max(1.0), style.line_height.max(style.size).max(1.0));
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        let mut borrowed = buffer.borrow_with(&mut self.font_system);
        borrowed.set_size(max_width, None);
        let weight = if style.bold { Weight::BOLD } else { Weight::NORMAL };
        let attrs = Attrs::new().family(Family::Name(style.family.as_str())).weight(weight);
        borrowed.set_text(text, &attrs, Shaping::Advanced, None);
        borrowed.shape_until_scroll(true);
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        for run in borrowed.layout_runs() {
            width = width.max(run.line_w);
            height = height.max(run.line_top + run.line_height);
        }
        LogicalSize::new(width, height.max(metrics.line_height))
    }
}

impl Renderer for SoftwareRenderer {
    fn render(&mut self, list: &DisplayList, logical_size: LogicalSize, scale: ScaleFactor) -> Result<Frame, RenderError> {
        let mut frame = Frame::new(scale.physical_size(logical_size))?;
        let mut clips = Vec::<PhysicalRect>::new();
        for command in list.commands() {
            let clip = Self::current_clip(&frame, &clips).unwrap_or_default();
            match command {
                DrawCommand::Clear(color) => Self::fill_rect(&mut frame, Self::frame_bounds(&frame), *color, Self::frame_bounds(&frame)),
                DrawCommand::FillRect { rect, color } => Self::fill_rect(&mut frame, scale.physical_rect(*rect), *color, clip),
                DrawCommand::StrokeRect { rect, color, width } => {
                    let physical_width = (*width * scale.get()).round().max(1.0) as u32;
                    Self::stroke_rect(&mut frame, scale.physical_rect(*rect), *color, physical_width, clip);
                }
                DrawCommand::Text { rect, text, style } => self.render_text(&mut frame, scale.physical_rect(*rect), text, style, scale, clip),
                DrawCommand::Image { rect, image } => Self::render_image(&mut frame, scale.physical_rect(*rect), image, clip),
                DrawCommand::Svg { rect, bytes } => Self::render_svg(&mut frame, scale.physical_rect(*rect), bytes, clip)?,
                DrawCommand::PushClip(rect) => clips.push(scale.physical_rect(*rect)),
                DrawCommand::PopClip => {
                    if clips.pop().is_none() {
                        return Err(RenderError::ClipStackUnderflow);
                    }
                }
            }
        }
        Ok(frame)
    }
}
