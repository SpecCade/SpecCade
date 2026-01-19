//! Texture stdlib functions for procedural texture generation.
//!
//! Provides helper functions for creating texture graph nodes and graphs.

mod graph;
mod nodes;
mod spec;
mod trimsheet;

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

/// Valid texture output formats.
pub(crate) const TEXTURE_FORMATS: &[&str] = &["png", "jpg", "exr", "tga"];

/// Registers texture stdlib functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    nodes::register(builder);
    graph::register(builder);
    spec::register(builder);
    trimsheet::register(builder);
}
