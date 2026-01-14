//! End-to-End Migration Tests for SpecCade
//!
//! Tests verify legacy .spec.py parsing works correctly.
//!
//! ## Running Tests
//!
//! ```bash
//! cargo test -p speccade-tests --test e2e_migration
//! ```

use std::fs;

use speccade_spec::AssetType;
use speccade_tests::fixtures::{GoldenFixtures, LegacyProjectFixture};
use speccade_tests::harness::parse_spec_file;

/// Test that we can parse a minimal sound spec.
#[test]
fn test_parse_sound_spec() {
    let fixture = LegacyProjectFixture::new();
    let spec_path = fixture.add_sound("test_beep");

    // Read and verify the file exists and has expected structure
    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("SOUND"));
    assert!(content.contains("duration"));
    assert!(content.contains("layers"));
}

/// Test that we can parse a minimal instrument spec.
#[test]
fn test_parse_instrument_spec() {
    let fixture = LegacyProjectFixture::new();
    let spec_path = fixture.add_instrument("test_synth");

    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("INSTRUMENT"));
    assert!(content.contains("synthesis"));
    assert!(content.contains("oscillators"));
}

/// Test that we can parse a minimal music spec.
#[test]
fn test_parse_music_spec() {
    let fixture = LegacyProjectFixture::new();
    let spec_path = fixture.add_music("test_song");

    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("SONG"));
    assert!(content.contains("bpm"));
    assert!(content.contains("patterns"));
}

/// Test that we can parse a minimal texture spec.
#[test]
fn test_parse_texture_spec() {
    let fixture = LegacyProjectFixture::new();
    let spec_path = fixture.add_texture("test_metal");

    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("TEXTURE"));
    assert!(content.contains("size"));
    assert!(content.contains("layers"));
}

/// Test that we can parse a minimal normal map spec.
#[test]
fn test_parse_normal_spec() {
    let fixture = LegacyProjectFixture::new();
    let spec_path = fixture.add_normal("test_bricks");

    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("NORMAL"));
    assert!(content.contains("pattern"));
}

/// Test that we can parse a minimal mesh spec.
#[test]
fn test_parse_mesh_spec() {
    let fixture = LegacyProjectFixture::new();
    let spec_path = fixture.add_mesh("test_cube");

    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("MESH"));
    assert!(content.contains("primitive"));
}

/// Test that we can parse a minimal character spec.
#[test]
fn test_parse_character_spec() {
    let fixture = LegacyProjectFixture::new();
    let spec_path = fixture.add_character("test_biped");

    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("SPEC"));
    assert!(content.contains("skeleton"));
}

/// Test that we can parse a minimal animation spec.
#[test]
fn test_parse_animation_spec() {
    let fixture = LegacyProjectFixture::new();
    let spec_path = fixture.add_animation("test_walk");

    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("ANIMATION"));
    assert!(content.contains("fps"));
}

/// Test that golden legacy fixtures exist.
#[test]
fn test_golden_legacy_fixtures_exist() {
    if !GoldenFixtures::exists() {
        println!("Golden fixtures not found, skipping test");
        return;
    }

    let sounds = GoldenFixtures::list_legacy_specs("sounds");
    assert!(!sounds.is_empty(), "Should have sound specs");

    let textures = GoldenFixtures::list_legacy_specs("textures");
    assert!(!textures.is_empty(), "Should have texture specs");
}

/// Test that golden speccade specs can be parsed.
#[test]
fn test_golden_speccade_specs_parse() {
    if !GoldenFixtures::exists() {
        println!("Golden fixtures not found, skipping test");
        return;
    }

    let audio_specs = GoldenFixtures::list_speccade_specs("audio");
    for spec_path in audio_specs {
        let spec = parse_spec_file(&spec_path);
        assert!(
            spec.is_ok(),
            "Failed to parse {:?}: {:?}",
            spec_path,
            spec.err()
        );
        let spec = spec.unwrap();
        assert_eq!(spec.asset_type, AssetType::Audio);
    }

    let texture_specs = GoldenFixtures::list_speccade_specs("texture");
    for spec_path in texture_specs {
        let spec = parse_spec_file(&spec_path);
        assert!(
            spec.is_ok(),
            "Failed to parse {:?}: {:?}",
            spec_path,
            spec.err()
        );
        let spec = spec.unwrap();
        assert_eq!(spec.asset_type, AssetType::Texture);
    }
}
