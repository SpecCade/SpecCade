# Music Compose IR Reference

The Pattern IR is a compact, operator-based authoring layer for tracker music. Instead of writing hundreds of individual note events, you compose small generators using operators like `stack`, `emit`, and `repeat`.

## Overview

**Recipe kind:** `music.tracker_song_compose_v1`

**Mental model:**
- A tracker pattern is a grid: `(row, channel) -> cell`
- Pattern IR builds this grid by composing generators with operators
- The backend expands Pattern IR into flat `TrackerPattern.data[]` sorted by `(row, channel)`

**Workflow:**
1. Write compact compose spec
2. Expand: `speccade expand --spec FILE` (review expanded JSON)
3. Generate: `speccade generate --spec FILE` (produces XM/IT)

---

## Top-Level Structure

```json
{
  "recipe": {
    "kind": "music.tracker_song_compose_v1",
    "params": {
      "format": "xm",           // "xm" or "it"
      "bpm": 150,
      "speed": 6,               // Ticks per row
      "channels": 8,
      "loop": false,
      "instruments": [...],
      "defs": { ... },          // Reusable pattern fragments
      "patterns": { ... },      // Named patterns with programs
      "arrangement": [...]      // Play order
    }
  }
}
```

### Instruments

Each instrument must use **exactly one** source (mutually exclusive):

| Method | Field | Best For |
|--------|-------|----------|
| **External ref** (recommended) | `ref` | Reusable instruments across songs |
| **Inline audio_v1** | `synthesis_audio_v1` | Complex one-off instruments |
| **WAV sample** | `wav` | Pre-recorded samples |
| **Deprecated synthesis** | `synthesis` | Quick prototyping only |

#### External Instrument Reference (Recommended)

Point to a separate `audio_v1` spec file for modular, reusable instruments:

```json
{
  "instruments": [
    {
      "name": "kick",
      "ref": "../audio/kick_punchy.spec.json",
      "envelope": { "attack": 0.001, "decay": 0.2, "sustain": 0.0, "release": 0.1 },
      "default_volume": 64
    },
    {
      "name": "bass",
      "ref": "../audio/bass_saw.spec.json",
      "base_note": "C2",
      "envelope": { "attack": 0.01, "decay": 0.1, "sustain": 0.8, "release": 0.2 }
    }
  ]
}
```

The referenced file must be an `audio_v1` spec (`asset_type: "audio"`, `recipe.kind: "audio_v1"`).

**Overrides:** Tracker-level `base_note`, `envelope`, and `sample_rate` override the audio_v1 values.

#### Inline Audio V1

Embed full `audio_v1` params directly (same power as audio specs, but not reusable):

```json
{
  "instruments": [
    {
      "name": "fm_bell",
      "synthesis_audio_v1": {
        "base_note": "C5",
        "duration_seconds": 2.0,
        "sample_rate": 44100,
        "layers": [
          {
            "synthesis": { "type": "fm_synth", "carrier_freq": 440.0, "modulator_freq": 880.0, "modulation_index": 2.5 },
            "volume": 0.8
          }
        ]
      },
      "envelope": { "attack": 0.01, "decay": 0.3, "sustain": 0.4, "release": 0.5 }
    }
  ]
}
```

#### Deprecated Synthesis (Quick Prototyping)

Simple built-in waveforms for quick tests (limited options):

```json
{
  "instruments": [
    { "name": "test_tone", "base_note": "C4", "synthesis": { "type": "sine" } },
    { "name": "test_noise", "base_note": "C1", "synthesis": { "type": "noise", "periodic": false } }
  ]
}
```

Deprecated types: `sine`, `square`, `sawtooth`, `triangle`, `noise`

### Defs (Reusable Fragments)

```json
{
  "defs": {
    "four_on_floor": {
      "op": "emit",
      "at": { "op": "range", "start": 0, "step": 16, "count": 4 },
      "cell": { "channel": 0, "note": "C4", "inst": 0, "vol": 64 }
    },
    "offbeat_hats": {
      "op": "emit",
      "at": { "op": "range", "start": 8, "step": 16, "count": 4 },
      "cell": { "channel": 1, "note": "C1", "inst": 1, "vol": 32 }
    }
  }
}
```

### Patterns

```json
{
  "patterns": {
    "verse": {
      "rows": 64,
      "program": {
        "op": "stack",
        "merge": "error",
        "parts": [
          { "op": "ref", "name": "four_on_floor" },
          { "op": "ref", "name": "offbeat_hats" }
        ]
      }
    }
  }
}
```

### Arrangement

```json
{
  "arrangement": [
    { "pattern": "verse", "repeat": 4 },
    { "pattern": "chorus", "repeat": 2 }
  ]
}
```

---

## Structural Operators

### `stack` - Overlay Patterns

Combine multiple pattern expressions into one, layering cells.

```json
{
  "op": "stack",
  "merge": "error",           // "error" | "merge_fields" | "last_wins"
  "parts": [
    { "op": "ref", "name": "kick_layer" },
    { "op": "ref", "name": "snare_layer" },
    { "op": "ref", "name": "hat_layer" }
  ]
}
```

**Merge policies:**
- `"error"` - Fail on any double-write to same `(row, channel)`. Use at top level to catch bugs.
- `"merge_fields"` - Merge non-conflicting fields; error if same field has different values. Default.
- `"last_wins"` - Later writer wins conflicts. Use for intentional overrides (ghost notes, fills).

### `concat` - Concatenate in Time

Join parts sequentially in time.

```json
{
  "op": "concat",
  "parts": [
    { "len_rows": 32, "body": { "op": "ref", "name": "intro" } },
    { "len_rows": 32, "body": { "op": "ref", "name": "verse" } }
  ]
}
```

### `repeat` - Repeat Pattern

Repeat a sub-expression N times with automatic time shift.

```json
{
  "op": "repeat",
  "times": 4,
  "len_rows": 16,
  "body": { "op": "ref", "name": "one_bar" }
}
```

### `shift` - Time Shift

Shift all produced cells by N rows (can be negative).

```json
{
  "op": "shift",
  "rows": 8,
  "body": { "op": "ref", "name": "upbeat" }
}
```

### `slice` - Time Window

Keep only events in `[start, start+len)`.

```json
{
  "op": "slice",
  "start": 0,
  "len": 32,
  "body": { "op": "ref", "name": "long_pattern" }
}
```

### `ref` - Reference Def

Reference a reusable fragment from `defs`.

```json
{ "op": "ref", "name": "four_on_floor" }
```

Rules:
- `name` must exist in `defs`
- Refs can point to other refs (nesting allowed)
- Cycles are errors (`a -> b -> a`)

---

## Emission Operators

### `emit` - Single Cell at Positions

Emit a cell template at each position from a time expression.

```json
{
  "op": "emit",
  "at": { "op": "range", "start": 0, "step": 4, "count": 16 },
  "cell": {
    "channel": 0,
    "note": "C4",
    "inst": 0,
    "vol": 64
  }
}
```

### `emit_seq` - Sequence at Positions

Emit a sequence of values aligned to time positions.

```json
{
  "op": "emit_seq",
  "at": { "op": "range", "start": 0, "step": 4, "count": 8 },
  "cell": { "channel": 2, "inst": 3, "vol": 56 },
  "note_seq": {
    "mode": "cycle",
    "values": ["F1", "C2", "G1", "D2"]
  }
}
```

**Sequence modes:**
- `"cycle"` - Loop through values: `values[i % len]`
- `"once"` - One-to-one mapping; requires `values.len() == positions.len()`

Supported sequence fields:
- `note_seq` - Note values
- `vol_seq` - Volume values
- `inst_seq` - Instrument indices

---

## Time Expressions (TimeExpr)

### `range` - Arithmetic Sequence

Generate rows at regular intervals.

```json
{ "op": "range", "start": 0, "step": 4, "count": 16 }
// Produces: [0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 48, 52, 56, 60]
```

### `list` - Explicit Rows

Specify exact row indices.

```json
{ "op": "list", "rows": [0, 3, 8, 11, 16, 19] }
```

### `euclid` - Euclidean Rhythm

Distribute `pulses` hits evenly across `steps` using Bjorklund algorithm.

```json
{
  "op": "euclid",
  "pulses": 5,      // Number of hits
  "steps": 16,      // Pattern length in steps
  "rotate": 0,      // Rotate pattern (0 = no rotation)
  "stride": 4,      // Multiply step index to get row
  "offset": 0       // Add to final row positions
}
```

Example: `euclid(5, 16)` produces hits at steps [0, 3, 6, 9, 13] before stride/offset.

### `pattern` - Mini-Notation

Compact string notation where `x` = hit, `.` = rest.

```json
{
  "op": "pattern",
  "pattern": "x...x...x...x...",
  "stride": 1,
  "offset": 0
}
// Equivalent to list: [0, 4, 8, 12]
```

---

## Transform Operators

### `transform` - Apply Transforms

Wrap any pattern expression with transformations.

```json
{
  "op": "transform",
  "ops": [
    { "op": "transpose_semitones", "semitones": 12 },
    { "op": "vol_mul", "mul": 3, "div": 4 }
  ],
  "body": { "op": "ref", "name": "bass_line" }
}
```

### `transpose_semitones`

Shift notes up/down by semitones. Special notes (`"---"`, `"OFF"`) are preserved.

```json
{ "op": "transpose_semitones", "semitones": 12 }  // Up one octave
{ "op": "transpose_semitones", "semitones": -7 }  // Down a fifth
```

### `vol_mul`

Scale volume by rational number (avoids float nondeterminism).

```json
{ "op": "vol_mul", "mul": 3, "div": 4 }  // 75% volume
{ "op": "vol_mul", "mul": 1, "div": 2 }  // 50% volume
```

### `set`

Set fields on cells when not already present.

```json
{ "op": "set", "inst": 5, "vol": 48 }
```

---

## Variation Operators

### `prob` - Probabilistic Filtering

Deterministically drop events with given probability.

```json
{
  "op": "prob",
  "p_permille": 350,        // 35% chance to keep each event (0-1000)
  "seed_salt": "hat_ghosts", // Required: makes RNG unique
  "body": {
    "op": "emit",
    "at": { "op": "range", "start": 1, "step": 2, "count": 32 },
    "cell": { "channel": 5, "note": "C1", "inst": 2, "vol": 16 }
  }
}
```

### `choose` - Weighted Random Choice

Deterministically pick one branch based on weights.

```json
{
  "op": "choose",
  "seed_salt": "end_fill",
  "choices": [
    { "weight": 3, "body": { "op": "ref", "name": "no_fill" } },
    { "weight": 1, "body": { "op": "ref", "name": "snare_roll" } }
  ]
}
```

**Determinism:** Same `spec.seed` + `pattern_name` + `seed_salt` = same output.

---

## Cell Template

The basic unit produced by emission operators.

```json
{
  "channel": 0,              // Required: channel index (0-based)
  "note": "C4",              // Optional: "C4", "F#2", "---", "...", "OFF"
  "inst": 0,                 // Required: instrument index (0-based)
  "vol": 64,                 // Optional: 0-64
  "effect": 0,               // Optional: effect code
  "param": 0,                // Optional: effect parameter
  "effect_name": "arpeggio", // Optional: named effect (alternative to effect/param)
  "effect_xy": [1, 2]        // Optional: X, Y nibbles for effect param
}
```

**Note formats:**
- `"C4"`, `"F#2"`, `"Db5"` - Note + octave
- `"---"` or `"..."` - No note (rest)
- `"OFF"` - Note off
- `"^^^"` - Note cut (IT only)

---

## RFC-0004: Musical Helpers (Optional)

### Named Channels & Instruments

Use string aliases instead of numeric indices.

```json
{
  "channel_ids": { "kick": 0, "snare": 1, "hat": 2, "bass": 3 },
  "instrument_ids": { "kick": 0, "snare": 1, "hat": 2, "bass": 3 }
}
```

Then in cells:
```json
{ "channel": "kick", "inst": "kick", "note": "C4" }
```

### TimeBase (Bars/Beats)

Author in musical time instead of raw rows.

```json
{
  "timebase": {
    "beats_per_bar": 4,
    "rows_per_beat": 4
  }
}
```

Then patterns can specify `bars` instead of `rows`:
```json
{
  "patterns": {
    "verse": {
      "bars": 4,  // = 64 rows with default timebase
      "program": { ... }
    }
  }
}
```

Time expressions can use beats:
```json
{ "op": "beat_range", "start": {"bar": 0, "beat": 0}, "step": {"beats": 1}, "count": 16 }
```

### Harmony (Key/Chords)

Define key and chord progression for scale-aware authoring.

```json
{
  "harmony": {
    "key": { "root": "C", "scale": "minor" },
    "chords": [
      { "at": {"bar": 0}, "chord": {"symbol": "Cm"} },
      { "at": {"bar": 2}, "chord": {"symbol": "Fm"} },
      { "at": {"bar": 4}, "chord": {"symbol": "G7"} }
    ]
  }
}
```

### PitchSeq (Scale Degree Authoring)

Author melodies using scale degrees or chord tones.

```json
{
  "pitch_seq": {
    "kind": "scale_degree",  // or "chord_tone"
    "mode": "cycle",
    "values": ["1", "3", "5", "8"],
    "octave": 2
  }
}
```

---

## CLI Commands

```bash
# Validate compose spec
speccade validate --spec my_song.json

# Expand to canonical music.tracker_song_v1 JSON (for debugging)
speccade expand --spec my_song.json

# Generate XM/IT file
speccade generate --spec my_song.json --out-root ./output
```

---

## Examples

### Minimal Beat with External Instruments

```json
{
  "recipe": {
    "kind": "music.tracker_song_compose_v1",
    "params": {
      "format": "xm",
      "bpm": 150,
      "speed": 6,
      "channels": 4,
      "instruments": [
        {
          "name": "kick",
          "ref": "../audio/kick_punchy.spec.json",
          "envelope": { "attack": 0.001, "decay": 0.2, "sustain": 0.0, "release": 0.1 },
          "default_volume": 64
        },
        {
          "name": "hat",
          "ref": "../audio/hat_closed.spec.json",
          "envelope": { "attack": 0.001, "decay": 0.05, "sustain": 0.0, "release": 0.05 },
          "default_volume": 32
        }
      ],
      "patterns": {
        "p0": {
          "rows": 16,
          "program": {
            "op": "stack",
            "merge": "merge_fields",
            "parts": [
              {
                "op": "emit",
                "at": { "op": "range", "start": 0, "step": 4, "count": 4 },
                "cell": { "channel": 0, "note": "C4", "inst": 0, "vol": 64 }
              },
              {
                "op": "emit",
                "at": { "op": "range", "start": 0, "step": 1, "count": 16 },
                "cell": { "channel": 1, "note": "C1", "inst": 1, "vol": 32 }
              }
            ]
          }
        }
      },
      "arrangement": [{ "pattern": "p0", "repeat": 1 }]
    }
  }
}
```

### Full Song with Instrument Library

For production songs, create a shared instrument library:

```
my_project/
├── instruments/
│   ├── kick_punchy.spec.json      # audio_v1 spec
│   ├── snare_tight.spec.json
│   ├── hat_closed.spec.json
│   ├── bass_saw.spec.json
│   └── lead_fm.spec.json
└── songs/
    └── track_01.spec.json         # compose spec referencing ../instruments/
```

Then reference instruments by path:
```json
{
  "instruments": [
    { "name": "kick", "ref": "../instruments/kick_punchy.spec.json", "envelope": {...} },
    { "name": "snare", "ref": "../instruments/snare_tight.spec.json", "envelope": {...} },
    { "name": "bass", "ref": "../instruments/bass_saw.spec.json", "base_note": "C2", "envelope": {...} }
  ]
}
```

### Eurobeat 4 Bars with Defs and Variation

See `docs/examples/music/compose_eurobeat_4bars.json` for a complete example with:
- Reusable `defs` for kick, snare, hat layers
- `emit_seq` for bass and lead melodies
- `prob` for ghost note variation
- `choose` for random fills
- `stack` with `merge: "error"` at top level

---

## Best Practices

1. **Use external instruments (`ref`)** - Create reusable `audio_v1` specs for instruments; share across songs
2. **Use `defs` for reusable layers** - Extract kick, snare, hat patterns into defs
3. **Use `merge: "error"` at top level** - Catches accidental cell collisions early
4. **Use `last_wins` for intentional overrides** - Ghost notes, velocity shaping
5. **Keep variations small** - Use `prob`/`choose` for ghost notes and fills, not core structure
6. **Review expanded output** - Always run `speccade expand` before generating
7. **Use named channels/instruments** - Avoid index errors with RFC-0004 helpers
8. **Test incrementally** - Build patterns layer by layer, validating each step
9. **Avoid deprecated `synthesis`** - Only use for quick prototyping; prefer `ref` or `synthesis_audio_v1`
