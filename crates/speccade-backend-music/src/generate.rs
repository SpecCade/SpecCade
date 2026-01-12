//! Main entry point for music generation from SpecCade specs.
//!
//! This module provides the public API for generating tracker modules (XM and IT)
//! from SpecCade `MusicTrackerSongV1Params` specifications.
//!
//! # Module Organization
//!
//! The generation logic is split into specialized modules:
//! - [`crate::envelope`] - Envelope conversion (ADSR to tracker format)
//! - [`crate::xm_gen`] - XM (FastTracker II) format generation
//! - [`crate::it_gen`] - IT (Impulse Tracker) format generation
//!
//! This module serves as the thin dispatcher and re-exports shared types.

use std::path::Path;

use speccade_spec::recipe::audio::{
    AudioLayer, AudioV1Params, NoiseType, NoteSpec as AudioNoteSpec, Synthesis as AudioSynthesis,
    Waveform,
};
use speccade_spec::recipe::music::{
    InstrumentSynthesis, MusicTrackerSongComposeV1Params, MusicTrackerSongV1Params, PatternNote,
    TrackerFormat, TrackerInstrument,
};
use speccade_spec::BackendError;
use thiserror::Error;

use crate::note::{midi_to_freq, DEFAULT_IT_SYNTH_MIDI_NOTE, DEFAULT_SYNTH_MIDI_NOTE};
use crate::synthesis::{derive_instrument_seed, load_wav_sample};
use crate::compose::expand_compose;

// Re-export format-specific generators (internal use)
pub(crate) use crate::it_gen::generate_it;
pub(crate) use crate::xm_gen::generate_xm;

/// Error type for music generation.
#[derive(Debug, Error)]
pub enum GenerateError {
    /// Invalid parameter value.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Pattern not found in spec.
    #[error("Pattern not found: {0}")]
    PatternNotFound(String),

    /// IO error during writing.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Unsupported synthesis type.
    #[error("Unsupported synthesis type: {0}")]
    UnsupportedSynthesis(String),

    /// Sample loading error.
    #[error("Sample loading error: {0}")]
    SampleLoadError(String),

    /// Instrument definition error.
    #[error("Instrument error: {0}")]
    InstrumentError(String),

    /// Automation error.
    #[error("Automation error: {0}")]
    AutomationError(String),

    /// Compose expansion error.
    #[error("Compose expansion error: {0}")]
    ComposeExpand(#[from] crate::compose::ExpandError),
}

impl BackendError for GenerateError {
    fn code(&self) -> &'static str {
        match self {
            GenerateError::InvalidParameter(_) => "MUSIC_001",
            GenerateError::PatternNotFound(_) => "MUSIC_002",
            GenerateError::IoError(_) => "MUSIC_003",
            GenerateError::UnsupportedSynthesis(_) => "MUSIC_004",
            GenerateError::SampleLoadError(_) => "MUSIC_005",
            GenerateError::InstrumentError(_) => "MUSIC_006",
            GenerateError::AutomationError(_) => "MUSIC_007",
            GenerateError::ComposeExpand(err) => err.code(),
        }
    }

    fn category(&self) -> &'static str {
        "music"
    }
}

/// Result of music generation.
pub struct GenerateResult {
    /// Generated tracker module bytes.
    pub data: Vec<u8>,
    /// BLAKE3 hash of the generated data.
    pub hash: String,
    /// File extension ("xm" or "it").
    pub extension: &'static str,
}

/// Generate a tracker module from a SpecCade music spec.
///
/// This is the main entry point for music generation. It dispatches to
/// format-specific generators based on the `format` field in params.
///
/// # Arguments
/// * `params` - Music tracker song parameters from the spec
/// * `seed` - Base seed for deterministic generation
/// * `spec_dir` - Directory containing the spec file (for resolving relative sample paths)
///
/// # Returns
/// `GenerateResult` containing the module bytes, hash, and extension
///
/// # Example
/// ```ignore
/// use speccade_backend_music::generate::generate_music;
/// use speccade_spec::recipe::music::MusicTrackerSongV1Params;
/// use std::path::Path;
///
/// let params = MusicTrackerSongV1Params { ... };
/// let spec_dir = Path::new("path/to/spec/dir");
/// let result = generate_music(&params, 42, spec_dir)?;
/// std::fs::write(format!("song.{}", result.extension), &result.data)?;
/// ```
pub fn generate_music(
    params: &MusicTrackerSongV1Params,
    seed: u32,
    spec_dir: &Path,
) -> Result<GenerateResult, GenerateError> {
    match params.format {
        TrackerFormat::Xm => generate_xm(params, seed, spec_dir),
        TrackerFormat::It => generate_it(params, seed, spec_dir),
    }
}

/// Generate a tracker module from compose (Pattern IR) params.
pub fn generate_music_compose(
    params: &MusicTrackerSongComposeV1Params,
    seed: u32,
    spec_dir: &Path,
) -> Result<GenerateResult, GenerateError> {
    let expanded = expand_compose(params, seed)?;
    generate_music(&expanded, seed, spec_dir)
}

// ============================================================================
// Shared utilities used by xm_gen and it_gen
// ============================================================================

/// Baked tracker sample data for a `TrackerInstrument`.
#[derive(Debug, Clone)]
pub(crate) struct BakedInstrumentSample {
    /// 16-bit mono PCM bytes (little-endian i16).
    pub pcm16_mono: Vec<u8>,
    /// Natural sample rate of the PCM data.
    pub sample_rate: u32,
    /// Base MIDI note the sample is tuned to.
    pub base_midi: u8,
    /// Optional loop points in samples: (loop_start, loop_end).
    pub loop_points: Option<(u32, u32)>,
}

fn neutralize_audio_layer_envelopes(params: &mut AudioV1Params) {
    for layer in &mut params.layers {
        layer.envelope.attack = 0.0;
        layer.envelope.decay = 0.0;
        layer.envelope.sustain = 1.0;
        layer.envelope.release = 0.0;
    }
}

/// Bake a tracker instrument into a mono sample.
///
/// Sources (exactly one):
/// - `wav`: load PCM from disk
/// - `ref`: load an external `audio_v1` spec and render via `speccade-backend-audio`
/// - `synthesis_audio_v1`: inline `audio_v1` params, rendered via `speccade-backend-audio`
/// - `synthesis` (legacy): mapped to `audio_v1` oscillator/noise then rendered
pub(crate) fn bake_instrument_sample(
    instr: &TrackerInstrument,
    base_seed: u32,
    index: u32,
    spec_dir: &Path,
    format: TrackerFormat,
) -> Result<BakedInstrumentSample, GenerateError> {
    const MAX_TRACKER_SAMPLE_SECONDS: f64 = 6.0;

    let instr_seed = derive_instrument_seed(base_seed, index);

    let mut sources = Vec::new();
    if instr.r#ref.is_some() {
        sources.push("ref");
    }
    if instr.wav.is_some() {
        sources.push("wav");
    }
    if instr.synthesis_audio_v1.is_some() {
        sources.push("synthesis_audio_v1");
    }
    if instr.synthesis.is_some() {
        sources.push("synthesis");
    }

    if sources.is_empty() {
        return Err(GenerateError::InstrumentError(format!(
            "Instrument '{}' must set exactly one of: ref, wav, synthesis_audio_v1, synthesis",
            instr.name
        )));
    }
    if sources.len() > 1 {
        return Err(GenerateError::InstrumentError(format!(
            "Instrument '{}' must set exactly one of: ref, wav, synthesis_audio_v1, synthesis (got: {})",
            instr.name,
            sources.join(", ")
        )));
    }

    // Resolve the base MIDI note for pitch mapping.
    let default_base_midi = match format {
        TrackerFormat::Xm => DEFAULT_SYNTH_MIDI_NOTE,
        TrackerFormat::It => DEFAULT_IT_SYNTH_MIDI_NOTE,
    };

    // Load PCM from WAV files directly.
    if let Some(ref wav_path) = instr.wav {
        let sample_path = spec_dir.join(wav_path);
        let (pcm16_mono, sample_rate) =
            load_wav_sample(&sample_path).map_err(GenerateError::SampleLoadError)?;

        let base_midi = parse_base_note_midi(
            instr.base_note.as_deref(),
            None,
            None,
            default_base_midi,
            &instr.name,
        )?;

        enforce_max_sample_len(
            &pcm16_mono,
            sample_rate,
            MAX_TRACKER_SAMPLE_SECONDS,
            &instr.name,
        )?;

        return Ok(BakedInstrumentSample {
            pcm16_mono,
            sample_rate,
            base_midi,
            loop_points: None,
        });
    }

    // Legacy `synthesis: { type: sample }` is equivalent to `wav`.
    if let Some(InstrumentSynthesis::Sample { path, base_note }) = instr.synthesis.as_ref() {
        let sample_path = spec_dir.join(path);
        let (pcm16_mono, sample_rate) =
            load_wav_sample(&sample_path).map_err(GenerateError::SampleLoadError)?;

        let base_midi = parse_base_note_midi(
            instr.base_note.as_deref(),
            base_note.as_deref(),
            None,
            default_base_midi,
            &instr.name,
        )?;

        enforce_max_sample_len(
            &pcm16_mono,
            sample_rate,
            MAX_TRACKER_SAMPLE_SECONDS,
            &instr.name,
        )?;

        return Ok(BakedInstrumentSample {
            pcm16_mono,
            sample_rate,
            base_midi,
            loop_points: None,
        });
    }

    // Everything else is baked via audio_v1 -> backend-audio.
    let mut audio_params = if let Some(ref ref_path) = instr.r#ref {
        load_audio_v1_params_from_ref(ref_path, spec_dir)?
    } else if let Some(ref params) = instr.synthesis_audio_v1 {
        params.clone()
    } else if let Some(ref legacy) = instr.synthesis {
        legacy_synthesis_to_audio_v1_params(instr, legacy, format)?
    } else {
        return Err(GenerateError::InstrumentError(format!(
            "Instrument '{}' must set exactly one of: ref, wav, synthesis_audio_v1, synthesis",
            instr.name
        )));
    };

    // Instrument-level overrides / precedence.
    if let Some(sample_rate) = instr.sample_rate {
        audio_params.sample_rate = sample_rate;
    }
    if let Some(base_note) = instr
        .base_note
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        audio_params.base_note = Some(AudioNoteSpec::NoteName(base_note.to_string()));
    }

    // Loop strategy is driven by the tracker envelope: sustained instruments loop, one-shots don't.
    let want_loop = instr.envelope.sustain > 0.0;

    // Tracker envelopes also apply at playback time in XM/IT. If we baked audio_v1 layer
    // envelopes into the sample, we'd effectively apply an amplitude envelope twice (once in
    // the baked PCM, and again in the tracker). This is especially problematic for short,
    // percussive envelopes where tick quantization can collapse points and mute the sound.
    //
    // Rule: for music instruments, tracker envelopes are authoritative; audio_v1 layer envelopes
    // are neutralized when baking samples.
    neutralize_audio_layer_envelopes(&mut audio_params);

    // Loop points are derived from the tracker envelope (not the audio_v1 layer envelope).
    audio_params.generate_loop_points = false;

    // Render via unified audio backend (deterministic by seed).
    let gen =
        speccade_backend_audio::generate_from_params(&audio_params, instr_seed).map_err(|e| {
            GenerateError::InstrumentError(format!(
                "Failed to bake audio_v1 instrument '{}' to tracker sample: {}",
                instr.name, e
            ))
        })?;

    let pcm =
        speccade_backend_audio::wav::extract_pcm_data(&gen.wav.wav_data).ok_or_else(|| {
            GenerateError::InstrumentError(format!(
                "audio_v1 backend returned an invalid WAV buffer for instrument '{}'",
                instr.name
            ))
        })?;

    let pcm16_mono = if gen.wav.is_stereo {
        downmix_pcm16_stereo_to_mono(pcm).ok_or_else(|| {
            GenerateError::InstrumentError(format!(
                "audio_v1 backend returned invalid stereo PCM for instrument '{}'",
                instr.name
            ))
        })?
    } else {
        pcm.to_vec()
    };

    // Safety: cap sample length to keep module sizes reasonable.
    enforce_max_sample_len(
        &pcm16_mono,
        gen.wav.sample_rate,
        MAX_TRACKER_SAMPLE_SECONDS,
        &instr.name,
    )?;

    // Resolve base_midi for pitch mapping (tracker-level base_note overrides audio_v1 base_note).
    let audio_base_note = audio_params.base_note.as_ref();
    let (audio_base_note_str, audio_base_note_midi) = match audio_base_note {
        Some(AudioNoteSpec::NoteName(s)) => (Some(s.as_str()), None),
        Some(AudioNoteSpec::MidiNote(n)) => (None, Some(*n)),
        None => (None, None),
    };
    let base_midi = parse_base_note_midi(
        instr.base_note.as_deref(),
        audio_base_note_str,
        audio_base_note_midi,
        default_base_midi,
        &instr.name,
    )?;

    let sample_rate = gen.wav.sample_rate;

    // Loop points: loop from tracker attack+decay to the end of the sample.
    //
    // Trackers will keep looping the sample even after note-off; the instrument volume envelope
    // handles the release phase by fading volume down.
    let loop_points = if want_loop {
        let sample_len = (pcm16_mono.len() / 2) as u32;
        if sample_len < 2 {
            return Err(GenerateError::InstrumentError(format!(
                "Instrument '{}' sample is too short to loop ({} samples)",
                instr.name, sample_len
            )));
        }

        let loop_start = ((instr.envelope.attack + instr.envelope.decay) * sample_rate as f64)
            .round()
            .clamp(0.0, (sample_len - 1) as f64) as u32;
        let loop_end = sample_len;
        if loop_start + 1 > loop_end {
            return Err(GenerateError::InstrumentError(format!(
                "Instrument '{}' has invalid loop region: start={} end={}",
                instr.name, loop_start, loop_end,
            )));
        }

        Some((loop_start, loop_end))
    } else {
        None
    };

    Ok(BakedInstrumentSample {
        pcm16_mono,
        sample_rate,
        base_midi,
        loop_points,
    })
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

fn parse_base_note_midi(
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

fn enforce_max_sample_len(
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

fn load_audio_v1_params_from_ref(
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

fn legacy_synthesis_to_audio_v1_params(
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
        }],
        pitch_envelope: None,
        generate_loop_points: want_loop,
        master_filter: None,
    })
}

fn downmix_pcm16_stereo_to_mono(pcm: &[u8]) -> Option<Vec<u8>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::audio::Envelope;
    use speccade_spec::recipe::music::{ArrangementEntry, PatternNote, TrackerPattern};
    use std::collections::HashMap;

    fn create_test_params() -> MusicTrackerSongV1Params {
        let envelope = Envelope {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
        };

        let instrument = TrackerInstrument {
            name: "Test Lead".to_string(),
            synthesis: Some(InstrumentSynthesis::Pulse { duty_cycle: 0.5 }),
            envelope: envelope.clone(),
            default_volume: Some(64),
            ..Default::default()
        };

        let mut notes = HashMap::new();
        notes.insert(
            "0".to_string(),
            vec![
                PatternNote {
                    row: 0,
                    note: "C4".to_string(),
                    inst: 0,
                    vol: Some(64),
                    ..Default::default()
                },
                PatternNote {
                    row: 4,
                    note: "E4".to_string(),
                    inst: 0,
                    vol: Some(64),
                    ..Default::default()
                },
                PatternNote {
                    row: 8,
                    note: "G4".to_string(),
                    inst: 0,
                    vol: Some(64),
                    ..Default::default()
                },
                PatternNote {
                    row: 12,
                    note: "OFF".to_string(),
                    inst: 0,
                    ..Default::default()
                },
            ],
        );
        let pattern = TrackerPattern {
            rows: 16,
            notes: Some(notes),
            data: None,
        };

        let mut patterns = HashMap::new();
        patterns.insert("intro".to_string(), pattern);

        MusicTrackerSongV1Params {
            format: TrackerFormat::Xm,
            bpm: 120,
            speed: 6,
            channels: 4,
            r#loop: true,
            instruments: vec![instrument],
            patterns,
            arrangement: vec![ArrangementEntry {
                pattern: "intro".to_string(),
                repeat: 1,
            }],
            ..Default::default()
        }
    }

    #[test]
    fn test_generate_xm() {
        let params = create_test_params();
        let spec_dir = Path::new(".");
        let result = generate_music(&params, 42, spec_dir).unwrap();

        assert_eq!(result.extension, "xm");
        assert!(!result.data.is_empty());
        assert_eq!(result.hash.len(), 64);
    }

    #[test]
    fn test_generate_it() {
        let mut params = create_test_params();
        params.format = TrackerFormat::It;

        let spec_dir = Path::new(".");
        let result = generate_music(&params, 42, spec_dir).unwrap();

        assert_eq!(result.extension, "it");
        assert!(!result.data.is_empty());
        assert_eq!(result.hash.len(), 64);
    }

    #[test]
    fn test_determinism() {
        let params = create_test_params();
        let spec_dir = Path::new(".");

        let result1 = generate_music(&params, 42, spec_dir).unwrap();
        let result2 = generate_music(&params, 42, spec_dir).unwrap();

        assert_eq!(result1.hash, result2.hash);
        assert_eq!(result1.data, result2.data);
    }

    #[test]
    fn test_different_seeds_different_output() {
        // Use noise synthesis which uses the seed
        let mut params = create_test_params();
        params.instruments[0].synthesis = Some(InstrumentSynthesis::Noise { periodic: false });

        let spec_dir = Path::new(".");
        let result1 = generate_music(&params, 42, spec_dir).unwrap();
        let result2 = generate_music(&params, 43, spec_dir).unwrap();

        // Different seeds should produce different hashes (due to noise synthesis)
        assert_ne!(result1.hash, result2.hash);
    }

    #[test]
    fn test_different_seeds_same_output_for_pure_oscillator() {
        // Oscillators are deterministic and should not use RNG; seeds should not affect output.
        let params = create_test_params();
        let spec_dir = Path::new(".");

        let result1 = generate_music(&params, 42, spec_dir).unwrap();
        let result2 = generate_music(&params, 43, spec_dir).unwrap();

        assert_eq!(result1.hash, result2.hash);
        assert_eq!(result1.data, result2.data);
    }

    #[test]
    fn test_invalid_channels() {
        let mut params = create_test_params();
        params.channels = 100; // Invalid for XM (max 32)

        let spec_dir = Path::new(".");
        let result = generate_music(&params, 42, spec_dir);
        assert!(result.is_err());
    }

    // =========================================================================
    // External instrument reference tests
    // =========================================================================

    #[test]
    fn test_external_ref_file_not_found() {
        // Test that missing external reference returns a clear error
        let instr = TrackerInstrument {
            name: "Missing".to_string(),
            r#ref: Some("nonexistent/file.json".to_string()),
            ..Default::default()
        };

        let err =
            bake_instrument_sample(&instr, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap_err();
        assert!(err
            .to_string()
            .contains("Failed to read external instrument spec"));
    }

    #[test]
    fn test_bake_instrument_sample_rejects_multiple_sources() {
        let instr = TrackerInstrument {
            name: "Bad".to_string(),
            wav: Some("samples/kick.wav".to_string()),
            synthesis: Some(InstrumentSynthesis::Sine),
            ..Default::default()
        };

        let err =
            bake_instrument_sample(&instr, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap_err();
        assert!(err.to_string().contains("exactly one of"));
    }

    #[test]
    fn test_bake_instrument_sample_from_ref_supports_advanced_audio_v1() {
        let spec_dir =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../golden/speccade/specs/music");

        let instr = TrackerInstrument {
            name: "FM Ref".to_string(),
            r#ref: Some("../audio/audio_instrument_fm_advanced.spec.json".to_string()),
            // Sustain > 0 => loopable sample.
            envelope: Envelope {
                attack: 0.01,
                decay: 0.05,
                sustain: 0.7,
                release: 0.2,
            },
            ..Default::default()
        };

        let baked = bake_instrument_sample(&instr, 42, 0, &spec_dir, TrackerFormat::Xm).unwrap();
        assert!(!baked.pcm16_mono.is_empty());
        assert!(
            baked.loop_points.is_some(),
            "sustained instruments should loop"
        );
    }

    #[test]
    fn test_bake_instrument_sample_inline_audio_v1_downmixes_stereo() {
        let instr = TrackerInstrument {
            name: "Stereo Inline".to_string(),
            synthesis_audio_v1: Some(AudioV1Params {
                base_note: Some(AudioNoteSpec::NoteName("A4".to_string())),
                duration_seconds: 0.1,
                sample_rate: 22050,
                layers: vec![
                    AudioLayer {
                        synthesis: AudioSynthesis::Oscillator {
                            waveform: Waveform::Sine,
                            frequency: 440.0,
                            freq_sweep: None,
                            detune: None,
                            duty: None,
                        },
                        envelope: Envelope::default(),
                        volume: 0.5,
                        pan: -1.0,
                        delay: None,
                    },
                    AudioLayer {
                        synthesis: AudioSynthesis::Oscillator {
                            waveform: Waveform::Sine,
                            frequency: 440.0,
                            freq_sweep: None,
                            detune: None,
                            duty: None,
                        },
                        envelope: Envelope::default(),
                        volume: 0.5,
                        pan: 1.0,
                        delay: None,
                    },
                ],
                pitch_envelope: None,
                generate_loop_points: false,
                master_filter: None,
            }),
            envelope: Envelope {
                attack: 0.01,
                decay: 0.05,
                sustain: 0.0,
                release: 0.05,
            },
            ..Default::default()
        };

        let baked =
            bake_instrument_sample(&instr, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap();
        assert!(!baked.pcm16_mono.is_empty());

        // Must be mono 16-bit PCM: duration 0.1s @ 22050 Hz => 2205 samples => 4410 bytes.
        assert_eq!(baked.pcm16_mono.len(), 2205 * 2);
    }

    #[test]
    fn test_loop_policy_uses_tracker_envelope_sustain() {
        let audio = AudioV1Params {
            base_note: Some(AudioNoteSpec::NoteName("A4".to_string())),
            duration_seconds: 0.2,
            sample_rate: 22050,
            layers: vec![AudioLayer {
                synthesis: AudioSynthesis::Oscillator {
                    waveform: Waveform::Sine,
                    frequency: 440.0,
                    freq_sweep: None,
                    detune: None,
                    duty: None,
                },
                envelope: Envelope::default(),
                volume: 1.0,
                pan: 0.0,
                delay: None,
            }],
            pitch_envelope: None,
            // Intentionally opposite of the tracker envelope tests below.
            generate_loop_points: true,
            master_filter: None,
        };

        let one_shot = TrackerInstrument {
            name: "One Shot".to_string(),
            synthesis_audio_v1: Some(audio.clone()),
            envelope: Envelope {
                attack: 0.01,
                decay: 0.05,
                sustain: 0.0,
                release: 0.05,
            },
            ..Default::default()
        };
        let baked =
            bake_instrument_sample(&one_shot, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap();
        assert!(baked.loop_points.is_none());

        let sustained = TrackerInstrument {
            name: "Sustain".to_string(),
            synthesis_audio_v1: Some(AudioV1Params {
                generate_loop_points: false,
                ..audio
            }),
            envelope: Envelope {
                attack: 0.01,
                decay: 0.05,
                sustain: 0.7,
                release: 0.2,
            },
            ..Default::default()
        };
        let baked =
            bake_instrument_sample(&sustained, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap();
        assert!(baked.loop_points.is_some());
    }

    #[test]
    fn test_audio_v1_base_note_midi_note_is_used_for_pitch_mapping() {
        let instr = TrackerInstrument {
            name: "Midi Base".to_string(),
            synthesis_audio_v1: Some(AudioV1Params {
                base_note: Some(AudioNoteSpec::MidiNote(69)),
                duration_seconds: 0.1,
                sample_rate: 22050,
                layers: vec![AudioLayer {
                    synthesis: AudioSynthesis::Oscillator {
                        waveform: Waveform::Sine,
                        frequency: 440.0,
                        freq_sweep: None,
                        detune: None,
                        duty: None,
                    },
                    envelope: Envelope::default(),
                    volume: 1.0,
                    pan: 0.0,
                    delay: None,
                }],
                pitch_envelope: None,
                generate_loop_points: false,
                master_filter: None,
            }),
            envelope: Envelope {
                attack: 0.01,
                decay: 0.05,
                sustain: 0.0,
                release: 0.05,
            },
            ..Default::default()
        };

        let baked =
            bake_instrument_sample(&instr, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap();
        assert_eq!(baked.base_midi, 69);
    }
}
