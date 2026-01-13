//! Audio effect types for the effect chain.

use serde::{Deserialize, Serialize};

/// Audio effect in the processing chain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Effect {
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

fn default_width() -> f64 {
    1.0
}
fn default_chorus_voices() -> u8 {
    2
}
fn default_sr_reduction() -> f64 {
    1.0
}
