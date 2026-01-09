//! Instrument sample generation.
//!
//! This module generates instrument samples from synth patch specifications.
//! Instruments differ from SFX in that they represent sustained musical notes
//! rather than short sound effects, and may include loop points for trackers.

use speccade_spec::recipe::audio_instrument::AudioInstrumentSynthPatchV1Params;

use crate::envelope::{AdsrEnvelope, AdsrParams};
use crate::error::{AudioError, AudioResult};
use crate::mixer;
use crate::rng::create_rng;
use crate::synthesis::fm::FmSynth;
use crate::synthesis::harmonics::HarmonicSynth;
use crate::synthesis::karplus::KarplusStrong;
use crate::synthesis::noise::{NoiseColor, NoiseSynth};
use crate::synthesis::oscillators::{SawSynth, SineSynth, SquareSynth, TriangleSynth};
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
    let midi_note = *notes.first().ok_or_else(|| {
        AudioError::invalid_param("notes", "No valid notes specified")
    })?;

    // Convert MIDI note to frequency
    let frequency = midi_to_frequency(midi_note);

    // Generate the instrument sample
    let mut rng = create_rng(seed);
    let mut samples = generate_synthesis(
        &params.synthesis,
        frequency,
        num_samples,
        sample_rate,
        &mut rng,
    )?;

    // Apply envelope
    let envelope = generate_envelope(&params.envelope, sample_rate, num_samples);
    for (sample, env) in samples.iter_mut().zip(envelope.iter()) {
        *sample *= env;
    }

    // Normalize
    mixer::normalize(&mut samples, -3.0);

    // Determine loop point if requested
    let loop_point = if params.generate_loop_points {
        Some(calculate_loop_point(&params.envelope, sample_rate, num_samples))
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
        } => {
            // Use the instrument frequency
            generate_oscillator_samples(waveform, frequency, freq_sweep.as_ref(), num_samples, sample_rate, rng)
        }
    };

    Ok(samples)
}

/// Generates oscillator samples based on waveform type.
fn generate_oscillator_samples(
    waveform: &Waveform,
    frequency: f64,
    freq_sweep: Option<&speccade_spec::recipe::audio_sfx::FreqSweep>,
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
                SineSynth::with_sweep(frequency, s.end_freq, s.curve).synthesize(num_samples, sample_rate, rng)
            } else {
                SineSynth::new(frequency).synthesize(num_samples, sample_rate, rng)
            }
        }
        Waveform::Square | Waveform::Pulse => {
            if let Some(s) = sweep {
                SquareSynth::with_sweep(frequency, s.end_freq, s.curve).synthesize(num_samples, sample_rate, rng)
            } else {
                SquareSynth::new(frequency).synthesize(num_samples, sample_rate, rng)
            }
        }
        Waveform::Sawtooth => {
            if let Some(s) = sweep {
                SawSynth::with_sweep(frequency, s.end_freq, s.curve).synthesize(num_samples, sample_rate, rng)
            } else {
                SawSynth::new(frequency).synthesize(num_samples, sample_rate, rng)
            }
        }
        Waveform::Triangle => {
            if let Some(s) = sweep {
                TriangleSynth::with_sweep(frequency, s.end_freq, s.curve).synthesize(num_samples, sample_rate, rng)
            } else {
                TriangleSynth::new(frequency).synthesize(num_samples, sample_rate, rng)
            }
        }
    }
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
        Filter::Lowpass { cutoff, resonance } => {
            synth = synth.with_lowpass(*cutoff, *resonance);
        }
        Filter::Highpass { cutoff, resonance } => {
            synth = synth.with_highpass(*cutoff, *resonance);
        }
        Filter::Bandpass {
            center,
            bandwidth,
            resonance,
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
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sine,
                frequency: 440.0, // Will be overridden by note
                freq_sweep: None,
            },
            envelope: Envelope {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.7,
                release: 0.2,
            },
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
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sawtooth,
                frequency: 440.0,
                freq_sweep: None,
            },
            envelope: Envelope {
                attack: 0.05,
                decay: 0.1,
                sustain: 0.7,
                release: 0.15,
            },
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
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Square,
                frequency: 440.0,
                freq_sweep: None,
            },
            envelope: Envelope::default(),
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
            synthesis: Synthesis::KarplusStrong {
                frequency: 440.0,
                decay: 0.996,
                blend: 0.7,
            },
            envelope: Envelope::default(),
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
            synthesis: Synthesis::FmSynth {
                carrier_freq: 440.0, // Will be overridden
                modulator_freq: 880.0,
                modulation_index: 2.0,
                freq_sweep: None,
            },
            envelope: Envelope::default(),
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
            synthesis: Synthesis::Additive {
                base_freq: 440.0, // Will be overridden
                harmonics: vec![1.0, 0.5, 0.25, 0.125],
            },
            envelope: Envelope::default(),
            notes: Some(vec![NoteSpec::MidiNote(69)]),
            generate_loop_points: false,
        };

        let result = generate_instrument(&params, 42).expect("generation should succeed");

        assert_eq!(result.notes, vec![69]);
        assert!(!result.wav.wav_data.is_empty());
    }
}
