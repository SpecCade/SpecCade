//! Mesh stdlib functions for static mesh generation.
//!
//! Provides helper functions for creating mesh primitives and modifiers.

mod baking;
mod modifiers;
mod primitives;
mod spec;

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::values::{dict::Dict, Heap, Value};

/// Helper to create a hashed key for dict insertion.
/// String hashing cannot fail, so we use expect.
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

/// Valid mesh primitive types.
pub(crate) const PRIMITIVES: &[&str] = &[
    "cube",
    "sphere",
    "cylinder",
    "cone",
    "torus",
    "plane",
    "ico_sphere",
];

/// Valid mesh output formats.
pub(crate) const MESH_FORMATS: &[&str] = &["glb", "gltf", "obj", "fbx"];

/// Registers mesh stdlib functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    primitives::register(builder);
    modifiers::register(builder);
    baking::register(builder);
    spec::register(builder);
}

#[cfg(test)]
mod tests {
    pub use super::super::tests::eval_to_json;
}
