//! Vocoder synthesis implementation.
//!
//! A vocoder transfers the spectral envelope from a modulator signal to a carrier signal.
//! Since we're generating from scratch (not processing existing audio), we create:
//! - A carrier signal (sawtooth, pulse, or noise)
//! - Modulator envelope patterns (simulated speech formants or rhythmic patterns)
//!
//! The vocoder works by:
//! 1. Splitting both modulator and carrier signals into frequency bands (filter bank)
//! 2. Extracting the amplitude envelope of each modulator band
//! 3. Applying those envelopes to the corresponding carrier bands
//! 4. Summing all bands to create the output

mod types;
mod synth;
mod processor;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{BandSpacing, CarrierType, VocoderBand};
pub use synth::VocoderSynth;
