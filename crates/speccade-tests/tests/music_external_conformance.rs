//! External player conformance smoke tests for XM/IT modules.
//!
//! This test validates that generated XM/IT modules can be decoded by an
//! external player (`ffmpeg`) and produce non-empty PCM output.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use speccade_backend_music::generate_music;
use speccade_spec::recipe::audio::Envelope;
use speccade_spec::recipe::music::{
    ArrangementEntry, InstrumentSynthesis, MusicTrackerSongV1Params, PatternNote, TrackerFormat,
    TrackerInstrument, TrackerPattern,
};

fn find_ffmpeg_bin() -> Option<String> {
    if let Ok(path) = std::env::var("SPECCADE_FFMPEG_BIN") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    let status = Command::new("ffmpeg").arg("-version").status().ok()?;
    if status.success() {
        Some("ffmpeg".to_string())
    } else {
        None
    }
}

fn create_test_spec(format: TrackerFormat) -> MusicTrackerSongV1Params {
    let mut patterns = HashMap::new();
    patterns.insert(
        "p0".to_string(),
        TrackerPattern {
            rows: 64,
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
                    row: 16,
                    channel: Some(1),
                    note: "E-4".to_string(),
                    inst: 0,
                    vol: Some(56),
                    ..Default::default()
                },
                PatternNote {
                    row: 32,
                    channel: Some(0),
                    note: "G-4".to_string(),
                    inst: 0,
                    vol: Some(52),
                    ..Default::default()
                },
                PatternNote {
                    row: 48,
                    channel: Some(1),
                    note: "C-5".to_string(),
                    inst: 0,
                    vol: Some(48),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        },
    );

    MusicTrackerSongV1Params {
        format,
        bpm: 128,
        speed: 6,
        channels: 4,
        r#loop: false,
        instruments: vec![TrackerInstrument {
            name: "lead".to_string(),
            synthesis: Some(InstrumentSynthesis::Pulse {
                duty_cycle: 0.5,
                base_note: None,
            }),
            envelope: Envelope {
                attack: 0.01,
                decay: 0.08,
                sustain: 0.7,
                release: 0.2,
            },
            ..Default::default()
        }],
        patterns,
        arrangement: vec![ArrangementEntry {
            pattern: "p0".to_string(),
            repeat: 1,
        }],
        ..Default::default()
    }
}

fn decode_with_ffmpeg(ffmpeg_bin: &str, module_path: &Path) -> Result<Vec<u8>, String> {
    let output = Command::new(ffmpeg_bin)
        .args([
            "-v",
            "error",
            "-i",
            module_path.to_str().unwrap(),
            "-f",
            "s16le",
            "-ac",
            "2",
            "-ar",
            "44100",
            "-t",
            "8",
            "pipe:1",
        ])
        .output()
        .map_err(|e| format!("failed to execute ffmpeg: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "ffmpeg failed for {}: {}",
            module_path.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(output.stdout)
}

#[test]
fn test_external_ffmpeg_decodes_generated_xm_and_it() {
    let Some(ffmpeg_bin) = find_ffmpeg_bin() else {
        eprintln!("Skipping external conformance test: ffmpeg not found");
        return;
    };

    let temp = tempfile::tempdir().expect("tempdir");
    let spec_dir = Path::new(".");

    let xm_result = generate_music(&create_test_spec(TrackerFormat::Xm), 42, spec_dir)
        .expect("XM generation should succeed");
    let it_result = generate_music(&create_test_spec(TrackerFormat::It), 42, spec_dir)
        .expect("IT generation should succeed");

    let xm_path: PathBuf = temp.path().join("smoke.xm");
    let it_path: PathBuf = temp.path().join("smoke.it");
    std::fs::write(&xm_path, &xm_result.data).expect("write xm");
    std::fs::write(&it_path, &it_result.data).expect("write it");

    let xm_pcm = decode_with_ffmpeg(&ffmpeg_bin, &xm_path).expect("ffmpeg decode xm");
    let it_pcm = decode_with_ffmpeg(&ffmpeg_bin, &it_path).expect("ffmpeg decode it");

    assert!(
        !xm_pcm.is_empty(),
        "ffmpeg produced empty PCM for XM: {}",
        xm_path.display()
    );
    assert!(
        !it_pcm.is_empty(),
        "ffmpeg produced empty PCM for IT: {}",
        it_path.display()
    );
    assert_eq!(
        xm_pcm.len(),
        it_pcm.len(),
        "decoded PCM length mismatch: XM={} IT={}",
        xm_pcm.len(),
        it_pcm.len()
    );
}
