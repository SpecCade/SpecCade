//! LFO modulation system for audio synthesis.
//!
//! This module implements Low Frequency Oscillator (LFO) modulation for various
//! synthesis parameters. LFOs generate slow-moving waveforms that modulate other
//! parameters over time, creating effects like vibrato, tremolo, and filter sweeps.

pub mod lfo;

#[cfg(test)]
mod tests;

pub use lfo::Lfo;
