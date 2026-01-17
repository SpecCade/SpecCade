//! Texture graph assembly functions.

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::list::AllocList;
use starlark::values::{dict::Dict, list::UnpackList, Heap, Value, ValueLike};

/// Helper to create a hashed key for dict insertion.
/// String hashing cannot fail, so we use expect.
fn hashed_key<'v>(heap: &'v Heap, key: &str) -> starlark::collections::Hashed<Value<'v>> {
    heap.alloc_str(key)
        .to_value()
        .get_hashed()
        .expect("string hashing cannot fail")
}

/// Helper to create an empty dict on the heap.
fn new_dict<'v>(_heap: &'v Heap) -> Dict<'v> {
    let map: SmallMap<Value<'v>, Value<'v>> = SmallMap::new();
    Dict::new(map)
}

/// Registers texture graph assembly functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_texture_graph_functions(builder);
}

#[starlark_module]
fn register_texture_graph_functions(builder: &mut GlobalsBuilder) {
    /// Creates a complete texture graph recipe params.
    ///
    /// # Arguments
    /// * `resolution` - [width, height] in pixels
    /// * `nodes` - List of texture nodes
    /// * `tileable` - Whether texture should tile seamlessly (default: True)
    ///
    /// # Returns
    /// A dict matching the TextureProceduralV1Params structure.
    ///
    /// # Example
    /// ```starlark
    /// texture_graph(
    ///     [64, 64],
    ///     [
    ///         noise_node("height", "perlin", 0.1, 4),
    ///         threshold_node("mask", "height", 0.5)
    ///     ]
    /// )
    /// ```
    fn texture_graph<'v>(
        resolution: UnpackList<i32>,
        nodes: UnpackList<Value<'v>>,
        #[starlark(default = true)] tileable: bool,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate resolution
        if resolution.items.len() != 2 {
            return Err(anyhow::anyhow!(
                "S101: texture_graph(): 'resolution' must be [width, height], got {} values",
                resolution.items.len()
            ));
        }
        let width = resolution.items[0];
        let height = resolution.items[1];
        if width <= 0 || height <= 0 {
            return Err(anyhow::anyhow!(
                "S103: texture_graph(): resolution values must be positive, got [{}, {}]",
                width, height
            ));
        }

        let mut dict = new_dict(heap);

        // resolution as list
        let res_list = heap.alloc(AllocList(vec![
            heap.alloc(width).to_value(),
            heap.alloc(height).to_value(),
        ]));
        dict.insert_hashed(
            hashed_key(heap, "resolution"),
            res_list,
        );

        dict.insert_hashed(
            hashed_key(heap, "tileable"),
            heap.alloc(tileable).to_value(),
        );

        // nodes as list
        let nodes_list = heap.alloc(AllocList(nodes.items));
        dict.insert_hashed(
            hashed_key(heap, "nodes"),
            nodes_list,
        );

        Ok(dict)
    }
}
