//! UI element generation (nine-slice panels, icon sets, and item cards).
//!
//! This module implements deterministic UI element generators:
//! - Nine-slice panels with corner/edge/center regions
//! - Icon set atlases with labeled frames
//! - Item card templates with rarity variants

mod gutter;
mod icon_set;
mod item_card;
mod nine_slice;

pub use icon_set::*;
pub use item_card::*;
pub use nine_slice::*;
