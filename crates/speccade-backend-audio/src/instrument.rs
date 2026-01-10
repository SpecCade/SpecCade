//! Instrument sample generation.
//!
//! This module generates instrument samples from synth patch specifications.
//! Instruments differ from SFX in that they represent sustained musical notes
//! rather than short sound effects, and may include loop points for trackers.

use speccade_spec::recipe::audio_instrument::{
    AudioInstrumentSynthPatchV1Params, InstrumentSynthesis,
};

use crate::envelope::{AdsrEnvelope, AdsrParams};
use crate::error::{AudioError, AudioResult};
use crate::mixer;
use crate::rng::create_rng;
use crate::synthesis::fm::FmSynth;
use crate::synthesis::harmonics::HarmonicSynth;
use crate::synthesis::karplus::KarplusStrong;
use crate::synthesis::metallic::MetallicSynth;
use crate::synthesis::noise::{NoiseColor, NoiseSynth};
use crate::synthesis::oscillators::{SawSynth, SineSynth, SquareSynth, TriangleSynth};
use crate::synthesis::pitched_body::PitchedBody;
use crate::synthesis::{FrequencySweep, SweepCurve, Synthesizer};
use crate::wav::WavResult;
use speccade_spec::recipe::audio_sfx::{Filter, NoiseType, Synthesis, Waveform};

/// Result of instrument generation.
#[derive(Debug)]
pub struct GenerateInstrumentResult {
    /// WAV file data.
    pub wav: WavResult,
    /// Note(s) generated (MIDI note numbers).
    pub notes: Vec<u8>,
    /// Loop point in samples (if generated).
    pub loop_point: Option<usize>,
}

/// Generates an instrument from parameters.
///
/// # Arguments
/// * `params` - Instrument synthesis parameters
/// * `seed` - RNG seed for deterministic generation
///
/// # Returns
/// Generated WAV file and metadata
pub fn generate_instrument(
    params: &AudioInstrumentSynthPatchV1Params,
    seed: u32,
) -> AudioResult<GenerateInstrumentResult> {
    let sample_rate = params.sample_rate as f64;
    let num_samples = (params.note_duration_seconds * sample_rate).ceil() as usize;

    // Determine which notes to generate
    let notes = if let Some(note_specs) = &params.notes {
        // Convert note specs to MIDI note numbers
        note_specs
            .iter()
            .filter_map(|spec| spec.to_midi_note())
            .collect::<Vec<_>>()
    } else {
        // Default to A4 (MIDI 69, 440 Hz)
        vec![69]
    };

    // For now, generate only the first note
    // Multi-note support can be added later if needed
    let midi_note = *notes
        .first()
        .ok_or_else(|| AudioError::invalid_param("notes", "No valid notes specified"))?;

    // Convert MIDI note to frequency
    let frequency = midi_to_frequency(midi_note);

    // Generate the instrument sample
    let mut rng = create_rng(seed);
    let mut samples = generate_instrument_synthesis(
        &params.synthesis,
        frequency,
        num_samples,
        sample_rate,
        &mut rng,
    )?;

    // Apply pitch envelope if specified
    if let Some(ref pitch_env) = params.pitch_envelope {
        let pitch_curve = generate_pitch_envelope_curve(pitch_env, sample_rate, num_samples);
        // Re-generate with pitch modulation
        samples = generate_synthesis_with_pitch_modulation(
            &params.synthesis,
            frequency,
            &pitch_curve,
            num_samples,
            sample_rate,
            &mut rng,
        )?;
    }

    // Apply envelope
    let envelope = generate_envelope(&params.envelope, sample_rate, num_samples);
    for (sample, env) in samples.iter_mut().zip(envelope.iter()) {
        *sample *= env;
    }

    // Normalize
    mixer::normalize(&mut samples, -3.0);

    // Determine loop point if requested
    let loop_point = if params.generate_loop_points {
        Some(calculate_loop_point(
            &params.envelope,
            sample_rate,
            num_samples,
        ))
    } else {
        None
    };

    // Convert to WAV
    let wav = WavResult::from_mono(&samples, params.sample_rate);

    Ok(GenerateInstrumentResult {
        wav,
        notes: vec![midi_note],
        loop_point,
    })
}

/// Generates synthesis samples based on InstrumentSynthesis type.
fn generate_instrument_synthesis(
    synthesis: &InstrumentSynthesis,
    frequency: f64,
    num_samples: usize,
    sample_rate: f64,
    rng: &mut rand_pcg::Pcg32,
) -> AudioResult<Vec<f64>> {
    match synthesis {
        InstrumentSynthesis::FmOperators { operators } => {
            generate_fm_operators(operators, frequency, num_samples, sample_rate, rng)
        }
        InstrumentSynthesis::Simple { synthesis } => {
            generate_synthesis(synthesis, frequency, num_samples, sample_rate, rng)
        }
    }
}

/// Generates synthesis samples with pitch envelope modulation.
fn generate_synthesis_with_pitch_modulation(
    inst_synthesis: &InstrumentSynthesis,
    base_frequency: f64,
    pitch_curve: &[f64],
    num_samples: usize,
    sample_rate: f64,
    rng: &mut rand_pcg::Pcg32,
) -> AudioResult<Vec<f64>> {
    // For FM operators with pitch envelope, need special handling
    if let InstrumentSynthesis::FmOperators { operators } = inst_synthesis {
        return generate_fm_operators_with_pitch(
            operators,
            base_frequency,
            pitch_curve,
            num_samples,
            sample_rate,
            rng,
        );
    }

    // For simple synthesis, extract inner synthesis
    let synthesis = match inst_synthesis {
        InstrumentSynthesis::Simple { synthesis } => synthesis,
        _ => unreachable!(),
    };
    use crate::oscillator::{PhaseAccumulator, TWO_PI};

    // For pitch envelope, we need to generate sample-by-sample
    let mut output = vec![0.0; num_samples];

    match synthesis {
        Synthesis::Oscillator {
            waveform,
            detune,
            duty,
            ..
        } => {
            let detune_mult = if let Some(detune_cents) = detune {
                2.0_f64.powf(*detune_cents / 1200.0)
            } else {
                1.0
            };

            let duty_cycle = duty.unwrap_or(0.5);
            let mut phase_acc = PhaseAccumulator::new(sample_rate);

            for i in 0..num_samples {
                let freq = base_frequency * detune_mult * pitch_curve[i];
                let phase = phase_acc.advance(freq);

                let sample = match waveform {
                    Waveform::Sine => crate::oscillator::sine(phase),
                    Waveform::Square | Waveform::Pulse => {
                        crate::oscillator::square(phase, duty_cycle)
                    }
                    Waveform::Sawtooth => crate::oscillator::sawtooth(phase),
                    Waveform::Triangle => crate::oscillator::triangle(phase),
                };

                output[i] = sample;
            }

            Ok(output)
        }

        Synthesis::MultiOscillator { oscillators, .. } => {
            for osc_config in oscillators {
                let detune_mult = if let Some(detune_cents) = osc_config.detune {
                    2.0_f64.powf(detune_cents / 1200.0)
                } else {
                    1.0
                };

                let duty = osc_config.duty.unwrap_or(0.5);
                let phase_offset = osc_config.phase.unwrap_or(0.0);
                let volume = osc_config.volume;
                let mut phase_acc = PhaseAccumulator::new(sample_rate);

                for i in 0..num_samples {
                    let freq = base_frequency * detune_mult * pitch_curve[i];
                    let mut phase = phase_acc.advance(freq);
                    phase += phase_offset;

                    while phase >= TWO_PI {
                        phase -= TWO_PI;
                    }

                    let sample = match osc_config.waveform {
                        Waveform::Sine => crate::oscillator::sine(phase),
                        Waveform::Square | Waveform::Pulse => {
                            crate::oscillator::square(phase, duty)
                        }
                        Waveform::Sawtooth => crate::oscillator::sawtooth(phase),
                        Waveform::Triangle => crate::oscillator::triangle(phase),
                    };

                    output[i] += sample * volume;
                }
            }

            // Normalize by oscillator count
            let count = oscillators.len().max(1) as f64;
            for sample in &mut output {
                *sample /= count;
            }

            Ok(output)
        }

        // For other synthesis types, fall back to regenerating without pitch envelope
        _ => generate_synthesis(synthesis, base_frequency, num_samples, sample_rate, rng),
    }
}

/// Generates synthesis samples at a specific frequency.
fn generate_synthesis(
    synthesis: &Synthesis,
    frequency: f64,
    num_samples: usize,
    sample_rate: f64,
    rng: &mut rand_pcg::Pcg32,
) -> AudioResult<Vec<f64>> {
    let samples = match synthesis {
        Synthesis::FmSynth {
            carrier_freq: _,
            modulator_freq,
            modulation_index,
            freq_sweep,
        } => {
            // For instruments, use the note frequency as carrier
            let mut synth = FmSynth::new(frequency, *modulator_freq, *modulation_index);

            if let Some(sweep) = freq_sweep {
                let curve = convert_sweep_curve(&sweep.curve);
                synth = synth.with_sweep(FrequencySweep::new(frequency, sweep.end_freq, curve));
            }

            synth.synthesize(num_samples, sample_rate, rng)
        }

        Synthesis::KarplusStrong { decay, blend, .. } => {
            // Use the instrument frequency instead of spec frequency
            let synth = KarplusStrong::new(frequency, *decay, *blend);
            synth.synthesize(num_samples, sample_rate, rng)
        }

        Synthesis::NoiseBurst { noise_type, filter } => {
            let color = convert_noise_type(noise_type);
            let mut synth = NoiseSynth::new(color);

            if let Some(f) = filter {
                synth = apply_noise_filter(synth, f);
            }

            synth.synthesize(num_samples, sample_rate, rng)
        }

        Synthesis::Additive {
            base_freq: _,
            harmonics,
        } => {
            // Use the instrument frequency as base
            let synth = HarmonicSynth::new(frequency, harmonics.clone());
            synth.synthesize(num_samples, sample_rate, rng)
        }

        Synthesis::Oscillator {
            waveform,
            frequency: _,
            freq_sweep,
            detune,
            duty,
        } => {
            // Apply detune if specified
            let actual_freq = if let Some(detune_cents) = detune {
                apply_detune(frequency, *detune_cents)
            } else {
                frequency
            };

            // Generate oscillator with duty cycle support
            generate_oscillator_with_duty(
                waveform,
                actual_freq,
                freq_sweep.as_ref(),
                *duty,
                num_samples,
                sample_rate,
                rng,
            )
        }

        Synthesis::MultiOscillator {
            frequency: _,
            oscillators,
            freq_sweep,
        } => {
            // Use the instrument frequency as base
            generate_multi_oscillator(
                frequency,
                oscillators,
                freq_sweep.as_ref(),
                num_samples,
                sample_rate,
                rng,
            )
        }

        Synthesis::PitchedBody {
            start_freq,
            end_freq,
        } => {
            let synth = PitchedBody::new(*start_freq, *end_freq);
            synth.synthesize(num_samples, sample_rate, rng)
        }

        Synthesis::Metallic {
            base_freq,
            num_partials,
            inharmonicity,
        } => {
            let synth = MetallicSynth::new(*base_freq, *num_partials, *inharmonicity);
            synth.synthesize(num_samples, sample_rate, rng)
        }
    };

    Ok(samples)
}

/// Generates oscillator samples with duty cycle support.
fn generate_oscillator_with_duty(
    waveform: &Waveform,
    frequency: f64,
    freq_sweep: Option<&speccade_spec::recipe::audio_sfx::FreqSweep>,
    duty: Option<f64>,
    num_samples: usize,
    sample_rate: f64,
    rng: &mut rand_pcg::Pcg32,
) -> Vec<f64> {
    let sweep = freq_sweep.map(|s| {
        let curve = convert_sweep_curve(&s.curve);
        FrequencySweep::new(frequency, s.end_freq, curve)
    });

    match waveform {
        Waveform::Sine => {
            if let Some(s) = sweep {
                SineSynth::with_sweep(frequency, s.end_freq, s.curve).synthesize(
                    num_samples,
                    sample_rate,
                    rng,
                )
            } else {
                SineSynth::new(frequency).synthesize(num_samples, sample_rate, rng)
            }
        }
        Waveform::Square | Waveform::Pulse => {
            let duty_cycle = duty.unwrap_or(0.5);
            if let Some(s) = sweep {
                let mut synth = SquareSynth::with_sweep(frequency, s.end_freq, s.curve);
                synth.duty = duty_cycle;
                synth.synthesize(num_samples, sample_rate, rng)
            } else {
                SquareSynth::pulse(frequency, duty_cycle).synthesize(num_samples, sample_rate, rng)
            }
        }
        Waveform::Sawtooth => {
            if let Some(s) = sweep {
                SawSynth::with_sweep(frequency, s.end_freq, s.curve).synthesize(
                    num_samples,
                    sample_rate,
                    rng,
                )
            } else {
                SawSynth::new(frequency).synthesize(num_samples, sample_rate, rng)
            }
        }
        Waveform::Triangle => {
            if let Some(s) = sweep {
                TriangleSynth::with_sweep(frequency, s.end_freq, s.curve).synthesize(
                    num_samples,
                    sample_rate,
                    rng,
                )
            } else {
                TriangleSynth::new(frequency).synthesize(num_samples, sample_rate, rng)
            }
        }
    }
}

/// Generates multi-oscillator stack samples.
fn generate_multi_oscillator(
    base_frequency: f64,
    oscillators: &[speccade_spec::recipe::audio_sfx::OscillatorConfig],
    freq_sweep: Option<&speccade_spec::recipe::audio_sfx::FreqSweep>,
    num_samples: usize,
    sample_rate: f64,
    rng: &mut rand_pcg::Pcg32,
) -> Vec<f64> {
    use crate::oscillator::{PhaseAccumulator, TWO_PI};

    let mut output = vec![0.0; num_samples];

    // Sweep applied to all oscillators
    let sweep_curve = freq_sweep.map(|s| {
        let curve = convert_sweep_curve(&s.curve);
        FrequencySweep::new(base_frequency, s.end_freq, curve)
    });

    for osc_config in oscillators {
        // Calculate oscillator frequency with detune
        let detune_mult = if let Some(detune_cents) = osc_config.detune {
            2.0_f64.powf(detune_cents / 1200.0)
        } else {
            1.0
        };

        let duty = osc_config.duty.unwrap_or(0.5);
        let phase_offset = osc_config.phase.unwrap_or(0.0);
        let volume = osc_config.volume;

        // Generate oscillator samples
        let mut phase_acc = PhaseAccumulator::new(sample_rate);

        for i in 0..num_samples {
            let base_freq = if let Some(ref sweep) = sweep_curve {
                sweep.at(i as f64 / num_samples as f64)
            } else {
                base_frequency
            };

            let freq = base_freq * detune_mult;
            let mut phase = phase_acc.advance(freq);
            phase += phase_offset;

            // Wrap phase
            while phase >= TWO_PI {
                phase -= TWO_PI;
            }

            let sample = match osc_config.waveform {
                Waveform::Sine => crate::oscillator::sine(phase),
                Waveform::Square | Waveform::Pulse => crate::oscillator::square(phase, duty),
                Waveform::Sawtooth => crate::oscillator::sawtooth(phase),
                Waveform::Triangle => crate::oscillator::triangle(phase),
            };

            output[i] += sample * volume;
        }
    }

    // Normalize by oscillator count to prevent clipping
    let count = oscillators.len().max(1) as f64;
    for sample in &mut output {
        *sample /= count;
    }

    output
}

/// Applies detune to a base frequency.
///
/// Detune is specified in cents (100 cents = 1 semitone).
/// Formula: freq_actual = freq * 2^(detune / 1200)
fn apply_detune(frequency: f64, detune_cents: f64) -> f64 {
    frequency * 2.0_f64.powf(detune_cents / 1200.0)
}

/// Generates a pitch envelope curve.
///
/// Returns a vector of frequency multipliers (1.0 = no change).
fn generate_pitch_envelope_curve(
    pitch_env: &speccade_spec::recipe::audio_sfx::PitchEnvelope,
    sample_rate: f64,
    num_samples: usize,
) -> Vec<f64> {
    let duration = num_samples as f64 / sample_rate;
    let attack_samples = (pitch_env.attack * sample_rate) as usize;
    let decay_samples = (pitch_env.decay * sample_rate) as usize;
    let release_samples = (pitch_env.release * sample_rate) as usize;
    let sustain_samples =
        num_samples.saturating_sub(attack_samples + decay_samples + release_samples);

    let mut curve = Vec::with_capacity(num_samples);

    // Convert depth from semitones to frequency multiplier
    let depth_multiplier = 2.0_f64.powf(pitch_env.depth / 12.0);

    // Attack phase: 1.0 -> depth_multiplier
    for i in 0..attack_samples {
        let t = i as f64 / attack_samples.max(1) as f64;
        let multiplier = 1.0 + (depth_multiplier - 1.0) * t;
        curve.push(multiplier);
    }

    // Decay phase: depth_multiplier -> sustain_level * depth_multiplier
    for i in 0..decay_samples {
        let t = i as f64 / decay_samples.max(1) as f64;
        let start = depth_multiplier;
        let end = 1.0 + (depth_multiplier - 1.0) * pitch_env.sustain;
        let multiplier = start + (end - start) * t;
        curve.push(multiplier);
    }

    // Sustain phase: hold at sustain_level * depth_multiplier
    let sustain_multiplier = 1.0 + (depth_multiplier - 1.0) * pitch_env.sustain;
    for _ in 0..sustain_samples {
        curve.push(sustain_multiplier);
    }

    // Release phase: sustain_level * depth_multiplier -> 1.0
    for i in 0..release_samples {
        let t = i as f64 / release_samples.max(1) as f64;
        let start = sustain_multiplier;
        let multiplier = start + (1.0 - start) * t;
        curve.push(multiplier);
    }

    // Ensure exact length
    curve.resize(num_samples, 1.0);
    curve
}

/// Converts spec sweep curve to internal representation.
fn convert_sweep_curve(curve: &speccade_spec::recipe::audio_sfx::SweepCurve) -> SweepCurve {
    match curve {
        speccade_spec::recipe::audio_sfx::SweepCurve::Linear => SweepCurve::Linear,
        speccade_spec::recipe::audio_sfx::SweepCurve::Exponential => SweepCurve::Exponential,
        speccade_spec::recipe::audio_sfx::SweepCurve::Logarithmic => SweepCurve::Logarithmic,
    }
}

/// Converts spec noise type to internal representation.
fn convert_noise_type(noise_type: &NoiseType) -> NoiseColor {
    match noise_type {
        NoiseType::White => NoiseColor::White,
        NoiseType::Pink => NoiseColor::Pink,
        NoiseType::Brown => NoiseColor::Brown,
    }
}

/// Applies filter configuration to noise synthesizer.
fn apply_noise_filter(mut synth: NoiseSynth, filter: &Filter) -> NoiseSynth {
    match filter {
        Filter::Lowpass {
            cutoff, resonance, ..
        } => {
            synth = synth.with_lowpass(*cutoff, *resonance);
        }
        Filter::Highpass {
            cutoff, resonance, ..
        } => {
            synth = synth.with_highpass(*cutoff, *resonance);
        }
        Filter::Bandpass {
            center,
            bandwidth,
            resonance,
            ..
        } => {
            synth = synth.with_bandpass(*center, *bandwidth, *resonance);
        }
    }
    synth
}

/// Generates an ADSR envelope for the given duration.
fn generate_envelope(
    env: &speccade_spec::recipe::audio_sfx::Envelope,
    sample_rate: f64,
    num_samples: usize,
) -> Vec<f64> {
    let params = AdsrParams::new(env.attack, env.decay, env.sustain, env.release);
    let duration = num_samples as f64 / sample_rate;
    AdsrEnvelope::generate_fixed_duration(&params, sample_rate, duration)
}

/// Calculates the loop point based on the envelope.
///
/// The loop point is typically set after the attack phase, during the sustain.
fn calculate_loop_point(
    env: &speccade_spec::recipe::audio_sfx::Envelope,
    sample_rate: f64,
    _num_samples: usize,
) -> usize {
    // Loop point is at the end of attack + decay phases
    let loop_time = env.attack + env.decay;
    (loop_time * sample_rate) as usize
}

/// Converts a MIDI note number to frequency in Hz.
fn midi_to_frequency(midi_note: u8) -> f64 {
    440.0 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0)
}

/// Generates FM synthesis with multiple operators.
fn generate_fm_operators(
    operators: &[speccade_spec::recipe::audio_instrument::FmOperator],
    base_frequency: f64,
    num_samples: usize,
    sample_rate: f64,
    rng: &mut rand_pcg::Pcg32,
) -> AudioResult<Vec<f64>> {
    use crate::oscillator::{PhaseAccumulator, TWO_PI};

    if operators.is_empty() {
        return Ok(vec![0.0; num_samples]);
    }

    let mut output = vec![0.0; num_samples];

    // Generate each operator
    for op in operators {
        let op_frequency = base_frequency * op.ratio;
        let detune_mult = if op.detune != 0.0 {
            2.0_f64.powf(op.detune / 1200.0)
        } else {
            1.0
        };
        let actual_freq = op_frequency * detune_mult;

        // Generate operator envelope if specified
        let op_envelope = if let Some(ref env) = op.envelope {
            generate_envelope(env, sample_rate, num_samples)
        } else {
            vec![1.0; num_samples]
        };

        let mut phase_acc = PhaseAccumulator::new(sample_rate);

        for i in 0..num_samples {
            let phase = phase_acc.advance(actual_freq);

            let sample = match op.waveform {
                Waveform::Sine => crate::oscillator::sine(phase),
                Waveform::Square | Waveform::Pulse => crate::oscillator::square(phase, 0.5),
                Waveform::Sawtooth => crate::oscillator::sawtooth(phase),
                Waveform::Triangle => crate::oscillator::triangle(phase),
            };

            output[i] += sample * op.level * op_envelope[i];
        }
    }

    // Normalize by operator count
    let count = operators.len().max(1) as f64;
    for sample in &mut output {
        *sample /= count;
    }

    Ok(output)
}

/// Generates FM operators with pitch envelope modulation.
fn generate_fm_operators_with_pitch(
    operators: &[speccade_spec::recipe::audio_instrument::FmOperator],
    base_frequency: f64,
    pitch_curve: &[f64],
    num_samples: usize,
    sample_rate: f64,
    rng: &mut rand_pcg::Pcg32,
) -> AudioResult<Vec<f64>> {
    use crate::oscillator::{PhaseAccumulator, TWO_PI};

    if operators.is_empty() {
        return Ok(vec![0.0; num_samples]);
    }

    let mut output = vec![0.0; num_samples];

    // Generate each operator
    for op in operators {
        let detune_mult = if op.detune != 0.0 {
            2.0_f64.powf(op.detune / 1200.0)
        } else {
            1.0
        };

        // Generate operator envelope if specified
        let op_envelope = if let Some(ref env) = op.envelope {
            generate_envelope(env, sample_rate, num_samples)
        } else {
            vec![1.0; num_samples]
        };

        let mut phase_acc = PhaseAccumulator::new(sample_rate);

        for i in 0..num_samples {
            let modulated_base_freq = base_frequency * pitch_curve[i];
            let op_frequency = modulated_base_freq * op.ratio * detune_mult;
            let phase = phase_acc.advance(op_frequency);

            let sample = match op.waveform {
                Waveform::Sine => crate::oscillator::sine(phase),
                Waveform::Square | Waveform::Pulse => crate::oscillator::square(phase, 0.5),
                Waveform::Sawtooth => crate::oscillator::sawtooth(phase),
                Waveform::Triangle => crate::oscillator::triangle(phase),
            };

            output[i] += sample * op.level * op_envelope[i];
        }
    }

    // Normalize by operator count
    let count = operators.len().max(1) as f64;
    for sample in &mut output {
        *sample /= count;
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::audio_instrument::NoteSpec;
    use speccade_spec::recipe::audio_sfx::Envelope;

    #[test]
    fn test_midi_to_frequency() {
        // A4 = 440 Hz
        assert!((midi_to_frequency(69) - 440.0).abs() < 0.001);
        // C4 ~= 261.63 Hz
        assert!((midi_to_frequency(60) - 261.63).abs() < 0.1);
        // A3 = 220 Hz
        assert!((midi_to_frequency(57) - 220.0).abs() < 0.001);
    }

    #[test]
    fn test_generate_instrument_basic() {
        let params = AudioInstrumentSynthPatchV1Params {
            note_duration_seconds: 1.0,
            sample_rate: 44100,
            synthesis: InstrumentSynthesis::Simple { synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sine,
                frequency: 440.0, // Will be overridden by note
                freq_sweep: None,
                detune: None,
                duty: None,
            }},
            envelope: Envelope {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.7,
                release: 0.2,
            },
            pitch_envelope: None,
            notes: Some(vec![NoteSpec::MidiNote(69)]), // A4
            generate_loop_points: false,
        };

        let result = generate_instrument(&params, 42).expect("generation should succeed");

        assert_eq!(result.notes, vec![69]);
        assert_eq!(result.wav.sample_rate, 44100);
        assert!(!result.wav.is_stereo);
        assert!(result.wav.num_samples > 0);
        assert!(result.loop_point.is_none());
    }

    #[test]
    fn test_generate_instrument_with_loop_point() {
        let params = AudioInstrumentSynthPatchV1Params {
            note_duration_seconds: 1.0,
            sample_rate: 44100,
            synthesis: InstrumentSynthesis::Simple { synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sawtooth,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: None,
            }},
            envelope: Envelope {
                attack: 0.05,
                decay: 0.1,
                sustain: 0.7,
                release: 0.15,
            },
            pitch_envelope: None,
            notes: None, // Use default A4
            generate_loop_points: true,
        };

        let result = generate_instrument(&params, 42).expect("generation should succeed");

        assert!(result.loop_point.is_some());
        let loop_point = result.loop_point.unwrap();
        // Loop point should be after attack + decay
        let expected_loop = ((0.05 + 0.1) * 44100.0) as usize;
        assert_eq!(loop_point, expected_loop);
    }

    #[test]
    fn test_generate_instrument_note_name() {
        let params = AudioInstrumentSynthPatchV1Params {
            note_duration_seconds: 0.5,
            sample_rate: 22050,
            synthesis: InstrumentSynthesis::Simple { synthesis: Synthesis::Oscillator {
                waveform: Waveform::Square,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: None,
            }},
            envelope: Envelope::default(),
            pitch_envelope: None,
            notes: Some(vec![NoteSpec::NoteName("C4".to_string())]),
            generate_loop_points: false,
        };

        let result = generate_instrument(&params, 42).expect("generation should succeed");

        assert_eq!(result.notes, vec![60]); // C4 = MIDI 60
    }

    #[test]
    fn test_generate_instrument_determinism() {
        let params = AudioInstrumentSynthPatchV1Params {
            note_duration_seconds: 0.5,
            sample_rate: 44100,
            synthesis: InstrumentSynthesis::Simple { synthesis: Synthesis::KarplusStrong {
                frequency: 440.0,
                decay: 0.996,
                blend: 0.7,
            }},
            envelope: Envelope::default(),
            pitch_envelope: None,
            notes: Some(vec![NoteSpec::MidiNote(69)]),
            generate_loop_points: false,
        };

        let result1 = generate_instrument(&params, 42).expect("first generation");
        let result2 = generate_instrument(&params, 42).expect("second generation");

        assert_eq!(result1.wav.pcm_hash, result2.wav.pcm_hash);
    }

    #[test]
    fn test_calculate_loop_point() {
        let env = Envelope {
            attack: 0.1,
            decay: 0.2,
            sustain: 0.5,
            release: 0.2,
        };

        let loop_point = calculate_loop_point(&env, 44100.0, 44100);
        // Should be at 0.1 + 0.2 = 0.3 seconds = 13230 samples
        assert_eq!(loop_point, 13230);
    }

    #[test]
    fn test_generate_instrument_fm() {
        let params = AudioInstrumentSynthPatchV1Params {
            note_duration_seconds: 0.3,
            sample_rate: 44100,
            synthesis: InstrumentSynthesis::Simple { synthesis: Synthesis::FmSynth {
                carrier_freq: 440.0, // Will be overridden
                modulator_freq: 880.0,
                modulation_index: 2.0,
                freq_sweep: None,
            }},
            envelope: Envelope::default(),
            pitch_envelope: None,
            notes: Some(vec![NoteSpec::MidiNote(60)]), // C4
            generate_loop_points: false,
        };

        let result = generate_instrument(&params, 42).expect("generation should succeed");

        assert_eq!(result.notes, vec![60]);
        assert!(!result.wav.wav_data.is_empty());
    }

    #[test]
    fn test_generate_instrument_additive() {
        let params = AudioInstrumentSynthPatchV1Params {
            note_duration_seconds: 0.5,
            sample_rate: 44100,
            synthesis: InstrumentSynthesis::Simple { synthesis: Synthesis::Additive {
                base_freq: 440.0, // Will be overridden
                harmonics: vec![1.0, 0.5, 0.25, 0.125],
            }},
            envelope: Envelope::default(),
            pitch_envelope: None,
            notes: Some(vec![NoteSpec::MidiNote(69)]),
            generate_loop_points: false,
        };

        let result = generate_instrument(&params, 42).expect("generation should succeed");

        assert_eq!(result.notes, vec![69]);
        assert!(!result.wav.wav_data.is_empty());
    }
}
