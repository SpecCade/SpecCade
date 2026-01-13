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

use serde::Serialize;
use speccade_spec::recipe::audio::{
    AudioLayer, AudioV1Params, NoiseType, NoteSpec as AudioNoteSpec, Synthesis as AudioSynthesis,
    Waveform,
};
use speccade_spec::recipe::music::{
    InstrumentSynthesis, MusicTrackerSongComposeV1Params, MusicTrackerSongV1Params, PatternNote,
    TrackerFormat, TrackerInstrument, TrackerLoopMode,
};
use speccade_spec::BackendError;
use thiserror::Error;

use crate::compose::expand_compose;
use crate::note::{midi_to_freq, DEFAULT_IT_SYNTH_MIDI_NOTE, DEFAULT_SYNTH_MIDI_NOTE};
use crate::synthesis::{derive_instrument_seed, load_wav_sample};

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

/// Loop mode actually used for an instrument sample in the generated module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChosenLoopMode {
    None,
    Forward,
    PingPong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopSeamMetrics {
    /// Absolute amplitude delta at the loop boundary.
    pub amp_jump: u32,
    /// Absolute delta between end/start slopes at the loop boundary.
    pub slope_jump: u32,
}

/// Loop diagnostics for a single tracker instrument.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MusicInstrumentLoopReport {
    pub index: u32,
    pub name: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub override_mode: Option<TrackerLoopMode>,

    pub chosen_mode: ChosenLoopMode,

    pub base_midi: u8,
    pub base_freq_hz: f64,
    pub sample_rate: u32,
    pub sample_len: u32,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desired_start: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_start: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_end: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_len: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forward_corr: Option<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crossfade_samples: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crossfade_ms: Option<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seam: Option<LoopSeamMetrics>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pingpong_start_slope: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pingpong_end_slope: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dc_removed_mean: Option<i64>,
}

/// Loop diagnostics for the generated tracker module.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MusicLoopReport {
    /// The generated module format ("xm" or "it").
    pub extension: String,
    pub instruments: Vec<MusicInstrumentLoopReport>,
}

/// Result of music generation.
pub struct GenerateResult {
    /// Generated tracker module bytes.
    pub data: Vec<u8>,
    /// BLAKE3 hash of the generated data.
    pub hash: String,
    /// File extension ("xm" or "it").
    pub extension: &'static str,
    /// Optional loop diagnostics, intended for developer workflows.
    pub loop_report: Option<MusicLoopReport>,
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
    /// Optional loop region for sustained instruments.
    pub loop_region: Option<LoopRegion>,
}

/// Sample loop mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum LoopMode {
    Forward,
    PingPong,
}

/// Loop points for tracker samples.
///
/// In both XM and IT, loop end is stored as an absolute index (not a length) here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LoopRegion {
    pub start: u32,
    pub end: u32,
    pub mode: LoopMode,
}

fn neutralize_audio_layer_envelopes(params: &mut AudioV1Params) {
    for layer in &mut params.layers {
        layer.envelope.attack = 0.0;
        layer.envelope.decay = 0.0;
        layer.envelope.sustain = 1.0;
        layer.envelope.release = 0.0;
    }
}

fn pcm16_mono_to_i16(pcm16_mono: &[u8]) -> Result<Vec<i16>, GenerateError> {
    if !pcm16_mono.len().is_multiple_of(2) {
        return Err(GenerateError::InstrumentError(
            "invalid PCM16 buffer: length is not a multiple of 2".to_string(),
        ));
    }
    let mut out = Vec::with_capacity(pcm16_mono.len() / 2);
    for chunk in pcm16_mono.chunks_exact(2) {
        out.push(i16::from_le_bytes([chunk[0], chunk[1]]));
    }
    Ok(out)
}

fn i16_to_pcm16_mono(samples: &[i16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * 2);
    for &sample in samples {
        out.extend_from_slice(&sample.to_le_bytes());
    }
    out
}

fn remove_dc_offset_i16_in_place(samples: &mut [i16]) -> i64 {
    if samples.is_empty() {
        return 0;
    }

    let sum: i64 = samples.iter().map(|&s| s as i64).sum();
    let mean = sum / samples.len() as i64;

    // Avoid churn from tiny rounding offsets.
    if mean.abs() < 2 {
        return 0;
    }

    for sample in samples.iter_mut() {
        let v = (*sample as i64) - mean;
        *sample = v.clamp(i16::MIN as i64, i16::MAX as i64) as i16;
    }

    mean
}

fn correlation_i16_stride(
    samples: &[i16],
    a_start: usize,
    b_start: usize,
    len: usize,
    stride: usize,
) -> Option<f64> {
    if len == 0 || stride == 0 {
        return None;
    }
    if a_start.checked_add(len)? > samples.len() || b_start.checked_add(len)? > samples.len() {
        return None;
    }

    let mut dot: i128 = 0;
    let mut norm_a: i128 = 0;
    let mut norm_b: i128 = 0;

    for i in (0..len).step_by(stride) {
        let a = samples[a_start + i] as i128;
        let b = samples[b_start + i] as i128;
        dot += a * b;
        norm_a += a * a;
        norm_b += b * b;
    }

    if norm_a == 0 || norm_b == 0 {
        return None;
    }

    let denom = (norm_a as f64 * norm_b as f64).sqrt();
    if denom == 0.0 {
        return None;
    }

    Some(dot as f64 / denom)
}

#[derive(Debug, Clone, Copy)]
struct ForwardLoopCandidate {
    start: usize,
    end_exclusive: usize,
    corr: f64,
}

fn find_best_forward_loop_candidate(
    samples: &[i16],
    desired_start: usize,
    radius: usize,
    sample_rate: u32,
    samples_per_cycle: u32,
    min_loop_len: usize,
) -> Option<ForwardLoopCandidate> {
    if samples.len() < 4 {
        return None;
    }

    let start_min = desired_start.saturating_sub(radius).max(1);
    let start_max = (desired_start + radius)
        .min(samples.len().saturating_sub(min_loop_len + 2))
        .max(start_min);

    let start_step = (samples_per_cycle / 8).clamp(16, 128) as usize;
    let end_step_coarse = (samples_per_cycle / 8).clamp(16, 256) as usize;
    let end_refine_radius = end_step_coarse.saturating_mul(2).max(64);

    let mut best: Option<ForwardLoopCandidate> = None;
    let mut best_score: f64 = f64::NEG_INFINITY;

    for start in (start_min..=start_max).step_by(start_step) {
        // Use a long-ish match window so slow beating phase gets captured. Cap at 1s.
        let remaining = samples.len().saturating_sub(start);
        // Important: keep this strictly smaller than half the remaining audio so we can compare
        // two non-overlapping windows. Otherwise any signal (including noise) can "match" by
        // overlapping the same samples, producing a false high correlation.
        let match_len = (remaining / 3).min(sample_rate as usize);
        if match_len < 1024 {
            continue;
        }

        // Downsampled correlation stride to keep this cheap, scaled by base period.
        let stride = (samples_per_cycle / 16).clamp(8, 128) as usize;

        // Earliest end that still allows an end-window aligned to the loop start window.
        let end_min = start
            .saturating_add(min_loop_len)
            .max(start.saturating_add(match_len.saturating_mul(2)))
            .min(samples.len());

        if end_min >= samples.len() {
            continue;
        }

        // Coarse scan.
        let mut best_end: Option<(usize, f64)> = None;
        let end_max = samples.len();
        for end_exclusive in (end_min..=end_max).step_by(end_step_coarse) {
            if end_exclusive < match_len {
                continue;
            }
            let b_start = end_exclusive.saturating_sub(match_len);
            if b_start < start.saturating_add(match_len) {
                continue;
            }
            let Some(corr) = correlation_i16_stride(samples, start, b_start, match_len, stride)
            else {
                continue;
            };

            // Prefer longer loops slightly, but correlation dominates.
            let loop_len = end_exclusive.saturating_sub(start) as f64;
            let score = corr + (loop_len / sample_rate as f64) * 0.01;

            if score > best_end.map(|(_, s)| s).unwrap_or(f64::NEG_INFINITY) {
                best_end = Some((end_exclusive, score));
            }
        }

        let Some((coarse_end, _)) = best_end else {
            continue;
        };

        // Refine around coarse best.
        let refine_start = coarse_end.saturating_sub(end_refine_radius).max(end_min);
        let refine_end = (coarse_end + end_refine_radius).min(samples.len());

        let mut refined_best: Option<ForwardLoopCandidate> = None;
        let mut refined_score: f64 = f64::NEG_INFINITY;
        for end_exclusive in (refine_start..=refine_end).step_by(4) {
            if end_exclusive < match_len {
                continue;
            }
            let b_start = end_exclusive.saturating_sub(match_len);
            if b_start < start.saturating_add(match_len) {
                continue;
            }
            let Some(corr) = correlation_i16_stride(samples, start, b_start, match_len, stride)
            else {
                continue;
            };

            let loop_len = end_exclusive.saturating_sub(start) as f64;
            let score = corr + (loop_len / sample_rate as f64) * 0.01;

            if score > refined_score {
                refined_score = score;
                refined_best = Some(ForwardLoopCandidate {
                    start,
                    end_exclusive,
                    corr,
                });
            }
        }

        let Some(candidate) = refined_best else {
            continue;
        };

        let loop_len = candidate.end_exclusive.saturating_sub(candidate.start);
        if loop_len < min_loop_len + 2 {
            continue;
        }

        if refined_score > best_score {
            best_score = refined_score;
            best = Some(candidate);
        }
    }

    best
}

fn find_best_pingpong_loop_start_near(samples: &[i16], target: usize, radius: usize) -> usize {
    if samples.len() < 2 {
        return target.min(samples.len().saturating_sub(1));
    }

    // For ping-pong loops, the start boundary turn-around goes:
    //   ... start+2, start+1, start, start+1, start+2 ...
    // so we want `samples[start]` and `samples[start+1]` to be as close as possible.
    let start = target.saturating_sub(radius);
    let end = (target + radius).min(samples.len().saturating_sub(2));

    let slope_weight: i64 = 16;

    let mut best = target.min(samples.len().saturating_sub(2));
    let mut best_score: i64 = i64::MAX;
    for i in start..=end {
        let a = samples[i] as i32;
        let b = samples[i + 1] as i32;
        let forward_slope = (b - a).abs() as i64;
        let dist = (i as i64 - target as i64).abs();
        let score = forward_slope * slope_weight + dist;
        if score < best_score {
            best = i;
            best_score = score;
        }
    }

    best
}

fn find_best_pingpong_loop_end_in_range(
    samples: &[i16],
    search_start: usize,
    search_end_inclusive: usize,
) -> Option<usize> {
    if samples.len() < 2 || search_start > search_end_inclusive {
        return None;
    }

    // For ping-pong loops, the end boundary turn-around goes:
    //   ... end-3, end-2, end-1, end-2, end-3 ...
    // so we want `samples[end-1]` and `samples[end-2]` to be as close as possible.
    //
    // Note: we treat `end_idx` as the last *included* sample in the loop (XM/IT store end as an
    // absolute index/length, so callers convert `end_idx` to `loop_end = end_idx + 1`).
    let start = search_start.max(1);
    let end = search_end_inclusive.min(samples.len().saturating_sub(1));
    if start > end {
        return None;
    }

    let slope_weight: i64 = 16;

    let mut best: usize = end;
    let mut best_score: i64 = i64::MAX;
    for i in start..=end {
        let a = samples[i - 1] as i32;
        let b = samples[i] as i32;
        let backward_slope = (b - a).abs() as i64;
        let dist_to_end = (end - i) as i64;
        let score = backward_slope * slope_weight + dist_to_end;
        if score < best_score {
            best = i;
            best_score = score;
        }
    }

    Some(best)
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
) -> Result<(BakedInstrumentSample, MusicInstrumentLoopReport), GenerateError> {
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

    let override_mode = instr.loop_mode;
    let want_loop = match override_mode {
        Some(TrackerLoopMode::None) => false,
        Some(TrackerLoopMode::Forward) | Some(TrackerLoopMode::PingPong) => true,
        Some(TrackerLoopMode::Auto) | None => instr.envelope.sustain > 0.0,
    };

    let (mut pcm16_mono, sample_rate, base_midi) = if let Some(ref wav_path) = instr.wav {
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

        (pcm16_mono, sample_rate, base_midi)
    } else if let Some(InstrumentSynthesis::Sample { path, base_note }) = instr.synthesis.as_ref() {
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

        (pcm16_mono, sample_rate, base_midi)
    } else {
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
        let gen = speccade_backend_audio::generate_from_params(&audio_params, instr_seed).map_err(
            |e| {
                GenerateError::InstrumentError(format!(
                    "Failed to bake audio_v1 instrument '{}' to tracker sample: {}",
                    instr.name, e
                ))
            },
        )?;

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

        (pcm16_mono, gen.wav.sample_rate, base_midi)
    };

    // Safety: cap sample length to keep module sizes reasonable.
    enforce_max_sample_len(
        &pcm16_mono,
        sample_rate,
        MAX_TRACKER_SAMPLE_SECONDS,
        &instr.name,
    )?;

    let base_freq_hz = midi_to_freq(base_midi);

    let mut desired_start_report: Option<u32> = None;
    let mut forward_corr_report: Option<f64> = None;
    let mut crossfade_samples_report: Option<u32> = None;
    let mut crossfade_ms_report: Option<f64> = None;
    let mut seam_report: Option<LoopSeamMetrics> = None;
    let mut pingpong_start_slope_report: Option<u32> = None;
    let mut pingpong_end_slope_report: Option<u32> = None;
    let mut dc_removed_mean_report: Option<i64> = None;

    // Loop points: sustained instruments loop by default. Trackers do not crossfade loop
    // boundaries, so we need to generate loop regions that are click-resistant by construction.
    //
    // Strategy:
    // - Forward: pick a self-similar region, bake a crossfade into the tail, and shift the loop
    //   start forward by the crossfade length so the wrap boundary lands on adjacent samples.
    // - Ping-pong: pick turn-around points with minimal local slope (no end->start jump).
    let loop_region = if want_loop {
        const FORWARD_LOOP_MIN_CORR: f64 = 0.85;

        let mut samples = pcm16_mono_to_i16(&pcm16_mono)?;
        let dc_mean = remove_dc_offset_i16_in_place(&mut samples);
        if dc_mean != 0 {
            dc_removed_mean_report = Some(dc_mean);
        }

        let sample_len = samples.len() as u32;
        if sample_len < 4 {
            return Err(GenerateError::InstrumentError(format!(
                "Instrument '{}' sample is too short to loop ({} samples)",
                instr.name, sample_len
            )));
        }

        let desired_start = ((instr.envelope.attack + instr.envelope.decay) * sample_rate as f64)
            .round()
            .clamp(0.0, (sample_len - 1) as f64) as u32;
        desired_start_report = Some(desired_start);

        let samples_per_cycle = (sample_rate as f64 / base_freq_hz).round().max(1.0) as u32;
        // Search at least ~20ms, but also at least half a waveform cycle for very low base notes.
        let radius = (sample_rate / 50).max(samples_per_cycle / 2).max(64) as usize;

        let min_loop_len = (sample_rate / 20).max(256) as usize; // ~50ms, minimum 256 samples

        // Candidate forward loop (if enabled). We always compute this in auto-mode so the report
        // can explain why forward was rejected.
        let forward_raw = if matches!(override_mode, Some(TrackerLoopMode::PingPong)) {
            None
        } else {
            find_best_forward_loop_candidate(
                &samples,
                desired_start as usize,
                radius,
                sample_rate,
                samples_per_cycle,
                min_loop_len,
            )
        };
        if let Some(c) = forward_raw {
            forward_corr_report = Some(c.corr);
        }

        let forward = match override_mode {
            Some(TrackerLoopMode::Forward) => {
                forward_raw.or_else(|| {
                    // Fall back to a best-effort forward loop covering the sample tail.
                    let end_exclusive = samples.len();
                    let start = desired_start
                        .min(sample_len.saturating_sub(2) as u32)
                        .max(1) as usize;
                    if end_exclusive.saturating_sub(start) < 4 {
                        return None;
                    }
                    Some(ForwardLoopCandidate {
                        start,
                        end_exclusive,
                        corr: correlation_i16_stride(
                            &samples,
                            start,
                            end_exclusive.saturating_sub(1024).max(start + 1),
                            1024.min(end_exclusive.saturating_sub(start + 1)),
                            8,
                        )
                        .unwrap_or(0.0),
                    })
                })
            }
            Some(TrackerLoopMode::Auto) | None => {
                forward_raw.filter(|c| c.corr >= FORWARD_LOOP_MIN_CORR)
            }
            Some(TrackerLoopMode::PingPong) => None,
            Some(TrackerLoopMode::None) => None,
        };

        let mut region = None;
        if let Some(forward) = forward {
            let loop_len = forward.end_exclusive.saturating_sub(forward.start);
            let desired_xfade_len = (sample_rate / 100).max(128) as usize; // ~10ms
            let max_xfade_len = (sample_rate / 25).max(512) as usize; // cap ~40ms
            let xfade_len = desired_xfade_len
                .min(max_xfade_len)
                .min(loop_len / 2)
                .max(2);

            if loop_len >= xfade_len + 2 && forward.start + xfade_len + 1 < forward.end_exclusive {
                let head_start = forward.start;
                let tail_start = forward.end_exclusive - xfade_len;

                let xfade_corr =
                    correlation_i16_stride(&samples, head_start, tail_start, xfade_len, 4)
                        .unwrap_or(0.0);
                let use_constant_power = xfade_corr < 0.6;

                for i in 0..xfade_len {
                    let t = i as f64 / (xfade_len - 1) as f64;
                    let (fade_out, fade_in) = if use_constant_power {
                        let a = (t * std::f64::consts::FRAC_PI_2).cos();
                        let b = (t * std::f64::consts::FRAC_PI_2).sin();
                        (a, b)
                    } else {
                        (1.0 - t, t)
                    };

                    let a = samples[tail_start + i] as f64;
                    let b = samples[head_start + i] as f64;
                    let mixed = (a * fade_out + b * fade_in)
                        .round()
                        .clamp(i16::MIN as f64, i16::MAX as f64)
                        as i16;
                    samples[tail_start + i] = mixed;
                }

                let loop_start = (forward.start + xfade_len) as u32;
                let loop_end = forward.end_exclusive.min(samples.len()) as u32;
                crossfade_samples_report = Some(xfade_len as u32);
                crossfade_ms_report = Some(xfade_len as f64 * 1000.0 / sample_rate as f64);

                if loop_start + 1 < loop_end && loop_end >= 2 {
                    let start = loop_start as usize;
                    let end = loop_end as usize;
                    let amp_jump = (samples[end - 1] as i32 - samples[start] as i32).unsigned_abs();
                    let slope_end = samples[end - 1] as i32 - samples[end.saturating_sub(2)] as i32;
                    let slope_start = samples[start + 1] as i32 - samples[start] as i32;
                    let slope_jump = (slope_end - slope_start).unsigned_abs();
                    seam_report = Some(LoopSeamMetrics {
                        amp_jump,
                        slope_jump,
                    });
                }

                region = Some(LoopRegion {
                    start: loop_start,
                    end: loop_end,
                    mode: LoopMode::Forward,
                });
            }
        }

        // Fallback (or forced): ping-pong loop.
        if region.is_none() {
            let desired_start_idx = desired_start as usize;
            let start_idx =
                find_best_pingpong_loop_start_near(&samples, desired_start_idx, radius) as u32;

            let min_loop_len = (sample_rate / 20).max(256); // ~50ms, minimum 256 samples
            let min_end_idx = (start_idx + min_loop_len).min(sample_len.saturating_sub(1)) as usize;
            let tail_window = sample_rate.max(4096) as usize; // ~1s tail search, minimum 4096 samples
            let end_search_end = samples.len().saturating_sub(1);
            let end_search_start = min_end_idx.max(end_search_end.saturating_sub(tail_window));

            let end_idx =
                find_best_pingpong_loop_end_in_range(&samples, end_search_start, end_search_end)
                    .unwrap_or(end_search_end)
                    .min(samples.len().saturating_sub(1)) as u32;

            let end = (end_idx + 1).min(sample_len);
            let start = start_idx.min(end.saturating_sub(1));

            if start + 1 > end {
                return Err(GenerateError::InstrumentError(format!(
                    "Instrument '{}' has invalid loop region: start={} end={}",
                    instr.name, start, end,
                )));
            }

            let start_usize = start as usize;
            let end_usize = end as usize;
            if start_usize + 1 < samples.len() {
                pingpong_start_slope_report = Some(
                    (samples[start_usize + 1] as i32 - samples[start_usize] as i32).unsigned_abs(),
                );
            }
            if end_usize >= 2 && end_usize <= samples.len() {
                pingpong_end_slope_report = Some(
                    (samples[end_usize - 1] as i32 - samples[end_usize - 2] as i32).unsigned_abs(),
                );
            }

            region = Some(LoopRegion {
                start,
                end,
                mode: LoopMode::PingPong,
            });
        }

        // Commit any DC-removal / crossfade edits.
        if region.is_some() {
            pcm16_mono = i16_to_pcm16_mono(&samples);
        }

        region
    } else {
        None
    };

    let chosen_mode = match loop_region.map(|r| r.mode) {
        Some(LoopMode::Forward) => ChosenLoopMode::Forward,
        Some(LoopMode::PingPong) => ChosenLoopMode::PingPong,
        None => ChosenLoopMode::None,
    };

    let sample_len = (pcm16_mono.len() / 2) as u32;

    let (loop_start, loop_end, loop_len) = if let Some(region) = loop_region {
        (
            Some(region.start),
            Some(region.end),
            Some(region.end.saturating_sub(region.start)),
        )
    } else {
        (None, None, None)
    };

    let report = MusicInstrumentLoopReport {
        index,
        name: instr.name.clone(),
        override_mode,
        chosen_mode,
        base_midi,
        base_freq_hz,
        sample_rate,
        sample_len,
        desired_start: desired_start_report,
        loop_start,
        loop_end,
        loop_len,
        forward_corr: forward_corr_report,
        crossfade_samples: crossfade_samples_report,
        crossfade_ms: crossfade_ms_report,
        seam: seam_report,
        pingpong_start_slope: pingpong_start_slope_report,
        pingpong_end_slope: pingpong_end_slope_report,
        dc_removed_mean: dc_removed_mean_report,
    };

    Ok((
        BakedInstrumentSample {
            pcm16_mono,
            sample_rate,
            base_midi,
            loop_region,
        },
        report,
    ))
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

        let (baked, _) =
            bake_instrument_sample(&instr, 42, 0, &spec_dir, TrackerFormat::Xm).unwrap();
        assert!(!baked.pcm16_mono.is_empty());
        assert!(
            baked.loop_region.is_some(),
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

        let (baked, _) =
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
        let (baked, _) =
            bake_instrument_sample(&one_shot, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap();
        assert!(baked.loop_region.is_none());

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
        let (baked, _) =
            bake_instrument_sample(&sustained, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap();
        assert!(baked.loop_region.is_some());
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

        let (baked, _) =
            bake_instrument_sample(&instr, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap();
        assert_eq!(baked.base_midi, 69);
    }

    #[test]
    fn test_sustained_sine_prefers_forward_loop_with_crossfade() {
        let instr = TrackerInstrument {
            name: "Sine Loop".to_string(),
            synthesis_audio_v1: Some(AudioV1Params {
                base_note: Some(AudioNoteSpec::NoteName("A4".to_string())),
                duration_seconds: 1.0,
                sample_rate: 44100,
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
                attack: 0.05,
                decay: 0.2,
                sustain: 0.7,
                release: 0.2,
            },
            ..Default::default()
        };

        let (baked, _) =
            bake_instrument_sample(&instr, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap();
        let loop_region = baked
            .loop_region
            .expect("sustained instruments should loop");
        assert_eq!(loop_region.mode, LoopMode::Forward);
        assert!(loop_region.end > loop_region.start);
    }

    #[test]
    fn test_sustained_noise_falls_back_to_pingpong_loop() {
        let instr = TrackerInstrument {
            name: "Noise Loop".to_string(),
            synthesis_audio_v1: Some(AudioV1Params {
                base_note: Some(AudioNoteSpec::NoteName("A4".to_string())),
                duration_seconds: 1.0,
                sample_rate: 44100,
                layers: vec![AudioLayer {
                    synthesis: AudioSynthesis::NoiseBurst {
                        noise_type: NoiseType::White,
                        filter: None,
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
                attack: 0.05,
                decay: 0.2,
                sustain: 0.7,
                release: 0.2,
            },
            ..Default::default()
        };

        let (baked, _) =
            bake_instrument_sample(&instr, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap();
        let loop_region = baked
            .loop_region
            .expect("sustained instruments should loop");
        assert_eq!(loop_region.mode, LoopMode::PingPong);
        assert!(loop_region.end > loop_region.start);
    }
}
