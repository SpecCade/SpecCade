//! Typed effect types for tracker modules with validation.
//!
//! This module provides a typed effect enum that can be validated
//! for format-specific parameter ranges (XM vs IT).

mod conversion;
pub mod it_codes;
#[cfg(test)]
mod tests;
mod tracker_options;
mod validation;
pub mod xm_codes;

pub use conversion::{decode_it_effect, decode_xm_effect, parse_effect_name};
pub use tracker_options::{AutomationEntry, ItOptions, PatternEffect};
pub use validation::EffectValidationError;

use serde::{Deserialize, Serialize};

/// Typed tracker effect command with validated parameters.
///
/// Effects can be specified either as typed variants or as raw effect codes.
/// The typed variants provide validation and documentation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TrackerEffect {
    /// Arpeggio effect - rapid cycling between note, note+x, note+y semitones.
    /// XM: 0xy, IT: Jxy
    Arpeggio {
        /// First semitone offset (0-15).
        x: u8,
        /// Second semitone offset (0-15).
        y: u8,
    },

    /// Portamento up - slide pitch up.
    /// XM: 1xx, IT: Fxx
    PortamentoUp {
        /// Speed (0-255, 0 uses previous value).
        speed: u8,
    },

    /// Portamento down - slide pitch down.
    /// XM: 2xx, IT: Exx
    PortamentoDown {
        /// Speed (0-255, 0 uses previous value).
        speed: u8,
    },

    /// Fine portamento up - precise pitch slide up.
    /// XM: E1x (extended), IT: FFx
    FinePortamentoUp {
        /// Fine speed (0-15).
        speed: u8,
    },

    /// Fine portamento down - precise pitch slide down.
    /// XM: E2x (extended), IT: EFx
    FinePortamentoDown {
        /// Fine speed (0-15).
        speed: u8,
    },

    /// Extra fine portamento up - very precise pitch slide up.
    /// XM: X1x, IT: uses fine porta
    ExtraFinePortaUp {
        /// Extra fine speed (0-15).
        speed: u8,
    },

    /// Extra fine portamento down - very precise pitch slide down.
    /// XM: X2x, IT: uses fine porta
    ExtraFinePortaDown {
        /// Extra fine speed (0-15).
        speed: u8,
    },

    /// Tone portamento - slide to target note.
    /// XM: 3xx, IT: Gxx
    TonePortamento {
        /// Speed (0-255, 0 uses previous value).
        speed: u8,
    },

    /// Vibrato - pitch oscillation.
    /// XM: 4xy, IT: Hxy
    Vibrato {
        /// Speed (0-15).
        speed: u8,
        /// Depth (0-15).
        depth: u8,
    },

    /// Tremolo - volume oscillation.
    /// XM: 7xy, IT: Rxy
    Tremolo {
        /// Speed (0-15).
        speed: u8,
        /// Depth (0-15).
        depth: u8,
    },

    /// Volume slide.
    /// XM: Axy, IT: Dxy
    VolumeSlide {
        /// Up speed (0-15, mutually exclusive with down except for fine slides).
        up: u8,
        /// Down speed (0-15, mutually exclusive with up except for fine slides).
        down: u8,
    },

    /// Set volume.
    /// XM: Cxx, IT: uses volume column (Mxx for channel volume)
    SetVolume {
        /// Volume (0-64).
        volume: u8,
    },

    /// Set panning position.
    /// XM: 8xx, IT: Xxx
    SetPanning {
        /// Panning (0-255, 0=left, 128=center, 255=right).
        pan: u8,
    },

    /// Sample offset - start playback from offset.
    /// XM: 9xx, IT: Oxx (uses SAx for high byte)
    SampleOffset {
        /// Offset in 256-byte units (0-255).
        offset: u8,
    },

    /// Position jump - jump to order position.
    /// XM: Bxx, IT: Bxx
    PositionJump {
        /// Order position (0-255).
        position: u8,
    },

    /// Pattern break - jump to row in next pattern.
    /// XM: Dxx, IT: Cxx
    PatternBreak {
        /// Row number (0-63 for XM, 0-255 for IT).
        row: u8,
    },

    /// Set speed (ticks per row).
    /// XM: Fxx (1-31), IT: Axx
    SetSpeed {
        /// Speed in ticks per row (1-31 for XM, 1-255 for IT).
        speed: u8,
    },

    /// Set tempo (BPM).
    /// XM: Fxx (32-255), IT: Txx
    SetTempo {
        /// Tempo in BPM (32-255).
        bpm: u8,
    },

    /// Retrigger note with volume change.
    /// XM: Rxy (multi-retrig), IT: Qxy
    Retrigger {
        /// Volume change type (0-15).
        /// 0=none, 1=-1, 2=-2, 3=-4, 4=-8, 5=-16, 6=*2/3, 7=*1/2
        /// 8=none, 9=+1, A=+2, B=+4, C=+8, D=+16, E=*3/2, F=*2
        volume_change: u8,
        /// Retrigger interval in ticks (1-15).
        interval: u8,
    },

    /// Note delay - delay note start by ticks.
    /// XM: EDx, IT: SDx
    NoteDelay {
        /// Delay in ticks (0-15).
        ticks: u8,
    },

    /// Note cut - cut note after ticks.
    /// XM: ECx, IT: SCx
    NoteCut {
        /// Delay in ticks before cut (0-15).
        ticks: u8,
    },

    /// Tremor - rapid volume on/off.
    /// XM: Txy, IT: Ixy
    Tremor {
        /// On time in ticks (0-15).
        on_time: u8,
        /// Off time in ticks (0-15).
        off_time: u8,
    },

    /// Global volume.
    /// XM: Gxx, IT: Vxx
    SetGlobalVolume {
        /// Global volume (0-64 for XM, 0-128 for IT).
        volume: u8,
    },

    /// Global volume slide.
    /// XM: Hxy, IT: Wxy
    GlobalVolumeSlide {
        /// Up speed (0-15).
        up: u8,
        /// Down speed (0-15).
        down: u8,
    },

    /// Panning slide.
    /// XM: Pxy, IT: Pxy
    PanningSlide {
        /// Left speed (0-15).
        left: u8,
        /// Right speed (0-15).
        right: u8,
    },

    /// Set channel volume (IT only).
    /// IT: Mxx
    SetChannelVolume {
        /// Channel volume (0-64).
        volume: u8,
    },

    /// Channel volume slide (IT only).
    /// IT: Nxy
    ChannelVolumeSlide {
        /// Up speed (0-15).
        up: u8,
        /// Down speed (0-15).
        down: u8,
    },

    /// Fine vibrato (IT only).
    /// IT: Uxy
    FineVibrato {
        /// Speed (0-15).
        speed: u8,
        /// Depth (0-15).
        depth: u8,
    },

    /// Set vibrato waveform.
    /// XM: E4x, IT: S3x
    SetVibratoWaveform {
        /// Waveform (0=sine, 1=ramp down, 2=square, 3=random).
        waveform: u8,
    },

    /// Set tremolo waveform.
    /// XM: E7x, IT: S4x
    SetTremoloWaveform {
        /// Waveform (0=sine, 1=ramp down, 2=square, 3=random).
        waveform: u8,
    },

    /// Set finetune.
    /// XM: E5x, IT: S2x
    SetFinetune {
        /// Finetune value (XM: -8 to +7 as 0-15, IT: 0-15).
        value: u8,
    },

    /// Pattern loop.
    /// XM: E6x, IT: SBx
    PatternLoop {
        /// Loop count (0=set loop start, 1-15=loop count).
        count: u8,
    },

    /// Tone portamento + volume slide.
    /// XM: 5xy, IT: Lxy
    TonePortaVolumeSlide {
        /// Volume up (0-15).
        up: u8,
        /// Volume down (0-15).
        down: u8,
    },

    /// Vibrato + volume slide.
    /// XM: 6xy, IT: Kxy
    VibratoVolumeSlide {
        /// Volume up (0-15).
        up: u8,
        /// Volume down (0-15).
        down: u8,
    },

    /// Key off with tick delay.
    /// XM: Kxx
    KeyOff {
        /// Tick to key off (0-255).
        tick: u8,
    },

    /// Set envelope position.
    /// XM: Lxx
    SetEnvelopePosition {
        /// Envelope tick position.
        position: u8,
    },

    /// Panbrello (IT only) - panning oscillation.
    /// IT: Yxy
    Panbrello {
        /// Speed (0-15).
        speed: u8,
        /// Depth (0-15).
        depth: u8,
    },

    /// Raw effect for backward compatibility.
    /// Use typed variants when possible for validation.
    Raw {
        /// Effect code (format-specific).
        code: u8,
        /// Effect parameter.
        param: u8,
    },
}
