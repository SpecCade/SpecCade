# Audio Spec Reference

This document covers audio-related asset types and recipes in SpecCade: sound effects and instrument samples.

## Table of Contents

- [Audio SFX](#audio-sfx)
  - [Layered Synth (layered_synth_v1)](#recipe-audio_sfxlayered_synth_v1)
  - [Synthesis Types](#synthesis-types)
  - [Filters](#filters)
  - [Envelopes](#envelopes)
- [Audio Instrument](#audio-instrument)
  - [Synth Patch (synth_patch_v1)](#recipe-audio_instrumentsynth_patch_v1)

---

## Audio SFX

**Asset Type:** `audio_sfx`
**Recipe Kinds:** `audio_sfx.layered_synth_v1`
**Output Formats:** WAV, OGG

One-shot sound effects using layered synthesis.

### Recipe: `audio_sfx.layered_synth_v1`

#### Required Params

| Param | Type | Description |
|-------|------|-------------|
| `duration_seconds` | float | Total duration in seconds |
| `layers` | array | Synthesis layers |

#### Optional Params

| Param | Type | Description | Default |
|-------|------|-------------|---------|
| `sample_rate` | integer | Sample rate in Hz | `44100` |
| `normalize` | boolean | Normalize to peak amplitude | `true` |
| `peak_db` | float | Target peak level in dB | `-1.0` |
| `master_envelope` | object | Master ADSR envelope | None |
| `master_filter` | object | Master filter | None |

#### Layer Structure

Each layer in the `layers` array defines an independent synthesis voice:

```json
{
  "synthesis": { ... },
  "envelope": { ... },
  "volume": 0.8,
  "pan": 0.0,
  "delay": 0.0,
  "duration": 0.5,
  "filter": { ... }
}
```

| Field | Type | Description | Required |
|-------|------|-------------|----------|
| `synthesis` | object | Synthesis configuration | Yes |
| `envelope` | object | ADSR envelope | Yes |
| `volume` | float | Layer amplitude (0.0-1.0) | No (default: 1.0) |
| `pan` | float | Stereo pan (-1.0 to 1.0) | No (default: 0.0) |
| `delay` | float | Start delay in seconds | No (default: 0.0) |
| `duration` | float | Per-layer duration override | No |
| `filter` | object | Layer filter | No |
| `comment` | string | Documentation comment | No |

---

### Synthesis Types

#### Oscillator

Basic waveform oscillator with optional frequency sweep.

```json
{
  "type": "oscillator",
  "waveform": "sine",
  "frequency": 440.0,
  "duty": 0.5,
  "freq_sweep": {
    "end_freq": 220.0,
    "curve": "exponential"
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `waveform` | string | `"sine"`, `"square"`, `"saw"`, `"triangle"` |
| `frequency` | float | Base frequency in Hz |
| `duty` | float | Duty cycle for square wave (0.0-1.0) |
| `freq_sweep.end_freq` | float | Target frequency for sweep |
| `freq_sweep.curve` | string | `"linear"` or `"exponential"` |

#### FM Synth

Frequency modulation synthesis.

```json
{
  "type": "fm_synth",
  "carrier_freq": 440.0,
  "mod_ratio": 2.0,
  "mod_index": 5.0,
  "index_decay": 3.0
}
```

| Field | Type | Description |
|-------|------|-------------|
| `carrier_freq` | float | Carrier frequency in Hz |
| `mod_ratio` | float | Modulator frequency ratio |
| `mod_index` | float | Modulation index (depth) |
| `index_decay` | float | Modulation index decay rate |

#### Karplus-Strong

Plucked string synthesis using Karplus-Strong algorithm.

```json
{
  "type": "karplus_strong",
  "frequency": 196.0,
  "damping": 0.996,
  "brightness": 0.7
}
```

| Field | Type | Description |
|-------|------|-------------|
| `frequency` | float | Fundamental frequency in Hz |
| `damping` | float | String damping factor (0.99-0.999) |
| `brightness` | float | High-frequency content (0.0-1.0) |

#### Noise Burst

Filtered noise generator.

```json
{
  "type": "noise_burst",
  "noise_type": "white"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `noise_type` | string | `"white"`, `"pink"`, `"brown"` |

#### Pitched Body

Frequency sweep oscillator, useful for drum-like sounds.

```json
{
  "type": "pitched_body",
  "start_freq": 200.0,
  "end_freq": 50.0
}
```

| Field | Type | Description |
|-------|------|-------------|
| `start_freq` | float | Starting frequency in Hz |
| `end_freq` | float | Ending frequency in Hz |

#### Metallic

Inharmonic partials for metallic/bell-like sounds.

```json
{
  "type": "metallic",
  "base_freq": 800.0,
  "num_partials": 6,
  "inharmonicity": 1.414
}
```

| Field | Type | Description |
|-------|------|-------------|
| `base_freq` | float | Base frequency in Hz |
| `num_partials` | integer | Number of partials |
| `inharmonicity` | float | Inharmonicity factor (1.0 = harmonic) |

#### Harmonics (Additive)

Additive synthesis with explicit harmonic frequencies and amplitudes.

```json
{
  "type": "harmonics",
  "freqs": [220.0, 440.0, 660.0, 880.0, 1100.0],
  "amplitudes": [1.0, 0.5, 0.33, 0.25, 0.2]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `freqs` | array | Frequencies in Hz |
| `amplitudes` | array | Amplitudes for each frequency |

---

### Filters

Filters can be applied per-layer or as a master filter.

#### Lowpass Filter

```json
{
  "type": "lowpass",
  "cutoff": 2000.0,
  "cutoff_end": 500.0,
  "resonance": 1.5
}
```

| Field | Type | Description |
|-------|------|-------------|
| `cutoff` | float | Cutoff frequency in Hz |
| `cutoff_end` | float | End cutoff for sweep (optional) |
| `resonance` | float | Filter resonance (Q) |

#### Highpass Filter

```json
{
  "type": "highpass",
  "cutoff": 500.0,
  "resonance": 0.7
}
```

#### Bandpass Filter

```json
{
  "type": "bandpass",
  "cutoff": 2000.0,
  "cutoff_low": 1000.0,
  "cutoff_high": 4000.0,
  "resonance": 1.0
}
```

| Field | Type | Description |
|-------|------|-------------|
| `cutoff_low` | float | Lower cutoff frequency |
| `cutoff_high` | float | Upper cutoff frequency |

---

### Envelopes

ADSR (Attack, Decay, Sustain, Release) envelope for amplitude shaping.

```json
{
  "attack": 0.01,
  "decay": 0.1,
  "sustain": 0.7,
  "release": 0.2
}
```

| Field | Type | Description | Range |
|-------|------|-------------|-------|
| `attack` | float | Attack time in seconds | >= 0 |
| `decay` | float | Decay time in seconds | >= 0 |
| `sustain` | float | Sustain level | 0.0 - 1.0 |
| `release` | float | Release time in seconds | >= 0 |

---

### Example: Laser Shot

```json
{
  "spec_version": 1,
  "asset_id": "laser_shot",
  "asset_type": "audio_sfx",
  "license": "CC0-1.0",
  "seed": 1002,
  "description": "FM synthesis laser shot - classic arcade sound",
  "outputs": [
    {"kind": "primary", "format": "wav", "path": "laser_shot.wav"}
  ],
  "recipe": {
    "kind": "audio_sfx.layered_synth_v1",
    "params": {
      "duration_seconds": 0.25,
      "sample_rate": 44100,
      "normalize": true,
      "peak_db": -1.0,
      "layers": [
        {
          "synthesis": {
            "type": "fm_synth",
            "carrier_freq": 1200,
            "mod_ratio": 2.5,
            "mod_index": 8.0,
            "index_decay": 20.0
          },
          "volume": 0.9,
          "envelope": {
            "attack": 0.001,
            "decay": 0.1,
            "sustain": 0.3,
            "release": 0.1
          }
        }
      ]
    }
  }
}
```

### Example: Explosion

```json
{
  "spec_version": 1,
  "asset_id": "explosion",
  "asset_type": "audio_sfx",
  "license": "CC0-1.0",
  "seed": 2001,
  "description": "Layered explosion with noise and pitched body",
  "outputs": [
    {"kind": "primary", "format": "wav", "path": "explosion.wav"}
  ],
  "recipe": {
    "kind": "audio_sfx.layered_synth_v1",
    "params": {
      "duration_seconds": 1.5,
      "sample_rate": 44100,
      "normalize": true,
      "peak_db": -0.5,
      "layers": [
        {
          "comment": "Low thump",
          "synthesis": {
            "type": "pitched_body",
            "start_freq": 150.0,
            "end_freq": 30.0
          },
          "volume": 0.8,
          "envelope": {
            "attack": 0.001,
            "decay": 0.3,
            "sustain": 0.1,
            "release": 0.5
          }
        },
        {
          "comment": "Noise burst",
          "synthesis": {
            "type": "noise_burst",
            "noise_type": "brown"
          },
          "filter": {
            "type": "lowpass",
            "cutoff": 2000.0,
            "cutoff_end": 300.0,
            "resonance": 0.7
          },
          "volume": 0.6,
          "envelope": {
            "attack": 0.001,
            "decay": 0.5,
            "sustain": 0.2,
            "release": 0.8
          }
        }
      ]
    }
  }
}
```

---

## Audio Instrument

**Asset Type:** `audio_instrument`
**Recipe Kinds:** `audio_instrument.synth_patch_v1`
**Output Formats:** WAV

Single-note instrument samples for tracker modules.

### Recipe: `audio_instrument.synth_patch_v1`

#### Required Params

| Param | Type | Description |
|-------|------|-------------|
| `synthesis` | object | Synthesis configuration |
| `envelope` | object | ADSR envelope |

#### Optional Params

| Param | Type | Description | Default |
|-------|------|-------------|---------|
| `base_note` | string | Base note (e.g., `"C4"`) | `"C4"` |
| `note_duration_seconds` | float | Sample duration | `1.0` |
| `sample_rate` | integer | Sample rate in Hz | `44100` |
| `notes` | array | Notes to generate (MIDI numbers or names) | `[60]` (C4) |
| `generate_loop_points` | boolean | Generate loop points for sustain | `false` |
| `pitch_envelope` | object | Pitch modulation envelope | None |
| `output` | object | Output settings | Default |

#### Synthesis Types

The instrument synthesis types are similar to audio_sfx but with some additional options:

##### Basic Oscillator Types

Simple waveform names for quick instrument definition:

```json
{
  "synthesis": {
    "type": "sine"
  }
}
```

Available types: `"sine"`, `"square"`, `"saw"`, `"sawtooth"`, `"triangle"`, `"noise"`

##### FM Synthesis

Multi-operator FM synthesis for complex timbres:

```json
{
  "synthesis": {
    "type": "fm",
    "operators": [
      {"ratio": 1.0, "level": 1.0, "envelope": {...}},
      {"ratio": 2.0, "level": 0.8, "envelope": {...}},
      {"ratio": 3.5, "level": 0.5, "envelope": {...}}
    ],
    "index": 4.0,
    "index_decay": 3.0
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `operators` | array | FM operator definitions |
| `operators[].ratio` | float | Frequency ratio |
| `operators[].level` | float | Operator level |
| `operators[].envelope` | object | Per-operator ADSR |
| `index` | float | Base modulation index |
| `index_decay` | float | Index decay rate |

##### Simple FM Synth

Simplified FM with carrier and modulator:

```json
{
  "synthesis": {
    "type": "fm_synth",
    "carrier_freq": 440.0,
    "modulator_freq": 880.0,
    "modulation_index": 2.5,
    "freq_sweep": {
      "end_freq": 110.0,
      "curve": "exponential"
    }
  }
}
```

##### Karplus-Strong

Plucked string synthesis:

```json
{
  "synthesis": {
    "type": "karplus_strong",
    "frequency": 440.0,
    "decay": 0.996,
    "blend": 0.7
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `frequency` | float | Fundamental frequency |
| `decay` | float | Damping factor (0.99-0.999) |
| `blend` | float | Brightness/blend factor |

##### Subtractive Synthesis

Filtered oscillator bank:

```json
{
  "synthesis": {
    "type": "subtractive",
    "oscillators": [
      {"waveform": "saw", "detune": 0},
      {"waveform": "saw", "detune": 7},
      {"waveform": "saw", "detune": -7}
    ],
    "filter": {
      "type": "lowpass",
      "cutoff": 4000.0,
      "cutoff_end": 800.0,
      "resonance": 2.0
    }
  }
}
```

##### Additive Synthesis

Explicit partials definition:

```json
{
  "synthesis": {
    "type": "additive",
    "partials": [
      [1.0, 1.0],
      [2.0, 0.5],
      [3.0, 0.33],
      [4.0, 0.25]
    ]
  }
}
```

Each partial is `[frequency_ratio, amplitude]`.

#### Pitch Envelope

Optional pitch modulation over time:

```json
{
  "pitch_envelope": {
    "attack": 0.01,
    "decay": 0.1,
    "sustain": 0.0,
    "release": 0.05,
    "depth": 12.0
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `depth` | float | Pitch deviation in semitones |

#### Output Settings

```json
{
  "output": {
    "duration": 2.0,
    "bit_depth": 16
  }
}
```

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `duration` | float | Output duration in seconds | `1.0` |
| `bit_depth` | integer | Bit depth (8, 16, 24) | `16` |

#### Multi-Note Generation

Generate samples at multiple pitches:

```json
{
  "notes": ["C3", "C4", "C5"]
}
```

Or using MIDI note numbers:

```json
{
  "notes": [48, 60, 72]
}
```

---

### Example: Plucked Bass

```json
{
  "spec_version": 1,
  "asset_id": "bass_pluck",
  "asset_type": "audio_instrument",
  "license": "CC0-1.0",
  "seed": 2001,
  "description": "Bass pluck - Karplus-Strong synthesis",
  "outputs": [
    {"kind": "primary", "format": "wav", "path": "bass_pluck.wav"}
  ],
  "recipe": {
    "kind": "audio_instrument.synth_patch_v1",
    "params": {
      "note_duration_seconds": 1.5,
      "sample_rate": 44100,
      "synthesis": {
        "type": "karplus_strong",
        "frequency": 440.0,
        "decay": 0.996,
        "blend": 0.7
      },
      "envelope": {
        "attack": 0.001,
        "decay": 0.05,
        "sustain": 0.9,
        "release": 0.5
      },
      "notes": ["C3", "C4", "C5"],
      "generate_loop_points": true
    }
  }
}
```

### Example: FM Bell

```json
{
  "spec_version": 1,
  "asset_id": "fm_bell",
  "asset_type": "audio_instrument",
  "license": "CC0-1.0",
  "seed": 3001,
  "description": "FM bell with inharmonic partials",
  "outputs": [
    {"kind": "primary", "format": "wav", "path": "fm_bell.wav"}
  ],
  "recipe": {
    "kind": "audio_instrument.synth_patch_v1",
    "params": {
      "base_note": "C4",
      "sample_rate": 44100,
      "synthesis": {
        "type": "fm",
        "operators": [
          {
            "ratio": 1.0,
            "level": 1.0,
            "envelope": {"attack": 0.001, "decay": 0.3, "sustain": 0.2, "release": 0.8}
          },
          {
            "ratio": 3.5,
            "level": 0.6,
            "envelope": {"attack": 0.001, "decay": 0.5, "sustain": 0.1, "release": 1.0}
          }
        ],
        "index": 5.0,
        "index_decay": 2.0
      },
      "envelope": {
        "attack": 0.001,
        "decay": 0.5,
        "sustain": 0.2,
        "release": 1.0
      },
      "output": {
        "duration": 2.0,
        "bit_depth": 16
      }
    }
  }
}
```

---

## Validation Rules

### Audio-Specific Validation

| Rule | Error Code | Description |
|------|------------|-------------|
| Duration must be positive | E030 | Invalid duration_seconds |
| Sample rate must be valid | E031 | Sample rate not 22050, 44100, or 48000 |
| Layer must have synthesis | E032 | Missing synthesis in layer |
| Layer must have envelope | E033 | Missing envelope in layer |
| Waveform must be known | E034 | Unknown waveform type |
| Synthesis type must be known | E035 | Unknown synthesis type |
| Frequencies must be positive | E036 | Invalid frequency value |
| Envelope values must be non-negative | E037 | Negative envelope time |

---

## Golden Corpus Specs

Reference implementations:

- `golden/speccade/audio_sfx_comprehensive.spec.json` - All synthesis types
- `golden/speccade/audio_instrument_comprehensive.spec.json` - Complex FM instrument
- `golden/speccade/audio_instrument_karplus.spec.json` - Plucked string
- `golden/speccade/audio_instrument_fm_simple.spec.json` - Basic FM
- `golden/speccade/audio_instrument_additive.spec.json` - Additive synthesis
- `golden/speccade/audio_instrument_subtractive.spec.json` - Subtractive synthesis
