//! Scratches pattern generator.

use super::Pattern2D;
use crate::rng::DeterministicRng;

/// Scratches pattern configuration.
#[derive(Debug, Clone)]
pub struct ScratchesPattern {
    /// Number of scratches to generate.
    pub count: u32,
    /// Minimum scratch length as fraction of texture size.
    pub min_length: f64,
    /// Maximum scratch length as fraction of texture size.
    pub max_length: f64,
    /// Scratch width in pixels.
    pub width: f64,
    /// Scratch depth (intensity).
    pub depth: f64,
    /// Seed for random generation.
    pub seed: u32,
    /// Texture width.
    tex_width: u32,
    /// Texture height.
    tex_height: u32,
    /// Generated scratches.
    scratches: Vec<Scratch>,
}

/// A single scratch.
#[derive(Debug, Clone)]
struct Scratch {
    /// Start X position.
    x1: f64,
    /// Start Y position.
    y1: f64,
    /// End X position.
    x2: f64,
    /// End Y position.
    y2: f64,
    /// Width.
    width: f64,
    /// Intensity.
    intensity: f64,
}

impl ScratchesPattern {
    /// Create a new scratches pattern.
    pub fn new(width: u32, height: u32, seed: u32) -> Self {
        let mut pattern = Self {
            count: 50,
            min_length: 0.05,
            max_length: 0.2,
            width: 1.5,
            depth: 0.5,
            seed,
            tex_width: width,
            tex_height: height,
            scratches: Vec::new(),
        };
        pattern.generate_scratches();
        pattern
    }

    /// Set the number of scratches.
    pub fn with_count(mut self, count: u32) -> Self {
        self.count = count;
        self.generate_scratches();
        self
    }

    /// Set the length range.
    pub fn with_length_range(mut self, min: f64, max: f64) -> Self {
        self.min_length = min;
        self.max_length = max;
        self.generate_scratches();
        self
    }

    /// Set the width.
    pub fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self.generate_scratches();
        self
    }

    /// Set the depth/intensity.
    pub fn with_depth(mut self, depth: f64) -> Self {
        self.depth = depth;
        self.generate_scratches();
        self
    }

    /// Generate the scratches.
    fn generate_scratches(&mut self) {
        let mut rng = DeterministicRng::new(self.seed);
        self.scratches.clear();

        let diag =
            ((self.tex_width * self.tex_width + self.tex_height * self.tex_height) as f64).sqrt();

        for _ in 0..self.count {
            // Random start position
            let x1 = rng.gen_f64() * self.tex_width as f64;
            let y1 = rng.gen_f64() * self.tex_height as f64;

            // Random angle
            let angle = rng.gen_f64() * std::f64::consts::PI * 2.0;

            // Random length
            let length =
                (self.min_length + rng.gen_f64() * (self.max_length - self.min_length)) * diag;

            // Calculate end position
            let x2 = x1 + angle.cos() * length;
            let y2 = y1 + angle.sin() * length;

            // Random width variation
            let width = self.width * (0.5 + rng.gen_f64());

            // Random intensity
            let intensity = self.depth * (0.3 + rng.gen_f64() * 0.7);

            self.scratches.push(Scratch {
                x1,
                y1,
                x2,
                y2,
                width,
                intensity,
            });
        }
    }

    /// Calculate distance from a point to a line segment.
    fn distance_to_segment(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len_sq = dx * dx + dy * dy;

        if len_sq < 1e-10 {
            // Line segment is a point
            let dpx = px - x1;
            let dpy = py - y1;
            return (dpx * dpx + dpy * dpy).sqrt();
        }

        // Project point onto line, clamping to segment
        let t = ((px - x1) * dx + (py - y1) * dy) / len_sq;
        let t = t.clamp(0.0, 1.0);

        let proj_x = x1 + t * dx;
        let proj_y = y1 + t * dy;

        let dpx = px - proj_x;
        let dpy = py - proj_y;
        (dpx * dpx + dpy * dpy).sqrt()
    }
}

impl Pattern2D for ScratchesPattern {
    fn sample(&self, x: u32, y: u32) -> f64 {
        let px = x as f64;
        let py = y as f64;

        let mut total_scratch: f64 = 0.0;

        for scratch in &self.scratches {
            // Handle wrapping for tileable textures
            for dy in [-1, 0, 1] {
                for dx in [-1, 0, 1] {
                    let offset_x = dx as f64 * self.tex_width as f64;
                    let offset_y = dy as f64 * self.tex_height as f64;

                    let dist = Self::distance_to_segment(
                        px,
                        py,
                        scratch.x1 + offset_x,
                        scratch.y1 + offset_y,
                        scratch.x2 + offset_x,
                        scratch.y2 + offset_y,
                    );

                    if dist < scratch.width * 3.0 {
                        // Smooth falloff
                        let falloff = 1.0 - (dist / scratch.width).clamp(0.0, 1.0);
                        let falloff = falloff * falloff; // Quadratic falloff

                        total_scratch = total_scratch.max(falloff * scratch.intensity);
                    }
                }
            }
        }

        // Return as height (scratches are indentations, so subtract from 1)
        1.0 - total_scratch.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scratches_deterministic() {
        let pattern1 = ScratchesPattern::new(256, 256, 42);
        let pattern2 = ScratchesPattern::new(256, 256, 42);

        for y in 0..50 {
            for x in 0..50 {
                assert_eq!(pattern1.sample(x, y), pattern2.sample(x, y));
            }
        }
    }

    #[test]
    fn test_scratches_range() {
        let pattern = ScratchesPattern::new(256, 256, 42);

        for y in 0..100 {
            for x in 0..100 {
                let v = pattern.sample(x, y);
                assert!((0.0..=1.0).contains(&v));
            }
        }
    }
}
