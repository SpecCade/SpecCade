//! Audio effect types for the effect chain.

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// Audio effect in the processing chain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Effect {
    /// Parametric EQ effect with cascaded biquad filters.
    ParametricEq {
        /// EQ bands to apply in order.
        bands: Vec<EqBand>,
    },
    /// Reverb effect.
    Reverb {
        /// Room size (0.0-1.0).
        room_size: f64,
        /// High frequency damping (0.0-1.0).
        damping: f64,
        /// Wet/dry mix (0.0-1.0).
        wet: f64,
        /// Stereo width (0.0-1.0).
        #[serde(default = "default_width")]
        width: f64,
    },
    /// Delay effect.
    Delay {
        /// Delay time in milliseconds (1-2000).
        time_ms: f64,
        /// Feedback amount (0.0-0.95).
        feedback: f64,
        /// Wet/dry mix (0.0-1.0).
        wet: f64,
        /// Enable ping-pong stereo delay.
        #[serde(default)]
        ping_pong: bool,
    },
    /// Chorus effect.
    Chorus {
        /// LFO rate in Hz.
        rate: f64,
        /// Modulation depth (0.0-1.0).
        depth: f64,
        /// Wet/dry mix (0.0-1.0).
        wet: f64,
        /// Number of voices (1-4).
        #[serde(default = "default_chorus_voices")]
        voices: u8,
    },
    /// Phaser effect.
    Phaser {
        /// LFO rate in Hz.
        rate: f64,
        /// Modulation depth (0.0-1.0).
        depth: f64,
        /// Number of allpass stages (2-12).
        stages: u8,
        /// Wet/dry mix (0.0-1.0).
        wet: f64,
    },
    /// Bitcrusher effect.
    Bitcrush {
        /// Bit depth (1-16).
        bits: u8,
        /// Sample rate reduction factor (1.0 = no reduction).
        #[serde(default = "default_sr_reduction")]
        sample_rate_reduction: f64,
    },
    /// Waveshaper distortion.
    Waveshaper {
        /// Drive amount (1.0-100.0).
        drive: f64,
        /// Shaping curve.
        #[serde(default)]
        curve: WaveshaperCurve,
        /// Wet/dry mix (0.0-1.0).
        wet: f64,
    },
    /// Dynamics compressor.
    Compressor {
        /// Threshold in dB (-60 to 0).
        threshold_db: f64,
        /// Compression ratio (1.0-20.0).
        ratio: f64,
        /// Attack time in ms (0.1-100).
        attack_ms: f64,
        /// Release time in ms (10-1000).
        release_ms: f64,
        /// Makeup gain in dB.
        #[serde(default)]
        makeup_db: f64,
    },
    /// Flanger effect.
    Flanger {
        /// LFO rate in Hz (0.1-10.0).
        rate: f64,
        /// Modulation depth (0.0-1.0).
        depth: f64,
        /// Feedback amount (-0.99 to 0.99, clamped for stability).
        feedback: f64,
        /// Base delay time in milliseconds (1-20 typical).
        delay_ms: f64,
        /// Wet/dry mix (0.0-1.0).
        wet: f64,
    },
    /// Brick-wall limiter effect.
    Limiter {
        /// Threshold in dB where limiting begins (-24 to 0).
        threshold_db: f64,
        /// Release time in ms for gain recovery (10-500).
        release_ms: f64,
        /// Lookahead time in ms for peak detection (1-10).
        lookahead_ms: f64,
        /// Maximum output level in dB (-6 to 0).
        ceiling_db: f64,
    },
    /// Gate/expander effect for tightening drums and noise reduction.
    GateExpander {
        /// Threshold in dB where gate opens (-60 to 0).
        threshold_db: f64,
        /// Expansion ratio when below threshold (1.0=off, >1.0=expander, inf=hard gate).
        ratio: f64,
        /// Attack time in ms to open gate (0.1-50).
        attack_ms: f64,
        /// Hold time in ms to stay open after signal drops (0-500).
        hold_ms: f64,
        /// Release time in ms to close gate (10-2000).
        release_ms: f64,
        /// Maximum attenuation depth in dB (-80 to 0).
        range_db: f64,
    },
    /// Stereo widener effect for enhancing stereo image.
    StereoWidener {
        /// Stereo width (0.0 = mono, 1.0 = normal, >1.0 = wider). Range: 0.0-2.0.
        width: f64,
        /// Processing algorithm.
        #[serde(default)]
        mode: StereoWidenerMode,
        /// Delay time in ms for Haas mode only (1-30 typical).
        #[serde(default = "default_haas_delay_ms")]
        delay_ms: f64,
    },
    /// Multi-tap delay effect with independent delay lines.
    MultiTapDelay {
        /// Delay taps to apply. Each tap has independent time, feedback, pan, level, and filter.
        taps: Vec<DelayTap>,
    },
    /// Tape saturation effect with warmth, wow/flutter, and hiss.
    TapeSaturation {
        /// Drive/saturation amount (1.0-20.0).
        drive: f64,
        /// DC bias before saturation (-0.5 to 0.5). Affects harmonic content.
        bias: f64,
        /// Wow rate in Hz (0.0-3.0). Low-frequency pitch modulation.
        wow_rate: f64,
        /// Flutter rate in Hz (0.0-20.0). Higher-frequency pitch modulation.
        flutter_rate: f64,
        /// Tape hiss level (0.0-0.1). Seeded noise added to output.
        hiss_level: f64,
    },
    /// Transient shaper effect for controlling attack punch and sustain.
    TransientShaper {
        /// Attack enhancement (-1.0 to 1.0). Negative = softer transients, positive = more punch.
        attack: f64,
        /// Sustain enhancement (-1.0 to 1.0). Negative = tighter, positive = fuller.
        sustain: f64,
        /// Output makeup gain in dB (-12 to +12).
        output_gain_db: f64,
    },
    /// Auto-filter / envelope follower effect for dynamic filter sweeps.
    AutoFilter {
        /// How much signal level affects filter (0.0-1.0).
        sensitivity: f64,
        /// Envelope attack time in ms (0.1-100).
        attack_ms: f64,
        /// Envelope release time in ms (10-1000).
        release_ms: f64,
        /// Filter sweep range (0.0-1.0).
        depth: f64,
        /// Base cutoff frequency when signal is quiet (100-8000 Hz).
        base_frequency: f64,
    },
    /// Cabinet simulation effect using cascaded biquad filters.
    CabinetSim {
        /// Cabinet type defining the filter curve.
        cabinet_type: CabinetType,
        /// Mic position (0.0 = close/bright, 1.0 = far/dark).
        #[serde(default)]
        mic_position: f64,
    },
    /// Rotary speaker (Leslie) effect with amplitude modulation and Doppler.
    RotarySpeaker {
        /// Rotation rate in Hz (0.5-10.0 typical, "slow" ~1 Hz, "fast" ~6 Hz).
        rate: f64,
        /// Effect intensity (0.0-1.0).
        depth: f64,
        /// Wet/dry mix (0.0-1.0).
        wet: f64,
    },
    /// Ring modulator effect that multiplies audio with a carrier oscillator.
    RingModulator {
        /// Carrier oscillator frequency in Hz (20-2000 typical).
        frequency: f64,
        /// Wet/dry mix (0.0-1.0).
        mix: f64,
    },
    /// Granular delay effect for shimmer and pitchy delays.
    GranularDelay {
        /// Delay time in milliseconds (10-2000).
        time_ms: f64,
        /// Feedback amount (0.0-0.95).
        feedback: f64,
        /// Grain window size in milliseconds (10-200).
        grain_size_ms: f64,
        /// Pitch shift per grain pass in semitones (-24 to +24).
        pitch_semitones: f64,
        /// Wet/dry mix (0.0-1.0).
        wet: f64,
    },
    /// True-peak limiter for broadcast/streaming compliance.
    ///
    /// Uses oversampling to detect and limit inter-sample peaks that exceed the
    /// ceiling. Essential for meeting loudness standards like EBU R128 or ATSC A/85.
    TruePeakLimiter {
        /// Maximum output level in dBTP (-6 to 0). Common values: -1.0 for streaming, -2.0 for broadcast.
        ceiling_db: f64,
        /// Release time in ms for gain recovery (10-500).
        release_ms: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WaveshaperCurve {
    #[default]
    Tanh,
    SoftClip,
    HardClip,
    Sine,
}

/// Cabinet type for cabinet simulation effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CabinetType {
    /// Classic 1x12 combo amp (bright, focused).
    #[default]
    #[serde(rename = "guitar_1x12")]
    Guitar1x12,
    /// Big 4x12 stack (full, warm).
    #[serde(rename = "guitar_4x12")]
    Guitar4x12,
    /// Bass 1x15 cabinet (deep, punchy).
    #[serde(rename = "bass_1x15")]
    Bass1x15,
    /// AM radio lo-fi (bandlimited).
    #[serde(rename = "radio")]
    Radio,
    /// Telephone line quality (narrow bandwidth).
    #[serde(rename = "telephone")]
    Telephone,
}

/// Processing mode for stereo widener effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StereoWidenerMode {
    /// L/R crossmix: new_L = (1 + width) * L - width * R
    #[default]
    Simple,
    /// Haas effect: delay one channel to create stereo image
    Haas,
    /// Mid/Side processing: scale side signal by width
    MidSide,
}

fn default_width() -> f64 {
    1.0
}
fn default_chorus_voices() -> u8 {
    2
}
fn default_sr_reduction() -> f64 {
    1.0
}
fn default_haas_delay_ms() -> f64 {
    10.0
}

/// A single tap in a multi-tap delay effect.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DelayTap {
    /// Delay time for this tap in milliseconds (1-2000).
    pub time_ms: f64,
    /// Feedback amount for this tap (0.0-0.99).
    pub feedback: f64,
    /// Stereo pan position for this tap (-1.0 to 1.0).
    pub pan: f64,
    /// Output level for this tap (0.0-1.0).
    pub level: f64,
    /// Low-pass filter cutoff in Hz. 0 or omitted means no filter.
    #[serde(default)]
    pub filter_cutoff: f64,
}

/// A single band in a parametric EQ.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EqBand {
    /// Center/corner frequency in Hz.
    pub frequency: f64,
    /// Gain in dB (typically -24 to +24).
    pub gain_db: f64,
    /// Q factor (bandwidth). Higher Q = narrower band (typically 0.1 to 10).
    pub q: f64,
    /// Type of EQ band.
    pub band_type: EqBandType,
}

/// Type of EQ band for parametric EQ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EqBandType {
    /// Low shelf: boost/cut frequencies below the frequency.
    Lowshelf,
    /// High shelf: boost/cut frequencies above the frequency.
    Highshelf,
    /// Peak (bell curve): boost/cut frequencies around the frequency.
    Peak,
    /// Notch: cut at the frequency (zero gain at center).
    Notch,
}
