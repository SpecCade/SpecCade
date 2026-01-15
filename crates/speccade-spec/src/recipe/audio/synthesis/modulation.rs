//! LFO and modulation types for synthesis.

use serde::{Deserialize, Serialize};

use super::basic_types::Waveform;

/// LFO (Low Frequency Oscillator) configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LfoConfig {
    /// Waveform type for the LFO.
    pub waveform: Waveform,
    /// LFO rate in Hz (typically 0.1-20 Hz).
    pub rate: f64,
    /// Modulation depth (0.0-1.0).
    pub depth: f64,
    /// Initial phase offset (0.0-1.0, optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<f64>,
}

/// Modulation target specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "target", rename_all = "snake_case", deny_unknown_fields)]
pub enum ModulationTarget {
    /// Modulate pitch (vibrato).
    Pitch {
        /// Maximum pitch deviation in semitones.
        semitones: f64,
    },
    /// Modulate volume (tremolo).
    Volume {
        /// Maximum amplitude reduction (0.0-1.0).
        ///
        /// Effective strength is `amount * config.depth`.
        amount: f64,
    },
    /// Modulate filter cutoff frequency.
    FilterCutoff {
        /// Maximum cutoff frequency change in Hz.
        amount: f64,
    },
    /// Modulate stereo pan.
    Pan {
        /// Maximum pan delta applied around the base `layer.pan` (0.0-1.0).
        ///
        /// Effective strength is `amount * config.depth`.
        amount: f64,
    },
    /// Modulate pulse width (duty cycle) of square/pulse oscillators.
    ///
    /// Only valid for `Synthesis::Oscillator` with `waveform: square|pulse` or
    /// `Synthesis::MultiOscillator` with at least one oscillator using `waveform: square|pulse`.
    PulseWidth {
        /// Maximum duty cycle delta around base duty (0.0-0.49).
        ///
        /// The effective duty is `clamp(base_duty + bipolar_lfo * amount * depth, 0.01, 0.99)`.
        amount: f64,
    },
    /// Modulate FM synthesis modulation index.
    ///
    /// Only valid for `Synthesis::FmSynth`.
    FmIndex {
        /// Maximum modulation index delta.
        ///
        /// The effective index is `max(base_index + bipolar_lfo * amount * depth, 0.0)`.
        amount: f64,
    },
    /// Modulate granular synthesis grain size.
    ///
    /// Only valid for `Synthesis::Granular`.
    GrainSize {
        /// Maximum grain size delta in milliseconds.
        ///
        /// The effective grain size is `clamp(base_size + bipolar_lfo * amount_ms * depth, 10.0, 500.0)`.
        amount_ms: f64,
    },
    /// Modulate granular synthesis grain density.
    ///
    /// Only valid for `Synthesis::Granular`.
    GrainDensity {
        /// Maximum grain density delta in grains/sec.
        ///
        /// The effective density is `clamp(base_density + bipolar_lfo * amount * depth, 1.0, 100.0)`.
        amount: f64,
    },
    /// Modulate delay time (post-FX only).
    ///
    /// Only valid in `AudioV1Params.post_fx_lfos`, not in `AudioLayer.lfo`.
    /// Applies to: `delay`, `multi_tap_delay`, `flanger`, `stereo_widener`, `granular_delay`.
    DelayTime {
        /// Maximum delay time delta in milliseconds.
        ///
        /// The effective time is `clamp(base_time_ms + bipolar_lfo * amount_ms, 1.0, 2000.0)`.
        amount_ms: f64,
    },
    /// Modulate reverb room size (post-FX only).
    ///
    /// Only valid in `AudioV1Params.post_fx_lfos`, not in `AudioLayer.lfo`.
    /// Applies to: `reverb`.
    ReverbSize {
        /// Maximum room size delta.
        ///
        /// The effective room_size is `clamp(base + bipolar_lfo * amount, 0.0, 1.0)`.
        amount: f64,
    },
    /// Modulate distortion drive (post-FX only).
    ///
    /// Only valid in `AudioV1Params.post_fx_lfos`, not in `AudioLayer.lfo`.
    /// Applies to: `waveshaper` (and `tape_saturation` when available).
    DistortionDrive {
        /// Maximum drive delta.
        ///
        /// The effective drive is `clamp(base + bipolar_lfo * amount, 1.0, 100.0)`.
        amount: f64,
    },
}

/// LFO modulation configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LfoModulation {
    /// LFO configuration.
    pub config: LfoConfig,
    /// Modulation target.
    pub target: ModulationTarget,
}
