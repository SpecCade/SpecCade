//! PBR texture map generators.
//!
//! Each module generates a specific type of PBR texture map.

mod albedo;
mod roughness;
mod metallic;
mod normal;
mod ao;
mod emissive;

pub use albedo::AlbedoGenerator;
pub use roughness::RoughnessGenerator;
pub use metallic::MetallicGenerator;
pub use normal::NormalGenerator;
pub use ao::AoGenerator;
pub use emissive::EmissiveGenerator;

use crate::color::Color;

/// A 2D texture buffer.
#[derive(Debug, Clone)]
pub struct TextureBuffer {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Pixel data (RGBA, row-major).
    pub data: Vec<Color>,
}

impl TextureBuffer {
    /// Create a new texture buffer filled with a color.
    pub fn new(width: u32, height: u32, fill: Color) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            data: vec![fill; size],
        }
    }

    /// Create a new texture buffer filled with black.
    pub fn new_black(width: u32, height: u32) -> Self {
        Self::new(width, height, Color::black())
    }

    /// Create a new texture buffer filled with white.
    pub fn new_white(width: u32, height: u32) -> Self {
        Self::new(width, height, Color::white())
    }

    /// Get a pixel at the given coordinates.
    #[inline]
    pub fn get(&self, x: u32, y: u32) -> Color {
        let idx = (y * self.width + x) as usize;
        self.data[idx]
    }

    /// Set a pixel at the given coordinates.
    #[inline]
    pub fn set(&mut self, x: u32, y: u32, color: Color) {
        let idx = (y * self.width + x) as usize;
        self.data[idx] = color;
    }

    /// Get a pixel with wrapping coordinates.
    #[inline]
    pub fn get_wrapped(&self, x: i32, y: i32) -> Color {
        let wx = x.rem_euclid(self.width as i32) as u32;
        let wy = y.rem_euclid(self.height as i32) as u32;
        self.get(wx, wy)
    }

    /// Sample with bilinear interpolation using normalized [0, 1] coordinates.
    pub fn sample_bilinear(&self, u: f64, v: f64) -> Color {
        let x = u * (self.width - 1) as f64;
        let y = v * (self.height - 1) as f64;

        let x0 = x.floor() as u32;
        let y0 = y.floor() as u32;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let fx = x - x.floor();
        let fy = y - y.floor();

        let c00 = self.get(x0, y0);
        let c10 = self.get(x1, y0);
        let c01 = self.get(x0, y1);
        let c11 = self.get(x1, y1);

        let c0 = c00.lerp(&c10, fx);
        let c1 = c01.lerp(&c11, fx);
        c0.lerp(&c1, fy)
    }

    /// Convert to grayscale buffer (single channel).
    pub fn to_grayscale(&self) -> Vec<f64> {
        self.data.iter().map(|c| c.luminance()).collect()
    }

    /// Create from a grayscale buffer.
    pub fn from_grayscale(gray: &[f64], width: u32, height: u32) -> Self {
        let data: Vec<Color> = gray.iter().map(|&v| Color::gray(v)).collect();
        Self { width, height, data }
    }

    /// Convert to 8-bit RGBA bytes.
    pub fn to_rgba8(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.data.len() * 4);
        for color in &self.data {
            let rgba = color.to_rgba8();
            bytes.extend_from_slice(&rgba);
        }
        bytes
    }

    /// Convert to 8-bit RGB bytes.
    pub fn to_rgb8(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.data.len() * 3);
        for color in &self.data {
            let rgb = color.to_rgb8();
            bytes.extend_from_slice(&rgb);
        }
        bytes
    }

    /// Convert to 8-bit grayscale bytes.
    pub fn to_gray8(&self) -> Vec<u8> {
        self.data
            .iter()
            .map(|c| (c.luminance().clamp(0.0, 1.0) * 255.0).round() as u8)
            .collect()
    }
}

/// Grayscale texture buffer (single channel).
#[derive(Debug, Clone)]
pub struct GrayscaleBuffer {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Pixel data (single channel, row-major).
    pub data: Vec<f64>,
}

impl GrayscaleBuffer {
    /// Create a new grayscale buffer filled with a value.
    pub fn new(width: u32, height: u32, fill: f64) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            data: vec![fill; size],
        }
    }

    /// Get a pixel at the given coordinates.
    #[inline]
    pub fn get(&self, x: u32, y: u32) -> f64 {
        let idx = (y * self.width + x) as usize;
        self.data[idx]
    }

    /// Set a pixel at the given coordinates.
    #[inline]
    pub fn set(&mut self, x: u32, y: u32, value: f64) {
        let idx = (y * self.width + x) as usize;
        self.data[idx] = value;
    }

    /// Get a pixel with wrapping coordinates.
    #[inline]
    pub fn get_wrapped(&self, x: i32, y: i32) -> f64 {
        let wx = x.rem_euclid(self.width as i32) as u32;
        let wy = y.rem_euclid(self.height as i32) as u32;
        self.get(wx, wy)
    }

    /// Convert to 8-bit bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data
            .iter()
            .map(|&v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
            .collect()
    }

    /// Convert to TextureBuffer.
    pub fn to_texture_buffer(&self) -> TextureBuffer {
        TextureBuffer::from_grayscale(&self.data, self.width, self.height)
    }
}
