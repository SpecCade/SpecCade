//! Foley sound effect helpers for layered impact and whoosh sounds.
//!
//! This module provides helper functions for creating layered foley sound effects,
//! specifically designed for common game audio needs like impacts (hits, punches,
//! explosions) and whooshes (swings, flyby, UI transitions).
//!
//! ## Impact Sound Design
//!
//! Impact sounds are typically composed of three distinct layers:
//!
//! 1. **Transient**: The initial attack/click that gives punch and presence.
//!    Usually very short (< 50ms) with high-frequency content.
//!
//! 2. **Body**: The main character of the impact that conveys weight and material.
//!    Medium duration with the fundamental tone/noise.
//!
//! 3. **Tail**: The resonance and decay that adds space and realism.
//!    Longer duration, often lower frequency content.
//!
//! ## Whoosh Sound Design
//!
//! Whoosh sounds are characterized by:
//!
//! - Filtered noise with a frequency sweep
//! - Direction (rising or falling pitch)
//! - Temporal envelope that shapes the intensity
//!
//! ## Example: Creating an Impact Sound
//!
//! ```starlark
//! # Define impact layers
//! layers = impact_builder(
//!     transient = {
//!         "synthesis": noise_burst("white"),
//!         "envelope": oneshot_envelope(1, 20, 0.0),
//!         "volume": 0.7,
//!         "filter": highpass(2000, 1.5)
//!     },
//!     body = {
//!         "synthesis": oscillator(100, "sine", 60),
//!         "envelope": oneshot_envelope(5, 80, 0.1),
//!         "volume": 0.9,
//!         "filter": lowpass(500, 1.0)
//!     },
//!     tail = {
//!         "synthesis": noise_burst("brown"),
//!         "envelope": oneshot_envelope(10, 300, 0.0),
//!         "volume": 0.4
//!     }
//! )
//! ```
//!
//! ## Example: Creating a Whoosh Sound
//!
//! ```starlark
//! # Rising whoosh (e.g., sword swing upward)
//! layer = whoosh_builder(
//!     direction = "rising",
//!     duration_ms = 200,
//!     start_freq = 300,
//!     end_freq = 2000
//! )
//!
//! # Falling whoosh (e.g., object flying past)
//! layer = whoosh_builder(
//!     direction = "falling",
//!     duration_ms = 300,
//!     start_freq = 1500,
//!     end_freq = 200,
//!     noise_type = "pink"
//! )
//! ```

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::list::AllocList;
use starlark::values::{dict::Dict, dict::DictRef, Heap, Value, ValueLike};

use super::super::validation::{validate_enum, validate_positive};

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

/// Valid noise types for whoosh sounds.
const NOISE_TYPES: &[&str] = &["white", "pink", "brown"];

/// Valid directions for whoosh sounds.
const WHOOSH_DIRECTIONS: &[&str] = &["rising", "falling"];

/// Registers foley functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_foley_functions(builder);
}

/// Helper to get a value from a dict by string key.
fn get_dict_value<'v>(dict: &DictRef<'v>, key: &str, _heap: &'v Heap) -> Option<Value<'v>> {
    for (k, v) in dict.iter() {
        if let Some(k_str) = k.unpack_str() {
            if k_str == key {
                return Some(v);
            }
        }
    }
    None
}

/// Helper to extract a float value from a dict field.
fn extract_float_from_value(value: Value, function: &str, param: &str) -> anyhow::Result<f64> {
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
        function,
        param,
        value.get_type()
    ))
}

/// Build an audio layer dict from layer config components.
fn build_layer_from_config<'v>(
    config: &DictRef<'v>,
    function: &str,
    layer_name: &str,
    heap: &'v Heap,
) -> anyhow::Result<Dict<'v>> {
    let mut layer = new_dict(heap);

    // synthesis is required
    let synthesis = get_dict_value(config, "synthesis", heap).ok_or_else(|| {
        anyhow::anyhow!(
            "S101: {}(): '{}' dict must have 'synthesis' field",
            function,
            layer_name
        )
    })?;
    layer.insert_hashed(hashed_key(heap, "synthesis"), synthesis);

    // envelope is required
    let envelope = get_dict_value(config, "envelope", heap).ok_or_else(|| {
        anyhow::anyhow!(
            "S101: {}(): '{}' dict must have 'envelope' field",
            function,
            layer_name
        )
    })?;
    layer.insert_hashed(hashed_key(heap, "envelope"), envelope);

    // volume is required
    let volume_value = get_dict_value(config, "volume", heap).ok_or_else(|| {
        anyhow::anyhow!(
            "S101: {}(): '{}' dict must have 'volume' field",
            function,
            layer_name
        )
    })?;
    let volume =
        extract_float_from_value(volume_value, function, &format!("{}.volume", layer_name))?;
    if !(0.0..=1.0).contains(&volume) {
        return Err(anyhow::anyhow!(
            "S103: {}(): '{}.volume' must be in range 0.0 to 1.0, got {}",
            function,
            layer_name,
            volume
        ));
    }
    layer.insert_hashed(hashed_key(heap, "volume"), heap.alloc(volume).to_value());

    // pan defaults to 0.0
    layer.insert_hashed(hashed_key(heap, "pan"), heap.alloc(0.0).to_value());

    // filter is optional
    if let Some(filter) = get_dict_value(config, "filter", heap) {
        layer.insert_hashed(hashed_key(heap, "filter"), filter);
    }

    Ok(layer)
}

#[starlark_module]
fn register_foley_functions(builder: &mut GlobalsBuilder) {
    /// Creates a layered impact sound effect from three component layers.
    ///
    /// Impact sounds are composed of three distinct layers that together create
    /// the perception of a collision or hit:
    ///
    /// - **Transient**: The initial click/snap that gives punch (1-50ms)
    /// - **Body**: The main tonal/noise character that conveys weight (20-200ms)
    /// - **Tail**: The decay/resonance that adds space (100-1000ms)
    ///
    /// Each layer is specified as a dict with:
    /// - `synthesis`: Synthesis dict (from oscillator(), noise_burst(), etc.)
    /// - `envelope`: Envelope dict (typically from oneshot_envelope())
    /// - `volume`: Layer volume 0.0-1.0
    /// - `filter`: Optional filter dict (from lowpass(), highpass(), bandpass())
    ///
    /// # Arguments
    /// * `transient` - Dict defining the transient layer
    /// * `body` - Dict defining the body layer
    /// * `tail` - Dict defining the tail layer
    ///
    /// # Returns
    /// A list of three audio_layer dicts ready for use in an audio spec.
    ///
    /// # Example
    /// ```starlark
    /// # Punchy impact with high-frequency click, low body, and rumble tail
    /// impact_builder(
    ///     transient = {
    ///         "synthesis": noise_burst("white"),
    ///         "envelope": oneshot_envelope(1, 15, 0.0),
    ///         "volume": 0.6,
    ///         "filter": highpass(3000, 2.0)
    ///     },
    ///     body = {
    ///         "synthesis": oscillator(80, "sine", 50),
    ///         "envelope": oneshot_envelope(3, 60, 0.0),
    ///         "volume": 0.8,
    ///         "filter": lowpass(200, 0.8)
    ///     },
    ///     tail = {
    ///         "synthesis": noise_burst("brown"),
    ///         "envelope": oneshot_envelope(20, 400, 0.0),
    ///         "volume": 0.3
    ///     }
    /// )
    /// ```
    fn impact_builder<'v>(
        #[starlark(require = named)] transient: Value<'v>,
        #[starlark(require = named)] body: Value<'v>,
        #[starlark(require = named)] tail: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Value<'v>> {
        // Validate transient is a dict
        let transient_dict = DictRef::from_value(transient).ok_or_else(|| {
            anyhow::anyhow!(
                "S102: impact_builder(): 'transient' expected dict, got {}",
                transient.get_type()
            )
        })?;

        // Validate body is a dict
        let body_dict = DictRef::from_value(body).ok_or_else(|| {
            anyhow::anyhow!(
                "S102: impact_builder(): 'body' expected dict, got {}",
                body.get_type()
            )
        })?;

        // Validate tail is a dict
        let tail_dict = DictRef::from_value(tail).ok_or_else(|| {
            anyhow::anyhow!(
                "S102: impact_builder(): 'tail' expected dict, got {}",
                tail.get_type()
            )
        })?;

        // Build each layer
        let transient_layer =
            build_layer_from_config(&transient_dict, "impact_builder", "transient", heap)?;
        let body_layer = build_layer_from_config(&body_dict, "impact_builder", "body", heap)?;
        let tail_layer = build_layer_from_config(&tail_dict, "impact_builder", "tail", heap)?;

        // Return as a list of layers
        let layers = vec![
            heap.alloc(transient_layer).to_value(),
            heap.alloc(body_layer).to_value(),
            heap.alloc(tail_layer).to_value(),
        ];

        Ok(heap.alloc(AllocList(layers)))
    }

    /// Creates a whoosh sound effect with a filtered noise sweep.
    ///
    /// Whoosh sounds are created using bandpass-filtered noise with a frequency
    /// sweep. The direction determines whether the filter sweeps up (rising)
    /// or down (falling) over the duration.
    ///
    /// Common uses:
    /// - Sword/weapon swings (rising, 100-300ms)
    /// - Objects flying past (falling then rising, 200-500ms)
    /// - UI transitions (either direction, 50-200ms)
    /// - Wind gusts (slow sweep, 500-2000ms)
    ///
    /// # Arguments
    /// * `direction` - Sweep direction: "rising" or "falling"
    /// * `duration_ms` - Duration in milliseconds
    /// * `start_freq` - Starting bandpass center frequency in Hz
    /// * `end_freq` - Ending bandpass center frequency in Hz
    /// * `noise_type` - Noise type: "white", "pink", "brown" (default: "white")
    ///
    /// # Returns
    /// A single audio_layer dict with noise synthesis and bandpass filter sweep.
    ///
    /// # Example
    /// ```starlark
    /// # Quick rising whoosh for sword swing
    /// whoosh_builder(
    ///     direction = "rising",
    ///     duration_ms = 150,
    ///     start_freq = 400,
    ///     end_freq = 2500
    /// )
    ///
    /// # Slow falling whoosh for descending object
    /// whoosh_builder(
    ///     direction = "falling",
    ///     duration_ms = 400,
    ///     start_freq = 2000,
    ///     end_freq = 300,
    ///     noise_type = "pink"
    /// )
    /// ```
    fn whoosh_builder<'v>(
        #[starlark(require = named)] direction: &str,
        #[starlark(require = named)] duration_ms: f64,
        #[starlark(require = named)] start_freq: f64,
        #[starlark(require = named)] end_freq: f64,
        #[starlark(default = "white")] noise_type: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate direction
        validate_enum(direction, WHOOSH_DIRECTIONS, "whoosh_builder", "direction")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate duration
        validate_positive(duration_ms, "whoosh_builder", "duration_ms")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate frequencies
        validate_positive(start_freq, "whoosh_builder", "start_freq")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(end_freq, "whoosh_builder", "end_freq")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate noise type
        validate_enum(noise_type, NOISE_TYPES, "whoosh_builder", "noise_type")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Build synthesis dict (noise_burst)
        let mut synthesis = new_dict(heap);
        synthesis.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("noise_burst").to_value(),
        );
        synthesis.insert_hashed(
            hashed_key(heap, "noise_type"),
            heap.alloc_str(noise_type).to_value(),
        );

        // Build envelope - whoosh shape with attack in first third, decay in last two thirds
        // Convert ms to seconds
        let duration_sec = duration_ms / 1000.0;
        let attack = duration_sec * 0.3; // First 30% is attack
        let decay = duration_sec * 0.1; // Small decay
        let release = duration_sec * 0.6; // Last 60% is release

        let mut envelope = new_dict(heap);
        envelope.insert_hashed(hashed_key(heap, "attack"), heap.alloc(attack).to_value());
        envelope.insert_hashed(hashed_key(heap, "decay"), heap.alloc(decay).to_value());
        envelope.insert_hashed(hashed_key(heap, "sustain"), heap.alloc(0.8).to_value());
        envelope.insert_hashed(hashed_key(heap, "release"), heap.alloc(release).to_value());

        // Build bandpass filter with sweep
        // For rising: start_freq -> end_freq (typically low to high)
        // For falling: start_freq -> end_freq (typically high to low)
        let (filter_start, filter_end) = match direction {
            "rising" => (start_freq, end_freq),
            "falling" => (start_freq, end_freq),
            _ => unreachable!(), // Already validated
        };

        let mut filter = new_dict(heap);
        filter.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("bandpass").to_value(),
        );
        filter.insert_hashed(
            hashed_key(heap, "center"),
            heap.alloc(filter_start).to_value(),
        );
        // Use moderate resonance for characteristic whoosh sound
        filter.insert_hashed(hashed_key(heap, "resonance"), heap.alloc(2.0).to_value());
        // Add sweep endpoint
        filter.insert_hashed(
            hashed_key(heap, "center_end"),
            heap.alloc(filter_end).to_value(),
        );

        // Build the audio layer
        let mut layer = new_dict(heap);
        layer.insert_hashed(
            hashed_key(heap, "synthesis"),
            heap.alloc(synthesis).to_value(),
        );
        layer.insert_hashed(
            hashed_key(heap, "envelope"),
            heap.alloc(envelope).to_value(),
        );
        layer.insert_hashed(hashed_key(heap, "volume"), heap.alloc(0.8).to_value());
        layer.insert_hashed(hashed_key(heap, "pan"), heap.alloc(0.0).to_value());
        layer.insert_hashed(hashed_key(heap, "filter"), heap.alloc(filter).to_value());

        Ok(layer)
    }
}

#[cfg(test)]
#[path = "foley_tests.rs"]
mod tests;
