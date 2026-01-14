//! Note and frequency conversion utilities for tracker modules.
//!
//! This module provides deterministic conversion between note names, MIDI numbers,
//! and frequencies for XM and IT tracker formats.

mod constants;
mod conversion;
mod frequency;
mod pitch;

#[cfg(test)]
mod tests;

// Re-export all public items to preserve API
pub use constants::{
    it, xm, DEFAULT_IT_SYNTH_MIDI_NOTE, DEFAULT_SAMPLE_RATE, DEFAULT_SYNTH_MIDI_NOTE,
    IT_BASE_FREQ, XM_BASE_FREQ,
};

pub use conversion::{
    it_note_to_name, it_note_to_xm, note_name_to_it, note_name_to_midi, note_name_to_xm,
    xm_note_to_it, xm_note_to_name,
};

pub use frequency::{freq_to_midi, midi_to_freq};

pub use pitch::{
    calculate_c5_speed, calculate_c5_speed_for_base_note, calculate_pitch_correction,
    calculate_xm_pitch_correction,
};
