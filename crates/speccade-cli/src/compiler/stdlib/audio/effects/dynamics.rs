//! Dynamics and EQ effects: limiter, parametric_eq, eq_band, transient_shaper

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::{dict::Dict, Heap, Value, ValueLike};

use crate::compiler::stdlib::validation::{validate_enum, validate_pan_range, validate_positive};

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

/// Registers dynamics and EQ effects functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_dynamics_effects(builder);
}

#[starlark_module]
fn register_dynamics_effects(builder: &mut GlobalsBuilder) {
    /// Creates a limiter effect.
    ///
    /// # Arguments
    /// * `threshold_db` - Threshold in dB where limiting begins (-24 to 0)
    /// * `release_ms` - Release time in ms (10-500)
    /// * `lookahead_ms` - Lookahead time in ms (1-10)
    /// * `ceiling_db` - Maximum output level in dB (-6 to 0)
    fn limiter<'v>(
        threshold_db: f64,
        #[starlark(default = 100.0)] release_ms: f64,
        #[starlark(default = 5.0)] lookahead_ms: f64,
        #[starlark(default = -0.3)] ceiling_db: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if !(-24.0..=0.0).contains(&threshold_db) {
            return Err(anyhow::anyhow!(
                "S103: limiter(): 'threshold_db' must be -24 to 0, got {}",
                threshold_db
            ));
        }
        if !(10.0..=500.0).contains(&release_ms) {
            return Err(anyhow::anyhow!(
                "S103: limiter(): 'release_ms' must be 10-500, got {}",
                release_ms
            ));
        }
        if !(1.0..=10.0).contains(&lookahead_ms) {
            return Err(anyhow::anyhow!(
                "S103: limiter(): 'lookahead_ms' must be 1-10, got {}",
                lookahead_ms
            ));
        }
        if !(-6.0..=0.0).contains(&ceiling_db) {
            return Err(anyhow::anyhow!(
                "S103: limiter(): 'ceiling_db' must be -6 to 0, got {}",
                ceiling_db
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("limiter").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "threshold_db"),
            heap.alloc(threshold_db).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "release_ms"),
            heap.alloc(release_ms).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "lookahead_ms"),
            heap.alloc(lookahead_ms).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "ceiling_db"),
            heap.alloc(ceiling_db).to_value(),
        );

        Ok(dict)
    }

    /// Creates a parametric EQ effect.
    ///
    /// # Arguments
    /// * `bands` - List of EQ band dicts from eq_band()
    fn parametric_eq<'v>(bands: Value<'v>, heap: &'v Heap) -> anyhow::Result<Dict<'v>> {
        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("parametric_eq").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "bands"), bands);

        Ok(dict)
    }

    /// Creates an EQ band configuration.
    ///
    /// # Arguments
    /// * `frequency` - Center/corner frequency in Hz
    /// * `gain_db` - Gain in dB (-24 to +24)
    /// * `q` - Q factor (bandwidth), typically 0.1 to 10
    /// * `band_type` - Band type: "lowshelf", "highshelf", "peak", "notch"
    fn eq_band<'v>(
        frequency: f64,
        gain_db: f64,
        q: f64,
        band_type: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "eq_band", "frequency").map_err(|e| anyhow::anyhow!(e))?;
        if !(-24.0..=24.0).contains(&gain_db) {
            return Err(anyhow::anyhow!(
                "S103: eq_band(): 'gain_db' must be -24 to +24, got {}",
                gain_db
            ));
        }
        if !(0.1..=10.0).contains(&q) {
            return Err(anyhow::anyhow!(
                "S103: eq_band(): 'q' must be 0.1-10.0, got {}",
                q
            ));
        }
        const BAND_TYPES: &[&str] = &["lowshelf", "highshelf", "peak", "notch"];
        validate_enum(band_type, BAND_TYPES, "eq_band", "band_type")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "gain_db"), heap.alloc(gain_db).to_value());
        dict.insert_hashed(hashed_key(heap, "q"), heap.alloc(q).to_value());
        dict.insert_hashed(
            hashed_key(heap, "band_type"),
            heap.alloc_str(band_type).to_value(),
        );

        Ok(dict)
    }

    /// Creates a transient shaper effect for controlling attack punch and sustain.
    ///
    /// # Arguments
    /// * `attack` - Attack enhancement (-1.0 to 1.0). Negative = softer transients, positive = more punch.
    /// * `sustain` - Sustain enhancement (-1.0 to 1.0). Negative = tighter, positive = fuller.
    /// * `output_gain_db` - Output makeup gain in dB (-12 to +12).
    ///
    /// # Returns
    /// A dict matching the Effect::TransientShaper IR structure.
    ///
    /// # Example
    /// ```starlark
    /// transient_shaper(attack = 0.5, sustain = -0.3, output_gain_db = 2.0)  # Punchier, tighter
    /// transient_shaper(attack = -0.5, sustain = 0.5, output_gain_db = 0.0)  # Softer attack, fuller body
    /// ```
    #[starlark(speculative_exec_safe)]
    fn transient_shaper<'v>(
        #[starlark(require = named)] attack: f64,
        #[starlark(require = named)] sustain: f64,
        #[starlark(require = named)] output_gain_db: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_pan_range(attack, "transient_shaper", "attack").map_err(|e| anyhow::anyhow!(e))?;
        validate_pan_range(sustain, "transient_shaper", "sustain")
            .map_err(|e| anyhow::anyhow!(e))?;
        if !(-12.0..=12.0).contains(&output_gain_db) {
            return Err(anyhow::anyhow!(
                "S103: transient_shaper(): 'output_gain_db' must be -12 to +12, got {}",
                output_gain_db
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("transient_shaper").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "attack"), heap.alloc(attack).to_value());
        dict.insert_hashed(hashed_key(heap, "sustain"), heap.alloc(sustain).to_value());
        dict.insert_hashed(
            hashed_key(heap, "output_gain_db"),
            heap.alloc(output_gain_db).to_value(),
        );

        Ok(dict)
    }

    /// Creates a true-peak limiter effect for broadcast/streaming compliance.
    ///
    /// Uses oversampling to detect and limit inter-sample peaks that exceed the
    /// ceiling. Essential for meeting loudness standards like EBU R128 or ATSC A/85.
    ///
    /// # Arguments
    /// * `ceiling_db` - Maximum output level in dBTP (-6 to 0). Common values: -1.0 for streaming, -2.0 for broadcast.
    /// * `release_ms` - Release time in ms for gain recovery (10-500).
    ///
    /// # Returns
    /// A dict matching the Effect::TruePeakLimiter IR structure.
    ///
    /// # Example
    /// ```starlark
    /// true_peak_limiter(ceiling_db = -1.0, release_ms = 100.0)  # Streaming
    /// true_peak_limiter(ceiling_db = -2.0, release_ms = 200.0)  # Broadcast
    /// ```
    #[starlark(speculative_exec_safe)]
    fn true_peak_limiter<'v>(
        #[starlark(require = named)] ceiling_db: f64,
        #[starlark(require = named, default = 100.0)] release_ms: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if !(-6.0..=0.0).contains(&ceiling_db) {
            return Err(anyhow::anyhow!(
                "S103: true_peak_limiter(): 'ceiling_db' must be -6 to 0, got {}",
                ceiling_db
            ));
        }
        if !(10.0..=500.0).contains(&release_ms) {
            return Err(anyhow::anyhow!(
                "S103: true_peak_limiter(): 'release_ms' must be 10-500, got {}",
                release_ms
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("true_peak_limiter").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "ceiling_db"),
            heap.alloc(ceiling_db).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "release_ms"),
            heap.alloc(release_ms).to_value(),
        );

        Ok(dict)
    }
}
