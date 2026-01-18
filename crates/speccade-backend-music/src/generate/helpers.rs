//! Helper functions for music generation.
//!
//! This module contains utility functions for parsing, conversion, and legacy synthesis support.

use std::path::Path;

use speccade_spec::recipe::audio::{
    AudioLayer, AudioV1Params, NoiseType, NoteSpec as AudioNoteSpec, Synthesis as AudioSynthesis,
    Waveform,
};
use speccade_spec::recipe::music::{
    InstrumentSynthesis, PatternNote, TrackerFormat, TrackerInstrument,
};

use super::GenerateError;
use crate::note::{midi_to_freq, DEFAULT_IT_SYNTH_MIDI_NOTE, DEFAULT_SYNTH_MIDI_NOTE};

pub(super) fn neutralize_audio_layer_envelopes(params: &mut AudioV1Params) {
    for layer in &mut params.layers {
        layer.envelope.attack = 0.0;
        layer.envelope.decay = 0.0;
        layer.envelope.sustain = 1.0;
        layer.envelope.release = 0.0;
    }
}

pub(crate) fn parse_base_note_midi(
    instrument_base_note: Option<&str>,
    fallback_base_note: Option<&str>,
    fallback_midi: Option<u8>,
    default_base_midi: u8,
    instrument_name: &str,
) -> Result<u8, GenerateError> {
    let parse_name = |raw: &str| {
        let cleaned = raw.trim().replace('-', "");
        speccade_spec::recipe::audio::parse_note_name(&cleaned)
    };

    if let Some(note) = instrument_base_note.and_then(|s| {
        let s = s.trim();
        (!s.is_empty()).then_some(s)
    }) {
        return parse_name(note).ok_or_else(|| {
            GenerateError::InstrumentError(format!(
                "Instrument '{}' has invalid base_note '{}'",
                instrument_name, note
            ))
        });
    }

    if let Some(note) = fallback_base_note.and_then(|s| {
        let s = s.trim();
        (!s.is_empty()).then_some(s)
    }) {
        return parse_name(note).ok_or_else(|| {
            GenerateError::InstrumentError(format!(
                "Instrument '{}' has invalid base_note '{}'",
                instrument_name, note
            ))
        });
    }

    if let Some(midi) = fallback_midi {
        if midi <= 127 {
            return Ok(midi);
        }
        return Err(GenerateError::InstrumentError(format!(
            "Instrument '{}' has invalid base_note MIDI value {}",
            instrument_name, midi
        )));
    }

    Ok(default_base_midi)
}

pub(super) fn enforce_max_sample_len(
    pcm16_mono: &[u8],
    sample_rate: u32,
    max_seconds: f64,
    instrument_name: &str,
) -> Result<(), GenerateError> {
    let len_samples = pcm16_mono.len() / 2;
    let max_samples = (sample_rate as f64 * max_seconds).ceil() as usize;
    if len_samples > max_samples {
        return Err(GenerateError::InstrumentError(format!(
            "Instrument '{}' sample is too long ({} samples @ {} Hz). Max is {:.2}s.",
            instrument_name, len_samples, sample_rate, max_seconds
        )));
    }
    Ok(())
}

pub(crate) fn load_audio_v1_params_from_ref(
    ref_path: &str,
    spec_dir: &Path,
) -> Result<AudioV1Params, GenerateError> {
    use speccade_spec::Spec;

    let full_path = spec_dir.join(ref_path);
    let json_content = std::fs::read_to_string(&full_path).map_err(|e| {
        GenerateError::InstrumentError(format!(
            "Failed to read external instrument spec '{}': {}",
            ref_path, e
        ))
    })?;

    let spec: Spec = serde_json::from_str(&json_content).map_err(|e| {
        GenerateError::InstrumentError(format!(
            "Failed to parse external instrument spec '{}': {}",
            ref_path, e
        ))
    })?;

    if spec.asset_type != speccade_spec::AssetType::Audio {
        return Err(GenerateError::InstrumentError(format!(
            "External instrument spec '{}' has asset_type '{}' (expected 'audio')",
            ref_path, spec.asset_type
        )));
    }

    let recipe = spec.recipe.as_ref().ok_or_else(|| {
        GenerateError::InstrumentError(format!(
            "External instrument spec '{}' has no recipe",
            ref_path
        ))
    })?;

    if recipe.kind != "audio_v1" {
        return Err(GenerateError::InstrumentError(format!(
            "External instrument spec '{}' has unsupported recipe kind '{}', expected 'audio_v1'",
            ref_path, recipe.kind
        )));
    }

    serde_json::from_value(recipe.params.clone()).map_err(|e| {
        GenerateError::InstrumentError(format!(
            "Failed to parse audio_v1 params from '{}': {}",
            ref_path, e
        ))
    })
}

pub(super) fn legacy_synthesis_to_audio_v1_params(
    instr: &TrackerInstrument,
    synthesis: &InstrumentSynthesis,
    format: TrackerFormat,
) -> Result<AudioV1Params, GenerateError> {
    // Legacy synth is baked as a single-layer audio_v1 oscillator/noise.
    let default_base_midi = match format {
        TrackerFormat::Xm => DEFAULT_SYNTH_MIDI_NOTE,
        TrackerFormat::It => DEFAULT_IT_SYNTH_MIDI_NOTE,
    };
    let base_midi = parse_base_note_midi(
        instr.base_note.as_deref(),
        None,
        None,
        default_base_midi,
        &instr.name,
    )?;

    let sample_rate = instr
        .sample_rate
        .unwrap_or(crate::note::DEFAULT_SAMPLE_RATE);
    let want_loop = instr.envelope.sustain > 0.0;

    let sustain_pad = 0.25;
    let duration_seconds = if want_loop {
        (instr.envelope.attack + instr.envelope.decay + sustain_pad + instr.envelope.release)
            .clamp(0.05, 2.0)
    } else {
        (instr.envelope.attack + instr.envelope.decay + instr.envelope.release).clamp(0.02, 1.0)
    };

    let synthesis = match synthesis {
        InstrumentSynthesis::Pulse { duty_cycle } => AudioSynthesis::Oscillator {
            waveform: Waveform::Pulse,
            frequency: midi_to_freq(base_midi),
            freq_sweep: None,
            detune: None,
            duty: Some(*duty_cycle),
        },
        InstrumentSynthesis::Square => AudioSynthesis::Oscillator {
            waveform: Waveform::Square,
            frequency: midi_to_freq(base_midi),
            freq_sweep: None,
            detune: None,
            duty: None,
        },
        InstrumentSynthesis::Triangle => AudioSynthesis::Oscillator {
            waveform: Waveform::Triangle,
            frequency: midi_to_freq(base_midi),
            freq_sweep: None,
            detune: None,
            duty: None,
        },
        InstrumentSynthesis::Sawtooth => AudioSynthesis::Oscillator {
            waveform: Waveform::Sawtooth,
            frequency: midi_to_freq(base_midi),
            freq_sweep: None,
            detune: None,
            duty: None,
        },
        InstrumentSynthesis::Sine => AudioSynthesis::Oscillator {
            waveform: Waveform::Sine,
            frequency: midi_to_freq(base_midi),
            freq_sweep: None,
            detune: None,
            duty: None,
        },
        InstrumentSynthesis::Noise { .. } => AudioSynthesis::NoiseBurst {
            noise_type: NoiseType::White,
            filter: None,
        },
        InstrumentSynthesis::Sample { .. } => {
            return Err(GenerateError::InstrumentError(format!(
                "Instrument '{}' uses legacy 'synthesis: {{ type: sample }}' which should have been handled as wav",
                instr.name
            )));
        }
    };

    Ok(AudioV1Params {
        base_note: instr
            .base_note
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| AudioNoteSpec::NoteName(s.to_string())),
        duration_seconds,
        sample_rate,
        layers: vec![AudioLayer {
            synthesis,
            envelope: instr.envelope.clone(),
            volume: 1.0,
            pan: 0.0,
            delay: None,
            filter: None,
            lfo: None,
        }],
        pitch_envelope: None,
        loop_config: None,
        generate_loop_points: want_loop,
        master_filter: None,
        effects: vec![],
        post_fx_lfos: vec![],
    })
}

pub(super) fn downmix_pcm16_stereo_to_mono(pcm: &[u8]) -> Option<Vec<u8>> {
    if !pcm.len().is_multiple_of(4) {
        return None;
    }
    let mut out = Vec::with_capacity(pcm.len() / 2);

    for frame in pcm.chunks_exact(4) {
        let l = i16::from_le_bytes([frame[0], frame[1]]) as i32;
        let r = i16::from_le_bytes([frame[2], frame[3]]) as i32;
        let mixed = ((l + r) / 2) as i16;
        out.extend_from_slice(&mixed.to_le_bytes());
    }

    Some(out)
}

fn midi_to_note_name(midi: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let note = (midi % 12) as usize;
    let octave = (midi as i32 / 12) - 1;
    format!("{}{}", NAMES[note], octave)
}

/// Extract the base_note from an InstrumentSynthesis::Sample, if present.
///
/// Note: This is kept for backwards compatibility with Sample synthesis type
/// which has base_note inside the enum variant. For other synthesis types,
/// base_note is now at the TrackerInstrument level.
///
/// # Returns
/// - `Some(note_name)` if Sample synthesis has base_note set
/// - `None` otherwise
#[allow(dead_code)]
pub(crate) fn get_synthesis_base_note(synthesis: &InstrumentSynthesis) -> Option<&str> {
    // TODO: Once the spec schema adds base_note to all synthesis types, implement this:
    //
    // match synthesis {
    //     InstrumentSynthesis::Pulse { base_note, .. } => base_note.as_deref(),
    //     InstrumentSynthesis::Triangle { base_note, .. } => base_note.as_deref(),
    //     InstrumentSynthesis::Sawtooth { base_note, .. } => base_note.as_deref(),
    //     InstrumentSynthesis::Sine { base_note, .. } => base_note.as_deref(),
    //     InstrumentSynthesis::Noise { base_note, .. } => base_note.as_deref(),
    //     InstrumentSynthesis::Sample { base_note, .. } => base_note.as_deref(),
    // }
    //
    // For now, only Sample type has base_note in the schema:
    match synthesis {
        InstrumentSynthesis::Sample { base_note, .. } => base_note.as_deref(),
        _ => None, // Other synthesis types don't have base_note in schema yet
    }
}

/// Resolve an effective note name for a pattern event.
///
/// SpecCade treats an *omitted* / empty `note` field as "trigger the instrument at its base note".
/// That base note is sourced from (in priority order):
/// 1) `TrackerInstrument.base_note`
/// 2) `TrackerInstrument.synthesis.type == sample`'s `base_note`
/// 3) A format-specific default provided by the caller
///
/// Explicit `"---"` / `"..."` (no note), `"OFF"` / `"==="` (note off/cut), etc. are preserved
/// by returning the original `note` string when it is non-empty.
pub(crate) fn resolve_pattern_note_name<'a>(
    note: &'a PatternNote,
    instruments: &'a [TrackerInstrument],
    default_note: &'static str,
) -> Result<std::borrow::Cow<'a, str>, GenerateError> {
    use std::borrow::Cow;

    let trimmed = note.note.trim();
    if !trimmed.is_empty() {
        return Ok(Cow::Borrowed(trimmed));
    }

    let instrument = instruments.get(note.inst as usize).ok_or_else(|| {
        GenerateError::InvalidParameter(format!(
            "pattern references instrument {} but only {} instrument(s) are defined",
            note.inst,
            instruments.len()
        ))
    })?;

    if let Some(base_note) = instrument
        .base_note
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return Ok(Cow::Borrowed(base_note));
    }

    if let Some(InstrumentSynthesis::Sample {
        base_note: Some(base_note),
        ..
    }) = instrument.synthesis.as_ref()
    {
        let base_note = base_note.trim();
        if !base_note.is_empty() {
            return Ok(Cow::Borrowed(base_note));
        }
    }

    if let Some(AudioV1Params {
        base_note: Some(audio_base),
        ..
    }) = instrument.synthesis_audio_v1.as_ref()
    {
        match audio_base {
            AudioNoteSpec::NoteName(name) => {
                let name = name.trim();
                if !name.is_empty() {
                    return Ok(Cow::Borrowed(name));
                }
            }
            AudioNoteSpec::MidiNote(midi) => {
                return Ok(Cow::Owned(midi_to_note_name(*midi)));
            }
        }
    }

    Ok(Cow::Borrowed(default_note))
}

/// Extract effect code and parameter from PatternEffect, handling effect_xy nibbles.
///
/// This is used by both XM and IT pattern converters.
#[allow(dead_code)]
pub(crate) fn extract_effect_code_param<F>(
    effect: &speccade_spec::recipe::music::PatternEffect,
    name_to_code: F,
) -> Result<(Option<u8>, u8), GenerateError>
where
    F: Fn(&str) -> Option<u8>,
{
    // Calculate parameter from effect_xy nibbles if present
    let param = if let Some((x, y)) = effect.effect_xy {
        // Convert nibbles to byte: param = (x << 4) | y
        (x << 4) | y
    } else {
        effect.param.unwrap_or_default()
    };

    // Get effect code from type name
    let code = if let Some(ref type_name) = effect.r#type {
        name_to_code(type_name)
    } else {
        None
    };

    Ok((code, param))
}
