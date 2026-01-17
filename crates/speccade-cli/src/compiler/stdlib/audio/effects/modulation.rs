//! Modulation effects: flanger, waveshaper, auto_filter, rotary_speaker, ring_modulator

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::{dict::Dict, Heap, Value, ValueLike};

use crate::compiler::stdlib::validation::{validate_enum, validate_unit_range};

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

/// Registers modulation effects functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_modulation_effects(builder);
}

#[starlark_module]
fn register_modulation_effects(builder: &mut GlobalsBuilder) {
    /// Creates a flanger effect.
    ///
    /// # Arguments
    /// * `rate` - LFO rate in Hz (0.1-10.0)
    /// * `depth` - Modulation depth 0.0-1.0
    /// * `feedback` - Feedback amount -0.99 to 0.99
    /// * `delay_ms` - Base delay time in ms (1-20)
    /// * `wet` - Wet/dry mix 0.0-1.0
    fn flanger<'v>(
        rate: f64,
        depth: f64,
        feedback: f64,
        delay_ms: f64,
        wet: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if !(0.1..=10.0).contains(&rate) {
            return Err(anyhow::anyhow!("S103: flanger(): 'rate' must be 0.1-10.0, got {}", rate));
        }
        validate_unit_range(depth, "flanger", "depth")
            .map_err(|e| anyhow::anyhow!(e))?;
        if !(-0.99..=0.99).contains(&feedback) {
            return Err(anyhow::anyhow!("S103: flanger(): 'feedback' must be -0.99 to 0.99, got {}", feedback));
        }
        if !(1.0..=20.0).contains(&delay_ms) {
            return Err(anyhow::anyhow!("S103: flanger(): 'delay_ms' must be 1-20, got {}", delay_ms));
        }
        validate_unit_range(wet, "flanger", "wet")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("flanger").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "rate"),
            heap.alloc(rate).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "depth"),
            heap.alloc(depth).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "feedback"),
            heap.alloc(feedback).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "delay_ms"),
            heap.alloc(delay_ms).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "wet"),
            heap.alloc(wet).to_value(),
        );

        Ok(dict)
    }

    /// Creates a waveshaper distortion effect.
    ///
    /// # Arguments
    /// * `drive` - Drive amount 1.0-100.0
    /// * `curve` - Shaping curve: "tanh", "soft_clip", "hard_clip", "sine"
    /// * `wet` - Wet/dry mix 0.0-1.0
    fn waveshaper<'v>(
        drive: f64,
        #[starlark(default = "tanh")] curve: &str,
        #[starlark(default = 1.0)] wet: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if !(1.0..=100.0).contains(&drive) {
            return Err(anyhow::anyhow!("S103: waveshaper(): 'drive' must be 1.0-100.0, got {}", drive));
        }
        const CURVES: &[&str] = &["tanh", "soft_clip", "hard_clip", "sine"];
        validate_enum(curve, CURVES, "waveshaper", "curve")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(wet, "waveshaper", "wet")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("waveshaper").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "drive"),
            heap.alloc(drive).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "curve"),
            heap.alloc_str(curve).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "wet"),
            heap.alloc(wet).to_value(),
        );

        Ok(dict)
    }

    /// Creates an auto-filter / envelope follower effect for dynamic filter sweeps.
    ///
    /// # Arguments
    /// * `sensitivity` - How much signal level affects filter (0.0-1.0)
    /// * `attack_ms` - Envelope attack time in ms (0.1-100)
    /// * `release_ms` - Envelope release time in ms (10-1000)
    /// * `depth` - Filter sweep range (0.0-1.0)
    /// * `base_frequency` - Base cutoff frequency when signal is quiet (100-8000 Hz)
    ///
    /// # Returns
    /// A dict matching the Effect::AutoFilter IR structure.
    ///
    /// # Example
    /// ```starlark
    /// auto_filter(sensitivity = 0.7, attack_ms = 5.0, release_ms = 100.0, depth = 0.8, base_frequency = 200.0)
    /// ```
    #[starlark(speculative_exec_safe)]
    fn auto_filter<'v>(
        #[starlark(require = named)] sensitivity: f64,
        #[starlark(require = named)] attack_ms: f64,
        #[starlark(require = named)] release_ms: f64,
        #[starlark(require = named)] depth: f64,
        #[starlark(require = named)] base_frequency: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_unit_range(sensitivity, "auto_filter", "sensitivity")
            .map_err(|e| anyhow::anyhow!(e))?;
        if !(0.1..=100.0).contains(&attack_ms) {
            return Err(anyhow::anyhow!(
                "S103: auto_filter(): 'attack_ms' must be 0.1-100, got {}",
                attack_ms
            ));
        }
        if !(10.0..=1000.0).contains(&release_ms) {
            return Err(anyhow::anyhow!(
                "S103: auto_filter(): 'release_ms' must be 10-1000, got {}",
                release_ms
            ));
        }
        validate_unit_range(depth, "auto_filter", "depth")
            .map_err(|e| anyhow::anyhow!(e))?;
        if !(100.0..=8000.0).contains(&base_frequency) {
            return Err(anyhow::anyhow!(
                "S103: auto_filter(): 'base_frequency' must be 100-8000, got {}",
                base_frequency
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("auto_filter").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "sensitivity"),
            heap.alloc(sensitivity).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "attack_ms"),
            heap.alloc(attack_ms).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "release_ms"),
            heap.alloc(release_ms).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "depth"),
            heap.alloc(depth).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "base_frequency"),
            heap.alloc(base_frequency).to_value(),
        );

        Ok(dict)
    }

    /// Creates a rotary speaker (Leslie) effect with amplitude modulation and Doppler.
    ///
    /// # Arguments
    /// * `rate` - Rotation rate in Hz (0.5-10.0 typical, "slow" ~1 Hz, "fast" ~6 Hz)
    /// * `depth` - Effect intensity (0.0-1.0)
    /// * `wet` - Wet/dry mix (0.0-1.0)
    ///
    /// # Returns
    /// A dict matching the Effect::RotarySpeaker IR structure.
    ///
    /// # Example
    /// ```starlark
    /// rotary_speaker(rate = 1.0, depth = 0.7, wet = 0.8)  # Slow Leslie
    /// rotary_speaker(rate = 6.0, depth = 0.9, wet = 1.0)  # Fast Leslie
    /// ```
    #[starlark(speculative_exec_safe)]
    fn rotary_speaker<'v>(
        #[starlark(require = named)] rate: f64,
        #[starlark(require = named)] depth: f64,
        #[starlark(require = named)] wet: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if !(0.5..=10.0).contains(&rate) {
            return Err(anyhow::anyhow!(
                "S103: rotary_speaker(): 'rate' must be 0.5-10.0, got {}",
                rate
            ));
        }
        validate_unit_range(depth, "rotary_speaker", "depth")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(wet, "rotary_speaker", "wet")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("rotary_speaker").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "rate"),
            heap.alloc(rate).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "depth"),
            heap.alloc(depth).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "wet"),
            heap.alloc(wet).to_value(),
        );

        Ok(dict)
    }

    /// Creates a ring modulator effect that multiplies audio with a carrier oscillator.
    ///
    /// # Arguments
    /// * `frequency` - Carrier oscillator frequency in Hz (20-2000 typical)
    /// * `mix` - Wet/dry mix (0.0-1.0)
    ///
    /// # Returns
    /// A dict matching the Effect::RingModulator IR structure.
    ///
    /// # Example
    /// ```starlark
    /// ring_modulator(frequency = 200.0, mix = 0.5)  # Metallic tones
    /// ring_modulator(frequency = 50.0, mix = 1.0)  # Full ring mod effect
    /// ```
    #[starlark(speculative_exec_safe)]
    fn ring_modulator<'v>(
        #[starlark(require = named)] frequency: f64,
        #[starlark(require = named)] mix: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if !(20.0..=2000.0).contains(&frequency) {
            return Err(anyhow::anyhow!(
                "S103: ring_modulator(): 'frequency' must be 20-2000, got {}",
                frequency
            ));
        }
        validate_unit_range(mix, "ring_modulator", "mix")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("ring_modulator").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "mix"),
            heap.alloc(mix).to_value(),
        );

        Ok(dict)
    }
}
