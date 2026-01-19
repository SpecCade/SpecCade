//! Track layout and automation hint builders for cue templates.
//!
//! These helpers generate suggested track arrangements and automation hints
//! for different cue types (loops, stingers, transitions).

use starlark::values::dict::Dict;
use starlark::values::list::AllocList;
use starlark::values::{Heap, ValueLike};

use crate::compiler::stdlib::validation::{validate_enum, validate_non_empty, validate_positive_int};
use super::util::{hashed_key, new_dict};

/// Builds a suggested track layout for loop cues based on intensity.
pub fn build_loop_track_layout<'v>(heap: &'v Heap, intensity: &str, channels: i32) -> Dict<'v> {
    let mut layout = new_dict(heap);

    // Build track suggestions based on intensity
    let tracks: Vec<(&str, &str)> = match intensity {
        "low" => {
            // Sparse, ambient arrangement
            vec![
                ("pad", "Ambient pad / drone"),
                ("bass", "Subtle bass"),
                ("melody", "Sparse melodic elements"),
                ("perc", "Light percussion / texture"),
            ]
        }
        "main" => {
            // Balanced arrangement
            vec![
                ("drums", "Main drum pattern"),
                ("bass", "Bass line"),
                ("chord", "Chord / harmony"),
                ("lead", "Lead melody"),
                ("pad", "Background pad"),
                ("perc", "Additional percussion"),
                ("fx", "Sound effects / accents"),
                ("aux", "Auxiliary / fills"),
            ]
        }
        "hi" => {
            // Full, driving arrangement
            vec![
                ("kick", "Kick drum"),
                ("snare", "Snare / clap"),
                ("hihat", "Hi-hat pattern"),
                ("perc", "Additional percussion"),
                ("bass", "Driving bass"),
                ("chord1", "Main chord stabs"),
                ("chord2", "Chord layers"),
                ("lead", "Lead melody"),
                ("counter", "Counter melody"),
                ("pad", "Background pad"),
                ("fx", "Impacts / risers"),
                ("aux", "Fills / transitions"),
            ]
        }
        _ => vec![],
    };

    // Only include as many tracks as we have channels
    let track_count = (tracks.len() as i32).min(channels);
    let suggestions = build_track_suggestions(heap, &tracks, track_count);

    layout.insert_hashed(
        hashed_key(heap, "suggested_tracks"),
        heap.alloc(AllocList(suggestions)).to_value(),
    );
    layout.insert_hashed(
        hashed_key(heap, "intensity"),
        heap.alloc_str(intensity).to_value(),
    );

    layout
}

/// Builds a suggested track layout for stinger cues.
pub fn build_stinger_track_layout<'v>(
    heap: &'v Heap,
    stinger_type: &str,
    channels: i32,
) -> Dict<'v> {
    let mut layout = new_dict(heap);

    // Stinger-specific track suggestions
    let tracks: Vec<(&str, &str)> = match stinger_type {
        "victory" | "levelup" => vec![
            ("fanfare", "Main fanfare melody"),
            ("chord", "Triumphant chord"),
            ("bass", "Bass accent"),
            ("cymbal", "Cymbal crash / shimmer"),
        ],
        "defeat" => vec![
            ("melody", "Descending melody"),
            ("chord", "Minor / diminished chord"),
            ("bass", "Low drone"),
            ("fx", "Sad effect / reverb tail"),
        ],
        "pickup" | "discovery" => vec![
            ("chime", "Bright chime / bell"),
            ("sparkle", "Sparkle effect"),
            ("bass", "Short bass pluck"),
            ("pad", "Quick pad swell"),
        ],
        "danger" | "alert" => vec![
            ("brass", "Warning brass stab"),
            ("strings", "Tense strings"),
            ("bass", "Low rumble"),
            ("fx", "Alert sound effect"),
        ],
        _ => vec![
            ("melody", "Main melody"),
            ("chord", "Harmony"),
            ("bass", "Bass"),
            ("fx", "Effects"),
        ],
    };

    let track_count = (tracks.len() as i32).min(channels);
    let suggestions = build_track_suggestions(heap, &tracks, track_count);

    layout.insert_hashed(
        hashed_key(heap, "suggested_tracks"),
        heap.alloc(AllocList(suggestions)).to_value(),
    );
    layout.insert_hashed(
        hashed_key(heap, "stinger_type"),
        heap.alloc_str(stinger_type).to_value(),
    );

    layout
}

/// Builds a suggested track layout for transition cues.
pub fn build_transition_track_layout<'v>(
    heap: &'v Heap,
    transition_type: &str,
    _from_intensity: &str,
    _to_intensity: &str,
) -> Dict<'v> {
    let mut layout = new_dict(heap);

    let tracks: Vec<(&str, &str)> = match transition_type {
        "build" => vec![
            ("drums", "Building drum pattern"),
            ("bass", "Rising bass line"),
            ("riser", "Synth riser / sweep"),
            ("snare", "Snare roll / buildup"),
            ("fx", "Tension effects"),
        ],
        "breakdown" => vec![
            ("drums", "Fading drums"),
            ("bass", "Descending bass"),
            ("pad", "Releasing pad"),
            ("fx", "Decay effects"),
        ],
        "bridge" => vec![
            ("drums", "Neutral drum pattern"),
            ("bass", "Sustained bass"),
            ("chord", "Bridge chord"),
            ("melody", "Connecting melody"),
        ],
        "fill" => vec![
            ("snare", "Snare fill"),
            ("toms", "Tom pattern"),
            ("crash", "Cymbal accent"),
        ],
        _ => vec![
            ("main", "Main transition element"),
            ("support", "Supporting element"),
            ("fx", "Transition effects"),
        ],
    };

    let suggestions = build_track_suggestions(heap, &tracks, tracks.len() as i32);

    layout.insert_hashed(
        hashed_key(heap, "suggested_tracks"),
        heap.alloc(AllocList(suggestions)).to_value(),
    );
    layout.insert_hashed(
        hashed_key(heap, "transition_type"),
        heap.alloc_str(transition_type).to_value(),
    );

    layout
}

/// Builds automation hints for transition cues.
pub fn build_transition_automation_hints<'v>(
    heap: &'v Heap,
    transition_type: &str,
    from_intensity: &str,
    to_intensity: &str,
) -> Dict<'v> {
    let mut hints = new_dict(heap);

    // Determine volume direction
    let volume_direction = match (from_intensity, to_intensity) {
        ("low", "main") | ("low", "hi") | ("main", "hi") => "crescendo",
        ("hi", "main") | ("hi", "low") | ("main", "low") => "decrescendo",
        _ => "steady",
    };

    hints.insert_hashed(
        hashed_key(heap, "volume_direction"),
        heap.alloc_str(volume_direction).to_value(),
    );

    // Transition-specific suggestions
    let suggestions: Vec<&str> = match transition_type {
        "build" => vec![
            "Use volume_fade to gradually increase levels",
            "Add snare roll with increasing density",
            "Include rising filter sweep via effect commands",
            "Consider tempo_change for dramatic builds",
        ],
        "breakdown" => vec![
            "Use volume_fade to gradually decrease levels",
            "Strip down to sparse elements over time",
            "Include falling filter sweep",
            "End with sustained reverb tail",
        ],
        "bridge" => vec![
            "Maintain consistent energy level",
            "Use melodic variation to connect sections",
            "Keep rhythmic foundation steady",
        ],
        "fill" => vec![
            "Focus on percussion elements",
            "Keep duration short (1-2 beats typical)",
            "End with accent on downbeat",
        ],
        _ => vec!["Customize automation based on musical context"],
    };

    let mut suggestion_values = Vec::new();
    for s in suggestions {
        suggestion_values.push(heap.alloc_str(s).to_value());
    }
    hints.insert_hashed(
        hashed_key(heap, "suggestions"),
        heap.alloc(AllocList(suggestion_values)).to_value(),
    );

    hints
}

/// Helper to build track suggestion dicts from a list of (name, description) pairs.
fn build_track_suggestions<'v>(
    heap: &'v Heap,
    tracks: &[(&str, &str)],
    track_count: i32,
) -> Vec<starlark::values::Value<'v>> {
    let mut suggestions = Vec::new();

    for (i, (track_name, description)) in tracks.iter().enumerate() {
        if i >= track_count as usize {
            break;
        }
        let mut track = new_dict(heap);
        track.insert_hashed(hashed_key(heap, "channel"), heap.alloc(i as i32).to_value());
        track.insert_hashed(
            hashed_key(heap, "name"),
            heap.alloc_str(track_name).to_value(),
        );
        track.insert_hashed(
            hashed_key(heap, "description"),
            heap.alloc_str(description).to_value(),
        );
        suggestions.push(heap.alloc(track).to_value());
    }

    suggestions
}

/// Builds a loop cue template with the specified parameters.
pub fn build_loop_cue<'v>(
    heap: &'v Heap,
    name: &str,
    intensity: &str,
    bpm: i32,
    measures: i32,
    rows_per_beat: i32,
    channels: i32,
    format: &str,
) -> anyhow::Result<Dict<'v>> {
    validate_non_empty(name, "loop_cue", "name").map_err(|e| anyhow::anyhow!(e))?;
    validate_enum(format, &["xm", "it"], "loop_cue", "format").map_err(|e| anyhow::anyhow!(e))?;

    if !(30..=300).contains(&bpm) {
        return Err(anyhow::anyhow!(
            "S103: loop_cue(): 'bpm' must be 30-300, got {}",
            bpm
        ));
    }
    if !(1..=64).contains(&measures) {
        return Err(anyhow::anyhow!(
            "S103: loop_cue(): 'measures' must be 1-64, got {}",
            measures
        ));
    }
    validate_positive_int(rows_per_beat as i64, "loop_cue", "rows_per_beat")
        .map_err(|e| anyhow::anyhow!(e))?;
    validate_positive_int(channels as i64, "loop_cue", "channels")
        .map_err(|e| anyhow::anyhow!(e))?;

    let max_channels = if format == "xm" { 32 } else { 64 };
    if channels > max_channels {
        return Err(anyhow::anyhow!(
            "S103: loop_cue(): 'channels' must be 1-{} for {} format, got {}",
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
        heap.alloc_str("loop").to_value(),
    );
    dict.insert_hashed(hashed_key(heap, "name"), heap.alloc_str(name).to_value());
    dict.insert_hashed(
        hashed_key(heap, "intensity"),
        heap.alloc_str(intensity).to_value(),
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

    // Suggested song params
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
    song_params.insert_hashed(hashed_key(heap, "loop"), heap.alloc(true).to_value());
    song_params.insert_hashed(
        hashed_key(heap, "restart_position"),
        heap.alloc(0).to_value(),
    );
    song_params.insert_hashed(
        hashed_key(heap, "pattern_rows"),
        heap.alloc(total_rows).to_value(),
    );
    dict.insert_hashed(
        hashed_key(heap, "song_params"),
        heap.alloc(song_params).to_value(),
    );

    // Suggested track layout based on intensity
    let track_layout = build_loop_track_layout(heap, intensity, channels);
    dict.insert_hashed(
        hashed_key(heap, "track_layout"),
        heap.alloc(track_layout).to_value(),
    );

    Ok(dict)
}
