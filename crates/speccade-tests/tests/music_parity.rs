//! XM/IT Structural Parity Tests
//!
//! These tests verify that when the same music spec is generated as both XM and IT,
//! the outputs have structurally equivalent content (matching instrument counts,
//! sample counts, pattern counts, rows per pattern, etc.).
//!
//! # Running Tests
//!
//! ```bash
//! cargo test -p speccade-tests -- music_parity
//! ```

use std::collections::HashMap;
use std::path::Path;

use speccade_backend_music::parity::{check_parity, check_parity_detailed, ParityReport};
use speccade_backend_music::{generate_music, GenerateResult};
use speccade_spec::recipe::audio::Envelope;
use speccade_spec::recipe::music::{
    ArrangementEntry, InstrumentSynthesis, MusicTrackerSongV1Params, PatternNote, TrackerFormat,
    TrackerInstrument, TrackerPattern,
};

/// Create a minimal music spec for testing.
fn create_test_spec(format: TrackerFormat) -> MusicTrackerSongV1Params {
    let mut patterns = HashMap::new();
    patterns.insert(
        "intro".to_string(),
        TrackerPattern {
            rows: 32,
            data: Some(vec![
                PatternNote {
                    row: 0,
                    channel: Some(0),
                    note: "C-4".to_string(),
                    inst: 0,
                    vol: Some(64),
                    ..Default::default()
                },
                PatternNote {
                    row: 8,
                    channel: Some(0),
                    note: "E-4".to_string(),
                    inst: 0,
                    vol: Some(60),
                    ..Default::default()
                },
                PatternNote {
                    row: 16,
                    channel: Some(0),
                    note: "G-4".to_string(),
                    inst: 0,
                    vol: Some(56),
                    ..Default::default()
                },
                PatternNote {
                    row: 24,
                    channel: Some(1),
                    note: "C-3".to_string(),
                    inst: 1,
                    vol: Some(48),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        },
    );
    patterns.insert(
        "verse".to_string(),
        TrackerPattern {
            rows: 64,
            data: Some(vec![
                PatternNote {
                    row: 0,
                    channel: Some(0),
                    note: "D-4".to_string(),
                    inst: 0,
                    vol: Some(64),
                    ..Default::default()
                },
                PatternNote {
                    row: 16,
                    channel: Some(0),
                    note: "F-4".to_string(),
                    inst: 0,
                    vol: Some(64),
                    ..Default::default()
                },
                PatternNote {
                    row: 32,
                    channel: Some(0),
                    note: "A-4".to_string(),
                    inst: 0,
                    vol: Some(64),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        },
    );

    MusicTrackerSongV1Params {
        format,
        bpm: 120,
        speed: 6,
        channels: 4,
        r#loop: false,
        instruments: vec![
            TrackerInstrument {
                name: "lead".to_string(),
                synthesis: Some(InstrumentSynthesis::Pulse {
                    duty_cycle: 0.5,
                    base_note: None,
                }),
                envelope: Envelope {
                    attack: 0.01,
                    decay: 0.1,
                    sustain: 0.7,
                    release: 0.2,
                },
                ..Default::default()
            },
            TrackerInstrument {
                name: "bass".to_string(),
                synthesis: Some(InstrumentSynthesis::Sine { base_note: None }),
                envelope: Envelope {
                    attack: 0.005,
                    decay: 0.15,
                    sustain: 0.6,
                    release: 0.25,
                },
                ..Default::default()
            },
        ],
        patterns,
        arrangement: vec![
            ArrangementEntry {
                pattern: "intro".to_string(),
                repeat: 1,
            },
            ArrangementEntry {
                pattern: "verse".to_string(),
                repeat: 2,
            },
        ],
        ..Default::default()
    }
}

/// Generate both XM and IT from a spec template.
fn generate_both(
    seed: u32,
) -> Result<(GenerateResult, GenerateResult), speccade_backend_music::GenerateError> {
    let spec_dir = Path::new(".");

    let xm_spec = create_test_spec(TrackerFormat::Xm);
    let it_spec = create_test_spec(TrackerFormat::It);

    let xm_result = generate_music(&xm_spec, seed, spec_dir)?;
    let it_result = generate_music(&it_spec, seed, spec_dir)?;

    Ok((xm_result, it_result))
}

/// Check that the parity report indicates structural equivalence.
fn assert_parity(report: &ParityReport) {
    assert!(
        report.is_parity,
        "Parity check failed with {} mismatches:\n{}",
        report.mismatches.len(),
        report
            .mismatches
            .iter()
            .map(|m| format!("  - {}", m))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ============================================================================
// Basic Parity Tests
// ============================================================================

#[test]
fn test_xm_it_basic_parity() {
    let (xm, it) = generate_both(42).expect("Generation should succeed");

    assert_eq!(xm.extension, "xm");
    assert_eq!(it.extension, "it");

    let report = check_parity(&xm.data, &it.data).expect("Parity check should complete");

    // Report structure should be populated
    assert!(report.xm_summary.is_some());
    assert!(report.it_summary.is_some());

    // Check parity
    assert_parity(&report);
}

#[test]
fn test_xm_it_instrument_count_parity() {
    let (xm, it) = generate_both(123).expect("Generation should succeed");

    let report = check_parity(&xm.data, &it.data).expect("Parity check should complete");

    let xm_summary = report.xm_summary.as_ref().expect("XM summary");
    let it_summary = report.it_summary.as_ref().expect("IT summary");

    // Both should have 2 instruments
    assert_eq!(xm_summary.instrument_count, 2);
    assert_eq!(it_summary.instrument_count, 2);
}

#[test]
fn test_xm_it_pattern_count_parity() {
    let (xm, it) = generate_both(456).expect("Generation should succeed");

    let report = check_parity(&xm.data, &it.data).expect("Parity check should complete");

    let xm_summary = report.xm_summary.as_ref().expect("XM summary");
    let it_summary = report.it_summary.as_ref().expect("IT summary");

    // Both should have same number of patterns
    // Note: The arrangement expands to 3 pattern instances (intro + verse + verse)
    // but the actual pattern definitions are 2 (intro, verse)
    assert_eq!(xm_summary.pattern_count, it_summary.pattern_count);
}

#[test]
fn test_xm_it_pattern_rows_parity() {
    let (xm, it) = generate_both(789).expect("Generation should succeed");

    let report = check_parity(&xm.data, &it.data).expect("Parity check should complete");

    let xm_summary = report.xm_summary.as_ref().expect("XM summary");
    let it_summary = report.it_summary.as_ref().expect("IT summary");

    // Pattern rows should match for corresponding patterns
    assert_eq!(xm_summary.pattern_rows, it_summary.pattern_rows);
}

#[test]
fn test_xm_it_tempo_bpm_parity() {
    let (xm, it) = generate_both(999).expect("Generation should succeed");

    let report = check_parity(&xm.data, &it.data).expect("Parity check should complete");

    let xm_summary = report.xm_summary.as_ref().expect("XM summary");
    let it_summary = report.it_summary.as_ref().expect("IT summary");

    // Tempo and BPM should match
    assert_eq!(xm_summary.tempo, it_summary.tempo);
    assert_eq!(xm_summary.bpm, it_summary.bpm);
}

#[test]
fn test_xm_it_detailed_parity_with_loop_restart() {
    let spec_dir = Path::new(".");
    let mut xm_spec = create_test_spec(TrackerFormat::Xm);
    xm_spec.r#loop = true;
    xm_spec.restart_position = Some(1);

    let mut it_spec = create_test_spec(TrackerFormat::It);
    it_spec.r#loop = true;
    it_spec.restart_position = Some(1);

    let xm = generate_music(&xm_spec, 42, spec_dir).expect("XM generation should succeed");
    let it = generate_music(&it_spec, 42, spec_dir).expect("IT generation should succeed");

    let report =
        check_parity_detailed(&xm.data, &it.data).expect("Detailed parity check should complete");
    assert_parity(&report);
}

// ============================================================================
// Determinism Tests
// ============================================================================

#[test]
fn test_parity_deterministic_same_seed() {
    // Generate twice with the same seed
    let (xm1, it1) = generate_both(42).expect("First generation should succeed");
    let (xm2, it2) = generate_both(42).expect("Second generation should succeed");

    // Both XM outputs should be identical
    assert_eq!(xm1.hash, xm2.hash, "XM outputs should be deterministic");

    // Both IT outputs should be identical
    assert_eq!(it1.hash, it2.hash, "IT outputs should be deterministic");

    // Parity should hold for both pairs
    let report1 = check_parity(&xm1.data, &it1.data).expect("First parity check");
    let report2 = check_parity(&xm2.data, &it2.data).expect("Second parity check");

    assert_parity(&report1);
    assert_parity(&report2);
}

#[test]
fn test_parity_different_seeds_same_structure() {
    // Different seeds should produce different audio content but same structure
    let (xm1, it1) = generate_both(100).expect("Seed 100 generation");
    let (xm2, it2) = generate_both(200).expect("Seed 200 generation");

    let report1 = check_parity(&xm1.data, &it1.data).expect("Seed 100 parity");
    let report2 = check_parity(&xm2.data, &it2.data).expect("Seed 200 parity");

    // Both should maintain structural parity
    assert_parity(&report1);
    assert_parity(&report2);

    // Structure should be identical across seeds (same spec)
    let xm1_summary = report1.xm_summary.as_ref().unwrap();
    let xm2_summary = report2.xm_summary.as_ref().unwrap();

    assert_eq!(xm1_summary.instrument_count, xm2_summary.instrument_count);
    assert_eq!(xm1_summary.pattern_count, xm2_summary.pattern_count);
    assert_eq!(xm1_summary.pattern_rows, xm2_summary.pattern_rows);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_parity_minimal_spec() {
    let spec_dir = Path::new(".");

    // Minimal spec with one instrument and one pattern
    let mut patterns = HashMap::new();
    patterns.insert(
        "p1".to_string(),
        TrackerPattern {
            rows: 16,
            data: Some(vec![PatternNote {
                row: 0,
                channel: Some(0),
                note: "C-4".to_string(),
                inst: 0,
                vol: None,
                ..Default::default()
            }]),
            ..Default::default()
        },
    );

    let base_spec = MusicTrackerSongV1Params {
        format: TrackerFormat::Xm, // Will be overwritten
        bpm: 120,
        speed: 6,
        channels: 2,
        r#loop: false,
        instruments: vec![TrackerInstrument {
            name: "test".to_string(),
            synthesis: Some(InstrumentSynthesis::Sine { base_note: None }),
            envelope: Envelope {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.5,
                release: 0.1,
            },
            ..Default::default()
        }],
        patterns: patterns.clone(),
        arrangement: vec![ArrangementEntry {
            pattern: "p1".to_string(),
            repeat: 1,
        }],
        ..Default::default()
    };

    let xm_spec = MusicTrackerSongV1Params {
        format: TrackerFormat::Xm,
        ..base_spec.clone()
    };
    let it_spec = MusicTrackerSongV1Params {
        format: TrackerFormat::It,
        ..base_spec
    };

    let xm = generate_music(&xm_spec, 42, spec_dir).expect("XM generation");
    let it = generate_music(&it_spec, 42, spec_dir).expect("IT generation");

    let report = check_parity(&xm.data, &it.data).expect("Parity check");
    assert_parity(&report);
}

#[test]
fn test_parity_report_display() {
    let (xm, it) = generate_both(42).expect("Generation should succeed");

    let report = check_parity(&xm.data, &it.data).expect("Parity check should complete");

    // Report should produce valid display output
    let display = format!("{}", report);
    assert!(
        display.contains("Parity:"),
        "Display should contain parity status"
    );
    assert!(display.contains("XM:"), "Display should contain XM summary");
    assert!(display.contains("IT:"), "Display should contain IT summary");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_parity_invalid_xm_data() {
    let invalid_xm = vec![0u8; 100];
    let (_, it) = generate_both(42).expect("IT generation should succeed");

    let result = check_parity(&invalid_xm, &it.data);
    assert!(result.is_err(), "Should fail with invalid XM data");
}

#[test]
fn test_parity_invalid_it_data() {
    let (xm, _) = generate_both(42).expect("XM generation should succeed");
    let invalid_it = vec![0u8; 100];

    let result = check_parity(&xm.data, &invalid_it);
    assert!(result.is_err(), "Should fail with invalid IT data");
}
