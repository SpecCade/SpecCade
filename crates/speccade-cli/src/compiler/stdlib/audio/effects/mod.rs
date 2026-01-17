//! Effect functions for audio processing
//!
//! This module provides functions for creating audio effects like reverb, delay,
//! compression, modulation, and spatial effects.

use starlark::environment::GlobalsBuilder;

mod basic;
mod dynamics;
mod modulation;
mod spatial;

/// Registers all effects functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    basic::register(builder);
    dynamics::register(builder);
    modulation::register(builder);
    spatial::register(builder);
}
