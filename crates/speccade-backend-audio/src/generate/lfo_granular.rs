//! LFO modulation for granular synthesis.

use std::f64::consts::PI;

use speccade_spec::recipe::audio::{AudioLayer, GranularSource, Synthesis};

use crate::modulation::lfo::{apply_grain_density_modulation, apply_grain_size_modulation, Lfo};
use crate::oscillator::PhaseAccumulator;

/// Parameters for LFO grain size modulation.
pub struct LfoGrainSizeParams<'a> {
    /// The audio layer to modulate
    pub layer: &'a AudioLayer,
    /// Number of samples to generate
    pub num_samples: usize,
    /// Sample rate in Hz
    pub sample_rate: f64,
    /// LFO instance
    pub lfo: &'a mut Lfo,
    /// Grain size modulation amount in milliseconds
    pub amount_ms: f64,
    /// Modulation depth (0.0-1.0)
    pub depth: f64,
    /// RNG for LFO and granular synthesis
    pub rng: &'a mut rand_pcg::Pcg32,
}

/// Parameters for LFO grain density modulation.
pub struct LfoGrainDensityParams<'a> {
    /// The audio layer to modulate
    pub layer: &'a AudioLayer,
    /// Number of samples to generate
    pub num_samples: usize,
    /// Sample rate in Hz
    pub sample_rate: f64,
    /// LFO instance
    pub lfo: &'a mut Lfo,
    /// Grain density modulation amount in grains/sec
    pub amount: f64,
    /// Modulation depth (0.0-1.0)
    pub depth: f64,
    /// RNG for LFO and granular synthesis
    pub rng: &'a mut rand_pcg::Pcg32,
}

/// Applies LFO grain size modulation to a granular synthesis layer.
///
/// Modulates grain size per-grain by sampling LFO at each grain start time.
pub fn apply_lfo_grain_size_modulation(params: LfoGrainSizeParams<'_>) -> Vec<f64> {
    let LfoGrainSizeParams {
        layer,
        num_samples,
        sample_rate,
        lfo,
        amount_ms,
        depth,
        rng,
    } = params;

    match &layer.synthesis {
        Synthesis::Granular {
            source,
            grain_size_ms,
            grain_density,
            pitch_spread,
            position_spread,
            pan_spread,
        } => synthesize_granular_with_lfo_modulation(
            source,
            *grain_size_ms,
            *grain_density,
            *pitch_spread,
            *position_spread,
            *pan_spread,
            num_samples,
            sample_rate,
            lfo,
            Some(amount_ms),
            None,
            depth,
            rng,
        ),
        _ => {
            // Unsupported synthesis type - return silence
            vec![0.0; num_samples]
        }
    }
}

/// Applies LFO grain density modulation to a granular synthesis layer.
///
/// Modulates grain density per-grain by sampling LFO at each grain start time.
pub fn apply_lfo_grain_density_modulation(params: LfoGrainDensityParams<'_>) -> Vec<f64> {
    let LfoGrainDensityParams {
        layer,
        num_samples,
        sample_rate,
        lfo,
        amount,
        depth,
        rng,
    } = params;

    match &layer.synthesis {
        Synthesis::Granular {
            source,
            grain_size_ms,
            grain_density,
            pitch_spread,
            position_spread,
            pan_spread,
        } => synthesize_granular_with_lfo_modulation(
            source,
            *grain_size_ms,
            *grain_density,
            *pitch_spread,
            *position_spread,
            *pan_spread,
            num_samples,
            sample_rate,
            lfo,
            None,
            Some(amount),
            depth,
            rng,
        ),
        _ => {
            // Unsupported synthesis type - return silence
            vec![0.0; num_samples]
        }
    }
}

/// Granular synthesis with optional LFO modulation of grain size and/or density.
///
/// Parameters are sampled from the LFO at each grain start time rather than per-sample.
#[allow(clippy::too_many_arguments)]
fn synthesize_granular_with_lfo_modulation(
    source: &GranularSource,
    base_grain_size_ms: f64,
    base_grain_density: f64,
    pitch_spread: f64,
    position_spread: f64,
    pan_spread: f64,
    num_samples: usize,
    sample_rate: f64,
    lfo: &mut Lfo,
    grain_size_amount_ms: Option<f64>,
    grain_density_amount: Option<f64>,
    depth: f64,
    rng: &mut rand_pcg::Pcg32,
) -> Vec<f64> {
    use rand::Rng;

    // For stereo output, we'll generate interleaved samples [L, R, L, R, ...]
    // For mono output (pan_spread == 0.0), we'll generate [M, M, M, ...]
    let is_stereo = pan_spread > 0.0;
    let output_samples = if is_stereo {
        num_samples * 2
    } else {
        num_samples
    };

    let mut output = vec![0.0; output_samples];

    // Calculate initial grain interval for pre-generating LFO curve
    let initial_grain_interval_samples = (sample_rate / base_grain_density).max(1.0) as usize;

    // Generate grains with LFO-modulated parameters
    let mut current_sample = 0usize;
    while current_sample < num_samples {
        // Get LFO value at grain start time
        let lfo_value = lfo.next_sample(rng);

        // Calculate modulated grain size
        let grain_size_ms = if let Some(amount_ms) = grain_size_amount_ms {
            apply_grain_size_modulation(base_grain_size_ms, lfo_value, amount_ms, depth)
        } else {
            base_grain_size_ms
        };

        // Calculate modulated grain density
        let grain_density = if let Some(amount) = grain_density_amount {
            apply_grain_density_modulation(base_grain_density, lfo_value, amount, depth)
        } else {
            base_grain_density
        };

        // Calculate grain parameters in samples
        let grain_size_samples = ((grain_size_ms / 1000.0) * sample_rate) as usize;
        if grain_size_samples == 0 {
            current_sample += initial_grain_interval_samples.max(1);
            continue;
        }

        // Calculate interval based on current density
        let grain_interval_samples = (sample_rate / grain_density).max(1.0) as usize;

        // Apply position jitter
        let jitter = if position_spread > 0.0 {
            let jitter_range = (position_spread * grain_interval_samples as f64 * 0.5) as i32;
            if jitter_range > 0 {
                rng.gen_range(-jitter_range..=jitter_range)
            } else {
                0
            }
        } else {
            0
        };

        let grain_start = (current_sample as i32 + jitter)
            .max(0)
            .min(num_samples as i32 - 1) as usize;

        // Generate pitch shift for this grain
        let pitch_shift = if pitch_spread > 0.0 {
            let semitones = rng.gen_range(-pitch_spread..=pitch_spread);
            2.0_f64.powf(semitones / 12.0)
        } else {
            1.0
        };

        // Generate pan for this grain (0.0 = center, -1.0 = left, 1.0 = right)
        let pan = if is_stereo {
            rng.gen_range(-pan_spread..=pan_spread)
        } else {
            0.0
        };

        // Convert pan to left/right gains using equal power panning
        let (left_gain, right_gain) = if is_stereo {
            let pan_angle = (pan + 1.0) * 0.5 * PI / 2.0;
            (pan_angle.cos(), pan_angle.sin())
        } else {
            (1.0, 1.0)
        };

        // Generate grain samples
        let grain_samples =
            generate_grain(source, grain_size_samples, sample_rate, pitch_shift, rng);

        // Apply Hann window and add to output buffer (overlap-add)
        for (i, &grain_sample) in grain_samples.iter().enumerate() {
            let output_idx = grain_start + i;
            if output_idx >= num_samples {
                break;
            }

            // Apply Hann window
            let window = 0.5 * (1.0 - (2.0 * PI * i as f64 / grain_size_samples as f64).cos());
            let windowed_sample = grain_sample * window;

            if is_stereo {
                // Stereo output
                let left_idx = output_idx * 2;
                let right_idx = output_idx * 2 + 1;
                output[left_idx] += windowed_sample * left_gain;
                output[right_idx] += windowed_sample * right_gain;
            } else {
                // Mono output
                output[output_idx] += windowed_sample;
            }
        }

        current_sample += grain_interval_samples;
    }

    // Normalize by approximate grain overlap
    let avg_grain_size_samples = (base_grain_size_ms / 1000.0) * sample_rate;
    let avg_grain_interval_samples = sample_rate / base_grain_density;
    let overlap_factor = (avg_grain_size_samples / avg_grain_interval_samples).max(1.0);
    let normalization = 1.0 / overlap_factor.sqrt();

    for sample in &mut output {
        *sample *= normalization;
    }

    // If stereo, convert to mono for layer output
    if is_stereo {
        let mut mono_samples = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let left = output[i * 2];
            let right = output[i * 2 + 1];
            mono_samples.push((left + right) * 0.5);
        }
        mono_samples
    } else {
        output
    }
}

/// Generates a single grain of audio.
fn generate_grain(
    source: &GranularSource,
    grain_size_samples: usize,
    sample_rate: f64,
    pitch_shift: f64,
    rng: &mut rand_pcg::Pcg32,
) -> Vec<f64> {
    match source {
        GranularSource::Noise { noise_type } => {
            generate_noise_grain(grain_size_samples, noise_type, rng)
        }
        GranularSource::Tone {
            waveform,
            frequency,
        } => generate_tone_grain(
            grain_size_samples,
            sample_rate,
            *frequency * pitch_shift,
            waveform,
        ),
        GranularSource::Formant {
            frequency,
            formant_freq,
        } => generate_formant_grain(
            grain_size_samples,
            sample_rate,
            *frequency * pitch_shift,
            *formant_freq,
        ),
    }
}

/// Generates a noise-based grain.
fn generate_noise_grain(
    grain_size_samples: usize,
    noise_type: &speccade_spec::recipe::audio::NoiseType,
    rng: &mut rand_pcg::Pcg32,
) -> Vec<f64> {
    use speccade_spec::recipe::audio::NoiseType;

    match noise_type {
        NoiseType::White => crate::oscillator::white_noise(rng, grain_size_samples),
        NoiseType::Pink => crate::oscillator::pink_noise(rng, grain_size_samples),
        NoiseType::Brown => crate::oscillator::brown_noise(rng, grain_size_samples),
    }
}

/// Generates a tone-based grain.
fn generate_tone_grain(
    grain_size_samples: usize,
    sample_rate: f64,
    frequency: f64,
    waveform: &speccade_spec::recipe::audio::Waveform,
) -> Vec<f64> {
    use speccade_spec::recipe::audio::Waveform;

    let mut phase_acc = PhaseAccumulator::new(sample_rate);
    let mut samples = Vec::with_capacity(grain_size_samples);

    for _ in 0..grain_size_samples {
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

/// Generates a formant-based grain.
fn generate_formant_grain(
    grain_size_samples: usize,
    sample_rate: f64,
    frequency: f64,
    formant_freq: f64,
) -> Vec<f64> {
    let mut carrier_phase = PhaseAccumulator::new(sample_rate);
    let mut formant_phase = PhaseAccumulator::new(sample_rate);
    let mut samples = Vec::with_capacity(grain_size_samples);

    for _ in 0..grain_size_samples {
        let carrier = crate::oscillator::sawtooth(carrier_phase.advance(frequency));
        let formant = crate::oscillator::sine(formant_phase.advance(formant_freq));

        // Ring modulation
        let sample = carrier * (0.5 + 0.5 * formant);
        samples.push(sample);
    }

    samples
}
