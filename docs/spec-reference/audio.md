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
| `effects` | array | no | `[]` | Effect chain applied after mixing (see Effects Chain section) |

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
| `filter` | object | no | Optional filter applied to this layer |
| `lfo` | object | no | Optional LFO modulation (see below) |

#### Envelope

```json
{
  "attack": 0.01,
  "decay": 0.1,
  "sustain": 0.5,
  "release": 0.2
}
```

#### LFO Modulation

Each layer can have an optional `lfo` field for Low Frequency Oscillator modulation. This enables effects like vibrato, tremolo, filter sweeps, and auto-panning.

```json
{
  "lfo": {
    "config": {
      "waveform": "sine",
      "rate": 5.0,
      "depth": 0.5,
      "phase": 0.0
    },
    "target": { "target": "pitch", "semitones": 2.0 }
  }
}
```

**LFO Config:**
| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `waveform` | string | yes | `sine`, `square`, `sawtooth`, `triangle`, or `pulse` |
| `rate` | number | yes | LFO rate in Hz (typically 0.1-20) |
| `depth` | number | yes | Modulation depth (0.0-1.0) |
| `phase` | number | no | Initial phase offset (0.0-1.0) |

**Modulation Targets:**
- `{ "target": "pitch", "semitones": 2.0 }` — Vibrato (pitch deviation in semitones)
- `{ "target": "volume" }` — Tremolo (amplitude modulation)
- `{ "target": "filter_cutoff", "amount": 1000.0 }` — Filter sweep (Hz change)
- `{ "target": "pan" }` — Auto-panning (stereo sweep)

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

#### `wavetable`

Wavetable synthesis generates sound from pre-computed waveform frames that can be smoothly morphed between. Supports unison mode with detuning for thick sounds.

```json
{
  "type": "wavetable",
  "table": "basic",
  "frequency": 440.0,
  "position": 0.5,
  "position_sweep": { "end_position": 1.0, "curve": "linear" },
  "voices": 4,
  "detune": 15.0
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `table` | string | yes | Wavetable source: `basic`, `analog`, `digital`, `pwm`, `formant`, `organ` |
| `frequency` | number | yes | Base frequency in Hz |
| `position` | number | no | Position in wavetable (0.0-1.0, default 0.0). Morphs between 64 frames |
| `position_sweep` | object | no | Sweep position over duration with `end_position` and `curve` |
| `voices` | integer | no | Unison voices (1-8, default 1) |
| `detune` | number | no | Detune amount in cents for unison spread |

**Wavetable Sources:**
- `basic` — Sine → saw → square → pulse morphing
- `analog` — Classic analog-style waves with harmonic content
- `digital` — Harsh digital tones with high harmonics
- `pwm` — Pulse width modulation (duty cycle 0.05 to 0.95)
- `formant` — Vocal-like formant synthesis
- `organ` — Drawbar organ harmonics (9 simulated drawbars)

#### `granular`

Granular synthesis generates sound by combining many short audio fragments called "grains". Produces evolving, textured sounds with natural irregularity.

```json
{
  "type": "granular",
  "source": { "type": "noise", "noise_type": "white" },
  "grain_size_ms": 50.0,
  "grain_density": 20.0,
  "pitch_spread": 2.0,
  "position_spread": 0.3,
  "pan_spread": 0.8
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `source` | object | yes | Grain source material (see below) |
| `grain_size_ms` | number | yes | Grain size in milliseconds (10-500) |
| `grain_density` | number | yes | Grains per second (1-100) |
| `pitch_spread` | number | no | Random pitch variation in semitones (default 0) |
| `position_spread` | number | no | Timing jitter as fraction of grain interval (0.0-1.0, default 0) |
| `pan_spread` | number | no | Stereo spread (0.0 = mono, 1.0 = full stereo, default 0) |

**Granular Sources:**
- `{ "type": "noise", "noise_type": "white" }` — White, pink, or brown noise grains
- `{ "type": "tone", "waveform": "sine", "frequency": 440.0 }` — Pitched waveform grains
- `{ "type": "formant", "frequency": 220.0, "formant_freq": 800.0 }` — Formant-based grains

### Filters

Filters are tagged unions with `type`:

- `lowpass`: `{ "type": "lowpass", "cutoff": 2000.0, "resonance": 0.7, "cutoff_end": 500.0 }`
- `highpass`: `{ "type": "highpass", "cutoff": 200.0, "resonance": 0.7, "cutoff_end": 2000.0 }`
- `bandpass`: `{ "type": "bandpass", "center": 800.0, "resonance": 0.7, "center_end": 1200.0 }`

Sweep fields like `cutoff_end` / `center_end` are optional.

### Effects Chain

The `effects` array on `AudioV1Params` applies post-processing effects after mixing all layers. Effects are processed in order.

```json
{
  "effects": [
    { "type": "reverb", "room_size": 0.7, "damping": 0.5, "wet": 0.3, "width": 1.0 },
    { "type": "compressor", "threshold_db": -12, "ratio": 4.0, "attack_ms": 5, "release_ms": 100 }
  ]
}
```

#### `reverb`

Freeverb-based room reverb.

```json
{ "type": "reverb", "room_size": 0.7, "damping": 0.5, "wet": 0.3, "width": 1.0 }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `room_size` | number | yes | — | Room size (0.0-1.0) |
| `damping` | number | yes | — | High-frequency absorption (0.0-1.0) |
| `wet` | number | yes | — | Wet/dry mix (0.0-1.0) |
| `width` | number | no | `1.0` | Stereo width (0.0-1.0) |

#### `delay`

Echo/delay effect with optional ping-pong mode.

```json
{ "type": "delay", "time_ms": 250, "feedback": 0.4, "wet": 0.25, "ping_pong": true }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `time_ms` | number | yes | — | Delay time in ms (1-2000) |
| `feedback` | number | yes | — | Feedback amount (0.0-0.95) |
| `wet` | number | yes | — | Wet/dry mix (0.0-1.0) |
| `ping_pong` | boolean | no | `false` | Stereo ping-pong mode |

#### `chorus`

Modulated delay for thickening/detuning.

```json
{ "type": "chorus", "rate": 1.5, "depth": 0.4, "wet": 0.3, "voices": 2 }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `rate` | number | yes | — | LFO rate in Hz |
| `depth` | number | yes | — | Modulation depth (0.0-1.0) |
| `wet` | number | yes | — | Wet/dry mix (0.0-1.0) |
| `voices` | integer | no | `2` | Number of voices (1-4) |

#### `phaser`

Allpass filter sweep effect.

```json
{ "type": "phaser", "rate": 0.5, "depth": 0.6, "stages": 4, "wet": 0.5 }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `rate` | number | yes | — | LFO rate in Hz |
| `depth` | number | yes | — | Modulation depth (0.0-1.0) |
| `stages` | integer | yes | — | Number of allpass stages (2-12) |
| `wet` | number | yes | — | Wet/dry mix (0.0-1.0) |

#### `bitcrush`

Digital degradation effect.

```json
{ "type": "bitcrush", "bits": 8, "sample_rate_reduction": 4.0 }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `bits` | integer | yes | — | Bit depth (1-16) |
| `sample_rate_reduction` | number | no | `1.0` | Sample rate reduction factor |

#### `waveshaper`

Distortion/saturation effect.

```json
{ "type": "waveshaper", "drive": 5.0, "curve": "tanh", "wet": 0.5 }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `drive` | number | yes | — | Drive amount (1.0-100.0) |
| `curve` | string | no | `tanh` | Shaping curve: `tanh`, `soft_clip`, `hard_clip`, `sine` |
| `wet` | number | yes | — | Wet/dry mix (0.0-1.0) |

#### `compressor`

Dynamics compression.

```json
{ "type": "compressor", "threshold_db": -12, "ratio": 4.0, "attack_ms": 5, "release_ms": 100, "makeup_db": 3 }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `threshold_db` | number | yes | — | Threshold in dB (-60 to 0) |
| `ratio` | number | yes | — | Compression ratio (1.0-20.0) |
| `attack_ms` | number | yes | — | Attack time in ms (0.1-100) |
| `release_ms` | number | yes | — | Release time in ms (10-1000) |
| `makeup_db` | number | no | `0` | Makeup gain in dB |

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
