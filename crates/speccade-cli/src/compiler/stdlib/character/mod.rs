//! Character (skeletal mesh) stdlib functions.
//!
//! Provides helper functions for creating skeletal meshes with skeletons,
//! body parts, materials, and skinning settings.

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

/// Valid skeleton presets.
pub(crate) const SKELETON_PRESETS: &[&str] = &["humanoid_basic_v1"];

/// Valid mesh primitives (shared with mesh module).
pub(crate) const PRIMITIVES: &[&str] = &[
    "cube",
    "sphere",
    "cylinder",
    "cone",
    "torus",
    "plane",
    "ico_sphere",
];

/// Valid UV modes for texturing.
pub(crate) const UV_MODES: &[&str] =
    &["cylinder_project", "box_project", "sphere_project", "smart"];

/// Registers character stdlib functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    helpers::register(builder);
    spec::register(builder);
}

#[cfg(test)]
mod tests {
    pub use super::super::tests::eval_to_json;
}
