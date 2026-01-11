# Music Spec Reference

This document covers tracker module generation in SpecCade.

## Overview

**Asset Type:** `music`  
**Recipe Kinds:** `music.tracker_song_v1`  
**Output Formats:** XM, IT

SpecCade generates fully playable tracker modules with embedded instruments and patterns.

## Outputs

For `music.tracker_song_v1`, `speccade generate` writes one or more `primary` outputs:

- `outputs[]` must contain **at least one** entry with `kind: "primary"`.
- In **single-output** specs, the sole `primary` output `format` must match `recipe.params.format`.
- In **multi-output** specs, you may declare up to **two** `primary` outputs: one `"xm"` and one `"it"`. The generator writes both formats from the same params.

Example:

```json
{
  "kind": "primary",
  "format": "xm",
  "path": "songs/drum_loop.xm"
}
```

## Recipe: `music.tracker_song_v1`

### Params

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `format` | string | yes | — | `"xm"` or `"it"` |
| `bpm` | integer | yes | — | Validated at generate time (`30..=300`) |
| `speed` | integer | yes | — | Validated at generate time (`1..=31`) |
| `channels` | integer | yes | — | XM: `1..=32`, IT: `1..=64` |
| `name` | string | no | omitted | Module internal name |
| `title` | string | no | omitted | Display title (metadata) |
| `loop` | boolean | no | `false` | XM only (IT currently ignores looping) |
| `restart_position` | integer | no | omitted | XM only; order-table index used when `loop: true` |
| `instruments` | array | no | `[]` | Instrument definitions |
| `patterns` | object | no | `{}` | Map of pattern name → pattern |
| `arrangement` | array | no | `[]` | Sequence of patterns by name |
| `automation` | array | no | `[]` | Volume fades / tempo changes |
| `it_options` | object | no | omitted | IT-only module options |

## Instruments

Each entry in `instruments[]` is a `TrackerInstrument`. You must specify **exactly one** of:

- `ref` (recommended for reuse; points to an `audio_v1` spec)
- `synthesis_audio_v1` (inline `audio_v1` params baked to a tracker sample)
- `wav` (external sample file)
- `synthesis` (legacy inline tracker synth; baked via `audio_v1`)

### Instrument Fields

| Field | Type | Notes |
|------:|------|------|
| `name` | string | Display name in tracker |
| `comment` | string | Optional, ignored by generator |
| `ref` | string | Path to an `audio` spec (see below) |
| `synthesis_audio_v1` | object | Inline `audio_v1` params baked to a tracker sample |
| `wav` | string | Path to a WAV file (relative to the music spec directory) |
| `synthesis` | object | Inline synthesis (see below) |
| `base_note` | string | Note name like `"C4"` / `"A#3"` (affects pitch correction) |
| `sample_rate` | integer | Optional; default is backend-defined (typically `22050`) |
| `envelope` | object | ADSR envelope (defaults if omitted) |
| `default_volume` | integer | Optional `0..=64` |

### Referential Instruments (`ref`)

`ref` loads an external spec file relative to the music spec’s directory. The referenced spec must be:

- `asset_type: "audio"`
- `recipe.kind: "audio_v1"`

All `audio_v1` synthesis types are supported: the audio backend is run and its PCM is baked into the
tracker module as a sample.

Notes / rules:

- **Mono policy:** baked samples are always mono; stereo `audio_v1` output is downmixed deterministically.
- **`base_note` precedence:** `TrackerInstrument.base_note` overrides `audio_v1.base_note` for pitch mapping.
- **`sample_rate` precedence:** `TrackerInstrument.sample_rate` overrides `audio_v1.sample_rate` when baking.
- **Envelope policy:** `TrackerInstrument.envelope` is the *only* amplitude envelope applied at playback time. When baking `audio_v1` instruments, per-layer `audio_v1` envelopes are neutralized to avoid “double enveloping”.
- **One-shot envelope note:** if `TrackerInstrument.envelope.sustain == 0`, the generator treats `release` as additional “tail” time (so one-shots decay without requiring an explicit note-off).
- **Loop policy:** if `TrackerInstrument.envelope.sustain > 0`, the baked sample loops (loop start is derived from `attack + decay`); otherwise it is a one-shot.
- **Safety limit:** extremely long baked samples are rejected to prevent huge modules.

### Inline Instrument Synthesis (`synthesis`)

`synthesis` is a legacy tagged union baked via `audio_v1` under the hood. Prefer `ref` or
`synthesis_audio_v1` for new content.

`synthesis` supports:

- `pulse`: `{ "type": "pulse", "duty_cycle": 0.5 }`
- `square`: `{ "type": "square" }`
- `triangle`: `{ "type": "triangle" }`
- `sawtooth`: `{ "type": "sawtooth" }`
- `sine`: `{ "type": "sine" }`
- `noise`: `{ "type": "noise", "periodic": false }`
- `sample`: `{ "type": "sample", "path": "samples/kick.wav", "base_note": "C4" }`

### Inline `audio_v1` Instruments (`synthesis_audio_v1`)

Use `synthesis_audio_v1` to inline the full `audio_v1` synthesis stack directly in a music spec.
This is baked into a tracker sample (mono) at generate time.

Example (single layer oscillator):

```json
{
  "name": "inline_sine",
  "synthesis_audio_v1": {
    "base_note": "A4",
    "duration_seconds": 0.25,
    "sample_rate": 22050,
    "layers": [
      {
        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
        "envelope": { "attack": 0.01, "decay": 0.1, "sustain": 0.5, "release": 0.2 },
        "volume": 1.0,
        "pan": 0.0
      }
    ]
  },
  "envelope": { "attack": 0.01, "decay": 0.1, "sustain": 0.7, "release": 0.2 }
}
```

Note: for music instruments, use `TrackerInstrument.envelope` to shape amplitude. The `audio_v1` layer `envelope` fields are still part of the `audio_v1` schema, but are neutralized during baking.

## Patterns

`patterns` is an object mapping pattern name → `TrackerPattern`.

Each pattern can use one of two formats:

- **Channel-keyed** (`notes`): `{ "notes": { "0": [ ... ], "1": [ ... ] } }`
- **Flat list** (`data`): `{ "data": [ { "row": 0, "channel": 0, ... }, ... ] }`

If both are present, they are merged and then sorted by `(row, channel)`.

### Pattern Fields

| Field | Type | Default |
|------:|------|---------|
| `rows` | integer | `0` |
| `notes` | object | omitted |
| `data` | array | omitted |

### Note Events

Each note event is a `PatternNote`:

| Field | Type | Notes |
|------:|------|------|
| `row` | integer | 0-indexed |
| `channel` | integer | Required for `data` format; ignored for `notes` format |
| `note` | string \| integer | Note name (e.g. `"C-4"`, `"C4"`) or MIDI number; may be omitted to trigger the instrument’s base note (see below) |
| `inst` | integer | Instrument index (0-based); alias: `instrument` |
| `vol` | integer | Optional volume (0–64); alias: `volume` |
| `effect` | integer | Optional effect code |
| `param` | integer | Optional effect parameter byte |
| `effect_name` | string | Optional effect name (backend maps to code) |
| `effect_xy` | [integer, integer] | Optional `[x, y]` nibbles → parameter byte |

Special note strings are format-specific, but these are commonly accepted:

- `"---"` / `"..."` → no note (instrument-only / volume / effect updates)
- `"OFF"` / `"==="` → note off / cut (format-dependent)

If `note` is omitted or empty, the generator triggers the instrument at:

- `instruments[inst].base_note` (if set), otherwise
- `instruments[inst].synthesis_audio_v1.base_note` (if set), otherwise
- XM: `"C4"` (C-4), IT: `"C5"` (C-5)

## Arrangement

`arrangement[]` is a list of entries:

```json
{ "pattern": "intro", "repeat": 2 }
```

Pattern names must exist in `patterns`. `repeat` defaults to `1`.

## Automation

Automation entries are tagged unions with `type`:

- `volume_fade`: fades a channel volume over a row range within a pattern
- `tempo_change`: changes tempo at a row within a pattern

Example:

```json
[
  { "type": "tempo_change", "pattern": "intro", "row": 0, "bpm": 140 },
  { "type": "volume_fade", "pattern": "intro", "channel": 0, "start_row": 0, "end_row": 63, "start_vol": 64, "end_vol": 0 }
]
```

## IT Options (`it_options`)

```json
{
  "stereo": true,
  "global_volume": 128,
  "mix_volume": 48
}
```

## Example (Minimal XM)

```json
{
  "spec_version": 1,
  "asset_id": "minimal_song",
  "asset_type": "music",
  "license": "CC0-1.0",
  "seed": 42,
  "outputs": [{ "kind": "primary", "format": "xm", "path": "minimal_song.xm" }],
  "recipe": {
    "kind": "music.tracker_song_v1",
    "params": {
      "format": "xm",
      "bpm": 125,
      "speed": 6,
      "channels": 4,
      "instruments": [{ "name": "lead", "synthesis": { "type": "square" } }],
      "patterns": {
        "intro": {
          "rows": 16,
          "data": [
            { "row": 0, "channel": 0, "note": "C-4", "instrument": 0, "volume": 64 },
            { "row": 4, "channel": 0, "note": "E-4", "instrument": 0, "volume": 64 },
            { "row": 8, "channel": 0, "note": "G-4", "instrument": 0, "volume": 64 }
          ]
        }
      },
      "arrangement": [{ "pattern": "intro", "repeat": 2 }]
    }
  }
}
```
