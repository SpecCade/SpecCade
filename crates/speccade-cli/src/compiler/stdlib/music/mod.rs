//! Music stdlib functions for tracker module creation.
//!
//! Provides helper functions for creating tracker instruments, patterns,
//! arrangements, and complete tracker songs.
//!
//! ## Overview
//!
//! The music stdlib supports two tracker formats:
//! - **XM**: FastTracker II Extended Module format (1-32 channels)
//! - **IT**: Impulse Tracker format (1-64 channels)
//!
//! ## Function Categories
//!
//! - **Instruments**: `tracker_instrument()`, `instrument_synthesis()` - Define instruments
//! - **Patterns**: `tracker_pattern()`, `pattern_note()` - Define patterns and notes
//! - **Song**: `tracker_song()`, `arrangement_entry()` - Compose complete songs
//! - **Effects**: `it_options()`, `automation_entry()` - Module options and automation

use starlark::environment::GlobalsBuilder;

mod util;
mod instruments;
mod patterns;
mod song;

/// Registers music stdlib functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    instruments::register(builder);
    patterns::register(builder);
    song::register(builder);
}
