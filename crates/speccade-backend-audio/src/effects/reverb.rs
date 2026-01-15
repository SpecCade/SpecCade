//! Freeverb-style reverb effect.
//!
//! Implementation of the Freeverb algorithm with 8 parallel comb filters
//! and 4 serial allpass filters.

use crate::error::{AudioError, AudioResult};
use crate::mixer::StereoOutput;

// Freeverb tuning constants (in samples at 44.1kHz)
const COMB_TUNINGS: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLPASS_TUNINGS: [usize; 4] = [556, 441, 341, 225];
const STEREO_SPREAD: usize = 23;

const FIXED_GAIN: f64 = 0.015;
const SCALE_WET: f64 = 3.0;
const SCALE_DAMPING: f64 = 0.4;
const SCALE_ROOM: f64 = 0.28;
const OFFSET_ROOM: f64 = 0.7;

/// Comb filter with feedback.
struct CombFilter {
    buffer: Vec<f64>,
    buffer_index: usize,
    filter_store: f64,
    damp1: f64,
    damp2: f64,
    feedback: f64,
}

impl CombFilter {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            buffer_index: 0,
            filter_store: 0.0,
            damp1: 0.0,
            damp2: 0.0,
            feedback: 0.0,
        }
    }

    fn set_damping(&mut self, val: f64) {
        self.damp1 = val;
        self.damp2 = 1.0 - val;
    }

    fn set_feedback(&mut self, val: f64) {
        self.feedback = val;
    }

    fn process(&mut self, input: f64) -> f64 {
        let output = self.buffer[self.buffer_index];

        // One-pole lowpass filter
        self.filter_store = (output * self.damp2) + (self.filter_store * self.damp1);

        self.buffer[self.buffer_index] = input + (self.filter_store * self.feedback);

        self.buffer_index += 1;
        if self.buffer_index >= self.buffer.len() {
            self.buffer_index = 0;
        }

        output
    }

    fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.filter_store = 0.0;
        self.buffer_index = 0;
    }
}

/// Allpass filter.
struct AllpassFilter {
    buffer: Vec<f64>,
    buffer_index: usize,
}

impl AllpassFilter {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            buffer_index: 0,
        }
    }

    fn process(&mut self, input: f64) -> f64 {
        let buf_out = self.buffer[self.buffer_index];
        let output = buf_out - input;

        self.buffer[self.buffer_index] = input + (buf_out * 0.5);

        self.buffer_index += 1;
        if self.buffer_index >= self.buffer.len() {
            self.buffer_index = 0;
        }

        output
    }

    fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.buffer_index = 0;
    }
}

/// Freeverb reverb processor.
struct Freeverb {
    combs_left: Vec<CombFilter>,
    combs_right: Vec<CombFilter>,
    allpasses_left: Vec<AllpassFilter>,
    allpasses_right: Vec<AllpassFilter>,
    wet: f64,
    wet1: f64,
    wet2: f64,
    dry: f64,
    width: f64,
}

impl Freeverb {
    fn new(sample_rate: f64) -> Self {
        let scale = sample_rate / 44100.0;

        let combs_left: Vec<_> = COMB_TUNINGS
            .iter()
            .map(|&size| CombFilter::new((size as f64 * scale) as usize))
            .collect();

        let combs_right: Vec<_> = COMB_TUNINGS
            .iter()
            .map(|&size| CombFilter::new(((size + STEREO_SPREAD) as f64 * scale) as usize))
            .collect();

        let allpasses_left: Vec<_> = ALLPASS_TUNINGS
            .iter()
            .map(|&size| AllpassFilter::new((size as f64 * scale) as usize))
            .collect();

        let allpasses_right: Vec<_> = ALLPASS_TUNINGS
            .iter()
            .map(|&size| AllpassFilter::new(((size + STEREO_SPREAD) as f64 * scale) as usize))
            .collect();

        Self {
            combs_left,
            combs_right,
            allpasses_left,
            allpasses_right,
            wet: 0.0,
            wet1: 0.0,
            wet2: 0.0,
            dry: 0.0,
            width: 0.0,
        }
    }

    fn set_room_size(&mut self, value: f64) {
        let room_size = (value * SCALE_ROOM) + OFFSET_ROOM;
        for comb in &mut self.combs_left {
            comb.set_feedback(room_size);
        }
        for comb in &mut self.combs_right {
            comb.set_feedback(room_size);
        }
    }

    fn set_damping(&mut self, value: f64) {
        let damp = value * SCALE_DAMPING;
        for comb in &mut self.combs_left {
            comb.set_damping(damp);
        }
        for comb in &mut self.combs_right {
            comb.set_damping(damp);
        }
    }

    fn set_wet(&mut self, value: f64) {
        self.wet = value * SCALE_WET;
        self.update_mix();
    }

    fn set_width(&mut self, value: f64) {
        self.width = value;
        self.update_mix();
    }

    fn set_dry(&mut self, value: f64) {
        self.dry = value;
    }

    fn update_mix(&mut self) {
        self.wet1 = self.wet * (self.width / 2.0 + 0.5);
        self.wet2 = self.wet * ((1.0 - self.width) / 2.0);
    }

    fn clear(&mut self) {
        for comb in &mut self.combs_left {
            comb.clear();
        }
        for comb in &mut self.combs_right {
            comb.clear();
        }
        for allpass in &mut self.allpasses_left {
            allpass.clear();
        }
        for allpass in &mut self.allpasses_right {
            allpass.clear();
        }
    }

    fn process(&mut self, input_left: f64, input_right: f64) -> (f64, f64) {
        let input = (input_left + input_right) * FIXED_GAIN;

        // Process comb filters in parallel
        let mut out_left = 0.0;
        for comb in &mut self.combs_left {
            out_left += comb.process(input);
        }

        let mut out_right = 0.0;
        for comb in &mut self.combs_right {
            out_right += comb.process(input);
        }

        // Process allpass filters in series
        for allpass in &mut self.allpasses_left {
            out_left = allpass.process(out_left);
        }

        for allpass in &mut self.allpasses_right {
            out_right = allpass.process(out_right);
        }

        // Mix wet/dry
        let left = out_left * self.wet1 + out_right * self.wet2 + input_left * self.dry;
        let right = out_right * self.wet1 + out_left * self.wet2 + input_right * self.dry;

        (left, right)
    }
}

/// Applies reverb effect to stereo audio.
pub fn apply(
    stereo: &mut StereoOutput,
    room_size: f64,
    damping: f64,
    wet: f64,
    width: f64,
    sample_rate: f64,
) -> AudioResult<()> {
    // Create a constant room_size curve for the non-modulated case
    let num_samples = stereo.left.len();
    let room_size_curve = vec![room_size; num_samples];
    apply_with_modulation(stereo, &room_size_curve, damping, wet, width, sample_rate)
}

/// Applies reverb effect to stereo audio with per-sample room_size modulation.
///
/// # Arguments
/// * `stereo` - Stereo audio to process
/// * `room_size_curve` - Per-sample room size values (0.0-1.0)
/// * `damping` - High-frequency absorption (0.0-1.0)
/// * `wet` - Wet/dry mix (0.0-1.0)
/// * `width` - Stereo width (0.0-1.0)
/// * `sample_rate` - Sample rate in Hz
pub fn apply_with_modulation(
    stereo: &mut StereoOutput,
    room_size_curve: &[f64],
    damping: f64,
    wet: f64,
    width: f64,
    sample_rate: f64,
) -> AudioResult<()> {
    // Validate parameters
    if !(0.0..=1.0).contains(&damping) {
        return Err(AudioError::invalid_param(
            "reverb.damping",
            format!("must be 0.0-1.0, got {}", damping),
        ));
    }
    if !(0.0..=1.0).contains(&wet) {
        return Err(AudioError::invalid_param(
            "reverb.wet",
            format!("must be 0.0-1.0, got {}", wet),
        ));
    }
    if !(0.0..=1.0).contains(&width) {
        return Err(AudioError::invalid_param(
            "reverb.width",
            format!("must be 0.0-1.0, got {}", width),
        ));
    }

    let mut reverb = Freeverb::new(sample_rate);
    reverb.set_damping(damping);
    reverb.set_wet(wet);
    reverb.set_width(width);
    reverb.set_dry(1.0 - wet);
    reverb.clear();

    let num_samples = stereo.left.len();
    let mut wet_left = Vec::with_capacity(num_samples);
    let mut wet_right = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        // Get modulated room_size for this sample
        let room_size = room_size_curve
            .get(i)
            .copied()
            .unwrap_or(0.5)
            .clamp(0.0, 1.0);
        reverb.set_room_size(room_size);

        let (left, right) = reverb.process(stereo.left[i], stereo.right[i]);
        wet_left.push(left);
        wet_right.push(right);
    }

    stereo.left = wet_left;
    stereo.right = wet_right;

    Ok(())
}
