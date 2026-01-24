//! UI asset recipe types (nine-slice panels, icon sets, item cards, and damage numbers).
//!
//! This module defines four recipe kinds:
//! - `ui.nine_slice_v1` - Nine-slice panel generation with corner/edge/center regions
//! - `ui.icon_set_v1` - Icon pack assembly with sprite frames
//! - `ui.item_card_v1` - Item card templates with rarity variants and customizable slots
//! - `ui.damage_number_v1` - Damage number sprites with style variants (normal, critical, healing)

mod damage_number;
mod icon_set;
mod item_card;
mod nine_slice;

pub use damage_number::*;
pub use icon_set::*;
pub use item_card::*;
pub use nine_slice::*;

/// Default padding/gutter size in pixels for UI atlases.
/// This provides mip-safe borders between atlas regions.
pub(crate) const DEFAULT_UI_PADDING: u32 = 2;

#[cfg(test)]
mod tests;
