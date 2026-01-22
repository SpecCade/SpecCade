//! UI element generation (nine-slice panels and icon sets).
//!
//! This module implements deterministic UI element generators:
//! - Nine-slice panels with corner/edge/center regions
//! - Icon set atlases with labeled frames

mod gutter;
mod icon_set;
mod nine_slice;

pub use icon_set::*;
pub use nine_slice::*;
