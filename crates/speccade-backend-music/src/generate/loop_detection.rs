//! Loop detection algorithms for tracker samples.
//!
//! This module contains algorithms to find optimal loop points for both forward and ping-pong
//! looping modes. The goal is to produce seamless loops that minimize audible artifacts.

use super::GenerateError;

pub(super) fn pcm16_mono_to_i16(pcm16_mono: &[u8]) -> Result<Vec<i16>, GenerateError> {
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

pub(super) fn i16_to_pcm16_mono(samples: &[i16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * 2);
    for &sample in samples {
        out.extend_from_slice(&sample.to_le_bytes());
    }
    out
}

pub(super) fn remove_dc_offset_i16_in_place(samples: &mut [i16]) -> i64 {
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

pub(super) fn correlation_i16_stride(
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
pub(super) struct ForwardLoopCandidate {
    pub start: usize,
    pub end_exclusive: usize,
    pub corr: f64,
}

pub(super) fn find_best_forward_loop_candidate(
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

pub(super) fn find_best_pingpong_loop_start_near(
    samples: &[i16],
    target: usize,
    radius: usize,
) -> usize {
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

pub(super) fn find_best_pingpong_loop_end_in_range(
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
