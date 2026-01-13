//! Vector synthesis with 2D crossfading between multiple sound sources.
//!
//! Vector synthesis places 2-4 sound sources at corners of a 2D space and
//! crossfades between them based on a position within that space. The position
//! can be animated over time to create evolving, morphing textures.
//!
//! Classic examples: Prophet VS, Korg Wavestation.
//!
//! ```text
//! Source A -------- Source B
//!     |                |
//!     |    position    |
//!     |       *        |
//!     |                |
//! Source C -------- Source D
//! ```

use rand_pcg::Pcg32;

use super::{SweepCurve, Synthesizer};
use crate::oscillator::{self, PhaseAccumulator};

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

/// Vector synthesizer with 4 sources at corners.
#[derive(Debug, Clone)]
pub struct VectorSynth {
    /// Base frequency in Hz.
    pub frequency: f64,
    /// Four sources at corners: [A (top-left), B (top-right), C (bottom-left), D (bottom-right)].
    pub sources: [VectorSource; 4],
    /// Static position (used if path is None).
    pub position: VectorPosition,
    /// Optional animated path.
    pub path: Option<VectorPath>,
    /// Whether the path should loop.
    pub path_loop: bool,
}

impl VectorSynth {
    /// Creates a new vector synthesizer with the given sources.
    pub fn new(frequency: f64, sources: [VectorSource; 4]) -> Self {
        Self {
            frequency,
            sources,
            position: VectorPosition::center(),
            path: None,
            path_loop: false,
        }
    }

    /// Sets the static position.
    pub fn with_position(mut self, position: VectorPosition) -> Self {
        self.position = position;
        self
    }

    /// Sets an animated path.
    pub fn with_path(mut self, path: VectorPath, loop_path: bool) -> Self {
        self.path = Some(path);
        self.path_loop = loop_path;
        self
    }

    /// Calculates the mix weights for the four corners based on position.
    /// Uses bilinear interpolation.
    fn calculate_mix_weights(position: &VectorPosition) -> [f64; 4] {
        let x = position.x;
        let y = position.y;

        // Bilinear interpolation weights:
        // A (top-left):     (1-x) * (1-y)
        // B (top-right):    x * (1-y)
        // C (bottom-left):  (1-x) * y
        // D (bottom-right): x * y
        [
            (1.0 - x) * (1.0 - y), // A
            x * (1.0 - y),         // B
            (1.0 - x) * y,         // C
            x * y,                 // D
        ]
    }

    /// Generates a sample for a single source at the given phase.
    fn generate_source_sample(
        source: &VectorSource,
        phase: f64,
        wavetable_position: f64,
        _rng: &mut Pcg32,
        noise_value: f64,
    ) -> f64 {
        match source.source_type {
            VectorSourceType::Sine => oscillator::sine(phase),
            VectorSourceType::Saw => oscillator::sawtooth(phase),
            VectorSourceType::Square => oscillator::square(phase, 0.5),
            VectorSourceType::Triangle => oscillator::triangle(phase),
            VectorSourceType::Noise => noise_value,
            VectorSourceType::Wavetable => {
                // Simple wavetable simulation using additive synthesis
                // The wavetable_position parameter adds variety
                let harmonics = (1.0 + wavetable_position * 7.0) as usize;
                let mut sample = 0.0;
                for h in 1..=harmonics.max(1) {
                    let harmonic_amp = 1.0 / (h as f64);
                    sample += harmonic_amp * (phase * h as f64).sin();
                }
                sample * 0.5
            }
        }
    }
}

impl Synthesizer for VectorSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, rng: &mut Pcg32) -> Vec<f64> {
        let mut output = vec![0.0; num_samples];

        // Create phase accumulators for each source
        let mut phase_accumulators: [PhaseAccumulator; 4] = [
            PhaseAccumulator::new(sample_rate),
            PhaseAccumulator::new(sample_rate),
            PhaseAccumulator::new(sample_rate),
            PhaseAccumulator::new(sample_rate),
        ];

        // Pre-generate noise buffer for deterministic noise sources
        let noise_buffer = oscillator::white_noise(rng, num_samples);

        // Calculate time step per sample
        let dt = 1.0 / sample_rate;

        for i in 0..num_samples {
            let time = i as f64 * dt;

            // Get current position (from path or static)
            let position = if let Some(ref path) = self.path {
                path.position_at(time, self.path_loop)
            } else {
                self.position
            };

            // Calculate mix weights
            let weights = Self::calculate_mix_weights(&position);

            // Generate and mix samples from each source
            let mut sample = 0.0;
            for (j, (source, weight)) in self.sources.iter().zip(weights.iter()).enumerate() {
                if *weight > 0.0001 {
                    // Skip sources with negligible weight
                    let freq = self.frequency * source.frequency_ratio;
                    let phase = phase_accumulators[j].advance(freq);
                    let wavetable_pos = position.x * 0.5 + position.y * 0.5; // Use position for wavetable variation
                    let source_sample = Self::generate_source_sample(
                        source,
                        phase,
                        wavetable_pos,
                        rng,
                        noise_buffer[i],
                    );
                    sample += source_sample * weight;
                } else {
                    // Still advance phase to maintain consistency
                    let freq = self.frequency * source.frequency_ratio;
                    phase_accumulators[j].advance(freq);
                }
            }

            output[i] = sample.clamp(-1.0, 1.0);
        }

        output
    }
}

// ============================================================================
// Preset factory functions
// ============================================================================

/// Creates an evolving pad preset.
///
/// A lush, slowly evolving pad sound that morphs between different timbres.
/// Uses a path that slowly moves around the vector space.
pub fn evolving_pad(frequency: f64, duration: f64) -> VectorSynth {
    let sources = [
        VectorSource::sine(1.0),     // A: Pure sine (fundamental)
        VectorSource::saw(1.0),      // B: Saw wave (bright)
        VectorSource::triangle(2.0), // C: Triangle (soft, octave up)
        VectorSource::wavetable(1.0), // D: Wavetable (complex)
    ];

    // Create a path that moves in a circle around the center
    let segment_duration = duration / 4.0;
    let path = VectorPath::new(vec![
        VectorPathPoint::new(VectorPosition::new(0.3, 0.3), segment_duration),
        VectorPathPoint::new(VectorPosition::new(0.7, 0.3), segment_duration),
        VectorPathPoint::new(VectorPosition::new(0.7, 0.7), segment_duration),
        VectorPathPoint::new(VectorPosition::new(0.3, 0.7), segment_duration),
    ]);

    VectorSynth::new(frequency, sources)
        .with_position(VectorPosition::new(0.3, 0.3))
        .with_path(path, true)
}

/// Creates a morphing texture preset.
///
/// A textured sound that morphs between noise and tonal sources,
/// creating interesting evolving textures.
pub fn morph_texture(frequency: f64, duration: f64) -> VectorSynth {
    let sources = [
        VectorSource::sine(1.0),       // A: Pure tone
        VectorSource::noise(),         // B: Noise
        VectorSource::square(0.5),     // C: Sub-octave square
        VectorSource::wavetable(2.0),  // D: Wavetable at octave
    ];

    // Path that sweeps from tonal to noisy and back
    let half_duration = duration / 2.0;
    let path = VectorPath::new(vec![
        VectorPathPoint::new(VectorPosition::new(0.0, 0.5), half_duration),
        VectorPathPoint::new(VectorPosition::new(1.0, 0.5), half_duration),
    ]);

    VectorSynth::new(frequency, sources)
        .with_position(VectorPosition::new(0.0, 0.5))
        .with_path(path, true)
}

/// Creates a sweep corners preset.
///
/// Demonstrates vector synthesis by sweeping through all four corners
/// in sequence, showcasing each source distinctly.
pub fn sweep_corners(frequency: f64, duration: f64) -> VectorSynth {
    let sources = [
        VectorSource::sine(1.0),      // A: Sine
        VectorSource::saw(1.0),       // B: Saw
        VectorSource::triangle(1.0),  // C: Triangle
        VectorSource::square(1.0),    // D: Square
    ];

    // Path that visits each corner
    let segment_duration = duration / 4.0;
    let path = VectorPath::new(vec![
        VectorPathPoint::new(VectorPosition::corner_a(), 0.0), // Start at A
        VectorPathPoint::new(VectorPosition::corner_b(), segment_duration),
        VectorPathPoint::new(VectorPosition::corner_d(), segment_duration),
        VectorPathPoint::new(VectorPosition::corner_c(), segment_duration),
        VectorPathPoint::new(VectorPosition::corner_a(), segment_duration),
    ]);

    VectorSynth::new(frequency, sources)
        .with_position(VectorPosition::corner_a())
        .with_path(path, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;

    #[test]
    fn test_vector_position_new() {
        let pos = VectorPosition::new(0.5, 0.5);
        assert!((pos.x - 0.5).abs() < 0.001);
        assert!((pos.y - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_vector_position_clamp() {
        let pos = VectorPosition::new(-0.5, 1.5);
        assert!((pos.x - 0.0).abs() < 0.001);
        assert!((pos.y - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_vector_mix_weights_center() {
        let pos = VectorPosition::center();
        let weights = VectorSynth::calculate_mix_weights(&pos);

        // At center (0.5, 0.5), all corners should have equal weight (0.25)
        for weight in &weights {
            assert!((*weight - 0.25).abs() < 0.001);
        }
    }

    #[test]
    fn test_vector_mix_weights_corners() {
        // Corner A (top-left)
        let weights_a = VectorSynth::calculate_mix_weights(&VectorPosition::corner_a());
        assert!((weights_a[0] - 1.0).abs() < 0.001); // A = 1.0
        assert!(weights_a[1].abs() < 0.001);         // B = 0.0
        assert!(weights_a[2].abs() < 0.001);         // C = 0.0
        assert!(weights_a[3].abs() < 0.001);         // D = 0.0

        // Corner D (bottom-right)
        let weights_d = VectorSynth::calculate_mix_weights(&VectorPosition::corner_d());
        assert!(weights_d[0].abs() < 0.001);         // A = 0.0
        assert!(weights_d[1].abs() < 0.001);         // B = 0.0
        assert!(weights_d[2].abs() < 0.001);         // C = 0.0
        assert!((weights_d[3] - 1.0).abs() < 0.001); // D = 1.0
    }

    #[test]
    fn test_vector_synth_basic() {
        let sources = [
            VectorSource::sine(1.0),
            VectorSource::saw(1.0),
            VectorSource::triangle(1.0),
            VectorSource::square(1.0),
        ];
        let synth = VectorSynth::new(440.0, sources)
            .with_position(VectorPosition::center());

        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!(s.is_finite());
            assert!(s.abs() <= 1.0);
        }
    }

    #[test]
    fn test_vector_synth_with_path() {
        let sources = [
            VectorSource::sine(1.0),
            VectorSource::saw(1.0),
            VectorSource::triangle(1.0),
            VectorSource::square(1.0),
        ];

        let path = VectorPath::new(vec![
            VectorPathPoint::new(VectorPosition::corner_a(), 0.1),
            VectorPathPoint::new(VectorPosition::corner_d(), 0.1),
        ]);

        let synth = VectorSynth::new(440.0, sources)
            .with_path(path, false);

        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!(s.is_finite());
            assert!(s.abs() <= 1.0);
        }
    }

    #[test]
    fn test_vector_synth_determinism() {
        let sources = [
            VectorSource::sine(1.0),
            VectorSource::noise(),
            VectorSource::triangle(1.0),
            VectorSource::wavetable(1.0),
        ];
        let synth = VectorSynth::new(440.0, sources)
            .with_position(VectorPosition::new(0.3, 0.7));

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_evolving_pad_preset() {
        let synth = evolving_pad(220.0, 1.0);

        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!(s.is_finite());
            assert!(s.abs() <= 1.0);
        }
    }

    #[test]
    fn test_morph_texture_preset() {
        let synth = morph_texture(330.0, 0.5);

        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!(s.is_finite());
            assert!(s.abs() <= 1.0);
        }
    }

    #[test]
    fn test_sweep_corners_preset() {
        let synth = sweep_corners(440.0, 2.0);

        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!(s.is_finite());
            assert!(s.abs() <= 1.0);
        }
    }

    #[test]
    fn test_vector_path_position_at() {
        let path = VectorPath::new(vec![
            VectorPathPoint::new(VectorPosition::new(0.0, 0.0), 0.0),
            VectorPathPoint::new(VectorPosition::new(1.0, 1.0), 1.0),
        ]);

        // At start
        let pos_start = path.position_at(0.0, false);
        assert!((pos_start.x - 0.0).abs() < 0.001);
        assert!((pos_start.y - 0.0).abs() < 0.001);

        // At middle
        let pos_mid = path.position_at(0.5, false);
        assert!((pos_mid.x - 0.5).abs() < 0.001);
        assert!((pos_mid.y - 0.5).abs() < 0.001);

        // At end
        let pos_end = path.position_at(1.0, false);
        assert!((pos_end.x - 1.0).abs() < 0.001);
        assert!((pos_end.y - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_vector_path_looping() {
        let path = VectorPath::new(vec![
            VectorPathPoint::new(VectorPosition::new(0.0, 0.0), 0.0),
            VectorPathPoint::new(VectorPosition::new(1.0, 0.0), 1.0),
        ]);

        // With looping, time 1.5 should be equivalent to time 0.5
        let pos_loop = path.position_at(1.5, true);
        let pos_half = path.position_at(0.5, false);
        assert!((pos_loop.x - pos_half.x).abs() < 0.001);
        assert!((pos_loop.y - pos_half.y).abs() < 0.001);
    }
}
