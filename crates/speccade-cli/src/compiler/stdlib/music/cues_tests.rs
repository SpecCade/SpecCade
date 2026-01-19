//! Tests for cue template functions.

use crate::compiler::stdlib::tests::eval_to_json;

#[test]
fn test_loop_low_defaults() {
    let result = eval_to_json("loop_low(name = \"ambient\")").unwrap();
    assert_eq!(result["cue_type"], "loop");
    assert_eq!(result["name"], "ambient");
    assert_eq!(result["intensity"], "low");
    assert_eq!(result["timing"]["bpm"], 90);
    assert_eq!(result["song_params"]["loop"], true);
    assert_eq!(result["song_params"]["channels"], 4);
}

#[test]
fn test_loop_main_defaults() {
    let result = eval_to_json("loop_main(name = \"gameplay\")").unwrap();
    assert_eq!(result["cue_type"], "loop");
    assert_eq!(result["intensity"], "main");
    assert_eq!(result["timing"]["bpm"], 120);
    assert_eq!(result["song_params"]["channels"], 8);
}

#[test]
fn test_loop_hi_defaults() {
    let result = eval_to_json("loop_hi(name = \"combat\")").unwrap();
    assert_eq!(result["cue_type"], "loop");
    assert_eq!(result["intensity"], "hi");
    assert_eq!(result["timing"]["bpm"], 140);
    assert_eq!(result["song_params"]["channels"], 12);
}

#[test]
fn test_loop_low_custom_params() {
    let result =
        eval_to_json("loop_low(name = \"menu\", bpm = 80, measures = 16, channels = 6)").unwrap();
    assert_eq!(result["timing"]["bpm"], 80);
    assert_eq!(result["timing"]["measures"], 16);
    assert_eq!(result["song_params"]["channels"], 6);
}

#[test]
fn test_loop_cue_explicit_intensity() {
    let result = eval_to_json("loop_cue(name = \"test\", intensity = \"hi\")").unwrap();
    assert_eq!(result["intensity"], "hi");
    assert_eq!(result["timing"]["bpm"], 140); // hi default
    assert_eq!(result["song_params"]["channels"], 12); // hi default
}

#[test]
fn test_loop_cue_custom_bpm() {
    let result =
        eval_to_json("loop_cue(name = \"custom\", intensity = \"main\", bpm = 100)").unwrap();
    assert_eq!(result["timing"]["bpm"], 100);
}

#[test]
fn test_loop_invalid_bpm() {
    let result = eval_to_json("loop_main(name = \"test\", bpm = 500)");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
    assert!(err.contains("bpm"));
}

#[test]
fn test_loop_invalid_measures() {
    let result = eval_to_json("loop_main(name = \"test\", measures = 100)");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
}

#[test]
fn test_loop_empty_name() {
    let result = eval_to_json("loop_main(name = \"\")");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S101"));
}

#[test]
fn test_loop_track_layout() {
    let result = eval_to_json("loop_main(name = \"test\")").unwrap();
    assert!(result["track_layout"]["suggested_tracks"].is_array());
    let tracks = result["track_layout"]["suggested_tracks"]
        .as_array()
        .unwrap();
    assert!(!tracks.is_empty());
    assert!(tracks[0]["name"].is_string());
    assert!(tracks[0]["description"].is_string());
}

#[test]
fn test_stinger_defaults() {
    let result = eval_to_json("stinger(name = \"pickup\")").unwrap();
    assert_eq!(result["cue_type"], "stinger");
    assert_eq!(result["name"], "pickup");
    assert_eq!(result["stinger_type"], "custom");
    assert_eq!(result["duration_beats"], 4);
    assert_eq!(result["song_params"]["loop"], false);
}

#[test]
fn test_stinger_victory() {
    let result =
        eval_to_json("stinger(name = \"win\", stinger_type = \"victory\", duration_beats = 8)")
            .unwrap();
    assert_eq!(result["stinger_type"], "victory");
    assert_eq!(result["duration_beats"], 8);
    // Check track layout has victory-appropriate tracks
    let tracks = result["track_layout"]["suggested_tracks"]
        .as_array()
        .unwrap();
    assert!(tracks.iter().any(|t| t["name"] == "fanfare"));
}

#[test]
fn test_stinger_with_tail() {
    let result = eval_to_json(
        "stinger(name = \"alert\", stinger_type = \"alert\", duration_beats = 2, tail_beats = 2)",
    )
    .unwrap();
    assert_eq!(result["duration_beats"], 2);
    assert_eq!(result["tail_beats"], 2);
    // Total rows should include tail
    let rows_per_beat = result["timing"]["rows_per_beat"].as_i64().unwrap();
    let total_rows = result["timing"]["total_rows"].as_i64().unwrap();
    assert_eq!(total_rows, 4 * rows_per_beat); // 2 + 2 beats
}

#[test]
fn test_stinger_invalid_type() {
    let result = eval_to_json("stinger(name = \"test\", stinger_type = \"invalid\")");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S104"));
}

#[test]
fn test_stinger_invalid_duration() {
    let result = eval_to_json("stinger(name = \"test\", duration_beats = 100)");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
}

#[test]
fn test_transition_defaults() {
    let result = eval_to_json("transition(name = \"bridge\")").unwrap();
    assert_eq!(result["cue_type"], "transition");
    assert_eq!(result["transition_type"], "bridge");
    assert_eq!(result["from_intensity"], "main");
    assert_eq!(result["to_intensity"], "main");
    assert_eq!(result["timing"]["measures"], 2);
    assert_eq!(result["song_params"]["loop"], false);
}

#[test]
fn test_transition_build() {
    let result = eval_to_json(
        "transition(name = \"to_combat\", transition_type = \"build\", from_intensity = \"main\", to_intensity = \"hi\")",
    ).unwrap();
    assert_eq!(result["transition_type"], "build");
    assert_eq!(result["from_intensity"], "main");
    assert_eq!(result["to_intensity"], "hi");
    // Check automation hints
    assert_eq!(result["automation_hints"]["volume_direction"], "crescendo");
}

#[test]
fn test_transition_breakdown() {
    let result = eval_to_json(
        "transition(name = \"calm_down\", transition_type = \"breakdown\", from_intensity = \"hi\", to_intensity = \"low\")",
    ).unwrap();
    assert_eq!(result["transition_type"], "breakdown");
    assert_eq!(
        result["automation_hints"]["volume_direction"],
        "decrescendo"
    );
}

#[test]
fn test_transition_fill() {
    let result =
        eval_to_json("transition(name = \"drum_fill\", transition_type = \"fill\", measures = 1)")
            .unwrap();
    assert_eq!(result["transition_type"], "fill");
    assert_eq!(result["timing"]["measures"], 1);
    // Fill should have percussion-focused tracks
    let tracks = result["track_layout"]["suggested_tracks"]
        .as_array()
        .unwrap();
    assert!(tracks.iter().any(|t| t["name"] == "snare"));
}

#[test]
fn test_transition_invalid_type() {
    let result = eval_to_json("transition(name = \"test\", transition_type = \"invalid\")");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S104"));
}

#[test]
fn test_transition_invalid_intensity() {
    let result = eval_to_json("transition(name = \"test\", from_intensity = \"ultra\")");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S104"));
}

#[test]
fn test_transition_invalid_measures() {
    let result = eval_to_json("transition(name = \"test\", measures = 20)");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
}

#[test]
fn test_loop_format_xm() {
    let result = eval_to_json("loop_main(name = \"test\", format = \"xm\")").unwrap();
    assert_eq!(result["song_params"]["format"], "xm");
}

#[test]
fn test_loop_format_it() {
    let result = eval_to_json("loop_main(name = \"test\", format = \"it\")").unwrap();
    assert_eq!(result["song_params"]["format"], "it");
}

#[test]
fn test_loop_invalid_format() {
    let result = eval_to_json("loop_main(name = \"test\", format = \"mod\")");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S104"));
}

#[test]
fn test_loop_channels_xm_limit() {
    let result = eval_to_json("loop_main(name = \"test\", format = \"xm\", channels = 40)");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
    assert!(err.contains("32"));
}

#[test]
fn test_loop_channels_it_limit() {
    let result = eval_to_json("loop_main(name = \"test\", format = \"it\", channels = 80)");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
    assert!(err.contains("64"));
}

#[test]
fn test_timing_calculation() {
    // 4 measures * 4 beats * 4 rows_per_beat = 64 rows
    let result =
        eval_to_json("loop_main(name = \"test\", measures = 4, rows_per_beat = 4)").unwrap();
    assert_eq!(result["timing"]["total_rows"], 64);
}
