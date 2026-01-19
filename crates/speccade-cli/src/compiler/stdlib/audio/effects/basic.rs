//! Basic audio effects: reverb, delay, compressor, chorus, phaser, bitcrush

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::{dict::Dict, Heap, Value, ValueLike};

use crate::compiler::stdlib::validation::{validate_positive, validate_unit_range};

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

/// Registers basic effects functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_basic_effects(builder);
}

#[starlark_module]
fn register_basic_effects(builder: &mut GlobalsBuilder) {
    fn reverb<'v>(
        #[starlark(default = 0.5)] decay: f64,
        #[starlark(default = 0.3)] wet: f64,
        #[starlark(default = 0.8)] room_size: f64,
        #[starlark(default = 1.0)] width: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_unit_range(wet, "reverb", "wet").map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(room_size, "reverb", "room_size").map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(width, "reverb", "width").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("reverb").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "room_size"),
            heap.alloc(room_size).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "damping"), heap.alloc(decay).to_value());
        dict.insert_hashed(hashed_key(heap, "wet"), heap.alloc(wet).to_value());
        dict.insert_hashed(hashed_key(heap, "width"), heap.alloc(width).to_value());

        Ok(dict)
    }

    /// Creates a delay effect.
    ///
    /// # Arguments
    /// * `time_ms` - Delay time in milliseconds (default: 250)
    /// * `feedback` - Feedback amount 0.0-1.0 (default: 0.4)
    /// * `wet` - Wet/dry mix 0.0-1.0 (default: 0.3)
    /// * `ping_pong` - Enable stereo ping-pong mode (default: False)
    ///
    /// # Returns
    /// A dict matching the Effect::Delay IR structure.
    ///
    /// # Example
    /// ```starlark
    /// delay()
    /// delay(500, 0.5, 0.4)
    /// delay(250, 0.4, 0.3, True)  # Ping-pong stereo delay
    /// ```
    fn delay<'v>(
        #[starlark(default = 250.0)] time_ms: f64,
        #[starlark(default = 0.4)] feedback: f64,
        #[starlark(default = 0.3)] wet: f64,
        #[starlark(default = false)] ping_pong: bool,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(time_ms, "delay", "time_ms").map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(feedback, "delay", "feedback").map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(wet, "delay", "wet").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "type"), heap.alloc_str("delay").to_value());
        dict.insert_hashed(hashed_key(heap, "time_ms"), heap.alloc(time_ms).to_value());
        dict.insert_hashed(
            hashed_key(heap, "feedback"),
            heap.alloc(feedback).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "wet"), heap.alloc(wet).to_value());
        dict.insert_hashed(
            hashed_key(heap, "ping_pong"),
            heap.alloc(ping_pong).to_value(),
        );

        Ok(dict)
    }

    /// Creates a compressor effect.
    ///
    /// # Arguments
    /// * `threshold_db` - Threshold in dB (default: -12)
    /// * `ratio` - Compression ratio (default: 4)
    /// * `attack_ms` - Attack time in ms (default: 10)
    /// * `release_ms` - Release time in ms (default: 100)
    /// * `makeup_db` - Makeup gain in dB (default: 0)
    ///
    /// # Returns
    /// A dict matching the Effect::Compressor IR structure.
    ///
    /// # Example
    /// ```starlark
    /// compressor()
    /// compressor(-18, 6, 5, 50)
    /// compressor(-12, 4, 10, 100, 3.0)  # With 3dB makeup gain
    /// ```
    fn compressor<'v>(
        #[starlark(default = -12.0)] threshold_db: f64,
        #[starlark(default = 4.0)] ratio: f64,
        #[starlark(default = 10.0)] attack_ms: f64,
        #[starlark(default = 100.0)] release_ms: f64,
        #[starlark(default = 0.0)] makeup_db: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(ratio, "compressor", "ratio").map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(attack_ms, "compressor", "attack_ms").map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(release_ms, "compressor", "release_ms")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("compressor").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "threshold_db"),
            heap.alloc(threshold_db).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "ratio"), heap.alloc(ratio).to_value());
        dict.insert_hashed(
            hashed_key(heap, "attack_ms"),
            heap.alloc(attack_ms).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "release_ms"),
            heap.alloc(release_ms).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "makeup_db"),
            heap.alloc(makeup_db).to_value(),
        );

        Ok(dict)
    }

    /// Creates a chorus effect.
    ///
    /// # Arguments
    /// * `rate` - Modulation rate in Hz (typically ~0.1-10)
    /// * `depth` - Modulation depth 0.0-1.0
    /// * `wet` - Wet/dry mix 0.0-1.0
    /// * `voices` - Number of chorus voices (1-4, default: 2)
    ///
    /// # Returns
    /// A dict matching the Effect::Chorus IR structure.
    ///
    /// # Example
    /// ```starlark
    /// chorus(1.5, 0.3, 0.25)
    /// chorus(0.8, 0.6, 0.4, voices = 4)
    /// ```
    fn chorus<'v>(
        rate: f64,
        depth: f64,
        wet: f64,
        #[starlark(default = 2)] voices: i32,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(rate, "chorus", "rate").map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(depth, "chorus", "depth").map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(wet, "chorus", "wet").map_err(|e| anyhow::anyhow!(e))?;
        if !(1..=4).contains(&voices) {
            return Err(anyhow::anyhow!(
                "S103: chorus(): 'voices' must be 1-4, got {}",
                voices
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("chorus").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "rate"), heap.alloc(rate).to_value());
        dict.insert_hashed(hashed_key(heap, "depth"), heap.alloc(depth).to_value());
        dict.insert_hashed(hashed_key(heap, "wet"), heap.alloc(wet).to_value());
        dict.insert_hashed(hashed_key(heap, "voices"), heap.alloc(voices).to_value());

        Ok(dict)
    }

    /// Creates a phaser effect.
    ///
    /// # Arguments
    /// * `rate` - LFO rate in Hz
    /// * `depth` - Modulation depth 0.0-1.0
    /// * `stages` - Number of allpass stages 2-12
    /// * `wet` - Wet/dry mix 0.0-1.0
    fn phaser<'v>(
        rate: f64,
        depth: f64,
        stages: i32,
        wet: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(rate, "phaser", "rate").map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(depth, "phaser", "depth").map_err(|e| anyhow::anyhow!(e))?;
        if !(2..=12).contains(&stages) {
            return Err(anyhow::anyhow!(
                "S103: phaser(): 'stages' must be 2-12, got {}",
                stages
            ));
        }
        validate_unit_range(wet, "phaser", "wet").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("phaser").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "rate"), heap.alloc(rate).to_value());
        dict.insert_hashed(hashed_key(heap, "depth"), heap.alloc(depth).to_value());
        dict.insert_hashed(hashed_key(heap, "stages"), heap.alloc(stages).to_value());
        dict.insert_hashed(hashed_key(heap, "wet"), heap.alloc(wet).to_value());

        Ok(dict)
    }

    /// Creates a bitcrusher effect.
    ///
    /// # Arguments
    /// * `bits` - Bit depth 1-16
    /// * `sample_rate_reduction` - Sample rate reduction factor (1.0 = no reduction)
    fn bitcrush<'v>(
        bits: i32,
        #[starlark(default = 1.0)] sample_rate_reduction: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if !(1..=16).contains(&bits) {
            return Err(anyhow::anyhow!(
                "S103: bitcrush(): 'bits' must be 1-16, got {}",
                bits
            ));
        }
        if sample_rate_reduction < 1.0 {
            return Err(anyhow::anyhow!(
                "S103: bitcrush(): 'sample_rate_reduction' must be >= 1.0, got {}",
                sample_rate_reduction
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("bitcrush").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "bits"), heap.alloc(bits).to_value());
        dict.insert_hashed(
            hashed_key(heap, "sample_rate_reduction"),
            heap.alloc(sample_rate_reduction).to_value(),
        );

        Ok(dict)
    }
}
