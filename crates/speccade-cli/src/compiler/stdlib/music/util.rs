//! Utility functions shared across music modules.

use starlark::collections::SmallMap;
use starlark::values::{dict::Dict, Heap, Value};

/// Helper to create a hashed key for dict insertion.
/// String hashing cannot fail, so we use expect.
pub fn hashed_key<'v>(heap: &'v Heap, key: &str) -> starlark::collections::Hashed<Value<'v>> {
    heap.alloc_str(key)
        .to_value()
        .get_hashed()
        .expect("string hashing cannot fail")
}

/// Helper to create an empty dict on the heap.
pub fn new_dict<'v>(_heap: &'v Heap) -> Dict<'v> {
    let map: SmallMap<Value<'v>, Value<'v>> = SmallMap::new();
    Dict::new(map)
}
