//! Core logic for baking tracker instruments into samples.
//!
//! This module handles the conversion of various instrument sources (WAV files, audio_v1 specs,
//! legacy synthesis) into tracker-compatible PCM samples with appropriate loop points.

use std::path::Path;

use speccade_spec::recipe::audio::NoteSpec as AudioNoteSpec;
use speccade_spec::recipe::music::{InstrumentSynthesis, TrackerFormat, TrackerInstrument};

use super::loop_detection::{
    correlation_i16_stride, find_best_forward_loop_candidate, find_best_pingpong_loop_end_in_range,
    find_best_pingpong_loop_start_near, i16_to_pcm16_mono, pcm16_mono_to_i16,
    remove_dc_offset_i16_in_place, ForwardLoopCandidate,
};
use super::{
    BakedInstrumentSample, ChosenLoopMode, GenerateError, LoopMode, LoopRegion, LoopSeamMetrics,
    MusicInstrumentLoopReport,
};
use crate::note::{midi_to_freq, DEFAULT_IT_SYNTH_MIDI_NOTE, DEFAULT_SYNTH_MIDI_NOTE};
use crate::synthesis::{derive_instrument_seed, load_wav_sample};

use super::helpers::{
    downmix_pcm16_stereo_to_mono, enforce_max_sample_len, legacy_synthesis_to_audio_v1_params,
    load_audio_v1_params_from_ref, neutralize_audio_layer_envelopes, parse_base_note_midi,
};

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
        Some(speccade_spec::recipe::music::TrackerLoopMode::None) => false,
        Some(speccade_spec::recipe::music::TrackerLoopMode::Forward)
        | Some(speccade_spec::recipe::music::TrackerLoopMode::PingPong) => true,
        Some(speccade_spec::recipe::music::TrackerLoopMode::Auto) | None => {
            instr.envelope.sustain > 0.0
        }
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
        let forward_raw = if matches!(
            override_mode,
            Some(speccade_spec::recipe::music::TrackerLoopMode::PingPong)
        ) {
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
            Some(speccade_spec::recipe::music::TrackerLoopMode::Forward) => {
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
            Some(speccade_spec::recipe::music::TrackerLoopMode::Auto) | None => {
                forward_raw.filter(|c| c.corr >= FORWARD_LOOP_MIN_CORR)
            }
            Some(speccade_spec::recipe::music::TrackerLoopMode::PingPong) => None,
            Some(speccade_spec::recipe::music::TrackerLoopMode::None) => None,
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
        pitch_deviation_cents: None,
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
