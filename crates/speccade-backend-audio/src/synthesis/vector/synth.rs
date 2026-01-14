//! Vector synthesizer implementation.

use rand_pcg::Pcg32;

use super::super::Synthesizer;
use super::types::{VectorPath, VectorPosition, VectorSource, VectorSourceType};
use crate::oscillator::{self, PhaseAccumulator};

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
    pub(crate) fn calculate_mix_weights(position: &VectorPosition) -> [f64; 4] {
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
