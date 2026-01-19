//! Cue template functions for adaptive game music.
//!
//! Provides helper functions for creating common music cue patterns used in
//! adaptive game audio systems. These templates help composers create
//! structured cues with appropriate settings for different gameplay contexts.
//!
//! ## Cue Types
//!
//! - **Loop variants**: `loop_low()`, `loop_main()`, `loop_hi()` - Different intensity versions
//!   of loopable music sections (exploration, combat, boss, etc.)
//! - **Stingers**: `stinger()` - Short one-shot musical phrases for events (victory, defeat,
//!   pickup, level-up, etc.)
//! - **Transitions**: `transition()` - Bridge sections between different music states

use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::{dict::Dict, none::NoneType, Heap, Value, ValueLike};

use crate::compiler::stdlib::validation::{
    validate_enum, validate_non_empty, validate_positive_int,
};

use super::cue_layouts::{
    build_loop_cue, build_stinger_track_layout, build_transition_automation_hints,
    build_transition_track_layout,
};
use super::util::{hashed_key, new_dict};

/// Valid intensity levels for loop cues.
const INTENSITY_LEVELS: &[&str] = &["low", "main", "hi"];

/// Valid stinger types.
const STINGER_TYPES: &[&str] = &[
    "victory",
    "defeat",
    "pickup",
    "levelup",
    "discovery",
    "danger",
    "alert",
    "custom",
];

/// Valid transition types.
const TRANSITION_TYPES: &[&str] = &[
    "build",     // Building intensity
    "breakdown", // Reducing intensity
    "bridge",    // Neutral transition
    "fill",      // Short drum fill style
    "custom",    // User-defined
];

/// Registers cue template functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_cue_functions(builder);
}

#[starlark_module]
fn register_cue_functions(builder: &mut GlobalsBuilder) {
    /// Creates a loop cue template with low intensity settings.
    ///
    /// Low intensity loops are typically used for exploration, menus, or calm
    /// gameplay moments. They feature sparse instrumentation and subtle arrangements.
    ///
    /// # Arguments
    /// * `name` - Cue name (required)
    /// * `bpm` - Beats per minute (30-300, default: 90)
    /// * `measures` - Number of measures (1-64, default: 8)
    /// * `rows_per_beat` - Rows per beat for pattern timing (default: 4)
    /// * `channels` - Number of channels (default: 4)
    /// * `format` - Tracker format: "xm" or "it" (default: "xm")
    ///
    /// # Returns
    /// A dict with cue metadata and suggested tracker_song parameters.
    ///
    /// # Example
    /// ```starlark
    /// loop_low(name = "explore_ambient")
    /// loop_low(name = "menu_music", bpm = 80, measures = 16)
    /// ```
    fn loop_low<'v>(
        #[starlark(require = named)] name: &str,
        #[starlark(default = 90)] bpm: i32,
        #[starlark(default = 8)] measures: i32,
        #[starlark(default = 4)] rows_per_beat: i32,
        #[starlark(default = 4)] channels: i32,
        #[starlark(default = "xm")] format: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        build_loop_cue(
            heap,
            name,
            "low",
            bpm,
            measures,
            rows_per_beat,
            channels,
            format,
        )
    }

    /// Creates a loop cue template with main/standard intensity settings.
    ///
    /// Main intensity loops are the default gameplay music. They feature balanced
    /// instrumentation suitable for general gameplay, walking, puzzles, etc.
    ///
    /// # Arguments
    /// * `name` - Cue name (required)
    /// * `bpm` - Beats per minute (30-300, default: 120)
    /// * `measures` - Number of measures (1-64, default: 8)
    /// * `rows_per_beat` - Rows per beat for pattern timing (default: 4)
    /// * `channels` - Number of channels (default: 8)
    /// * `format` - Tracker format: "xm" or "it" (default: "xm")
    ///
    /// # Returns
    /// A dict with cue metadata and suggested tracker_song parameters.
    ///
    /// # Example
    /// ```starlark
    /// loop_main(name = "gameplay_theme")
    /// loop_main(name = "level_1", bpm = 128, channels = 8)
    /// ```
    fn loop_main<'v>(
        #[starlark(require = named)] name: &str,
        #[starlark(default = 120)] bpm: i32,
        #[starlark(default = 8)] measures: i32,
        #[starlark(default = 4)] rows_per_beat: i32,
        #[starlark(default = 8)] channels: i32,
        #[starlark(default = "xm")] format: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        build_loop_cue(
            heap,
            name,
            "main",
            bpm,
            measures,
            rows_per_beat,
            channels,
            format,
        )
    }

    /// Creates a loop cue template with high intensity settings.
    ///
    /// High intensity loops are for action, combat, or boss encounters. They
    /// feature full instrumentation and driving rhythms.
    ///
    /// # Arguments
    /// * `name` - Cue name (required)
    /// * `bpm` - Beats per minute (30-300, default: 140)
    /// * `measures` - Number of measures (1-64, default: 8)
    /// * `rows_per_beat` - Rows per beat for pattern timing (default: 4)
    /// * `channels` - Number of channels (default: 12)
    /// * `format` - Tracker format: "xm" or "it" (default: "xm")
    ///
    /// # Returns
    /// A dict with cue metadata and suggested tracker_song parameters.
    ///
    /// # Example
    /// ```starlark
    /// loop_hi(name = "boss_battle")
    /// loop_hi(name = "combat_intense", bpm = 160, channels = 16)
    /// ```
    fn loop_hi<'v>(
        #[starlark(require = named)] name: &str,
        #[starlark(default = 140)] bpm: i32,
        #[starlark(default = 8)] measures: i32,
        #[starlark(default = 4)] rows_per_beat: i32,
        #[starlark(default = 12)] channels: i32,
        #[starlark(default = "xm")] format: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        build_loop_cue(
            heap,
            name,
            "hi",
            bpm,
            measures,
            rows_per_beat,
            channels,
            format,
        )
    }

    /// Creates a stinger cue template for one-shot musical events.
    ///
    /// Stingers are short, non-looping musical phrases triggered by game events.
    /// They typically play over the current music or interrupt it briefly.
    ///
    /// # Arguments
    /// * `name` - Cue name (required)
    /// * `stinger_type` - Type: "victory", "defeat", "pickup", "levelup", "discovery",
    ///   "danger", "alert", "custom" (default: "custom")
    /// * `duration_beats` - Duration in beats (1-32, default: 4)
    /// * `bpm` - Beats per minute (30-300, inherited from context or default: 120)
    /// * `rows_per_beat` - Rows per beat for pattern timing (default: 4)
    /// * `channels` - Number of channels (default: 4)
    /// * `format` - Tracker format: "xm" or "it" (default: "xm")
    /// * `tail_beats` - Optional decay/reverb tail in beats (default: 0)
    ///
    /// # Returns
    /// A dict with cue metadata and suggested tracker_song parameters.
    ///
    /// # Example
    /// ```starlark
    /// stinger(name = "coin_pickup", stinger_type = "pickup", duration_beats = 2)
    /// stinger(name = "level_complete", stinger_type = "victory", duration_beats = 8)
    /// stinger(name = "enemy_alert", stinger_type = "alert", duration_beats = 2, tail_beats = 2)
    /// ```
    fn stinger<'v>(
        #[starlark(require = named)] name: &str,
        #[starlark(default = "custom")] stinger_type: &str,
        #[starlark(default = 4)] duration_beats: i32,
        #[starlark(default = 120)] bpm: i32,
        #[starlark(default = 4)] rows_per_beat: i32,
        #[starlark(default = 4)] channels: i32,
        #[starlark(default = "xm")] format: &str,
        #[starlark(default = 0)] tail_beats: i32,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(name, "stinger", "name").map_err(|e| anyhow::anyhow!(e))?;
        validate_enum(stinger_type, STINGER_TYPES, "stinger", "stinger_type")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_enum(format, &["xm", "it"], "stinger", "format")
            .map_err(|e| anyhow::anyhow!(e))?;

        if !(30..=300).contains(&bpm) {
            return Err(anyhow::anyhow!(
                "S103: stinger(): 'bpm' must be 30-300, got {}",
                bpm
            ));
        }
        if !(1..=32).contains(&duration_beats) {
            return Err(anyhow::anyhow!(
                "S103: stinger(): 'duration_beats' must be 1-32, got {}",
                duration_beats
            ));
        }
        validate_positive_int(rows_per_beat as i64, "stinger", "rows_per_beat")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive_int(channels as i64, "stinger", "channels")
            .map_err(|e| anyhow::anyhow!(e))?;
        if !(0..=16).contains(&tail_beats) {
            return Err(anyhow::anyhow!(
                "S103: stinger(): 'tail_beats' must be 0-16, got {}",
                tail_beats
            ));
        }

        let max_channels = if format == "xm" { 32 } else { 64 };
        if channels > max_channels {
            return Err(anyhow::anyhow!(
                "S103: stinger(): 'channels' must be 1-{} for {} format, got {}",
                max_channels,
                format,
                channels
            ));
        }

        // Calculate total rows including tail
        let total_beats = duration_beats + tail_beats;
        let total_rows = total_beats * rows_per_beat;

        let mut dict = new_dict(heap);

        // Cue metadata
        dict.insert_hashed(
            hashed_key(heap, "cue_type"),
            heap.alloc_str("stinger").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "name"), heap.alloc_str(name).to_value());
        dict.insert_hashed(
            hashed_key(heap, "stinger_type"),
            heap.alloc_str(stinger_type).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "duration_beats"),
            heap.alloc(duration_beats).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "tail_beats"),
            heap.alloc(tail_beats).to_value(),
        );

        // Timing info
        let mut timing = new_dict(heap);
        timing.insert_hashed(hashed_key(heap, "bpm"), heap.alloc(bpm).to_value());
        timing.insert_hashed(
            hashed_key(heap, "rows_per_beat"),
            heap.alloc(rows_per_beat).to_value(),
        );
        timing.insert_hashed(
            hashed_key(heap, "total_rows"),
            heap.alloc(total_rows).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "timing"), heap.alloc(timing).to_value());

        // Suggested song params (non-looping)
        let mut song_params = new_dict(heap);
        song_params.insert_hashed(
            hashed_key(heap, "format"),
            heap.alloc_str(format).to_value(),
        );
        song_params.insert_hashed(hashed_key(heap, "bpm"), heap.alloc(bpm).to_value());
        song_params.insert_hashed(hashed_key(heap, "speed"), heap.alloc(6).to_value());
        song_params.insert_hashed(
            hashed_key(heap, "channels"),
            heap.alloc(channels).to_value(),
        );
        song_params.insert_hashed(hashed_key(heap, "loop"), heap.alloc(false).to_value());
        song_params.insert_hashed(
            hashed_key(heap, "pattern_rows"),
            heap.alloc(total_rows).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "song_params"),
            heap.alloc(song_params).to_value(),
        );

        // Suggested track layout for stingers (simpler than loops)
        let track_layout = build_stinger_track_layout(heap, stinger_type, channels);
        dict.insert_hashed(
            hashed_key(heap, "track_layout"),
            heap.alloc(track_layout).to_value(),
        );

        Ok(dict)
    }

    /// Creates a transition cue template for bridging between music states.
    ///
    /// Transitions are short sections that smoothly connect different music states.
    /// They can build intensity, break down, or provide a neutral bridge.
    ///
    /// # Arguments
    /// * `name` - Cue name (required)
    /// * `transition_type` - Type: "build", "breakdown", "bridge", "fill", "custom"
    ///   (default: "bridge")
    /// * `from_intensity` - Starting intensity: "low", "main", "hi" (default: "main")
    /// * `to_intensity` - Target intensity: "low", "main", "hi" (default: "main")
    /// * `measures` - Number of measures (1-8, default: 2)
    /// * `bpm` - Beats per minute (30-300, default: 120)
    /// * `rows_per_beat` - Rows per beat for pattern timing (default: 4)
    /// * `channels` - Number of channels (default: 8)
    /// * `format` - Tracker format: "xm" or "it" (default: "xm")
    ///
    /// # Returns
    /// A dict with cue metadata and suggested tracker_song parameters.
    ///
    /// # Example
    /// ```starlark
    /// transition(name = "to_combat", transition_type = "build", from_intensity = "main", to_intensity = "hi")
    /// transition(name = "combat_end", transition_type = "breakdown", from_intensity = "hi", to_intensity = "low")
    /// transition(name = "drum_fill", transition_type = "fill", measures = 1)
    /// ```
    fn transition<'v>(
        #[starlark(require = named)] name: &str,
        #[starlark(default = "bridge")] transition_type: &str,
        #[starlark(default = "main")] from_intensity: &str,
        #[starlark(default = "main")] to_intensity: &str,
        #[starlark(default = 2)] measures: i32,
        #[starlark(default = 120)] bpm: i32,
        #[starlark(default = 4)] rows_per_beat: i32,
        #[starlark(default = 8)] channels: i32,
        #[starlark(default = "xm")] format: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(name, "transition", "name").map_err(|e| anyhow::anyhow!(e))?;
        validate_enum(
            transition_type,
            TRANSITION_TYPES,
            "transition",
            "transition_type",
        )
        .map_err(|e| anyhow::anyhow!(e))?;
        validate_enum(
            from_intensity,
            INTENSITY_LEVELS,
            "transition",
            "from_intensity",
        )
        .map_err(|e| anyhow::anyhow!(e))?;
        validate_enum(to_intensity, INTENSITY_LEVELS, "transition", "to_intensity")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_enum(format, &["xm", "it"], "transition", "format")
            .map_err(|e| anyhow::anyhow!(e))?;

        if !(30..=300).contains(&bpm) {
            return Err(anyhow::anyhow!(
                "S103: transition(): 'bpm' must be 30-300, got {}",
                bpm
            ));
        }
        if !(1..=8).contains(&measures) {
            return Err(anyhow::anyhow!(
                "S103: transition(): 'measures' must be 1-8, got {}",
                measures
            ));
        }
        validate_positive_int(rows_per_beat as i64, "transition", "rows_per_beat")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive_int(channels as i64, "transition", "channels")
            .map_err(|e| anyhow::anyhow!(e))?;

        let max_channels = if format == "xm" { 32 } else { 64 };
        if channels > max_channels {
            return Err(anyhow::anyhow!(
                "S103: transition(): 'channels' must be 1-{} for {} format, got {}",
                max_channels,
                format,
                channels
            ));
        }

        // Calculate total rows (4 beats per measure is standard)
        let beats_per_measure = 4;
        let total_rows = measures * beats_per_measure * rows_per_beat;

        let mut dict = new_dict(heap);

        // Cue metadata
        dict.insert_hashed(
            hashed_key(heap, "cue_type"),
            heap.alloc_str("transition").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "name"), heap.alloc_str(name).to_value());
        dict.insert_hashed(
            hashed_key(heap, "transition_type"),
            heap.alloc_str(transition_type).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "from_intensity"),
            heap.alloc_str(from_intensity).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "to_intensity"),
            heap.alloc_str(to_intensity).to_value(),
        );

        // Timing info
        let mut timing = new_dict(heap);
        timing.insert_hashed(hashed_key(heap, "bpm"), heap.alloc(bpm).to_value());
        timing.insert_hashed(
            hashed_key(heap, "rows_per_beat"),
            heap.alloc(rows_per_beat).to_value(),
        );
        timing.insert_hashed(
            hashed_key(heap, "measures"),
            heap.alloc(measures).to_value(),
        );
        timing.insert_hashed(
            hashed_key(heap, "total_rows"),
            heap.alloc(total_rows).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "timing"), heap.alloc(timing).to_value());

        // Suggested song params (non-looping for transitions)
        let mut song_params = new_dict(heap);
        song_params.insert_hashed(
            hashed_key(heap, "format"),
            heap.alloc_str(format).to_value(),
        );
        song_params.insert_hashed(hashed_key(heap, "bpm"), heap.alloc(bpm).to_value());
        song_params.insert_hashed(hashed_key(heap, "speed"), heap.alloc(6).to_value());
        song_params.insert_hashed(
            hashed_key(heap, "channels"),
            heap.alloc(channels).to_value(),
        );
        song_params.insert_hashed(hashed_key(heap, "loop"), heap.alloc(false).to_value());
        song_params.insert_hashed(
            hashed_key(heap, "pattern_rows"),
            heap.alloc(total_rows).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "song_params"),
            heap.alloc(song_params).to_value(),
        );

        // Suggested track layout based on transition type
        let track_layout =
            build_transition_track_layout(heap, transition_type, from_intensity, to_intensity);
        dict.insert_hashed(
            hashed_key(heap, "track_layout"),
            heap.alloc(track_layout).to_value(),
        );

        // Automation suggestions for transitions
        let automation_hints =
            build_transition_automation_hints(heap, transition_type, from_intensity, to_intensity);
        dict.insert_hashed(
            hashed_key(heap, "automation_hints"),
            heap.alloc(automation_hints).to_value(),
        );

        Ok(dict)
    }

    /// Creates a loop cue with explicit intensity level.
    ///
    /// This is a generic version of loop_low/loop_main/loop_hi that accepts
    /// the intensity as a parameter.
    ///
    /// # Arguments
    /// * `name` - Cue name (required)
    /// * `intensity` - Intensity level: "low", "main", "hi" (required)
    /// * `bpm` - Beats per minute (30-300)
    /// * `measures` - Number of measures (1-64, default: 8)
    /// * `rows_per_beat` - Rows per beat for pattern timing (default: 4)
    /// * `channels` - Number of channels
    /// * `format` - Tracker format: "xm" or "it" (default: "xm")
    ///
    /// # Returns
    /// A dict with cue metadata and suggested tracker_song parameters.
    ///
    /// # Example
    /// ```starlark
    /// loop_cue(name = "ambient", intensity = "low", bpm = 80)
    /// loop_cue(name = "action", intensity = "hi", bpm = 150, channels = 16)
    /// ```
    fn loop_cue<'v>(
        #[starlark(require = named)] name: &str,
        #[starlark(require = named)] intensity: &str,
        #[starlark(default = NoneType)] bpm: Value<'v>,
        #[starlark(default = 8)] measures: i32,
        #[starlark(default = 4)] rows_per_beat: i32,
        #[starlark(default = NoneType)] channels: Value<'v>,
        #[starlark(default = "xm")] format: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_enum(intensity, INTENSITY_LEVELS, "loop_cue", "intensity")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Use intensity-appropriate defaults
        let (default_bpm, default_channels) = match intensity {
            "low" => (90, 4),
            "main" => (120, 8),
            "hi" => (140, 12),
            _ => (120, 8),
        };

        let actual_bpm = if bpm.is_none() {
            default_bpm
        } else {
            bpm.unpack_i32().unwrap_or(default_bpm)
        };

        let actual_channels = if channels.is_none() {
            default_channels
        } else {
            channels.unpack_i32().unwrap_or(default_channels)
        };

        build_loop_cue(
            heap,
            name,
            intensity,
            actual_bpm,
            measures,
            rows_per_beat,
            actual_channels,
            format,
        )
    }
}

#[cfg(test)]
#[path = "cues_tests.rs"]
mod tests;
