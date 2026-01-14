//! End-to-End Validation Tests for SpecCade
//!
//! Tests verify:
//! - Output validation (WAV, PNG, XM files)
//! - Spec validation
//!
//! ## Running Tests
//!
//! ```bash
//! cargo test -p speccade-tests --test e2e_validation
//! ```

use std::fs;

use speccade_spec::{AssetType, OutputFormat, OutputSpec, Spec};
use speccade_tests::fixtures::GoldenFixtures;
use speccade_tests::harness::{
    parse_spec_file, validate_png_file, validate_wav_file, validate_xm_file, TestHarness,
};

// ============================================================================
// Output Validation Tests
// ============================================================================

/// Test WAV file validation catches invalid files.
#[test]
fn test_wav_validation_rejects_invalid() {
    let harness = TestHarness::new();
    let bad_file = harness.path().join("bad.wav");

    // Empty file
    fs::write(&bad_file, b"").unwrap();
    assert!(validate_wav_file(&bad_file).is_err());

    // Too short
    fs::write(&bad_file, b"RIFF").unwrap();
    assert!(validate_wav_file(&bad_file).is_err());

    // Wrong magic
    fs::write(&bad_file, b"XXXX00000000WAVE").unwrap();
    assert!(validate_wav_file(&bad_file).is_err());
}

/// Test PNG file validation catches invalid files.
#[test]
fn test_png_validation_rejects_invalid() {
    let harness = TestHarness::new();
    let bad_file = harness.path().join("bad.png");

    // Empty file
    fs::write(&bad_file, b"").unwrap();
    assert!(validate_png_file(&bad_file).is_err());

    // Wrong magic
    fs::write(&bad_file, b"NOTAPNG!").unwrap();
    assert!(validate_png_file(&bad_file).is_err());
}

/// Test XM file validation catches invalid files.
#[test]
fn test_xm_validation_rejects_invalid() {
    let harness = TestHarness::new();
    let bad_file = harness.path().join("bad.xm");

    // Empty file
    fs::write(&bad_file, b"").unwrap();
    assert!(validate_xm_file(&bad_file).is_err());

    // Wrong header
    fs::write(&bad_file, b"Not an XM file at all!").unwrap();
    assert!(validate_xm_file(&bad_file).is_err());
}

/// Test that output format detection works correctly.
#[test]
fn test_output_format_dispatch() {
    let harness = TestHarness::new();

    // Create a minimal valid WAV file (44 bytes minimum)
    let wav_header: Vec<u8> = vec![
        b'R', b'I', b'F', b'F', // ChunkID
        36, 0, 0, 0, // ChunkSize (36 + data size)
        b'W', b'A', b'V', b'E', // Format
        b'f', b'm', b't', b' ', // Subchunk1ID
        16, 0, 0, 0, // Subchunk1Size
        1, 0, // AudioFormat (PCM)
        1, 0, // NumChannels
        0x22, 0x56, 0, 0, // SampleRate (22050)
        0x44, 0xAC, 0, 0, // ByteRate
        2, 0, // BlockAlign
        16, 0, // BitsPerSample
        b'd', b'a', b't', b'a', // Subchunk2ID
        0, 0, 0, 0, // Subchunk2Size
    ];

    let wav_file = harness.path().join("test.wav");
    fs::write(&wav_file, &wav_header).unwrap();

    let result = validate_wav_file(&wav_file);
    assert!(result.is_ok(), "Valid WAV should pass: {:?}", result.err());
}

// ============================================================================
// Spec Validation Tests
// ============================================================================

/// Test that a valid spec passes validation.
#[test]
fn test_valid_spec_passes() {
    let spec = Spec::builder("valid-spec-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .build();

    let result = speccade_spec::validate_spec(&spec);
    assert!(
        result.is_ok(),
        "Valid spec should pass: {:?}",
        result.errors
    );
}

/// Test that a spec without recipe fails generate validation.
#[test]
fn test_spec_without_recipe_fails_generate() {
    let spec = Spec::builder("no-recipe-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .build();

    let result = speccade_spec::validate_for_generate(&spec);
    assert!(
        !result.is_ok(),
        "Spec without recipe should fail generate validation"
    );
}

/// Test that all golden specs pass validation.
#[test]
fn test_golden_specs_pass_validation() {
    if !GoldenFixtures::exists() {
        println!("Golden fixtures not found, skipping");
        return;
    }

    let asset_types = [
        "audio",
        "music",
        "texture",
        "static_mesh",
        "skeletal_mesh",
        "skeletal_animation",
    ];

    for asset_type in &asset_types {
        let specs = GoldenFixtures::list_speccade_specs(asset_type);
        for spec_path in specs {
            let spec = parse_spec_file(&spec_path);
            assert!(
                spec.is_ok(),
                "Failed to parse {:?}: {:?}",
                spec_path,
                spec.err()
            );

            let spec = spec.unwrap();
            let result = speccade_spec::validate_spec(&spec);
            assert!(
                result.is_ok(),
                "Golden spec {:?} should pass validation: {:?}",
                spec_path,
                result.errors
            );
        }
    }
}
