//! Color utilities for texture generation.

/// RGBA color with f64 components (0.0 to 1.0 range).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    /// Create a new color with alpha = 1.0.
    pub const fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create a new color with alpha.
    pub const fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    /// Create a grayscale color.
    pub const fn gray(value: f64) -> Self {
        Self::rgb(value, value, value)
    }

    /// Create black.
    pub const fn black() -> Self {
        Self::rgb(0.0, 0.0, 0.0)
    }

    /// Create white.
    pub const fn white() -> Self {
        Self::rgb(1.0, 1.0, 1.0)
    }

    /// Create a color from HSV values.
    /// - h: hue in degrees (0-360)
    /// - s: saturation (0-1)
    /// - v: value/brightness (0-1)
    pub fn from_hsv(h: f64, s: f64, v: f64) -> Self {
        if s <= 0.0 {
            return Self::rgb(v, v, v);
        }

        let h = h % 360.0;
        let h = if h < 0.0 { h + 360.0 } else { h };
        let h = h / 60.0;

        let i = h.floor() as i32;
        let f = h - i as f64;
        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));

        match i {
            0 => Self::rgb(v, t, p),
            1 => Self::rgb(q, v, p),
            2 => Self::rgb(p, v, t),
            3 => Self::rgb(p, q, v),
            4 => Self::rgb(t, p, v),
            _ => Self::rgb(v, p, q),
        }
    }

    /// Convert to HSV values.
    /// Returns (hue in degrees 0-360, saturation 0-1, value 0-1).
    pub fn to_hsv(&self) -> (f64, f64, f64) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let delta = max - min;

        let v = max;

        if delta < 1e-10 {
            return (0.0, 0.0, v);
        }

        let s = delta / max;

        let h = if (self.r - max).abs() < 1e-10 {
            (self.g - self.b) / delta
        } else if (self.g - max).abs() < 1e-10 {
            2.0 + (self.b - self.r) / delta
        } else {
            4.0 + (self.r - self.g) / delta
        };

        let h = h * 60.0;
        let h = if h < 0.0 { h + 360.0 } else { h };

        (h, s, v)
    }

    /// Linearly interpolate between two colors.
    pub fn lerp(&self, other: &Color, t: f64) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    /// Clamp all components to [0.0, 1.0].
    pub fn clamp(&self) -> Color {
        Color {
            r: self.r.clamp(0.0, 1.0),
            g: self.g.clamp(0.0, 1.0),
            b: self.b.clamp(0.0, 1.0),
            a: self.a.clamp(0.0, 1.0),
        }
    }

    /// Convert to 8-bit RGBA.
    pub fn to_rgba8(&self) -> [u8; 4] {
        let c = self.clamp();
        [
            (c.r * 255.0).round() as u8,
            (c.g * 255.0).round() as u8,
            (c.b * 255.0).round() as u8,
            (c.a * 255.0).round() as u8,
        ]
    }

    /// Convert to 8-bit RGB.
    pub fn to_rgb8(&self) -> [u8; 3] {
        let c = self.clamp();
        [
            (c.r * 255.0).round() as u8,
            (c.g * 255.0).round() as u8,
            (c.b * 255.0).round() as u8,
        ]
    }

    /// Create from 8-bit RGBA.
    pub fn from_rgba8(rgba: [u8; 4]) -> Self {
        Self {
            r: rgba[0] as f64 / 255.0,
            g: rgba[1] as f64 / 255.0,
            b: rgba[2] as f64 / 255.0,
            a: rgba[3] as f64 / 255.0,
        }
    }

    /// Create from 8-bit RGB.
    pub fn from_rgb8(rgb: [u8; 3]) -> Self {
        Self {
            r: rgb[0] as f64 / 255.0,
            g: rgb[1] as f64 / 255.0,
            b: rgb[2] as f64 / 255.0,
            a: 1.0,
        }
    }

    /// Multiply color by a scalar.
    pub fn scale(&self, factor: f64) -> Color {
        Color {
            r: self.r * factor,
            g: self.g * factor,
            b: self.b * factor,
            a: self.a,
        }
    }

    /// Add two colors.
    pub fn add(&self, other: &Color) -> Color {
        Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
            a: self.a.max(other.a),
        }
    }

    /// Multiply two colors component-wise.
    pub fn multiply(&self, other: &Color) -> Color {
        Color {
            r: self.r * other.r,
            g: self.g * other.g,
            b: self.b * other.b,
            a: self.a * other.a,
        }
    }

    /// Screen blend mode.
    pub fn screen(&self, other: &Color) -> Color {
        Color {
            r: 1.0 - (1.0 - self.r) * (1.0 - other.r),
            g: 1.0 - (1.0 - self.g) * (1.0 - other.g),
            b: 1.0 - (1.0 - self.b) * (1.0 - other.b),
            a: self.a.max(other.a),
        }
    }

    /// Overlay blend mode.
    pub fn overlay(&self, other: &Color) -> Color {
        fn overlay_channel(base: f64, blend: f64) -> f64 {
            if base < 0.5 {
                2.0 * base * blend
            } else {
                1.0 - 2.0 * (1.0 - base) * (1.0 - blend)
            }
        }

        Color {
            r: overlay_channel(self.r, other.r),
            g: overlay_channel(self.g, other.g),
            b: overlay_channel(self.b, other.b),
            a: self.a.max(other.a),
        }
    }

    /// Soft light blend mode.
    pub fn soft_light(&self, other: &Color) -> Color {
        fn soft_light_channel(base: f64, blend: f64) -> f64 {
            if blend < 0.5 {
                base - (1.0 - 2.0 * blend) * base * (1.0 - base)
            } else {
                let d = if base < 0.25 {
                    ((16.0 * base - 12.0) * base + 4.0) * base
                } else {
                    base.sqrt()
                };
                base + (2.0 * blend - 1.0) * (d - base)
            }
        }

        Color {
            r: soft_light_channel(self.r, other.r),
            g: soft_light_channel(self.g, other.g),
            b: soft_light_channel(self.b, other.b),
            a: self.a.max(other.a),
        }
    }

    /// Luminance of the color (perceived brightness).
    pub fn luminance(&self) -> f64 {
        0.299 * self.r + 0.587 * self.g + 0.114 * self.b
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::black()
    }
}

/// Blend mode for combining layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Add,
    Screen,
    Overlay,
    SoftLight,
}

impl BlendMode {
    /// Blend source color over destination using this blend mode.
    pub fn blend(&self, dst: &Color, src: &Color, opacity: f64) -> Color {
        let blended = match self {
            BlendMode::Normal => *src,
            BlendMode::Multiply => dst.multiply(src),
            BlendMode::Add => dst.add(src),
            BlendMode::Screen => dst.screen(src),
            BlendMode::Overlay => dst.overlay(src),
            BlendMode::SoftLight => dst.soft_light(src),
        };

        // Apply opacity via linear interpolation
        dst.lerp(&blended, opacity * src.a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_gray() {
        let gray = Color::gray(0.5);
        assert!((gray.r - 0.5).abs() < 1e-10);
        assert!((gray.g - 0.5).abs() < 1e-10);
        assert!((gray.b - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_hsv_roundtrip() {
        let original = Color::rgb(0.8, 0.3, 0.5);
        let (h, s, v) = original.to_hsv();
        let restored = Color::from_hsv(h, s, v);

        assert!((original.r - restored.r).abs() < 1e-6);
        assert!((original.g - restored.g).abs() < 1e-6);
        assert!((original.b - restored.b).abs() < 1e-6);
    }

    #[test]
    fn test_lerp() {
        let black = Color::black();
        let white = Color::white();

        let mid = black.lerp(&white, 0.5);
        assert!((mid.r - 0.5).abs() < 1e-10);
        assert!((mid.g - 0.5).abs() < 1e-10);
        assert!((mid.b - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_rgba8_roundtrip() {
        let original = Color::rgb(0.5, 0.25, 0.75);
        let rgba = original.to_rgba8();
        let restored = Color::from_rgba8(rgba);

        // Allow for 8-bit quantization error
        assert!((original.r - restored.r).abs() < 0.01);
        assert!((original.g - restored.g).abs() < 0.01);
        assert!((original.b - restored.b).abs() < 0.01);
    }
}
