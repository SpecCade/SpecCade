//! Compose (Pattern IR) integration tests.

use std::fs;
use std::path::PathBuf;

use speccade_backend_music::{expand_compose, generate_music, generate_music_compose};
use speccade_spec::recipe::music::MusicTrackerSongV1Params;
use speccade_spec::Spec;

fn example_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn compose_example_paths() -> (PathBuf, PathBuf) {
    let root = example_root();
    let compose = root.join("docs/examples/music/compose_minimal_16rows.json");
    let expanded = root.join("docs/examples/music/compose_minimal_16rows.expanded.params.json");
    (compose, expanded)
}

#[test]
fn compose_expands_to_snapshot() {
    let (compose_path, expanded_path) = compose_example_paths();
    let compose_json = fs::read_to_string(&compose_path).expect("compose spec read");
    let spec = Spec::from_json(&compose_json).expect("compose spec parse");
    let recipe = spec.recipe.as_ref().expect("compose spec recipe");
    let params = recipe
        .as_music_tracker_song_compose()
        .expect("compose params parse");

    let expanded = expand_compose(&params, spec.seed).expect("expand compose");

    let expected_json = fs::read_to_string(&expanded_path).expect("expanded params read");
    let expected: MusicTrackerSongV1Params =
        serde_json::from_str(&expected_json).expect("expanded params parse");

    assert_eq!(expanded, expected);
}

#[test]
fn compose_generation_matches_expanded() {
    let (compose_path, expanded_path) = compose_example_paths();
    let compose_json = fs::read_to_string(&compose_path).expect("compose spec read");
    let spec = Spec::from_json(&compose_json).expect("compose spec parse");
    let recipe = spec.recipe.as_ref().expect("compose spec recipe");
    let params = recipe
        .as_music_tracker_song_compose()
        .expect("compose params parse");

    let expected_json = fs::read_to_string(&expanded_path).expect("expanded params read");
    let expected: MusicTrackerSongV1Params =
        serde_json::from_str(&expected_json).expect("expanded params parse");

    let spec_dir = compose_path.parent().expect("compose spec dir");

    let compose_gen =
        generate_music_compose(&params, spec.seed, spec_dir).expect("compose generate");
    let expanded_gen = generate_music(&expected, spec.seed, spec_dir).expect("expanded generate");

    assert_eq!(compose_gen.data, expanded_gen.data);
    assert_eq!(compose_gen.hash, expanded_gen.hash);
}

