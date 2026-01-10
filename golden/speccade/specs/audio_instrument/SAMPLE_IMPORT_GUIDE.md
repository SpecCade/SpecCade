# Sample-Based Instrument Guide

This document describes how to use .wav sample imports in SpecCade instruments.

## Current Schema Support

### Music Tracker Instruments (Supported)

The `music.tracker_song_v1` recipe already supports sample-based instruments via the
`InstrumentSynthesis::Sample` type:

```json
{
  "instruments": [
    {
      "name": "piano",
      "synthesis": {
        "type": "sample",
        "path": "samples/piano/c4.wav",
        "base_note": "C4"
      },
      "envelope": {
        "attack": 0.001,
        "decay": 0.8,
        "sustain": 0.3,
        "release": 0.5
      }
    }
  ]
}
```

**Fields:**
- `type`: Must be `"sample"`
- `path`: Relative path to .wav file (from spec location)
- `base_note`: (Optional) The MIDI note the sample represents (e.g., "C4", "A3")

**Example specs using this feature:**
- `specs/music/orchestral_demo.json`
- `specs/music/hybrid_band.json`

### Standalone Audio Instrument Specs (Feature Request)

The current `audio_instrument.synth_patch_v1` recipe kind only supports synthesis-based
generation. To add sample import support, a new recipe kind would be needed:

```json
{
  "spec_version": 1,
  "asset_id": "piano_sample",
  "asset_type": "audio_instrument",
  "recipe": {
    "kind": "audio_instrument.sample_import_v1",
    "params": {
      "samples": [
        {
          "note": "C4",
          "sample_path": "samples/piano/c4.wav"
        },
        {
          "note": "C5",
          "sample_path": "samples/piano/c5.wav"
        }
      ],
      "loop": {
        "enabled": true,
        "start_percent": 0.5,
        "end_percent": 0.9
      },
      "envelope": {
        "attack": 0.01,
        "decay": 0.1,
        "sustain": 0.7,
        "release": 0.3
      }
    }
  }
}
```

## Implementation Status

| Feature | Status | Location |
|---------|--------|----------|
| Sample synthesis type for music | Implemented | `InstrumentSynthesis::Sample` |
| WAV loading (mono conversion) | Implemented | `synthesis.rs::load_wav_sample` |
| WAV resampling (to 22050 Hz) | Implemented | `synthesis.rs::resample_linear` |
| Multi-channel to mono | Implemented | `synthesis.rs::convert_to_mono_f64` |
| Spec-relative path resolution | Partial | Needs CLI dispatch update |
| Standalone sample import recipe | Not Implemented | Needs new recipe kind |

## WAV File Requirements

All samples must meet these specifications:

| Property | Requirement |
|----------|-------------|
| Format | WAV (RIFF) |
| Encoding | PCM (Integer) |
| Bit depth | 8, 16, 24, or 32-bit |
| Sample rate | Any (will be resampled to 22050 Hz) |
| Channels | Mono or stereo (stereo converted to mono) |

## Recommended Schema Extensions

### 1. Sample Import Recipe for Audio Instruments

Add `audio_instrument.sample_import_v1` recipe kind:

```rust
// In speccade-spec/src/recipe/audio_instrument.rs

pub struct AudioInstrumentSampleImportV1Params {
    /// Multi-sample definitions
    pub samples: Vec<SampleEntry>,
    /// Global envelope
    pub envelope: Envelope,
    /// Loop configuration
    pub loop_config: Option<LoopConfig>,
    /// Output sample rate
    pub sample_rate: u32,
}

pub struct SampleEntry {
    /// MIDI note or note name for this sample
    pub note: NoteSpec,
    /// Path to the sample file (relative to spec)
    pub sample_path: String,
    /// Volume adjustment for this sample (0.0 - 1.0)
    pub volume: Option<f64>,
}

pub struct LoopConfig {
    /// Enable looping
    pub enabled: bool,
    /// Loop start position (0.0 - 1.0 as percentage of sample)
    pub start_percent: Option<f64>,
    /// Loop end position (0.0 - 1.0 as percentage of sample)
    pub end_percent: Option<f64>,
}
```

### 2. Path Resolution Update

The CLI dispatch layer needs to pass the spec file's directory to the backend:

```rust
// In speccade-cli/src/dispatch.rs

fn generate_music(spec: &Spec, spec_path: &Path) -> Result<Report> {
    let spec_dir = spec_path.parent().unwrap_or(Path::new("."));
    speccade_backend_music::generate_music(
        &params,
        spec.seed,
        spec_dir,  // New parameter
    )
}
```

## Directory Structure for Samples

Recommended sample organization:

```
project/
  specs/
    music/
      my_song.json       # References samples with relative paths
  samples/
    piano/
      c4.wav
      c5.wav
    drums/
      kick.wav
      snare.wav
    bass/
      e1.wav
```

In `my_song.json`:
```json
{
  "synthesis": {
    "type": "sample",
    "path": "../samples/piano/c4.wav",
    "base_note": "C4"
  }
}
```

Or from golden corpus:
```json
{
  "synthesis": {
    "type": "sample",
    "path": "samples/piano/c4.wav",
    "base_note": "C4"
  }
}
```

## See Also

- `IMPLEMENTATION_ORCHESTRATION.md` - Phase 3A: Music WAV Samples
- `docs/SPEC_REFERENCE.md` - Full spec reference
- `crates/speccade-backend-music/src/synthesis.rs` - WAV loading implementation
