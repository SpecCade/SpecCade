//! Audio stdlib functions for synthesis and effects.
//!
//! Provides helper functions for creating audio synthesis layers, envelopes,
//! oscillators, filters, and effects.

use starlark::environment::GlobalsBuilder;

mod effects;
mod filters;
mod layers;
mod modulation;
mod synthesis;

/// Registers audio stdlib functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    modulation::register(builder);
    synthesis::register(builder);
    filters::register(builder);
    effects::register(builder);
    layers::register(builder);
}
