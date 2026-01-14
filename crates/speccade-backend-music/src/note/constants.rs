//! Constants for note and frequency conversion utilities.

/// XM reference frequency (C-4 in XM format).
pub const XM_BASE_FREQ: f64 = 8363.0;

/// IT reference frequency (C-5 in IT format, plays at sample rate).
pub const IT_BASE_FREQ: f64 = 8363.0;

/// Default sample rate for generated samples.
pub const DEFAULT_SAMPLE_RATE: u32 = 22050;

/// Default MIDI note for synthesized samples (middle C = C4 = 261.6 Hz)
/// This is used by XM format, which uses C-4 as its reference pitch.
pub const DEFAULT_SYNTH_MIDI_NOTE: u8 = 60;

/// Default MIDI note for IT format synthesized samples (C5 = 523.25 Hz)
/// IT uses C-5 (IT note 60 = MIDI 72) as its reference pitch.
/// When no base_note is specified, IT samples are assumed to contain C-5 audio.
pub const DEFAULT_IT_SYNTH_MIDI_NOTE: u8 = 72;

/// Note values for XM format.
pub mod xm {
    /// No note present.
    pub const NOTE_NONE: u8 = 0;
    /// Note off command.
    pub const NOTE_OFF: u8 = 97;
    /// Minimum valid note (C-0).
    pub const NOTE_MIN: u8 = 1;
    /// Maximum valid note (B-7).
    pub const NOTE_MAX: u8 = 96;
}

/// Note values for IT format.
pub mod it {
    /// Minimum valid note (C-0).
    pub const NOTE_MIN: u8 = 0;
    /// Maximum valid note (B-9).
    pub const NOTE_MAX: u8 = 119;
    /// Note fade command.
    pub const NOTE_FADE: u8 = 253;
    /// Note cut command.
    pub const NOTE_CUT: u8 = 254;
    /// Note off command.
    pub const NOTE_OFF: u8 = 255;
}

/// Semitone offsets for note names (C=0, D=2, E=4, F=5, G=7, A=9, B=11).
pub(super) const SEMITONE_MAP: [(char, i8); 7] = [
    ('C', 0),
    ('D', 2),
    ('E', 4),
    ('F', 5),
    ('G', 7),
    ('A', 9),
    ('B', 11),
];
