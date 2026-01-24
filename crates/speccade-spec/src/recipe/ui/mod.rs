//! UI asset recipe types (nine-slice panels, icon sets, and item cards).
//!
//! This module defines three recipe kinds:
//! - `ui.nine_slice_v1` - Nine-slice panel generation with corner/edge/center regions
//! - `ui.icon_set_v1` - Icon pack assembly with sprite frames
//! - `ui.item_card_v1` - Item card templates with rarity variants and customizable slots

mod icon_set;
mod item_card;
mod nine_slice;

pub use icon_set::*;
pub use item_card::*;
pub use nine_slice::*;

/// Default padding/gutter size in pixels for UI atlases.
/// This provides mip-safe borders between atlas regions.
pub(crate) const DEFAULT_UI_PADDING: u32 = 2;

#[cfg(test)]
mod tests;
