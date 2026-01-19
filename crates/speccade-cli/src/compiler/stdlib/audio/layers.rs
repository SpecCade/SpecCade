//! Audio layer composition functions
//!
//! This module provides functions for creating audio layers with envelopes
//! and loop configurations. It includes helpers for creating paired one-shot
//! (attack) and loop (sustain) samples from the same synthesis.
//!
//! ## One-Shot vs Loop Pairing Workflow
//!
//! When creating instrument samples for trackers or samplers, you often need
//! two versions of the same sound:
//!
//! 1. **One-shot (attack)**: A transient sample with full ADSR that plays once.
//!    Use `oneshot_envelope()` for sharp, immediate sounds optimized for
//!    single-play use cases (drums, stabs, SFX).
//!
//! 2. **Loop (sustain)**: A sample designed to loop seamlessly during the
//!    sustain phase. Use `loop_envelope()` plus `with_loop_config()` to create
//!    sounds that sustain indefinitely when held (pads, leads, organs).
//!
//! ## Example: Creating Paired Samples
//!
//! ```starlark
//! # Define common synthesis parameters
//! synth = oscillator(440, "sawtooth")
//!
//! # One-shot version: punchy attack, full decay, no looping
//! oneshot_layer = audio_layer(
//!     synth,
//!     envelope = oneshot_envelope(10, 200, 0.7),  # 10ms attack, 200ms decay
//!     volume = 0.8
//! )
//!
//! # Loop version: quick attack, sustained, configured for looping
//! loop_layer = with_loop_config(
//!     audio_layer(
//!         synth,
//!         envelope = loop_envelope(10, 1.0, 50),  # 10ms attack, full sustain, 50ms release
//!         volume = 0.8
//!     ),
//!     crossfade_samples = 512
//! )
//! ```

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::{dict::Dict, dict::DictRef, none::NoneType, Heap, Value, ValueLike};

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
        function,
        param,
        value.get_type()
    ))
}

/// Helper function to extract an integer from a Starlark Value.
fn extract_int(value: Value, function: &str, param: &str) -> anyhow::Result<i64> {
    if let Some(i) = value.unpack_i32() {
        return Ok(i as i64);
    }
    // Also handle float values that are whole numbers
    if value.get_type() == "float" {
        if let Ok(f) = value.to_str().parse::<f64>() {
            if f.fract() == 0.0 {
                return Ok(f as i64);
            }
        }
    }
    Err(anyhow::anyhow!(
        "S102: {}(): '{}' expected int, got {}",
        function,
        param,
        value.get_type()
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
        validate_unit_range(volume, "audio_layer", "volume").map_err(|e| anyhow::anyhow!(e))?;
        validate_pan_range(pan, "audio_layer", "pan").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        // synthesis is required
        dict.insert_hashed(hashed_key(heap, "synthesis"), synthesis);

        // envelope - use default if not provided
        if envelope.is_none() {
            // Create default envelope inline
            let mut env_dict = new_dict(heap);
            env_dict.insert_hashed(hashed_key(heap, "attack"), heap.alloc(0.01).to_value());
            env_dict.insert_hashed(hashed_key(heap, "decay"), heap.alloc(0.1).to_value());
            env_dict.insert_hashed(hashed_key(heap, "sustain"), heap.alloc(0.5).to_value());
            env_dict.insert_hashed(hashed_key(heap, "release"), heap.alloc(0.2).to_value());
            dict.insert_hashed(
                hashed_key(heap, "envelope"),
                heap.alloc(env_dict).to_value(),
            );
        } else {
            dict.insert_hashed(hashed_key(heap, "envelope"), envelope);
        }

        dict.insert_hashed(hashed_key(heap, "volume"), heap.alloc(volume).to_value());
        dict.insert_hashed(hashed_key(heap, "pan"), heap.alloc(pan).to_value());

        // Optional: filter
        if !filter.is_none() {
            dict.insert_hashed(hashed_key(heap, "filter"), filter);
        }

        // Optional: lfo
        if !lfo.is_none() {
            dict.insert_hashed(hashed_key(heap, "lfo"), lfo);
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
            dict.insert_hashed(hashed_key(heap, "delay"), heap.alloc(delay_val).to_value());
        }

        Ok(dict)
    }

    /// Creates an envelope optimized for one-shot (attack transient) sounds.
    ///
    /// One-shot envelopes are designed for sounds that play once without looping.
    /// They have a sharp attack, a controllable decay, and the sustain level
    /// determines how much of the sound persists after the initial decay phase.
    /// Release is typically short since these sounds are not sustained.
    ///
    /// This is ideal for:
    /// - Drum hits and percussion
    /// - Stabs and plucks
    /// - Sound effects (impacts, UI sounds)
    /// - Attack transients of layered sounds
    ///
    /// # Arguments
    /// * `attack_ms` - Attack time in milliseconds (how fast the sound reaches peak)
    /// * `decay_ms` - Decay time in milliseconds (how fast the sound falls to sustain level)
    /// * `sustain_level` - Sustain level 0.0-1.0 (volume after decay phase)
    ///
    /// # Returns
    /// An envelope dict suitable for use with `audio_layer()`.
    ///
    /// # Example
    /// ```starlark
    /// # Punchy drum hit: fast attack, medium decay, no sustain
    /// oneshot_envelope(5, 150, 0.0)
    ///
    /// # Plucked string: instant attack, slow decay, slight sustain
    /// oneshot_envelope(1, 500, 0.2)
    ///
    /// # Stab: fast attack, short decay, medium sustain
    /// oneshot_envelope(10, 100, 0.5)
    /// ```
    fn oneshot_envelope<'v>(
        attack_ms: f64,
        decay_ms: f64,
        sustain_level: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate ranges
        if attack_ms < 0.0 {
            return Err(anyhow::anyhow!(
                "S103: oneshot_envelope(): 'attack_ms' must be >= 0, got {}",
                attack_ms
            ));
        }
        if decay_ms < 0.0 {
            return Err(anyhow::anyhow!(
                "S103: oneshot_envelope(): 'decay_ms' must be >= 0, got {}",
                decay_ms
            ));
        }
        validate_unit_range(sustain_level, "oneshot_envelope", "sustain_level")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Convert milliseconds to seconds
        let attack = attack_ms / 1000.0;
        let decay = decay_ms / 1000.0;
        // One-shots use a short release (50ms default) since they're not sustained
        let release = 0.05;

        let mut dict = new_dict(heap);
        dict.insert_hashed(hashed_key(heap, "attack"), heap.alloc(attack).to_value());
        dict.insert_hashed(hashed_key(heap, "decay"), heap.alloc(decay).to_value());
        dict.insert_hashed(
            hashed_key(heap, "sustain"),
            heap.alloc(sustain_level).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "release"), heap.alloc(release).to_value());

        Ok(dict)
    }

    /// Creates an envelope optimized for loopable (sustained) sounds.
    ///
    /// Loop envelopes are designed for sounds that will loop during the sustain phase.
    /// They have a quick attack to reach the sustain level, maintain a high sustain
    /// level for seamless looping, and have a controlled release for note-off events.
    ///
    /// This is ideal for:
    /// - Sustained pads and strings
    /// - Organ and synth leads
    /// - Ambient textures
    /// - Any sound that needs to play indefinitely when held
    ///
    /// # Arguments
    /// * `attack_ms` - Attack time in milliseconds (how fast the sound reaches sustain)
    /// * `sustain_level` - Sustain level 0.0-1.0 (typically 0.8-1.0 for loops)
    /// * `release_ms` - Release time in milliseconds (how fast sound fades on note-off)
    ///
    /// # Returns
    /// An envelope dict suitable for use with `audio_layer()`.
    ///
    /// # Example
    /// ```starlark
    /// # Organ: instant attack, full sustain, quick release
    /// loop_envelope(5, 1.0, 30)
    ///
    /// # Pad: slow attack, high sustain, long release
    /// loop_envelope(500, 0.9, 1000)
    ///
    /// # Lead synth: medium attack, full sustain, medium release
    /// loop_envelope(50, 1.0, 200)
    /// ```
    fn loop_envelope<'v>(
        attack_ms: f64,
        sustain_level: f64,
        release_ms: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate ranges
        if attack_ms < 0.0 {
            return Err(anyhow::anyhow!(
                "S103: loop_envelope(): 'attack_ms' must be >= 0, got {}",
                attack_ms
            ));
        }
        validate_unit_range(sustain_level, "loop_envelope", "sustain_level")
            .map_err(|e| anyhow::anyhow!(e))?;
        if release_ms < 0.0 {
            return Err(anyhow::anyhow!(
                "S103: loop_envelope(): 'release_ms' must be >= 0, got {}",
                release_ms
            ));
        }

        // Convert milliseconds to seconds
        let attack = attack_ms / 1000.0;
        let release = release_ms / 1000.0;
        // Loop envelopes use minimal decay since we want to reach sustain quickly
        let decay = 0.01;

        let mut dict = new_dict(heap);
        dict.insert_hashed(hashed_key(heap, "attack"), heap.alloc(attack).to_value());
        dict.insert_hashed(hashed_key(heap, "decay"), heap.alloc(decay).to_value());
        dict.insert_hashed(
            hashed_key(heap, "sustain"),
            heap.alloc(sustain_level).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "release"), heap.alloc(release).to_value());

        Ok(dict)
    }

    /// Adds loop configuration to an audio layer for seamless looping.
    ///
    /// This function takes an existing audio layer dict and adds loop point
    /// configuration to enable seamless looping during the sustain phase.
    /// Use this in combination with `loop_envelope()` for sustained instruments.
    ///
    /// Loop points are placed in the sustain region of the sound (after attack+decay)
    /// and crossfaded to eliminate clicks at loop boundaries.
    ///
    /// # Arguments
    /// * `layer` - An audio layer dict from `audio_layer()`
    /// * `loop_start` - Optional loop start sample position (None = auto-detect after attack+decay)
    /// * `loop_end` - Optional loop end sample position (None = end of audio)
    /// * `crossfade_samples` - Number of samples for crossfade at loop boundary (default: 441 = ~10ms at 44100Hz)
    ///
    /// # Returns
    /// The layer dict with added `loop_config` field.
    ///
    /// # Example
    /// ```starlark
    /// # Basic loop with default settings (auto-detect loop points)
    /// with_loop_config(audio_layer(oscillator(440), envelope = loop_envelope(10, 1.0, 50)))
    ///
    /// # Loop with custom crossfade for smoother transition
    /// with_loop_config(
    ///     audio_layer(oscillator(440), envelope = loop_envelope(10, 1.0, 50)),
    ///     crossfade_samples = 882  # ~20ms at 44100Hz
    /// )
    ///
    /// # Loop with explicit loop points (in samples)
    /// with_loop_config(
    ///     audio_layer(oscillator(440), envelope = loop_envelope(10, 1.0, 50)),
    ///     loop_start = 4410,   # Start at 100ms
    ///     loop_end = 22050,    # End at 500ms
    ///     crossfade_samples = 441
    /// )
    /// ```
    fn with_loop_config<'v>(
        layer: Value<'v>,
        #[starlark(default = NoneType)] loop_start: Value<'v>,
        #[starlark(default = NoneType)] loop_end: Value<'v>,
        #[starlark(default = 441)] crossfade_samples: i32,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate crossfade_samples
        if crossfade_samples < 0 {
            return Err(anyhow::anyhow!(
                "S103: with_loop_config(): 'crossfade_samples' must be >= 0, got {}",
                crossfade_samples
            ));
        }

        // Get the original layer dict
        let layer_dict = DictRef::from_value(layer).ok_or_else(|| {
            anyhow::anyhow!(
                "S102: with_loop_config(): 'layer' expected dict, got {}",
                layer.get_type()
            )
        })?;

        // Create a new dict with all existing layer fields
        let mut new_layer = new_dict(heap);
        for (k, v) in layer_dict.iter() {
            let key = k
                .get_hashed()
                .map_err(|e| anyhow::anyhow!("Failed to hash key: {}", e))?;
            new_layer.insert_hashed(key, v);
        }

        // Build loop_config dict
        let mut loop_config = new_dict(heap);
        loop_config.insert_hashed(hashed_key(heap, "enabled"), heap.alloc(true).to_value());

        // Add optional loop_start
        if !loop_start.is_none() {
            let start_val = extract_int(loop_start, "with_loop_config", "loop_start")?;
            if start_val < 0 {
                return Err(anyhow::anyhow!(
                    "S103: with_loop_config(): 'loop_start' must be >= 0, got {}",
                    start_val
                ));
            }
            loop_config.insert_hashed(
                hashed_key(heap, "start_sample"),
                heap.alloc(start_val as i32).to_value(),
            );
        }

        // Add optional loop_end
        if !loop_end.is_none() {
            let end_val = extract_int(loop_end, "with_loop_config", "loop_end")?;
            if end_val < 0 {
                return Err(anyhow::anyhow!(
                    "S103: with_loop_config(): 'loop_end' must be >= 0, got {}",
                    end_val
                ));
            }
            loop_config.insert_hashed(
                hashed_key(heap, "end_sample"),
                heap.alloc(end_val as i32).to_value(),
            );
        }

        // Convert crossfade_samples to milliseconds (assuming 44100Hz for default)
        // The LoopConfig uses crossfade_ms, so we convert samples to ms
        let crossfade_ms = (crossfade_samples as f64) / 44.1;
        loop_config.insert_hashed(
            hashed_key(heap, "crossfade_ms"),
            heap.alloc(crossfade_ms).to_value(),
        );

        // Enable snap to zero crossing for cleaner loops
        loop_config.insert_hashed(
            hashed_key(heap, "snap_to_zero_crossing"),
            heap.alloc(true).to_value(),
        );

        // Add loop_config to the layer
        new_layer.insert_hashed(
            hashed_key(heap, "loop_config"),
            heap.alloc(loop_config).to_value(),
        );

        Ok(new_layer)
    }
}

#[cfg(test)]
#[path = "layers_tests.rs"]
mod tests;
