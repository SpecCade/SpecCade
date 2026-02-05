# Music Tracker Format

SpecCade generates tracker modules in XM (FastTracker II) and IT (Impulse Tracker) formats. These are pattern-based music formats with channels, instruments, and effects.

## Recipe Types

### Basic Tracker Song
Pre-defined patterns and instruments.

```json
{
  "recipe": {
    "kind": "music.tracker_song_v1",
    "params": {
      "format": "xm",       // xm or it
      "tempo": 125,         // BPM
      "speed": 6,           // Ticks per row
      "channels": 8,        // Max channels
      "patterns": [...],
      "instruments": [...],
      "sequence": [0, 1, 0, 2]  // Pattern play order
    }
  }
}
```

### Compose IR (Pattern IR)

For compact, operator-based music authoring, use `music.tracker_song_compose_v1`.

See **`references/music-compose-ir.md`** for complete documentation of the Pattern IR format including:
- Structural operators (`stack`, `concat`, `repeat`, `ref`)
- Emission operators (`emit`, `emit_seq`)
- Time expressions (`range`, `list`, `euclid`, `pattern`)
- Variation operators (`prob`, `choose`)
- Musical helpers (named channels, timebase, harmony)

Use `speccade expand --spec FILE` to convert compose specs to canonical tracker JSON.

## XM Format Details

**Capabilities:**
- Up to 32 channels
- Up to 128 instruments
- Volume envelopes with sustain/loop points
- Panning envelopes
- Sample-based instruments

**Limitations:**
- No pitch envelopes
- 16-bit samples max
- Linear frequency slides only

## IT Format Details

**Capabilities:**
- Up to 64 channels
- New Note Action (NNA) for polyphony
- Pitch envelopes
- Filter envelopes
- Better sample interpolation
- 24-bit samples

**Advantages over XM:**
- More channels
- NNA allows held notes
- Filter support
- Better audio quality

## Pattern Structure

Patterns contain rows of note data across channels.

```json
{
  "patterns": [
    {
      "rows": 64,           // Pattern length (typically 64)
      "data": [
        {
          "row": 0,
          "channel": 0,
          "note": "C-4",    // Note + octave (C-0 to B-9)
          "instrument": 1,
          "volume": 64,     // 0-64
          "effect": "0",    // Effect column
          "effect_param": 0
        }
      ]
    }
  ]
}
```

**Note format:**
- `C-4` = C in octave 4
- `C#4` or `Db4` = C sharp / D flat
- `---` = no note
- `===` = note off
- `^^^` = note cut (IT only)

## Instruments

Each instrument must use **exactly one** source (mutually exclusive):

| Method | Field | Best For |
|--------|-------|----------|
| **External ref** (recommended) | `ref` | Reusable instruments across songs |
| **Inline audio_v1** | `synthesis_audio_v1` | Complex one-off instruments |
| **WAV sample** | `wav` | Pre-recorded samples |
| **Deprecated synthesis** | `synthesis` | Quick prototyping only |

### External Reference (Recommended)

```json
{
  "instruments": [
    {
      "name": "bass",
      "ref": "../audio/bass_saw.spec.json",
      "base_note": "C2",
      "envelope": { "attack": 0.01, "decay": 0.1, "sustain": 0.8, "release": 0.2 },
      "default_volume": 56
    }
  ]
}
```

The referenced file must be an `audio_v1` spec. Tracker-level `envelope` and `base_note` override the audio_v1 values.

### Inline Audio V1

```json
{
  "instruments": [
    {
      "name": "fm_bell",
      "synthesis_audio_v1": {
        "base_note": "C5",
        "duration_seconds": 2.0,
        "layers": [{ "synthesis": { "type": "fm_synth", "carrier_freq": 440.0, "modulator_freq": 880.0 }, "volume": 0.8 }]
      },
      "envelope": { "attack": 0.01, "decay": 0.3, "sustain": 0.4, "release": 0.5 }
    }
  ]
}
```

### Deprecated Synthesis (Prototyping Only)

```json
{
  "instruments": [
    { "name": "test", "base_note": "C4", "synthesis": { "type": "sine" } }
  ]
}
```

Deprecated types: `sine`, `saw`, `square`, `triangle`, `pulse`, `noise`

## Effects

Common tracker effects (effect column):

| Effect | Name | Description |
|--------|------|-------------|
| `0xy` | Arpeggio | Rapid note switch (x/y semitones) |
| `1xx` | Porta Up | Slide pitch up |
| `2xx` | Porta Down | Slide pitch down |
| `3xx` | Tone Porta | Slide to note |
| `4xy` | Vibrato | Pitch vibrato (speed/depth) |
| `Axy` | Vol Slide | Volume slide up/down |
| `Bxx` | Jump | Jump to pattern |
| `Cxx` | Set Volume | Set channel volume |
| `Dxx` | Pattern Break | Next pattern, row xx |
| `Exy` | Extended | Extended effects |
| `Fxx` | Set Speed/Tempo | Speed (<0x20) or BPM |

**IT-specific effects:**
| Effect | Name | Description |
|--------|------|-------------|
| `Gxx` | Glissando | Tone porta with quantize |
| `Hxy` | Tremor | On/off volume |
| `Zxx` | Filter | Set filter cutoff |

## Example: Simple Loop (music.tracker_song_v1)

```json
{
  "spec_version": 1,
  "asset_id": "simple_beat",
  "asset_type": "music",
  "license": "CC0-1.0",
  "seed": 42,
  "outputs": [{ "kind": "primary", "format": "xm", "path": "beat.xm" }],
  "recipe": {
    "kind": "music.tracker_song_v1",
    "params": {
      "format": "xm",
      "tempo": 120,
      "speed": 6,
      "channels": 4,
      "instruments": [
        {
          "name": "Kick",
          "ref": "../audio/kick_punchy.spec.json",
          "envelope": { "attack": 0.001, "decay": 0.2, "sustain": 0.0, "release": 0.1 }
        },
        {
          "name": "Snare",
          "ref": "../audio/snare_tight.spec.json",
          "envelope": { "attack": 0.001, "decay": 0.15, "sustain": 0.0, "release": 0.1 }
        }
      ],
      "patterns": [
        {
          "rows": 16,
          "data": [
            { "row": 0, "channel": 0, "note": "C-2", "instrument": 0, "volume": 64 },
            { "row": 4, "channel": 1, "note": "C-4", "instrument": 1, "volume": 48 },
            { "row": 8, "channel": 0, "note": "C-2", "instrument": 0, "volume": 64 },
            { "row": 12, "channel": 1, "note": "C-4", "instrument": 1, "volume": 48 }
          ]
        }
      ],
      "sequence": [0, 0, 0, 0]
    }
  }
}
```
