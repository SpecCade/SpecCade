//! Core types for vector synthesis.

use super::super::SweepCurve;

/// Source waveform type for vector synthesis corners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorSourceType {
    /// Sine wave.
    Sine,
    /// Sawtooth wave.
    Saw,
    /// Square wave with 50% duty cycle.
    Square,
    /// Triangle wave.
    Triangle,
    /// White noise.
    Noise,
    /// Wavetable-based source (uses additive harmonics for variety).
    Wavetable,
}

/// A single source in the vector synthesis grid.
#[derive(Debug, Clone)]
pub struct VectorSource {
    /// Type of waveform for this source.
    pub source_type: VectorSourceType,
    /// Frequency ratio relative to the base frequency (1.0 = unison).
    pub frequency_ratio: f64,
}

impl VectorSource {
    /// Creates a new vector source.
    pub fn new(source_type: VectorSourceType, frequency_ratio: f64) -> Self {
        Self {
            source_type,
            frequency_ratio: frequency_ratio.max(0.0625), // Clamp to reasonable range
        }
    }

    /// Creates a sine source at the given frequency ratio.
    pub fn sine(frequency_ratio: f64) -> Self {
        Self::new(VectorSourceType::Sine, frequency_ratio)
    }

    /// Creates a saw source at the given frequency ratio.
    pub fn saw(frequency_ratio: f64) -> Self {
        Self::new(VectorSourceType::Saw, frequency_ratio)
    }

    /// Creates a square source at the given frequency ratio.
    pub fn square(frequency_ratio: f64) -> Self {
        Self::new(VectorSourceType::Square, frequency_ratio)
    }

    /// Creates a triangle source at the given frequency ratio.
    pub fn triangle(frequency_ratio: f64) -> Self {
        Self::new(VectorSourceType::Triangle, frequency_ratio)
    }

    /// Creates a noise source.
    pub fn noise() -> Self {
        Self::new(VectorSourceType::Noise, 1.0)
    }

    /// Creates a wavetable source at the given frequency ratio.
    pub fn wavetable(frequency_ratio: f64) -> Self {
        Self::new(VectorSourceType::Wavetable, frequency_ratio)
    }
}

/// A position in the 2D vector space.
#[derive(Debug, Clone, Copy)]
pub struct VectorPosition {
    /// X position (0.0 = left, 1.0 = right).
    pub x: f64,
    /// Y position (0.0 = top, 1.0 = bottom).
    pub y: f64,
}

impl VectorPosition {
    /// Creates a new position.
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
        }
    }

    /// Position at corner A (top-left).
    pub fn corner_a() -> Self {
        Self::new(0.0, 0.0)
    }

    /// Position at corner B (top-right).
    pub fn corner_b() -> Self {
        Self::new(1.0, 0.0)
    }

    /// Position at corner C (bottom-left).
    pub fn corner_c() -> Self {
        Self::new(0.0, 1.0)
    }

    /// Position at corner D (bottom-right).
    pub fn corner_d() -> Self {
        Self::new(1.0, 1.0)
    }

    /// Position at the center.
    pub fn center() -> Self {
        Self::new(0.5, 0.5)
    }

    /// Linearly interpolates between two positions.
    pub fn lerp(&self, other: &VectorPosition, t: f64) -> VectorPosition {
        VectorPosition::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }
}

/// A point in a vector path animation.
#[derive(Debug, Clone)]
pub struct VectorPathPoint {
    /// Target position.
    pub position: VectorPosition,
    /// Duration in seconds to reach this position from the previous point.
    pub duration: f64,
}

impl VectorPathPoint {
    /// Creates a new path point.
    pub fn new(position: VectorPosition, duration: f64) -> Self {
        Self {
            position,
            duration: duration.max(0.0),
        }
    }
}

/// A path for animating the vector position over time.
#[derive(Debug, Clone)]
pub struct VectorPath {
    /// Sequence of positions with durations.
    pub points: Vec<VectorPathPoint>,
    /// Curve type for interpolation between points.
    pub curve: SweepCurve,
}

impl VectorPath {
    /// Creates a new path with linear interpolation.
    pub fn new(points: Vec<VectorPathPoint>) -> Self {
        Self {
            points,
            curve: SweepCurve::Linear,
        }
    }

    /// Creates a path with specified curve type.
    pub fn with_curve(mut self, curve: SweepCurve) -> Self {
        self.curve = curve;
        self
    }

    /// Gets the total duration of the path.
    pub fn total_duration(&self) -> f64 {
        self.points.iter().map(|p| p.duration).sum()
    }

    /// Gets the position at a given time, looping if requested.
    pub fn position_at(&self, time: f64, loop_path: bool) -> VectorPosition {
        if self.points.is_empty() {
            return VectorPosition::center();
        }

        let total_duration = self.total_duration();
        if total_duration <= 0.0 {
            return self.points[0].position;
        }

        // Handle looping
        let effective_time = if loop_path && time > total_duration {
            time % total_duration
        } else {
            time.min(total_duration)
        };

        // Find which segment we're in
        let mut accumulated = 0.0;
        let mut prev_position = self.points[0].position;

        for point in &self.points {
            let segment_end = accumulated + point.duration;

            if effective_time <= segment_end {
                if point.duration <= 0.0 {
                    return point.position;
                }

                let t = (effective_time - accumulated) / point.duration;
                let interpolated_t = match self.curve {
                    SweepCurve::Linear => t,
                    SweepCurve::Exponential => {
                        // Exponential ease-out
                        1.0 - (1.0 - t).powi(2)
                    }
                    SweepCurve::Logarithmic => {
                        // Logarithmic ease-in
                        t * t
                    }
                };

                return prev_position.lerp(&point.position, interpolated_t);
            }

            accumulated = segment_end;
            prev_position = point.position;
        }

        // Return last position
        self.points.last().map(|p| p.position).unwrap_or(VectorPosition::center())
    }
}
