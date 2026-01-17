//! Audio layer composition functions

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::{dict::Dict, none::NoneType, Heap, Value, ValueLike};

use super::super::validation::{validate_pan_range, validate_unit_range};

/// Helper to create a hashed key for dict insertion.
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

/// Helper function to extract a float from a Starlark Value.
fn extract_float(value: Value, function: &str, param: &str) -> anyhow::Result<f64> {
    if let Some(f) = value.unpack_i32() {
        return Ok(f as f64);
    }
    if value.get_type() == "float" {
        if let Ok(f) = value.to_str().parse::<f64>() {
            return Ok(f);
        }
    }
    Err(anyhow::anyhow!(
        "S102: {}(): '{}' expected float, got {}",
        function, param, value.get_type()
    ))
}

/// Registers layers functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_layers_functions(builder);
}

#[starlark_module]
fn register_layers_functions(builder: &mut GlobalsBuilder) {
    fn audio_layer<'v>(
        synthesis: Value<'v>,
        #[starlark(default = NoneType)] envelope: Value<'v>,
        #[starlark(default = 0.8)] volume: f64,
        #[starlark(default = 0.0)] pan: f64,
        #[starlark(default = NoneType)] filter: Value<'v>,
        #[starlark(default = NoneType)] lfo: Value<'v>,
        #[starlark(default = NoneType)] delay: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_unit_range(volume, "audio_layer", "volume")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_pan_range(pan, "audio_layer", "pan")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        // synthesis is required
        dict.insert_hashed(
            hashed_key(heap, "synthesis"),
            synthesis,
        );

        // envelope - use default if not provided
        if envelope.is_none() {
            // Create default envelope inline
            let mut env_dict = new_dict(heap);
            env_dict.insert_hashed(
                hashed_key(heap, "attack"),
                heap.alloc(0.01).to_value(),
            );
            env_dict.insert_hashed(
                hashed_key(heap, "decay"),
                heap.alloc(0.1).to_value(),
            );
            env_dict.insert_hashed(
                hashed_key(heap, "sustain"),
                heap.alloc(0.5).to_value(),
            );
            env_dict.insert_hashed(
                hashed_key(heap, "release"),
                heap.alloc(0.2).to_value(),
            );
            dict.insert_hashed(
                hashed_key(heap, "envelope"),
                heap.alloc(env_dict).to_value(),
            );
        } else {
            dict.insert_hashed(
                hashed_key(heap, "envelope"),
                envelope,
            );
        }

        dict.insert_hashed(
            hashed_key(heap, "volume"),
            heap.alloc(volume).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "pan"),
            heap.alloc(pan).to_value(),
        );

        // Optional: filter
        if !filter.is_none() {
            dict.insert_hashed(
                hashed_key(heap, "filter"),
                filter,
            );
        }

        // Optional: lfo
        if !lfo.is_none() {
            dict.insert_hashed(
                hashed_key(heap, "lfo"),
                lfo,
            );
        }

        // Optional: delay
        if !delay.is_none() {
            let delay_val = extract_float(delay, "audio_layer", "delay")?;
            if delay_val < 0.0 {
                return Err(anyhow::anyhow!(
                    "S103: audio_layer(): 'delay' must be >= 0, got {}",
                    delay_val
                ));
            }
            dict.insert_hashed(
                hashed_key(heap, "delay"),
                heap.alloc(delay_val).to_value(),
            );
        }

        Ok(dict)
    }
}
