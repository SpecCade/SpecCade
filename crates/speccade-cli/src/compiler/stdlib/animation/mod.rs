//! Animation (skeletal animation) stdlib functions.
//!
//! Provides helper functions for creating skeletal animations with keyframes,
//! bone transforms, and export settings.

mod helpers;
mod spec;

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::values::{dict::Dict, Heap, Value};

/// Helper to create a hashed key for dict insertion.
pub(crate) fn hashed_key<'v>(
    heap: &'v Heap,
    key: &str,
) -> starlark::collections::Hashed<Value<'v>> {
    heap.alloc_str(key)
        .to_value()
        .get_hashed()
        .expect("string hashing cannot fail")
}

/// Helper to create an empty dict on the heap.
pub(crate) fn new_dict<'v>(_heap: &'v Heap) -> Dict<'v> {
    let map: SmallMap<Value<'v>, Value<'v>> = SmallMap::new();
    Dict::new(map)
}

/// Valid skeleton presets (shared with character module).
pub(crate) const SKELETON_PRESETS: &[&str] = &["humanoid_basic_v1"];

/// Valid interpolation modes.
pub(crate) const INTERPOLATION_MODES: &[&str] = &["linear", "bezier", "constant"];

/// Valid animation formats.
pub(crate) const ANIMATION_FORMATS: &[&str] = &["glb", "gltf"];

/// Registers animation stdlib functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    helpers::register(builder);
    spec::register(builder);
}

#[cfg(test)]
mod tests {
    pub use super::super::tests::eval_to_json;
}
