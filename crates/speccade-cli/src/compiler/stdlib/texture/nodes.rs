//! Texture node creation functions.

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::list::AllocList;
use starlark::values::{dict::Dict, list::UnpackList, none::NoneType, Heap, Value, ValueLike};

use super::super::validation::{
    extract_float, validate_enum, validate_non_empty, validate_positive_int, validate_unit_range,
};

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

/// Valid noise algorithms.
const NOISE_ALGORITHMS: &[&str] = &["perlin", "simplex", "worley", "value", "fbm"];

/// Valid gradient directions.
const GRADIENT_DIRECTIONS: &[&str] = &["horizontal", "vertical", "radial"];

/// Valid stripe directions.
const STRIPE_DIRECTIONS: &[&str] = &["horizontal", "vertical"];

/// Registers texture node functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_texture_node_functions(builder);
}

#[starlark_module]
fn register_texture_node_functions(builder: &mut GlobalsBuilder) {
    /// Creates a noise texture node.
    ///
    /// # Arguments
    /// * `id` - Unique node identifier
    /// * `algorithm` - Noise algorithm: "perlin", "simplex", "worley", "value", "fbm"
    /// * `scale` - Noise scale factor (default: 0.1)
    /// * `octaves` - Number of octaves for fractal noise (default: 4)
    /// * `persistence` - Amplitude decay per octave (default: 0.5)
    /// * `lacunarity` - Frequency multiplier per octave (default: 2.0)
    ///
    /// # Returns
    /// A dict matching the TextureProceduralNode with Noise op.
    ///
    /// # Example
    /// ```starlark
    /// noise_node("height", "perlin", 0.1, 4)
    /// noise_node("detail", "simplex", 0.05, 6, 0.5, 2.0)
    /// ```
    fn noise_node<'v>(
        id: &str,
        #[starlark(default = "perlin")] algorithm: &str,
        #[starlark(default = 0.1)] scale: f64,
        #[starlark(default = 4)] octaves: i32,
        #[starlark(default = 0.5)] persistence: f64,
        #[starlark(default = 2.0)] lacunarity: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(id, "noise_node", "id").map_err(|e| anyhow::anyhow!(e))?;
        validate_enum(algorithm, NOISE_ALGORITHMS, "noise_node", "algorithm")
            .map_err(|e| anyhow::anyhow!(e))?;
        if scale <= 0.0 {
            return Err(anyhow::anyhow!(
                "S103: noise_node(): 'scale' must be positive, got {}",
                scale
            ));
        }
        validate_positive_int(octaves as i64, "noise_node", "octaves")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());
        dict.insert_hashed(hashed_key(heap, "type"), heap.alloc_str("noise").to_value());

        // Create noise config
        let mut noise_dict = new_dict(heap);
        noise_dict.insert_hashed(
            hashed_key(heap, "algorithm"),
            heap.alloc_str(algorithm).to_value(),
        );
        noise_dict.insert_hashed(hashed_key(heap, "scale"), heap.alloc(scale).to_value());
        noise_dict.insert_hashed(hashed_key(heap, "octaves"), heap.alloc(octaves).to_value());
        noise_dict.insert_hashed(
            hashed_key(heap, "persistence"),
            heap.alloc(persistence).to_value(),
        );
        noise_dict.insert_hashed(
            hashed_key(heap, "lacunarity"),
            heap.alloc(lacunarity).to_value(),
        );

        dict.insert_hashed(hashed_key(heap, "noise"), heap.alloc(noise_dict).to_value());

        Ok(dict)
    }

    /// Creates a gradient texture node.
    ///
    /// # Arguments
    /// * `id` - Unique node identifier
    /// * `direction` - Gradient direction: "horizontal", "vertical", "radial"
    /// * `start` - Start value (default: 0.0)
    /// * `end` - End value (default: 1.0)
    /// * `center` - Center point for radial gradient [x, y] (optional, default [0.5, 0.5])
    /// * `inner` - Inner radius for radial gradient (optional)
    /// * `outer` - Outer radius for radial gradient (optional)
    ///
    /// # Returns
    /// A dict matching the TextureProceduralNode with Gradient op.
    ///
    /// # Example
    /// ```starlark
    /// gradient_node("grad", "horizontal")
    /// gradient_node("vignette", "radial", 1.0, 0.0)
    /// gradient_node("custom_radial", "radial", 1.0, 0.0, [0.5, 0.5], 0.0, 1.0)
    /// ```
    fn gradient_node<'v>(
        id: &str,
        #[starlark(default = "horizontal")] direction: &str,
        #[starlark(default = 0.0)] start: f64,
        #[starlark(default = 1.0)] end: f64,
        #[starlark(default = NoneType)] center: Value<'v>,
        #[starlark(default = NoneType)] inner: Value<'v>,
        #[starlark(default = NoneType)] outer: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(id, "gradient_node", "id").map_err(|e| anyhow::anyhow!(e))?;
        validate_enum(direction, GRADIENT_DIRECTIONS, "gradient_node", "direction")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());
        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("gradient").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "direction"),
            heap.alloc_str(direction).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "start"), heap.alloc(start).to_value());
        dict.insert_hashed(hashed_key(heap, "end"), heap.alloc(end).to_value());

        // Add optional radial gradient parameters
        if !center.is_none() {
            // Extract list of 2 floats for center
            let list = center.iterate(heap).map_err(|e| {
                anyhow::anyhow!(
                    "S102: gradient_node(): 'center' expected list, got {}: {}",
                    center.get_type(),
                    e
                )
            })?;
            let items: Vec<_> = list.collect();
            if items.len() != 2 {
                return Err(anyhow::anyhow!(
                    "S101: gradient_node(): 'center' must be [x, y], got {} values",
                    items.len()
                ));
            }
            let center_list = heap.alloc(AllocList(vec![items[0], items[1]]));
            dict.insert_hashed(hashed_key(heap, "center"), center_list);
        }

        if !inner.is_none() {
            let inner_val =
                extract_float(inner, "gradient_node", "inner").map_err(|e| anyhow::anyhow!(e))?;
            dict.insert_hashed(hashed_key(heap, "inner"), heap.alloc(inner_val).to_value());
        }

        if !outer.is_none() {
            let outer_val =
                extract_float(outer, "gradient_node", "outer").map_err(|e| anyhow::anyhow!(e))?;
            dict.insert_hashed(hashed_key(heap, "outer"), heap.alloc(outer_val).to_value());
        }

        Ok(dict)
    }

    /// Creates a constant value texture node.
    ///
    /// # Arguments
    /// * `id` - Unique node identifier
    /// * `value` - Constant value (0.0-1.0)
    ///
    /// # Returns
    /// A dict matching the TextureProceduralNode with Constant op.
    ///
    /// # Example
    /// ```starlark
    /// constant_node("white", 1.0)
    /// constant_node("gray", 0.5)
    /// ```
    fn constant_node<'v>(id: &str, value: f64, heap: &'v Heap) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(id, "constant_node", "id").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());
        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("constant").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "value"), heap.alloc(value).to_value());

        Ok(dict)
    }

    /// Creates a threshold operation node.
    ///
    /// # Arguments
    /// * `id` - Unique node identifier
    /// * `input` - Input node id
    /// * `threshold` - Threshold value (default: 0.5)
    ///
    /// # Returns
    /// A dict matching the TextureProceduralNode with Threshold op.
    ///
    /// # Example
    /// ```starlark
    /// threshold_node("mask", "noise", 0.5)
    /// ```
    fn threshold_node<'v>(
        id: &str,
        input: &str,
        #[starlark(default = 0.5)] threshold: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(id, "threshold_node", "id").map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(input, "threshold_node", "input").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());
        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("threshold").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "input"), heap.alloc_str(input).to_value());
        dict.insert_hashed(
            hashed_key(heap, "threshold"),
            heap.alloc(threshold).to_value(),
        );

        Ok(dict)
    }

    /// Creates an invert operation node (1 - x).
    ///
    /// # Arguments
    /// * `id` - Unique node identifier
    /// * `input` - Input node id
    ///
    /// # Returns
    /// A dict matching the TextureProceduralNode with Invert op.
    ///
    /// # Example
    /// ```starlark
    /// invert_node("inverted", "noise")
    /// ```
    fn invert_node<'v>(id: &str, input: &str, heap: &'v Heap) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(id, "invert_node", "id").map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(input, "invert_node", "input").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());
        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("invert").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "input"), heap.alloc_str(input).to_value());

        Ok(dict)
    }

    /// Creates a color ramp mapping node.
    ///
    /// # Arguments
    /// * `id` - Unique node identifier
    /// * `input` - Input node id
    /// * `ramp` - List of hex colors (e.g., ["#000000", "#ffffff"])
    ///
    /// # Returns
    /// A dict matching the TextureProceduralNode with ColorRamp op.
    ///
    /// # Example
    /// ```starlark
    /// color_ramp_node("colored", "noise", ["#000000", "#ff0000", "#ffffff"])
    /// ```
    fn color_ramp_node<'v>(
        id: &str,
        input: &str,
        ramp: UnpackList<&str>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(id, "color_ramp_node", "id").map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(input, "color_ramp_node", "input").map_err(|e| anyhow::anyhow!(e))?;

        if ramp.items.len() < 2 {
            return Err(anyhow::anyhow!(
                "S101: color_ramp_node(): 'ramp' must have at least 2 colors"
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());
        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("color_ramp").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "input"), heap.alloc_str(input).to_value());

        // Convert ramp to list of values
        let ramp_values: Vec<Value> = ramp
            .items
            .iter()
            .map(|c| heap.alloc_str(c).to_value())
            .collect();
        let ramp_list = heap.alloc(AllocList(ramp_values));
        dict.insert_hashed(hashed_key(heap, "ramp"), ramp_list);

        Ok(dict)
    }

    /// Creates an add blend node (a + b).
    ///
    /// # Arguments
    /// * `id` - Unique node identifier
    /// * `a` - First input node id
    /// * `b` - Second input node id
    ///
    /// # Returns
    /// A dict matching the TextureProceduralNode with Add op.
    ///
    /// # Example
    /// ```starlark
    /// add_node("combined", "noise1", "noise2")
    /// ```
    fn add_node<'v>(id: &str, a: &str, b: &str, heap: &'v Heap) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(id, "add_node", "id").map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(a, "add_node", "a").map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(b, "add_node", "b").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());
        dict.insert_hashed(hashed_key(heap, "type"), heap.alloc_str("add").to_value());
        dict.insert_hashed(hashed_key(heap, "a"), heap.alloc_str(a).to_value());
        dict.insert_hashed(hashed_key(heap, "b"), heap.alloc_str(b).to_value());

        Ok(dict)
    }

    /// Creates a multiply blend node (a * b).
    ///
    /// # Arguments
    /// * `id` - Unique node identifier
    /// * `a` - First input node id
    /// * `b` - Second input node id
    ///
    /// # Returns
    /// A dict matching the TextureProceduralNode with Multiply op.
    ///
    /// # Example
    /// ```starlark
    /// multiply_node("masked", "noise", "gradient")
    /// ```
    fn multiply_node<'v>(id: &str, a: &str, b: &str, heap: &'v Heap) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(id, "multiply_node", "id").map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(a, "multiply_node", "a").map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(b, "multiply_node", "b").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());
        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("multiply").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "a"), heap.alloc_str(a).to_value());
        dict.insert_hashed(hashed_key(heap, "b"), heap.alloc_str(b).to_value());

        Ok(dict)
    }

    /// Creates a lerp (linear interpolation) node.
    ///
    /// # Arguments
    /// * `id` - Unique node identifier
    /// * `a` - First input node id
    /// * `b` - Second input node id
    /// * `t` - Interpolation factor node id (0 = a, 1 = b)
    ///
    /// # Returns
    /// A dict matching the TextureProceduralNode with Lerp op.
    ///
    /// # Example
    /// ```starlark
    /// lerp_node("blended", "noise1", "noise2", "mask")
    /// ```
    fn lerp_node<'v>(
        id: &str,
        a: &str,
        b: &str,
        t: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(id, "lerp_node", "id").map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(a, "lerp_node", "a").map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(b, "lerp_node", "b").map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(t, "lerp_node", "t").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());
        dict.insert_hashed(hashed_key(heap, "type"), heap.alloc_str("lerp").to_value());
        dict.insert_hashed(hashed_key(heap, "a"), heap.alloc_str(a).to_value());
        dict.insert_hashed(hashed_key(heap, "b"), heap.alloc_str(b).to_value());
        dict.insert_hashed(hashed_key(heap, "t"), heap.alloc_str(t).to_value());

        Ok(dict)
    }

    /// Creates a clamp node.
    ///
    /// # Arguments
    /// * `id` - Unique node identifier
    /// * `input` - Input node id
    /// * `min` - Minimum value (default: 0.0)
    /// * `max` - Maximum value (default: 1.0)
    ///
    /// # Returns
    /// A dict matching the TextureProceduralNode with Clamp op.
    ///
    /// # Example
    /// ```starlark
    /// clamp_node("clamped", "noise", 0.2, 0.8)
    /// ```
    fn clamp_node<'v>(
        id: &str,
        input: &str,
        #[starlark(default = 0.0)] min: f64,
        #[starlark(default = 1.0)] max: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(id, "clamp_node", "id").map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(input, "clamp_node", "input").map_err(|e| anyhow::anyhow!(e))?;
        if min > max {
            return Err(anyhow::anyhow!(
                "S103: clamp_node(): 'min' ({}) must be <= 'max' ({})",
                min,
                max
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());
        dict.insert_hashed(hashed_key(heap, "type"), heap.alloc_str("clamp").to_value());
        dict.insert_hashed(hashed_key(heap, "input"), heap.alloc_str(input).to_value());
        dict.insert_hashed(hashed_key(heap, "min"), heap.alloc(min).to_value());
        dict.insert_hashed(hashed_key(heap, "max"), heap.alloc(max).to_value());

        Ok(dict)
    }

    /// Creates a stripes pattern node.
    ///
    /// # Arguments
    /// * `id` - Unique node identifier
    /// * `direction` - Stripe direction: "horizontal" or "vertical"
    /// * `stripe_width` - Width of each stripe in pixels
    /// * `color1` - First stripe value (0.0-1.0, default: 0.0)
    /// * `color2` - Second stripe value (0.0-1.0, default: 1.0)
    ///
    /// # Returns
    /// A dict matching the TextureProceduralNode with Stripes op.
    ///
    /// # Example
    /// ```starlark
    /// stripes_node("lines", "horizontal", 4, 0.0, 1.0)
    /// ```
    fn stripes_node<'v>(
        id: &str,
        direction: &str,
        stripe_width: i32,
        #[starlark(default = 0.0)] color1: f64,
        #[starlark(default = 1.0)] color2: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(id, "stripes_node", "id").map_err(|e| anyhow::anyhow!(e))?;
        validate_enum(direction, STRIPE_DIRECTIONS, "stripes_node", "direction")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive_int(stripe_width as i64, "stripes_node", "stripe_width")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(color1, "stripes_node", "color1").map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(color2, "stripes_node", "color2").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());
        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("stripes").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "direction"),
            heap.alloc_str(direction).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "stripe_width"),
            heap.alloc(stripe_width).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "color1"), heap.alloc(color1).to_value());
        dict.insert_hashed(hashed_key(heap, "color2"), heap.alloc(color2).to_value());

        Ok(dict)
    }

    /// Creates a checkerboard pattern node.
    ///
    /// # Arguments
    /// * `id` - Unique node identifier
    /// * `tile_size` - Size of each tile in pixels
    /// * `color1` - First tile value (0.0-1.0, default: 0.0)
    /// * `color2` - Second tile value (0.0-1.0, default: 1.0)
    ///
    /// # Returns
    /// A dict matching the TextureProceduralNode with Checkerboard op.
    ///
    /// # Example
    /// ```starlark
    /// checkerboard_node("checker", 8, 0.0, 1.0)
    /// ```
    fn checkerboard_node<'v>(
        id: &str,
        tile_size: i32,
        #[starlark(default = 0.0)] color1: f64,
        #[starlark(default = 1.0)] color2: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(id, "checkerboard_node", "id").map_err(|e| anyhow::anyhow!(e))?;
        validate_positive_int(tile_size as i64, "checkerboard_node", "tile_size")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(color1, "checkerboard_node", "color1")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(color2, "checkerboard_node", "color2")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());
        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("checkerboard").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "tile_size"),
            heap.alloc(tile_size).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "color1"), heap.alloc(color1).to_value());
        dict.insert_hashed(hashed_key(heap, "color2"), heap.alloc(color2).to_value());

        Ok(dict)
    }

    /// Creates a grayscale conversion node.
    ///
    /// # Arguments
    /// * `id` - Unique node identifier
    /// * `input` - Input color node id
    ///
    /// # Returns
    /// A dict matching the TextureProceduralNode with ToGrayscale op.
    ///
    /// # Example
    /// ```starlark
    /// grayscale_node("gray", "colored_input")
    /// ```
    fn grayscale_node<'v>(id: &str, input: &str, heap: &'v Heap) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(id, "grayscale_node", "id").map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(input, "grayscale_node", "input").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());
        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("to_grayscale").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "input"), heap.alloc_str(input).to_value());

        Ok(dict)
    }

    /// Creates a palette quantization node.
    ///
    /// # Arguments
    /// * `id` - Unique node identifier
    /// * `input` - Input node id
    /// * `palette` - List of hex colors to quantize to
    ///
    /// # Returns
    /// A dict matching the TextureProceduralNode with Palette op.
    ///
    /// # Example
    /// ```starlark
    /// palette_node("retro", "colored", ["#000000", "#ff0000", "#00ff00", "#0000ff"])
    /// ```
    fn palette_node<'v>(
        id: &str,
        input: &str,
        palette: UnpackList<&str>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(id, "palette_node", "id").map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(input, "palette_node", "input").map_err(|e| anyhow::anyhow!(e))?;

        if palette.items.is_empty() {
            return Err(anyhow::anyhow!(
                "S101: palette_node(): 'palette' must have at least 1 color"
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());
        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("palette").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "input"), heap.alloc_str(input).to_value());

        // Convert palette to list of values
        let palette_values: Vec<Value> = palette
            .items
            .iter()
            .map(|c| heap.alloc_str(c).to_value())
            .collect();
        let palette_list = heap.alloc(AllocList(palette_values));
        dict.insert_hashed(hashed_key(heap, "palette"), palette_list);

        Ok(dict)
    }

    /// Creates an RGBA composition node.
    ///
    /// # Arguments
    /// * `id` - Unique node identifier
    /// * `r` - Red channel node id
    /// * `g` - Green channel node id
    /// * `b` - Blue channel node id
    /// * `a` - Alpha channel node id (optional)
    ///
    /// # Returns
    /// A dict matching the TextureProceduralNode with ComposeRgba op.
    ///
    /// # Example
    /// ```starlark
    /// compose_rgba_node("color", "red_channel", "green_channel", "blue_channel")
    /// compose_rgba_node("color_with_alpha", "r", "g", "b", "alpha")
    /// ```
    fn compose_rgba_node<'v>(
        id: &str,
        r: &str,
        g: &str,
        b: &str,
        #[starlark(default = NoneType)] a: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(id, "compose_rgba_node", "id").map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(r, "compose_rgba_node", "r").map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(g, "compose_rgba_node", "g").map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(b, "compose_rgba_node", "b").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());
        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("compose_rgba").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "r"), heap.alloc_str(r).to_value());
        dict.insert_hashed(hashed_key(heap, "g"), heap.alloc_str(g).to_value());
        dict.insert_hashed(hashed_key(heap, "b"), heap.alloc_str(b).to_value());

        // Add optional alpha channel
        if !a.is_none() {
            let a_str = a.unpack_str().ok_or_else(|| {
                anyhow::anyhow!(
                    "S102: compose_rgba_node(): 'a' expected string, got {}",
                    a.get_type()
                )
            })?;
            validate_non_empty(a_str, "compose_rgba_node", "a").map_err(|e| anyhow::anyhow!(e))?;
            dict.insert_hashed(hashed_key(heap, "a"), heap.alloc_str(a_str).to_value());
        }

        Ok(dict)
    }

    /// Creates a normal map from height field node.
    ///
    /// # Arguments
    /// * `id` - Unique node identifier
    /// * `input` - Input height field node id
    /// * `strength` - Normal map strength (default: 1.0)
    ///
    /// # Returns
    /// A dict matching the TextureProceduralNode with NormalFromHeight op.
    ///
    /// # Example
    /// ```starlark
    /// normal_from_height_node("normals", "heightmap", 1.0)
    /// ```
    fn normal_from_height_node<'v>(
        id: &str,
        input: &str,
        #[starlark(default = 1.0)] strength: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(id, "normal_from_height_node", "id").map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(input, "normal_from_height_node", "input")
            .map_err(|e| anyhow::anyhow!(e))?;
        if strength <= 0.0 {
            return Err(anyhow::anyhow!(
                "S103: normal_from_height_node(): 'strength' must be positive, got {}",
                strength
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());
        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("normal_from_height").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "input"), heap.alloc_str(input).to_value());
        dict.insert_hashed(
            hashed_key(heap, "strength"),
            heap.alloc(strength).to_value(),
        );

        Ok(dict)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::eval_to_json;

    // ========================================================================
    // noise_node() tests
    // ========================================================================

    #[test]
    fn test_noise_node_defaults() {
        let result = eval_to_json("noise_node(\"n1\")").unwrap();
        assert_eq!(result["id"], "n1");
        assert_eq!(result["type"], "noise");
        let noise = &result["noise"];
        assert_eq!(noise["algorithm"], "perlin");
        assert_eq!(noise["scale"], 0.1);
        assert_eq!(noise["octaves"], 4);
    }

    #[test]
    fn test_noise_node_custom() {
        let result = eval_to_json("noise_node(\"height\", \"simplex\", 0.05, 6)").unwrap();
        let noise = &result["noise"];
        assert_eq!(noise["algorithm"], "simplex");
        assert_eq!(noise["scale"], 0.05);
        assert_eq!(noise["octaves"], 6);
    }

    #[test]
    fn test_noise_node_invalid_algorithm() {
        let result = eval_to_json("noise_node(\"n\", \"fractal\")");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
        assert!(err.contains("algorithm"));
    }

    // ========================================================================
    // gradient_node() tests
    // ========================================================================

    #[test]
    fn test_gradient_node_defaults() {
        let result = eval_to_json("gradient_node(\"g1\")").unwrap();
        assert_eq!(result["id"], "g1");
        assert_eq!(result["type"], "gradient");
        assert_eq!(result["direction"], "horizontal");
        assert_eq!(result["start"], 0.0);
        assert_eq!(result["end"], 1.0);
    }

    #[test]
    fn test_gradient_node_vertical() {
        let result = eval_to_json("gradient_node(\"g2\", \"vertical\", 0.2, 0.8)").unwrap();
        assert_eq!(result["direction"], "vertical");
        assert_eq!(result["start"], 0.2);
        assert_eq!(result["end"], 0.8);
    }

    #[test]
    fn test_gradient_node_radial_with_params() {
        let result =
            eval_to_json("gradient_node(\"g3\", \"radial\", 1.0, 0.0, [0.5, 0.5], 0.0, 1.0)")
                .unwrap();
        assert_eq!(result["direction"], "radial");
        assert!(result["center"].is_array());
        let center = result["center"].as_array().unwrap();
        assert_eq!(center[0], 0.5);
        assert_eq!(center[1], 0.5);
        assert_eq!(result["inner"], 0.0);
        assert_eq!(result["outer"], 1.0);
    }

    // ========================================================================
    // constant_node() tests
    // ========================================================================

    #[test]
    fn test_constant_node() {
        let result = eval_to_json("constant_node(\"white\", 1.0)").unwrap();
        assert_eq!(result["id"], "white");
        assert_eq!(result["type"], "constant");
        assert_eq!(result["value"], 1.0);
    }

    // ========================================================================
    // threshold_node() tests
    // ========================================================================

    #[test]
    fn test_threshold_node_default() {
        let result = eval_to_json("threshold_node(\"mask\", \"noise\")").unwrap();
        assert_eq!(result["id"], "mask");
        assert_eq!(result["type"], "threshold");
        assert_eq!(result["input"], "noise");
        assert_eq!(result["threshold"], 0.5);
    }

    #[test]
    fn test_threshold_node_custom() {
        let result = eval_to_json("threshold_node(\"binary\", \"height\", 0.7)").unwrap();
        assert_eq!(result["threshold"], 0.7);
    }

    // ========================================================================
    // invert_node() tests
    // ========================================================================

    #[test]
    fn test_invert_node() {
        let result = eval_to_json("invert_node(\"inverted\", \"noise\")").unwrap();
        assert_eq!(result["id"], "inverted");
        assert_eq!(result["type"], "invert");
        assert_eq!(result["input"], "noise");
    }

    // ========================================================================
    // color_ramp_node() tests
    // ========================================================================

    #[test]
    fn test_color_ramp_node() {
        let result =
            eval_to_json("color_ramp_node(\"colored\", \"noise\", [\"#000000\", \"#ffffff\"])")
                .unwrap();
        assert_eq!(result["id"], "colored");
        assert_eq!(result["type"], "color_ramp");
        assert_eq!(result["input"], "noise");
        assert!(result["ramp"].is_array());
        assert_eq!(result["ramp"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_color_ramp_node_too_few_colors() {
        let result = eval_to_json("color_ramp_node(\"c\", \"n\", [\"#000000\"])");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S101"));
        assert!(err.contains("at least 2 colors"));
    }

    // ========================================================================
    // add_node() tests
    // ========================================================================

    #[test]
    fn test_add_node() {
        let result = eval_to_json("add_node(\"combined\", \"noise1\", \"noise2\")").unwrap();
        assert_eq!(result["id"], "combined");
        assert_eq!(result["type"], "add");
        assert_eq!(result["a"], "noise1");
        assert_eq!(result["b"], "noise2");
    }

    // ========================================================================
    // multiply_node() tests
    // ========================================================================

    #[test]
    fn test_multiply_node() {
        let result = eval_to_json("multiply_node(\"masked\", \"noise\", \"gradient\")").unwrap();
        assert_eq!(result["id"], "masked");
        assert_eq!(result["type"], "multiply");
        assert_eq!(result["a"], "noise");
        assert_eq!(result["b"], "gradient");
    }

    // ========================================================================
    // lerp_node() tests
    // ========================================================================

    #[test]
    fn test_lerp_node() {
        let result =
            eval_to_json("lerp_node(\"blended\", \"noise1\", \"noise2\", \"mask\")").unwrap();
        assert_eq!(result["id"], "blended");
        assert_eq!(result["type"], "lerp");
        assert_eq!(result["a"], "noise1");
        assert_eq!(result["b"], "noise2");
        assert_eq!(result["t"], "mask");
    }

    // ========================================================================
    // clamp_node() tests
    // ========================================================================

    #[test]
    fn test_clamp_node_defaults() {
        let result = eval_to_json("clamp_node(\"clamped\", \"noise\")").unwrap();
        assert_eq!(result["id"], "clamped");
        assert_eq!(result["type"], "clamp");
        assert_eq!(result["input"], "noise");
        assert_eq!(result["min"], 0.0);
        assert_eq!(result["max"], 1.0);
    }

    #[test]
    fn test_clamp_node_custom() {
        let result = eval_to_json("clamp_node(\"clamped\", \"noise\", 0.2, 0.8)").unwrap();
        assert_eq!(result["min"], 0.2);
        assert_eq!(result["max"], 0.8);
    }

    // ========================================================================
    // stripes_node() tests
    // ========================================================================

    #[test]
    fn test_stripes_node() {
        let result = eval_to_json("stripes_node(\"lines\", \"horizontal\", 4)").unwrap();
        assert_eq!(result["id"], "lines");
        assert_eq!(result["type"], "stripes");
        assert_eq!(result["direction"], "horizontal");
        assert_eq!(result["stripe_width"], 4);
        assert_eq!(result["color1"], 0.0);
        assert_eq!(result["color2"], 1.0);
    }

    #[test]
    fn test_stripes_node_vertical() {
        let result = eval_to_json("stripes_node(\"vlines\", \"vertical\", 8, 0.3, 0.7)").unwrap();
        assert_eq!(result["direction"], "vertical");
        assert_eq!(result["stripe_width"], 8);
        assert_eq!(result["color1"], 0.3);
        assert_eq!(result["color2"], 0.7);
    }

    // ========================================================================
    // checkerboard_node() tests
    // ========================================================================

    #[test]
    fn test_checkerboard_node() {
        let result = eval_to_json("checkerboard_node(\"checker\", 8)").unwrap();
        assert_eq!(result["id"], "checker");
        assert_eq!(result["type"], "checkerboard");
        assert_eq!(result["tile_size"], 8);
        assert_eq!(result["color1"], 0.0);
        assert_eq!(result["color2"], 1.0);
    }

    // ========================================================================
    // grayscale_node() tests
    // ========================================================================

    #[test]
    fn test_grayscale_node() {
        let result = eval_to_json("grayscale_node(\"gray\", \"colored\")").unwrap();
        assert_eq!(result["id"], "gray");
        assert_eq!(result["type"], "to_grayscale");
        assert_eq!(result["input"], "colored");
    }

    // ========================================================================
    // palette_node() tests
    // ========================================================================

    #[test]
    fn test_palette_node() {
        let result = eval_to_json(
            "palette_node(\"retro\", \"colored\", [\"#000000\", \"#ff0000\", \"#00ff00\"])",
        )
        .unwrap();
        assert_eq!(result["id"], "retro");
        assert_eq!(result["type"], "palette");
        assert_eq!(result["input"], "colored");
        assert!(result["palette"].is_array());
        assert_eq!(result["palette"].as_array().unwrap().len(), 3);
    }

    // ========================================================================
    // compose_rgba_node() tests
    // ========================================================================

    #[test]
    fn test_compose_rgba_node() {
        let result = eval_to_json("compose_rgba_node(\"color\", \"r\", \"g\", \"b\")").unwrap();
        assert_eq!(result["id"], "color");
        assert_eq!(result["type"], "compose_rgba");
        assert_eq!(result["r"], "r");
        assert_eq!(result["g"], "g");
        assert_eq!(result["b"], "b");
        assert!(result.get("a").is_none() || result["a"].is_null());
    }

    #[test]
    fn test_compose_rgba_node_with_alpha() {
        let result =
            eval_to_json("compose_rgba_node(\"color\", \"r\", \"g\", \"b\", \"alpha\")").unwrap();
        assert_eq!(result["a"], "alpha");
    }

    // ========================================================================
    // normal_from_height_node() tests
    // ========================================================================

    #[test]
    fn test_normal_from_height_node() {
        let result = eval_to_json("normal_from_height_node(\"normals\", \"height\")").unwrap();
        assert_eq!(result["id"], "normals");
        assert_eq!(result["type"], "normal_from_height");
        assert_eq!(result["input"], "height");
        assert_eq!(result["strength"], 1.0);
    }

    #[test]
    fn test_normal_from_height_node_custom() {
        let result = eval_to_json("normal_from_height_node(\"normals\", \"height\", 2.5)").unwrap();
        assert_eq!(result["strength"], 2.5);
    }
}
