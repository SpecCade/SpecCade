//! Spatial and delay effects: stereo_widener, delay_tap, multi_tap_delay, tape_saturation, cabinet_sim, granular_delay

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::{dict::Dict, Heap, Value, ValueLike};

use crate::compiler::stdlib::validation::{validate_enum, validate_pan_range, validate_unit_range};

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

/// Registers spatial and delay effects functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_spatial_effects(builder);
}

#[starlark_module]
fn register_spatial_effects(builder: &mut GlobalsBuilder) {
    /// Creates a stereo widener effect.
    ///
    /// # Arguments
    /// * `width` - Stereo width amount (0.0-2.0)
    /// * `mode` - Widening mode: "simple", "haas", or "mid_side" (default: "simple")
    /// * `delay_ms` - Haas delay in milliseconds (1-30, default: 10.0)
    ///
    /// # Returns
    /// A dict matching the Effect::StereoWidener IR structure.
    ///
    /// # Example
    /// ```starlark
    /// stereo_widener(width = 1.25)
    /// stereo_widener(width = 1.5, mode = "haas", delay_ms = 12.0)
    /// ```
    fn stereo_widener<'v>(
        #[starlark(require = named)] width: f64,
        #[starlark(require = named, default = "simple")] mode: &str,
        #[starlark(require = named, default = 10.0)] delay_ms: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if !(0.0..=2.0).contains(&width) {
            return Err(anyhow::anyhow!(
                "S103: stereo_widener(): 'width' must be 0.0-2.0, got {}",
                width
            ));
        }
        const MODES: &[&str] = &["simple", "haas", "mid_side"];
        validate_enum(mode, MODES, "stereo_widener", "mode").map_err(|e| anyhow::anyhow!(e))?;
        if !(1.0..=30.0).contains(&delay_ms) {
            return Err(anyhow::anyhow!(
                "S103: stereo_widener(): 'delay_ms' must be 1-30, got {}",
                delay_ms
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("stereo_widener").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "width"), heap.alloc(width).to_value());
        dict.insert_hashed(hashed_key(heap, "mode"), heap.alloc_str(mode).to_value());
        dict.insert_hashed(
            hashed_key(heap, "delay_ms"),
            heap.alloc(delay_ms).to_value(),
        );

        Ok(dict)
    }

    /// Creates a delay tap configuration for multi_tap_delay().
    ///
    /// # Arguments
    /// * `time_ms` - Delay time in milliseconds (1-2000)
    /// * `feedback` - Feedback amount (0.0-0.99)
    /// * `pan` - Stereo pan position (-1.0 to 1.0)
    /// * `level` - Output level (0.0-1.0)
    /// * `filter_cutoff` - Optional low-pass filter cutoff in Hz (0 = no filter)
    ///
    /// # Returns
    /// A dict matching the DelayTap structure.
    ///
    /// # Example
    /// ```starlark
    /// delay_tap(time_ms = 250, feedback = 0.4, pan = -0.5, level = 0.8)
    /// delay_tap(time_ms = 500, feedback = 0.3, pan = 0.5, level = 0.6, filter_cutoff = 2000)
    /// ```
    #[starlark(speculative_exec_safe)]
    fn delay_tap<'v>(
        #[starlark(require = named)] time_ms: f64,
        #[starlark(require = named)] feedback: f64,
        #[starlark(require = named)] pan: f64,
        #[starlark(require = named)] level: f64,
        #[starlark(require = named, default = 0.0)] filter_cutoff: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if !(1.0..=2000.0).contains(&time_ms) {
            return Err(anyhow::anyhow!(
                "S103: delay_tap(): 'time_ms' must be 1-2000, got {}",
                time_ms
            ));
        }
        if !(0.0..=0.99).contains(&feedback) {
            return Err(anyhow::anyhow!(
                "S103: delay_tap(): 'feedback' must be 0.0-0.99, got {}",
                feedback
            ));
        }
        validate_pan_range(pan, "delay_tap", "pan").map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(level, "delay_tap", "level").map_err(|e| anyhow::anyhow!(e))?;
        if filter_cutoff < 0.0 {
            return Err(anyhow::anyhow!(
                "S103: delay_tap(): 'filter_cutoff' must be >= 0, got {}",
                filter_cutoff
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "time_ms"), heap.alloc(time_ms).to_value());
        dict.insert_hashed(
            hashed_key(heap, "feedback"),
            heap.alloc(feedback).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "pan"), heap.alloc(pan).to_value());
        dict.insert_hashed(hashed_key(heap, "level"), heap.alloc(level).to_value());
        dict.insert_hashed(
            hashed_key(heap, "filter_cutoff"),
            heap.alloc(filter_cutoff).to_value(),
        );

        Ok(dict)
    }

    /// Creates a multi-tap delay effect.
    ///
    /// # Arguments
    /// * `taps` - List of delay tap dicts from delay_tap()
    ///
    /// # Returns
    /// A dict matching the Effect::MultiTapDelay IR structure.
    ///
    /// # Example
    /// ```starlark
    /// multi_tap_delay(taps = [
    ///     delay_tap(time_ms = 250, feedback = 0.4, pan = -0.5, level = 0.8),
    ///     delay_tap(time_ms = 500, feedback = 0.3, pan = 0.5, level = 0.6),
    /// ])
    /// ```
    #[starlark(speculative_exec_safe)]
    fn multi_tap_delay<'v>(
        #[starlark(require = named)] taps: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate taps is a list
        let _taps_iter = taps
            .iterate(heap)
            .map_err(|_| anyhow::anyhow!("S102: multi_tap_delay(): 'taps' must be a list"))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("multi_tap_delay").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "taps"), taps);

        Ok(dict)
    }

    /// Creates a tape saturation effect with warmth, wow/flutter, and hiss.
    ///
    /// # Arguments
    /// * `drive` - Drive/saturation amount (1.0-20.0)
    /// * `bias` - DC bias before saturation (-0.5 to 0.5). Affects harmonic content.
    /// * `wow_rate` - Wow rate in Hz (0.0-3.0). Low-frequency pitch modulation.
    /// * `flutter_rate` - Flutter rate in Hz (0.0-20.0). Higher-frequency pitch modulation.
    /// * `hiss_level` - Tape hiss level (0.0-0.1). Seeded noise added to output.
    ///
    /// # Returns
    /// A dict matching the Effect::TapeSaturation IR structure.
    ///
    /// # Example
    /// ```starlark
    /// tape_saturation(drive = 3.0, bias = 0.1, wow_rate = 0.5, flutter_rate = 5.0, hiss_level = 0.02)
    /// ```
    #[starlark(speculative_exec_safe)]
    fn tape_saturation<'v>(
        #[starlark(require = named)] drive: f64,
        #[starlark(require = named)] bias: f64,
        #[starlark(require = named)] wow_rate: f64,
        #[starlark(require = named)] flutter_rate: f64,
        #[starlark(require = named)] hiss_level: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if !(1.0..=20.0).contains(&drive) {
            return Err(anyhow::anyhow!(
                "S103: tape_saturation(): 'drive' must be 1.0-20.0, got {}",
                drive
            ));
        }
        if !(-0.5..=0.5).contains(&bias) {
            return Err(anyhow::anyhow!(
                "S103: tape_saturation(): 'bias' must be -0.5 to 0.5, got {}",
                bias
            ));
        }
        if !(0.0..=3.0).contains(&wow_rate) {
            return Err(anyhow::anyhow!(
                "S103: tape_saturation(): 'wow_rate' must be 0.0-3.0, got {}",
                wow_rate
            ));
        }
        if !(0.0..=20.0).contains(&flutter_rate) {
            return Err(anyhow::anyhow!(
                "S103: tape_saturation(): 'flutter_rate' must be 0.0-20.0, got {}",
                flutter_rate
            ));
        }
        if !(0.0..=0.1).contains(&hiss_level) {
            return Err(anyhow::anyhow!(
                "S103: tape_saturation(): 'hiss_level' must be 0.0-0.1, got {}",
                hiss_level
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("tape_saturation").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "drive"), heap.alloc(drive).to_value());
        dict.insert_hashed(hashed_key(heap, "bias"), heap.alloc(bias).to_value());
        dict.insert_hashed(
            hashed_key(heap, "wow_rate"),
            heap.alloc(wow_rate).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "flutter_rate"),
            heap.alloc(flutter_rate).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "hiss_level"),
            heap.alloc(hiss_level).to_value(),
        );

        Ok(dict)
    }

    /// Creates a cabinet simulation effect using cascaded biquad filters.
    ///
    /// # Arguments
    /// * `cabinet_type` - Cabinet type: "guitar_1x12", "guitar_4x12", "bass_1x15", "radio", "telephone"
    /// * `mic_position` - Mic position (0.0 = close/bright, 1.0 = far/dark). Default: 0.0
    ///
    /// # Returns
    /// A dict matching the Effect::CabinetSim IR structure.
    ///
    /// # Example
    /// ```starlark
    /// cabinet_sim(cabinet_type = "guitar_4x12")  # Classic 4x12 stack
    /// cabinet_sim(cabinet_type = "radio", mic_position = 0.5)  # AM radio lo-fi
    /// ```
    #[starlark(speculative_exec_safe)]
    fn cabinet_sim<'v>(
        #[starlark(require = named)] cabinet_type: &str,
        #[starlark(require = named, default = 0.0)] mic_position: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        const CABINET_TYPES: &[&str] = &[
            "guitar_1x12",
            "guitar_4x12",
            "bass_1x15",
            "radio",
            "telephone",
        ];
        validate_enum(cabinet_type, CABINET_TYPES, "cabinet_sim", "cabinet_type")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(mic_position, "cabinet_sim", "mic_position")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("cabinet_sim").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "cabinet_type"),
            heap.alloc_str(cabinet_type).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "mic_position"),
            heap.alloc(mic_position).to_value(),
        );

        Ok(dict)
    }

    /// Creates a granular delay effect for shimmer and pitchy delays.
    ///
    /// # Arguments
    /// * `time_ms` - Delay time in milliseconds (10-2000)
    /// * `feedback` - Feedback amount (0.0-0.95)
    /// * `grain_size_ms` - Grain window size in milliseconds (10-200)
    /// * `pitch_semitones` - Pitch shift per grain pass in semitones (-24 to +24)
    /// * `wet` - Wet/dry mix (0.0-1.0)
    ///
    /// # Returns
    /// A dict matching the Effect::GranularDelay IR structure.
    ///
    /// # Example
    /// ```starlark
    /// granular_delay(time_ms = 500, feedback = 0.6, grain_size_ms = 50, pitch_semitones = 12, wet = 0.5)  # Shimmer delay
    /// granular_delay(time_ms = 250, feedback = 0.4, grain_size_ms = 100, pitch_semitones = -7, wet = 0.3)  # Downward pitch
    /// ```
    #[starlark(speculative_exec_safe)]
    fn granular_delay<'v>(
        #[starlark(require = named)] time_ms: f64,
        #[starlark(require = named)] feedback: f64,
        #[starlark(require = named)] grain_size_ms: f64,
        #[starlark(require = named)] pitch_semitones: f64,
        #[starlark(require = named)] wet: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if !(10.0..=2000.0).contains(&time_ms) {
            return Err(anyhow::anyhow!(
                "S103: granular_delay(): 'time_ms' must be 10-2000, got {}",
                time_ms
            ));
        }
        if !(0.0..=0.95).contains(&feedback) {
            return Err(anyhow::anyhow!(
                "S103: granular_delay(): 'feedback' must be 0.0-0.95, got {}",
                feedback
            ));
        }
        if !(10.0..=200.0).contains(&grain_size_ms) {
            return Err(anyhow::anyhow!(
                "S103: granular_delay(): 'grain_size_ms' must be 10-200, got {}",
                grain_size_ms
            ));
        }
        if !(-24.0..=24.0).contains(&pitch_semitones) {
            return Err(anyhow::anyhow!(
                "S103: granular_delay(): 'pitch_semitones' must be -24 to +24, got {}",
                pitch_semitones
            ));
        }
        validate_unit_range(wet, "granular_delay", "wet").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("granular_delay").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "time_ms"), heap.alloc(time_ms).to_value());
        dict.insert_hashed(
            hashed_key(heap, "feedback"),
            heap.alloc(feedback).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "grain_size_ms"),
            heap.alloc(grain_size_ms).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "pitch_semitones"),
            heap.alloc(pitch_semitones).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "wet"), heap.alloc(wet).to_value());

        Ok(dict)
    }
}
