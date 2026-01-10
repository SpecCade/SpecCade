# Music Spec Reference

This document covers music tracker module generation in SpecCade.

## Table of Contents

- [Overview](#overview)
- [Recipe: music.tracker_song_v1](#recipe-musictracker_song_v1)
- [Instruments](#instruments)
  - [Referential vs Inline Instruments](#referential-vs-inline-instruments)
  - [Instrument Definition](#instrument-definition)
- [Patterns](#patterns)
  - [Note Events](#note-events)
  - [Effects](#effects)
- [Arrangement](#arrangement)
- [Automation](#automation)
- [Format-Specific Options](#format-specific-options)
- [Examples](#examples)

---

## Overview

**Asset Type:** `music`
**Recipe Kinds:** `music.tracker_song_v1`
**Output Formats:** XM, IT

Tracker module songs with instruments, patterns, and arrangement. SpecCade generates fully playable XM (FastTracker II) or IT (Impulse Tracker) modules with embedded samples.

---

## Recipe: `music.tracker_song_v1`

### Required Params

| Param | Type | Description |
|-------|------|-------------|
| `format` | string | Module format: `"xm"` or `"it"` |
| `instruments` | array | Instrument definitions |
| `patterns` | object | Pattern definitions |
| `arrangement` | array | Pattern order |

### Optional Params

| Param | Type | Description | Default |
|-------|------|-------------|---------|
| `name` | string | Internal song name | `""` |
| `title` | string | Display title | `""` |
| `bpm` | integer | Tempo in beats per minute | `125` |
| `speed` | integer | Tracker speed (ticks per row) | `6` |
| `channels` | integer | Number of channels | `4` |
| `loop` | boolean | Loop song | `true` |
| `restart_position` | integer | Loop restart position in arrangement | `0` |
| `automation` | array | Automation events | `[]` |
| `xm_options` | object | XM-specific options | `{}` |
| `it_options` | object | IT-specific options | `{}` |

---

## Instruments

### Referential vs Inline Instruments

Music specs support two approaches for defining instruments:

**Referential Instruments (Recommended):**
Reference a pre-generated `audio_instrument` spec by path. Benefits:
- Cached WAV samples reused across songs
- Smaller, cleaner music specs
- Separation between sound design and composition

**Inline Synthesis:**
Define synthesis parameters directly in the music spec. Useful for:
- Quick prototyping
- One-off unique sounds
- Simple waveforms

### Instrument Definition

Each instrument requires a `name` and exactly one of: `ref`, `wav`, or `synthesis`.

#### Referential Instrument

```json
{
  "name": "bass",
  "ref": "instruments/bass_pluck.spec.json"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Instrument name (displayed in tracker) |
| `ref` | string | Path to audio_instrument spec |

#### WAV Sample Reference

```json
{
  "name": "kick",
  "wav": "samples/kick.wav",
  "base_note": "C4"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `wav` | string | Path to WAV file |
| `base_note` | string | Sample pitch for playback |

#### Inline Synthesis

```json
{
  "name": "lead_synth",
  "base_note": "C5",
  "sample_rate": 44100,
  "synthesis": {
    "type": "sawtooth"
  },
  "envelope": {
    "attack": 0.01,
    "decay": 0.1,
    "sustain": 0.7,
    "release": 0.3
  }
}
```

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `base_note` | string | Sample pitch | `"C4"` (XM), `"C5"` (IT) |
| `sample_rate` | integer | Sample rate | `22050` |
| `synthesis` | object | Synthesis config | Required |
| `envelope` | object | ADSR envelope | Required |

**Available synthesis types:**
- `"sine"` - Sine wave
- `"square"` - Square wave
- `"saw"` / `"sawtooth"` - Sawtooth wave
- `"triangle"` - Triangle wave
- `"noise"` - White noise
- `"pulse"` - Pulse wave with configurable duty cycle

---

## Patterns

Patterns define note sequences organized by channel.

### Pattern Structure

```json
{
  "patterns": {
    "intro": {
      "rows": 64,
      "notes": {
        "0": [...],
        "1": [...],
        "2": [...]
      }
    },
    "verse": {
      "rows": 64,
      "notes": {...}
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `rows` | integer | Number of rows (typically 64) |
| `notes` | object | Channel-indexed note arrays |

Channels are zero-indexed strings: `"0"`, `"1"`, `"2"`, etc.

### Note Events

Each note event in a channel specifies position and playback parameters:

```json
{
  "row": 0,
  "note": "C4",
  "inst": 0,
  "vol": 64,
  "effect": 0,
  "param": 0
}
```

| Field | Type | Description | Required |
|-------|------|-------------|----------|
| `row` | integer | Row index (0-based) | Yes |
| `note` | string/integer | Note name or MIDI number | Yes |
| `inst` | integer | Instrument index (0-based) | Yes |
| `vol` | integer | Volume (0-64) | No |
| `effect` | integer | Effect command number | No |
| `param` | integer | Effect parameter | No |
| `effect_name` | string | Named effect (alternative to effect) | No |
| `effect_xy` | array | Effect X/Y parameters `[x, y]` | No |

#### Note Values

- Note names: `"C4"`, `"D#5"`, `"Gb3"`, etc.
- MIDI numbers: `60` (C4), `69` (A4), etc.
- Special values:
  - `"OFF"` or `97` - Note off/cut
  - `"---"` - Empty/continue

### Effects

Effects can be specified by number or by name.

#### Named Effects

```json
{"row": 0, "note": "C4", "inst": 0, "effect_name": "vibrato", "effect_xy": [4, 8]}
```

| Effect Name | Description | Parameters |
|-------------|-------------|------------|
| `arpeggio` | Rapid note switching | `[semitones1, semitones2]` |
| `porta_up` | Portamento up | `[0, speed]` |
| `porta_down` | Portamento down | `[0, speed]` |
| `porta_to_note` | Slide to note | `[0, speed]` |
| `vibrato` | Pitch vibrato | `[speed, depth]` |
| `tremolo` | Volume tremolo | `[speed, depth]` |
| `volume_slide` | Volume slide | `[up, down]` |
| `set_volume` | Set volume | Single value in `param` |
| `set_tempo` | Set BPM | Single value in `param` |
| `set_speed` | Set speed/ticks | Single value in `param` |

#### Numeric Effects

Standard tracker effect numbers can also be used directly:

| Effect | XM | IT | Description |
|--------|----|----|-------------|
| 0 | Arpeggio | Arpeggio | Rapid note switching |
| 1 | Porta Up | Porta Up | Pitch slide up |
| 2 | Porta Down | Porta Down | Pitch slide down |
| 3 | Porta To | Porta To | Slide to note |
| 4 | Vibrato | Vibrato | Pitch oscillation |
| 5 | Vol+Porta | Vol+Porta | Volume slide + porta |
| 6 | Vol+Vib | Vol+Vib | Volume slide + vibrato |
| 7 | Tremolo | Tremolo | Volume oscillation |
| 10 | Vol Slide | Vol Slide | Volume slide |
| 12 | Set Vol | Set Vol | Set volume |
| 15 | Set Speed | Set Speed | Set speed/tempo |

---

## Arrangement

The arrangement defines the order of patterns:

```json
{
  "arrangement": [
    {"pattern": "intro", "repeat": 1},
    {"pattern": "verse", "repeat": 2},
    {"pattern": "chorus", "repeat": 2},
    {"pattern": "bridge", "repeat": 1},
    {"pattern": "verse", "repeat": 1},
    {"pattern": "chorus", "repeat": 2},
    {"pattern": "outro", "repeat": 1}
  ]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `pattern` | string | Pattern name from `patterns` object |
| `repeat` | integer | Number of times to play |

---

## Automation

Automation events modify playback parameters over time:

### Volume Fade

```json
{
  "type": "volume_fade",
  "pattern": "intro",
  "channel": 0,
  "start_row": 0,
  "end_row": 32,
  "start_vol": 0,
  "end_vol": 64
}
```

| Field | Type | Description |
|-------|------|-------------|
| `pattern` | string | Target pattern |
| `channel` | integer | Target channel |
| `start_row` | integer | Fade start row |
| `end_row` | integer | Fade end row |
| `start_vol` | integer | Starting volume (0-64) |
| `end_vol` | integer | Ending volume (0-64) |

### Tempo Change

```json
{
  "type": "tempo_change",
  "pattern": "chorus",
  "row": 0,
  "bpm": 150
}
```

| Field | Type | Description |
|-------|------|-------------|
| `pattern` | string | Target pattern |
| `row` | integer | Row to apply change |
| `bpm` | integer | New tempo |

---

## Format-Specific Options

### XM Options

```json
{
  "xm_options": {
    "linear_frequency": true
  }
}
```

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `linear_frequency` | boolean | Use linear frequency table | `true` |

### IT Options

```json
{
  "it_options": {
    "stereo": true,
    "global_volume": 128,
    "mix_volume": 64
  }
}
```

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `stereo` | boolean | Enable stereo output | `true` |
| `global_volume` | integer | Global volume (0-128) | `128` |
| `mix_volume` | integer | Mixing volume (0-128) | `64` |

---

## Examples

### Example: Simple 4-Channel Song

```json
{
  "spec_version": 1,
  "asset_id": "simple_song",
  "asset_type": "music",
  "license": "CC0-1.0",
  "seed": 12345,
  "description": "Simple 4-channel tracker song",
  "outputs": [
    {"kind": "primary", "format": "xm", "path": "simple_song.xm"}
  ],
  "recipe": {
    "kind": "music.tracker_song_v1",
    "params": {
      "format": "xm",
      "bpm": 140,
      "speed": 6,
      "channels": 4,
      "loop": true,
      "instruments": [
        {
          "name": "lead",
          "synthesis": {"type": "square"},
          "envelope": {"attack": 0.01, "decay": 0.1, "sustain": 0.6, "release": 0.2}
        },
        {
          "name": "bass",
          "synthesis": {"type": "sine"},
          "envelope": {"attack": 0.005, "decay": 0.2, "sustain": 0.5, "release": 0.2}
        }
      ],
      "patterns": {
        "main": {
          "rows": 64,
          "notes": {
            "0": [
              {"row": 0, "note": "C4", "inst": 0, "vol": 64},
              {"row": 16, "note": "E4", "inst": 0, "vol": 64},
              {"row": 32, "note": "G4", "inst": 0, "vol": 64},
              {"row": 48, "note": "E4", "inst": 0, "vol": 64}
            ],
            "1": [
              {"row": 0, "note": "C2", "inst": 1, "vol": 56},
              {"row": 32, "note": "G2", "inst": 1, "vol": 56}
            ]
          }
        }
      },
      "arrangement": [
        {"pattern": "main", "repeat": 4}
      ]
    }
  }
}
```

### Example: IT Module with Referential Instruments

```json
{
  "spec_version": 1,
  "asset_id": "it_song",
  "asset_type": "music",
  "license": "CC0-1.0",
  "seed": 99999,
  "description": "IT module using external instrument specs",
  "outputs": [
    {"kind": "primary", "format": "it", "path": "it_song.it"}
  ],
  "recipe": {
    "kind": "music.tracker_song_v1",
    "params": {
      "format": "it",
      "bpm": 120,
      "speed": 6,
      "channels": 8,
      "instruments": [
        {"name": "kick", "ref": "audio_instrument/drum_kick.json"},
        {"name": "snare", "ref": "audio_instrument/drum_snare.json"},
        {"name": "bass", "ref": "audio_instrument/bass_electric.json"},
        {"name": "lead", "ref": "audio_instrument/saw_lead.json"}
      ],
      "patterns": {
        "verse": {
          "rows": 64,
          "notes": {
            "0": [
              {"row": 0, "note": "C4", "inst": 0, "vol": 64},
              {"row": 16, "note": "C4", "inst": 0, "vol": 60},
              {"row": 32, "note": "C4", "inst": 0, "vol": 64},
              {"row": 48, "note": "C4", "inst": 0, "vol": 60}
            ],
            "1": [
              {"row": 8, "note": "D4", "inst": 1, "vol": 56},
              {"row": 24, "note": "D4", "inst": 1, "vol": 56},
              {"row": 40, "note": "D4", "inst": 1, "vol": 56},
              {"row": 56, "note": "D4", "inst": 1, "vol": 56}
            ],
            "2": [
              {"row": 0, "note": "C2", "inst": 2, "vol": 64},
              {"row": 32, "note": "G2", "inst": 2, "vol": 64}
            ],
            "3": [
              {"row": 0, "note": "C4", "inst": 3, "vol": 48},
              {"row": 16, "note": "E4", "inst": 3, "vol": 48},
              {"row": 32, "note": "G4", "inst": 3, "vol": 48},
              {"row": 48, "note": "B4", "inst": 3, "vol": 48}
            ]
          }
        }
      },
      "arrangement": [
        {"pattern": "verse", "repeat": 4}
      ],
      "it_options": {
        "stereo": true,
        "global_volume": 128,
        "mix_volume": 64
      }
    }
  }
}
```

---

## Validation Rules

### Music-Specific Validation

| Rule | Error Code | Description |
|------|------------|-------------|
| Format must be "xm" or "it" | E040 | Invalid module format |
| BPM must be 32-255 | E041 | BPM out of range |
| Speed must be 1-31 | E042 | Speed out of range |
| Channels must be 1-64 | E043 | Channel count out of range |
| Pattern must have rows | E044 | Missing rows in pattern |
| Pattern referenced in arrangement must exist | E045 | Unknown pattern |
| Instrument index must be valid | E046 | Invalid instrument reference |
| Note must be valid | E047 | Invalid note value |
| Row must be within pattern bounds | E048 | Row exceeds pattern length |

---

## Golden Corpus Specs

Reference implementations:

- `golden/speccade/music_comprehensive.spec.json` - All features demonstrated
