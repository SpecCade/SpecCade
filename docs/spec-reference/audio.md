# Audio Spec Reference

This document covers audio generation in SpecCade.

## Overview

**Asset Type:** `audio`  
**Recipe Kinds:** `audio_v1`  
**Output Formats:** WAV

Audio generation uses a unified layered synthesis recipe for both one-shot SFX and pitched instrument samples.

**SSOT:** The authoritative `audio_v1` parameter surface is the Rust type `AudioV1Params` in `crates/speccade-spec/src/recipe/audio/`.

## Outputs

For `audio_v1`, `speccade generate` writes exactly one `primary` output:

- `outputs[]` must contain **exactly one** entry with `kind: "primary"`.
- The `primary` output must have `format: "wav"` and a `.wav` path.
- Other output kinds like `metadata` / `preview` are reserved and currently rejected by validation.

Example:

```json
{
  "kind": "primary",
  "format": "wav",
  "path": "sounds/laser_shot.wav"
}
```

Note: `speccade validate` and `speccade generate` write a `${asset_id}.report.json` sibling file next to the spec file. This is the structured metadata output today.

## Recipe: `audio_v1`

### Params

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `base_note` | string \| integer | no | omitted | MIDI note (0–127) or note name (e.g. `"C4"`) |
| `duration_seconds` | number | yes | — | Total rendered length in seconds |
| `sample_rate` | integer | no | `44100` | Sample rate in Hz |
| `layers` | array | yes | — | Synthesis layers to combine (can be empty for silence) |
| `pitch_envelope` | object | no | omitted | Pitch modulation envelope applied to all layers |
| `generate_loop_points` | boolean | no | `false` | If true, backend sets a loop point (currently `attack + decay` of the first layer envelope) |
| `master_filter` | object | no | omitted | Filter applied after mixing layers |

### Base Note Semantics (Music Integration)

`base_note` is primarily used by the music backend when an `audio` spec is referenced from a music instrument `ref`. It describes the pitch the rendered sample represents (so the tracker can transpose correctly).

### Mixing / Normalization

After layers are mixed (and `master_filter` is applied, if present), the backend normalizes the final signal to **-3 dB peak headroom** to prevent clipping. There is currently no per-spec flag to disable this; treat layer `volume` as a relative balance tool, not an absolute loudness guarantee.

### Audio Layers

Each entry in `layers[]` is an object with:

| Field | Type | Required | Notes |
|------:|------|:--------:|------|
| `synthesis` | object | yes | Synthesis configuration (see below) |
| `envelope` | object | yes | ADSR envelope |
| `volume` | number | yes | Recommended range `0.0..=1.0` |
| `pan` | number | yes | Recommended range `-1.0..=1.0` |
| `delay` | number | no | Layer start delay in seconds |

#### Envelope

```json
{
  "attack": 0.01,
  "decay": 0.1,
  "sustain": 0.5,
  "release": 0.2
}
```

### Synthesis Types

`synthesis` is a tagged union with `type`:

For the authoritative list of synthesis variants/fields, see `crates/speccade-spec/src/recipe/audio/synthesis.rs`.

#### `oscillator`

```json
{
  "type": "oscillator",
  "waveform": "sine",
  "frequency": 440.0,
  "freq_sweep": { "end_freq": 220.0, "curve": "exponential" },
  "detune": 7.0,
  "duty": 0.5
}
```

#### `fm_synth`

```json
{
  "type": "fm_synth",
  "carrier_freq": 440.0,
  "modulator_freq": 880.0,
  "modulation_index": 4.0,
  "freq_sweep": { "end_freq": 110.0, "curve": "linear" }
}
```

#### `karplus_strong`

```json
{ "type": "karplus_strong", "frequency": 110.0, "decay": 0.996, "blend": 0.7 }
```

#### `noise_burst`

```json
{ "type": "noise_burst", "noise_type": "white", "filter": { "type": "lowpass", "cutoff": 2000.0, "resonance": 0.7 } }
```

#### `additive`

```json
{ "type": "additive", "base_freq": 220.0, "harmonics": [1.0, 0.5, 0.33, 0.25] }
```

#### `multi_oscillator`

```json
{
  "type": "multi_oscillator",
  "frequency": 220.0,
  "oscillators": [
    { "waveform": "sawtooth", "volume": 1.0, "detune": 0.0 },
    { "waveform": "square", "volume": 0.8, "detune": 7.0, "duty": 0.4 }
  ],
  "freq_sweep": { "end_freq": 110.0, "curve": "exponential" }
}
```

#### `pitched_body`

```json
{ "type": "pitched_body", "start_freq": 600.0, "end_freq": 120.0 }
```

#### `metallic`

```json
{ "type": "metallic", "base_freq": 220.0, "num_partials": 8, "inharmonicity": 1.6 }
```

### Filters

Filters are tagged unions with `type`:

- `lowpass`: `{ "type": "lowpass", "cutoff": 2000.0, "resonance": 0.7, "cutoff_end": 500.0 }`
- `highpass`: `{ "type": "highpass", "cutoff": 200.0, "resonance": 0.7, "cutoff_end": 2000.0 }`
- `bandpass`: `{ "type": "bandpass", "center": 800.0, "resonance": 0.7, "center_end": 1200.0 }`

Sweep fields like `cutoff_end` / `center_end` are optional.

## Examples

### Minimal Beep

```json
{
  "spec_version": 1,
  "asset_id": "beep_01",
  "asset_type": "audio",
  "license": "CC0-1.0",
  "seed": 1,
  "outputs": [{ "kind": "primary", "format": "wav", "path": "beep_01.wav" }],
  "recipe": {
    "kind": "audio_v1",
    "params": {
      "duration_seconds": 0.2,
      "layers": [
        {
          "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
          "envelope": { "attack": 0.01, "decay": 0.05, "sustain": 0.6, "release": 0.05 },
          "volume": 0.8,
          "pan": 0.0
        }
      ]
    }
  }
}
```
