//! Spectral freeze synthesis implementation.
//!
//! Uses FFT to capture the spectral content of a source signal and sustain it
//! indefinitely via repeated inverse FFT with overlap-add.

use rand_pcg::Pcg32;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use speccade_spec::recipe::audio::{NoiseType, SpectralSource, Waveform};

use crate::oscillator::PhaseAccumulator;
use crate::synthesis::Synthesizer;

use std::f64::consts::PI;

/// Fixed FFT size for spectral freeze.
const FFT_SIZE: usize = 2048;

/// Fixed hop size for overlap-add.
const HOP_SIZE: usize = 512;

/// Spectral freeze synthesizer.
#[derive(Debug, Clone)]
pub struct SpectralFreezeSynth {
    /// Source material for spectral capture.
    source: SpectralSource,
}

impl SpectralFreezeSynth {
    /// Creates a new spectral freeze synthesizer.
    ///
    /// # Arguments
    /// * `source` - Source material to capture and freeze
    pub fn new(source: SpectralSource) -> Self {
        Self { source }
    }
}

impl Synthesizer for SpectralFreezeSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, rng: &mut Pcg32) -> Vec<f64> {
        // Step 1: Generate one deterministic source frame of FFT_SIZE samples
        let source_frame = generate_source_frame(&self.source, sample_rate, rng);

        // Step 2: Apply Hann window and compute FFT
        let windowed_frame: Vec<f64> = source_frame
            .iter()
            .enumerate()
            .map(|(i, &s)| s * hann_window(i, FFT_SIZE))
            .collect();

        // Convert to complex for FFT
        let mut spectrum: Vec<Complex<f64>> = windowed_frame
            .iter()
            .map(|&s| Complex::new(s, 0.0))
            .collect();

        // Compute forward FFT
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);
        fft.process(&mut spectrum);

        // Step 3: Render full output by repeated inverse FFT with overlap-add
        let mut output = vec![0.0; num_samples];

        // Prepare inverse FFT
        let ifft = planner.plan_fft_inverse(FFT_SIZE);

        // Calculate window sum for normalization (Hann window overlap-add at hop_size)
        // For Hann window with 75% overlap, the sum is approximately constant
        let window_sum = calculate_window_sum();

        // Render frames with overlap-add
        let mut frame_start: i64 = 0;
        while (frame_start as usize) < num_samples {
            // Clone spectrum for IFFT (IFFT modifies in place)
            let mut frame_spectrum = spectrum.clone();

            // Compute inverse FFT
            ifft.process(&mut frame_spectrum);

            // Extract real part and apply window, then overlap-add
            for (i, complex_sample) in frame_spectrum.iter().enumerate() {
                let output_idx = frame_start as usize + i;
                if output_idx >= num_samples {
                    break;
                }

                // IFFT result needs to be scaled by 1/N
                let sample = complex_sample.re / FFT_SIZE as f64;
                let windowed = sample * hann_window(i, FFT_SIZE);
                output[output_idx] += windowed / window_sum;
            }

            frame_start += HOP_SIZE as i64;
        }

        output
    }
}

/// Generates a source frame based on the spectral source type.
fn generate_source_frame(source: &SpectralSource, sample_rate: f64, rng: &mut Pcg32) -> Vec<f64> {
    match source {
        SpectralSource::Noise { noise_type } => generate_noise_frame(noise_type, rng),
        SpectralSource::Tone {
            waveform,
            frequency,
        } => generate_tone_frame(waveform, *frequency, sample_rate),
    }
}

/// Generates a noise-based source frame.
fn generate_noise_frame(noise_type: &NoiseType, rng: &mut Pcg32) -> Vec<f64> {
    match noise_type {
        NoiseType::White => crate::oscillator::white_noise(rng, FFT_SIZE),
        NoiseType::Pink => crate::oscillator::pink_noise(rng, FFT_SIZE),
        NoiseType::Brown => crate::oscillator::brown_noise(rng, FFT_SIZE),
    }
}

/// Generates a tone-based source frame.
fn generate_tone_frame(waveform: &Waveform, frequency: f64, sample_rate: f64) -> Vec<f64> {
    let mut phase_acc = PhaseAccumulator::new(sample_rate);
    let mut samples = Vec::with_capacity(FFT_SIZE);

    for _ in 0..FFT_SIZE {
        let phase = phase_acc.advance(frequency);
        let sample = match waveform {
            Waveform::Sine => crate::oscillator::sine(phase),
            Waveform::Square | Waveform::Pulse => crate::oscillator::square(phase, 0.5),
            Waveform::Sawtooth => crate::oscillator::sawtooth(phase),
            Waveform::Triangle => crate::oscillator::triangle(phase),
        };
        samples.push(sample);
    }

    samples
}

/// Computes the Hann window value at a given index.
#[inline]
fn hann_window(i: usize, size: usize) -> f64 {
    0.5 * (1.0 - (2.0 * PI * i as f64 / size as f64).cos())
}

/// Calculates the overlap-add window sum for normalization.
///
/// For Hann window with hop_size = FFT_SIZE/4 (75% overlap),
/// the sum of windows at any point is approximately constant.
fn calculate_window_sum() -> f64 {
    // Number of overlapping frames at any given sample point
    let num_overlaps = FFT_SIZE / HOP_SIZE;

    // Sum the Hann window values at the center point
    let mut sum = 0.0;
    for frame in 0..num_overlaps {
        let offset = frame * HOP_SIZE;
        // Sample position within each frame at the center of output
        // Use saturating subtraction to avoid overflow
        let center = FFT_SIZE / 2;
        if offset <= center {
            let pos_in_frame = center - offset;
            if pos_in_frame < FFT_SIZE {
                let w = hann_window(pos_in_frame, FFT_SIZE);
                sum += w;
            }
        }
    }

    sum.max(0.001) // Prevent division by zero
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;

    #[test]
    fn test_spectral_freeze_noise_basic() {
        let mut rng = create_rng(42);
        let source = SpectralSource::Noise {
            noise_type: NoiseType::White,
        };

        let synth = SpectralFreezeSynth::new(source);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        // Should produce some non-zero output
        assert!(samples.iter().any(|&s| s.abs() > 0.001));
    }

    #[test]
    fn test_spectral_freeze_tone_basic() {
        let mut rng = create_rng(42);
        let source = SpectralSource::Tone {
            waveform: Waveform::Sine,
            frequency: 440.0,
        };

        let synth = SpectralFreezeSynth::new(source);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        // Should produce some non-zero output
        assert!(samples.iter().any(|&s| s.abs() > 0.001));
    }

    #[test]
    fn test_spectral_freeze_determinism() {
        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let source = SpectralSource::Noise {
            noise_type: NoiseType::Pink,
        };

        let synth1 = SpectralFreezeSynth::new(source.clone());
        let samples1 = synth1.synthesize(4410, 44100.0, &mut rng1);

        let synth2 = SpectralFreezeSynth::new(source);
        let samples2 = synth2.synthesize(4410, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_spectral_freeze_output_bounded() {
        let mut rng = create_rng(42);
        let source = SpectralSource::Tone {
            waveform: Waveform::Sawtooth,
            frequency: 220.0,
        };

        let synth = SpectralFreezeSynth::new(source);
        let samples = synth.synthesize(44100, 44100.0, &mut rng);

        // Output should be reasonably bounded (envelope will shape final amplitude)
        let max_abs = samples.iter().map(|&s| s.abs()).fold(0.0, f64::max);
        assert!(max_abs < 10.0, "max_abs was {}", max_abs);
    }

    #[test]
    fn test_spectral_freeze_short_output() {
        let mut rng = create_rng(42);
        let source = SpectralSource::Tone {
            waveform: Waveform::Sine,
            frequency: 440.0,
        };

        let synth = SpectralFreezeSynth::new(source);
        // Very short output (less than one FFT frame)
        let samples = synth.synthesize(512, 44100.0, &mut rng);

        assert_eq!(samples.len(), 512);
    }

    #[test]
    fn test_hann_window() {
        // Window should be 0 at boundaries
        assert!((hann_window(0, 1024)).abs() < 1e-10);

        // Window should be 1 at center
        assert!((hann_window(512, 1024) - 1.0).abs() < 1e-10);
    }
}
