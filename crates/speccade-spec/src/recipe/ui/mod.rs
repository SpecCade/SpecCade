//! UI asset recipe types (nine-slice panels and icon sets).
//!
//! This module defines two recipe kinds:
//! - `ui.nine_slice_v1` - Nine-slice panel generation with corner/edge/center regions
//! - `ui.icon_set_v1` - Icon pack assembly with sprite frames

mod icon_set;
mod nine_slice;

pub use icon_set::*;
pub use nine_slice::*;

/// Default padding/gutter size in pixels for UI atlases.
/// This provides mip-safe borders between atlas regions.
pub(crate) const DEFAULT_UI_PADDING: u32 = 2;

#[cfg(test)]
mod tests;
