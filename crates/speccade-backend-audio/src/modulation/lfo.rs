//! LFO (Low Frequency Oscillator) implementation.
//!
//! Generates modulation signals for various synthesis parameters.

use crate::oscillator::{sawtooth, sine, square, triangle, PhaseAccumulator};
use rand_pcg::Pcg32;
use speccade_spec::recipe::audio::Waveform;

/// LFO generator.
#[derive(Debug, Clone)]
pub struct Lfo {
    /// LFO waveform type.
    waveform: Waveform,
    /// LFO rate in Hz.
    rate: f64,
    /// Phase accumulator for waveform generation.
    phase_acc: PhaseAccumulator,
}

impl Lfo {
    /// Creates a new LFO.
    ///
    /// # Arguments
    /// * `waveform` - The waveform type (sine, triangle, square, sawtooth, pulse)
    /// * `rate` - LFO rate in Hz (typically 0.1-20 Hz)
    /// * `sample_rate` - Audio sample rate
    /// * `initial_phase` - Initial phase offset (0.0-1.0)
    pub fn new(waveform: Waveform, rate: f64, sample_rate: f64, initial_phase: f64) -> Self {
        let mut phase_acc = PhaseAccumulator::new(sample_rate);

        // Set initial phase directly in radians (0.0-1.0 cycles).
        let initial_phase = initial_phase.clamp(0.0, 1.0);
        phase_acc.set_phase_radians(initial_phase * std::f64::consts::TAU);

        Self {
            waveform,
            rate,
            phase_acc,
        }
    }

    /// Generates the next LFO sample.
    ///
    /// Returns a value in the range [0.0, 1.0].
    ///
    /// # Arguments
    /// * `_rng` - Random number generator for sample-and-hold mode (unused for now)
    pub fn next_sample(&mut self, _rng: &mut Pcg32) -> f64 {
        let phase = self.phase_acc.advance(self.rate);

        // For pulse waveform, treat it as square with 50% duty
        let waveform = if self.waveform == Waveform::Pulse {
            Waveform::Square
        } else {
            self.waveform
        };

        let raw_value = match waveform {
            Waveform::Sine => sine(phase),
            Waveform::Square | Waveform::Pulse => square(phase, 0.5),
            Waveform::Sawtooth => sawtooth(phase),
            Waveform::Triangle => triangle(phase),
        };

        // Convert from [-1.0, 1.0] to [0.0, 1.0]
        ((raw_value + 1.0) * 0.5).clamp(0.0, 1.0)
    }

    /// Generates a buffer of LFO samples.
    ///
    /// # Arguments
    /// * `num_samples` - Number of samples to generate
    /// * `rng` - Random number generator
    ///
    /// # Returns
    /// Vector of LFO values in range [0.0, 1.0]
    pub fn generate(&mut self, num_samples: usize, rng: &mut Pcg32) -> Vec<f64> {
        let mut output = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            output.push(self.next_sample(rng));
        }
        output
    }
}

/// Applies pitch modulation to a frequency value.
///
/// # Arguments
/// * `frequency` - Base frequency in Hz
/// * `lfo_value` - LFO modulation value (0.0-1.0)
/// * `semitones` - Maximum pitch deviation in semitones
///
/// # Returns
/// Modulated frequency in Hz
pub fn apply_pitch_modulation(frequency: f64, lfo_value: f64, semitones: f64) -> f64 {
    apply_pitch_modulation_with_depth(frequency, lfo_value, semitones, 1.0)
}

/// Applies pitch modulation to a frequency value, with an explicit depth scalar.
///
/// `depth` scales the target amount (`semitones`) and should be in [0.0, 1.0].
pub fn apply_pitch_modulation_with_depth(
    frequency: f64,
    lfo_value: f64,
    semitones: f64,
    depth: f64,
) -> f64 {
    // Convert LFO value from [0.0, 1.0] to [-1.0, 1.0]
    let bipolar = (lfo_value - 0.5) * 2.0;
    // Convert semitones to frequency multiplier
    let pitch_shift = bipolar * semitones * depth.clamp(0.0, 1.0);
    frequency * 2.0_f64.powf(pitch_shift / 12.0)
}

/// Applies volume modulation to an amplitude value.
///
/// # Arguments
/// * `amplitude` - Base amplitude
/// * `lfo_value` - LFO modulation value (0.0-1.0)
/// * `depth` - Modulation depth (0.0-1.0)
///
/// # Returns
/// Modulated amplitude
pub fn apply_volume_modulation(amplitude: f64, lfo_value: f64, amount: f64, depth: f64) -> f64 {
    let amount = amount.clamp(0.0, 1.0);
    let strength = amount * depth.clamp(0.0, 1.0);
    amplitude * ((1.0 - strength) + lfo_value * strength)
}

/// Applies filter cutoff modulation.
///
/// # Arguments
/// * `base_cutoff` - Base cutoff frequency in Hz
/// * `lfo_value` - LFO modulation value (0.0-1.0)
/// * `amount` - Maximum cutoff change in Hz
///
/// # Returns
/// Modulated cutoff frequency in Hz
pub fn apply_filter_cutoff_modulation(
    base_cutoff: f64,
    lfo_value: f64,
    amount_hz: f64,
    depth: f64,
) -> f64 {
    // Convert LFO value from [0.0, 1.0] to [-1.0, 1.0]
    let bipolar = (lfo_value - 0.5) * 2.0;
    (base_cutoff + bipolar * amount_hz * depth.clamp(0.0, 1.0)).max(20.0)
}

/// Applies pan modulation.
///
/// # Arguments
/// * `base_pan` - Base pan position (-1.0 to 1.0)
/// * `lfo_value` - LFO modulation value (0.0-1.0)
/// * `depth` - Modulation depth (0.0-1.0)
///
/// # Returns
/// Modulated pan position (-1.0 to 1.0)
pub fn apply_pan_modulation(base_pan: f64, lfo_value: f64, amount: f64, depth: f64) -> f64 {
    let amount = amount.clamp(0.0, 1.0);
    let strength = amount * depth.clamp(0.0, 1.0);
    // Convert LFO value from [0.0, 1.0] to [-1.0, 1.0]
    let bipolar = (lfo_value - 0.5) * 2.0;
    (base_pan + bipolar * strength).clamp(-1.0, 1.0)
}

/// Applies pulse width (duty cycle) modulation.
///
/// # Arguments
/// * `base_duty` - Base duty cycle (typically 0.5)
/// * `lfo_value` - LFO modulation value (0.0-1.0)
/// * `amount` - Maximum duty cycle delta (0.0-0.49)
/// * `depth` - Modulation depth (0.0-1.0)
///
/// # Returns
/// Modulated duty cycle clamped to (0.01, 0.99)
pub fn apply_pulse_width_modulation(
    base_duty: f64,
    lfo_value: f64,
    amount: f64,
    depth: f64,
) -> f64 {
    let amount = amount.clamp(0.0, 0.49);
    let strength = amount * depth.clamp(0.0, 1.0);
    // Convert LFO value from [0.0, 1.0] to [-1.0, 1.0]
    let bipolar = (lfo_value - 0.5) * 2.0;
    (base_duty + bipolar * strength).clamp(0.01, 0.99)
}

/// Applies FM modulation index modulation.
///
/// # Arguments
/// * `base_index` - Base FM modulation index
/// * `lfo_value` - LFO modulation value (0.0-1.0)
/// * `amount` - Maximum index delta
/// * `depth` - Modulation depth (0.0-1.0)
///
/// # Returns
/// Modulated modulation index clamped to >= 0.0
pub fn apply_fm_index_modulation(base_index: f64, lfo_value: f64, amount: f64, depth: f64) -> f64 {
    let strength = amount * depth.clamp(0.0, 1.0);
    // Convert LFO value from [0.0, 1.0] to [-1.0, 1.0]
    let bipolar = (lfo_value - 0.5) * 2.0;
    (base_index + bipolar * strength).max(0.0)
}

/// Applies grain size modulation for granular synthesis.
///
/// # Arguments
/// * `base_size_ms` - Base grain size in milliseconds
/// * `lfo_value` - LFO modulation value (0.0-1.0)
/// * `amount_ms` - Maximum grain size delta in milliseconds
/// * `depth` - Modulation depth (0.0-1.0)
///
/// # Returns
/// Modulated grain size clamped to [10.0, 500.0] ms
pub fn apply_grain_size_modulation(
    base_size_ms: f64,
    lfo_value: f64,
    amount_ms: f64,
    depth: f64,
) -> f64 {
    let strength = amount_ms * depth.clamp(0.0, 1.0);
    // Convert LFO value from [0.0, 1.0] to [-1.0, 1.0]
    let bipolar = (lfo_value - 0.5) * 2.0;
    (base_size_ms + bipolar * strength).clamp(10.0, 500.0)
}

/// Applies grain density modulation for granular synthesis.
///
/// # Arguments
/// * `base_density` - Base grain density in grains/sec
/// * `lfo_value` - LFO modulation value (0.0-1.0)
/// * `amount` - Maximum density delta in grains/sec
/// * `depth` - Modulation depth (0.0-1.0)
///
/// # Returns
/// Modulated grain density clamped to [1.0, 100.0] grains/sec
pub fn apply_grain_density_modulation(
    base_density: f64,
    lfo_value: f64,
    amount: f64,
    depth: f64,
) -> f64 {
    let strength = amount * depth.clamp(0.0, 1.0);
    // Convert LFO value from [0.0, 1.0] to [-1.0, 1.0]
    let bipolar = (lfo_value - 0.5) * 2.0;
    (base_density + bipolar * strength).clamp(1.0, 100.0)
}

/// Applies delay time modulation for post-FX delay effects.
///
/// # Arguments
/// * `base_time_ms` - Base delay time in milliseconds
/// * `lfo_value` - LFO modulation value (0.0-1.0)
/// * `amount_ms` - Maximum delay time delta in milliseconds
/// * `depth` - Modulation depth (0.0-1.0)
///
/// # Returns
/// Modulated delay time clamped to [1.0, 2000.0] ms
pub fn apply_delay_time_modulation(
    base_time_ms: f64,
    lfo_value: f64,
    amount_ms: f64,
    depth: f64,
) -> f64 {
    let strength = amount_ms * depth.clamp(0.0, 1.0);
    // Convert LFO value from [0.0, 1.0] to [-1.0, 1.0]
    let bipolar = (lfo_value - 0.5) * 2.0;
    (base_time_ms + bipolar * strength).clamp(1.0, 2000.0)
}

/// Applies reverb room size modulation for post-FX reverb effects.
///
/// # Arguments
/// * `base_room_size` - Base room size (0.0-1.0)
/// * `lfo_value` - LFO modulation value (0.0-1.0)
/// * `amount` - Maximum room size delta
/// * `depth` - Modulation depth (0.0-1.0)
///
/// # Returns
/// Modulated room size clamped to [0.0, 1.0]
pub fn apply_reverb_size_modulation(
    base_room_size: f64,
    lfo_value: f64,
    amount: f64,
    depth: f64,
) -> f64 {
    let strength = amount * depth.clamp(0.0, 1.0);
    // Convert LFO value from [0.0, 1.0] to [-1.0, 1.0]
    let bipolar = (lfo_value - 0.5) * 2.0;
    (base_room_size + bipolar * strength).clamp(0.0, 1.0)
}

/// Applies distortion drive modulation for post-FX distortion effects.
///
/// # Arguments
/// * `base_drive` - Base drive level (typically 1.0-100.0)
/// * `lfo_value` - LFO modulation value (0.0-1.0)
/// * `amount` - Maximum drive delta
/// * `depth` - Modulation depth (0.0-1.0)
///
/// # Returns
/// Modulated drive clamped to [1.0, 100.0]
pub fn apply_distortion_drive_modulation(
    base_drive: f64,
    lfo_value: f64,
    amount: f64,
    depth: f64,
) -> f64 {
    let strength = amount * depth.clamp(0.0, 1.0);
    // Convert LFO value from [0.0, 1.0] to [-1.0, 1.0]
    let bipolar = (lfo_value - 0.5) * 2.0;
    (base_drive + bipolar * strength).clamp(1.0, 100.0)
}
