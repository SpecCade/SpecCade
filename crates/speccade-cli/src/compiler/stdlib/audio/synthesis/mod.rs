//! Synthesis functions for audio generation
//!
//! This module provides various synthesis algorithms including:
//! - Basic synthesis (oscillator, FM, AM, noise)
//! - Complex synthesis (additive, wavetable, granular)
//! - Physical modeling (drums, strings, modal)
//! - Exotic synthesis (vocoder, formant, vector, waveguide)

use starlark::environment::GlobalsBuilder;

mod basic;
mod complex;
mod exotic;
mod physical;

/// Registers all synthesis functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    basic::register(builder);
    complex::register(builder);
    physical::register(builder);
    exotic::register(builder);
}
