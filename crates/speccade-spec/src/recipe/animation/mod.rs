//! Skeletal animation recipe types.

mod animator_rig;
mod clip;
mod common;
mod constraints;
mod helpers;
mod ik_setup;
mod pose;
mod procedural;
mod rig;
mod skeletal;

// Re-export all public types
pub use animator_rig::*;
pub use clip::*;
pub use common::*;
pub use constraints::*;
pub use helpers::*;
pub use ik_setup::*;
pub use pose::*;
pub use procedural::*;
pub use rig::*;
pub use skeletal::*;

#[cfg(test)]
mod tests;
