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

#### `am_synth`

AM (Amplitude Modulation) synthesis modulates the amplitude of a carrier oscillator with a modulator oscillator. Produces tremolo effects and sidebands at (carrier +/- modulator) frequencies. Common uses include tremolo guitar, broadcast-style AM, and siren sounds.

```json
{
  "type": "am_synth",
  "carrier_freq": 440.0,
  "modulator_freq": 5.0,
  "modulation_depth": 0.8,
  "freq_sweep": { "end_freq": 220.0, "curve": "exponential" }
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `carrier_freq` | number | yes | Carrier frequency in Hz |
| `modulator_freq` | number | yes | Modulator frequency in Hz (typically lower for tremolo) |
| `modulation_depth` | number | yes | Modulation depth (0.0-1.0, where 1.0 = 100% modulation) |
| `freq_sweep` | object | no | Optional frequency sweep for carrier |

**Common Presets:**
- Tremolo: Low modulator frequency (4-8 Hz) for pulsing amplitude
- Broadcast AM: Moderate modulator frequency for radio-style modulation
- Siren: Higher modulator frequency (10-15 Hz) for warbling effects

#### `ring_mod_synth`

Ring Modulation synthesis multiplies two oscillators together to create sum and difference frequencies. Produces metallic timbres, robotic sounds, and bell-like tones. Unlike AM synthesis (which preserves the carrier), ring modulation produces only sidebands, resulting in more complex and inharmonic spectra.

```json
{
  "type": "ring_mod_synth",
  "carrier_freq": 440.0,
  "modulator_freq": 150.0,
  "mix": 0.8,
  "freq_sweep": { "end_freq": 220.0, "curve": "exponential" }
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `carrier_freq` | number | yes | Carrier frequency in Hz |
| `modulator_freq` | number | yes | Modulator frequency in Hz |
| `mix` | number | yes | Wet/dry mix (0.0 = dry carrier, 1.0 = full ring mod) |
| `freq_sweep` | object | no | Optional frequency sweep for carrier |

**Common Presets:**
- Metallic: Carrier 440 Hz, modulator 150 Hz for shimmering metallic tones
- Robotic: Carrier 220 Hz, modulator 80 Hz for robotic voice effects
- Bell: Carrier 880 Hz, modulator 220 Hz for bell-like timbres
- Dissonant: Close frequency ratios (e.g., carrier 440 Hz, modulator 430 Hz) for harsh dissonance
- Sci-Fi: Carrier 330 Hz, modulator 100 Hz for retro sci-fi effects

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

#### `pd_synth`

Phase Distortion synthesis (Casio CZ style). Creates complex timbres by warping the phase of a waveform non-linearly. The distortion amount typically decays over time, creating sounds that start bright and evolve to pure tones. Ideal for classic 80s synth sounds and evolving timbres.

```json
{
  "type": "pd_synth",
  "frequency": 220.0,
  "distortion": 4.0,
  "distortion_decay": 6.0,
  "waveform": "resonant",
  "freq_sweep": { "end_freq": 110.0, "curve": "exponential" }
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `frequency` | number | yes | Base frequency in Hz |
| `distortion` | number | yes | Initial distortion amount (0.0 = pure sine, higher = more harmonics). Typical range: 0.0-10.0 |
| `distortion_decay` | number | no | Distortion decay rate (higher = faster decay to pure sine). Default: 0.0 |
| `waveform` | string | yes | Distortion curve: `resonant`, `sawtooth`, or `pulse` |
| `freq_sweep` | object | no | Optional frequency sweep |

**Waveform Types:**
- `resonant` — Resonant filter-like tones using power curve distortion
- `sawtooth` — Asymmetric distortion creating saw-like harmonics
- `pulse` — Sharp phase transition creating square/pulse wave characteristics

**Common Presets:**
- Bass: `distortion: 4.0`, `distortion_decay: 6.0`, `waveform: "resonant"` (40-120 Hz)
- Organ: `distortion: 2.5`, `distortion_decay: 0.5`, `waveform: "sawtooth"` (200-800 Hz)
- Strings: `distortion: 3.0`, `distortion_decay: 2.0`, `waveform: "pulse"` (200-600 Hz)

#### `modal`

Modal synthesis simulates struck or bowed objects (bells, chimes, marimbas, xylophones) by modeling their resonant modes. Each mode is a decaying sine wave at a specific frequency ratio with its own amplitude and decay time. Ideal for metallic percussion, wooden bars, and bell-like tones.

```json
{
  "type": "modal",
  "frequency": 440.0,
  "modes": [
    { "freq_ratio": 1.0, "amplitude": 1.0, "decay_time": 2.0 },
    { "freq_ratio": 2.4, "amplitude": 0.6, "decay_time": 1.5 },
    { "freq_ratio": 4.0, "amplitude": 0.4, "decay_time": 1.0 }
  ],
  "excitation": "impulse",
  "freq_sweep": { "end_freq": 220.0, "curve": "exponential" }
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `frequency` | number | yes | Base frequency in Hz |
| `modes` | array | yes | Array of Mode objects defining resonant frequencies |
| `excitation` | string | yes | Excitation type: `impulse`, `noise`, or `pluck` |
| `freq_sweep` | object | no | Optional frequency sweep for all modes |

**Mode Parameters:**
| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `freq_ratio` | number | yes | Frequency ratio relative to fundamental (1.0 = fundamental) |
| `amplitude` | number | yes | Amplitude of this mode (0.0-1.0) |
| `decay_time` | number | yes | Decay time in seconds |

**Excitation Types:**
- `impulse` — Single impulse (sharp attack, like a hard mallet striking)
- `noise` — Noise burst (softer, more complex attack)
- `pluck` — Pluck-like excitation (quick attack with harmonic content)

**Common Presets:**
- Bell: Inharmonic partials (1.0, 2.0, 2.4, 3.0, 4.0) with long decay (3-4s)
- Chime: Tubular bell partials (1.0, 2.76, 5.4, 8.93) with medium decay (2-3s)
- Marimba: Warm wooden tone with near-harmonic partials (1.0, 4.0, 9.0) and short decay (1-1.5s)
- Glockenspiel: Bright metal bar tone with strong high partials (1.0, 2.71, 5.28, 8.65, 12.81) and long decay (2-2.5s)
- Vibraphone: Pure sustained tone (1.0, 4.0, 10.0) with very long decay (3.5s)
- Xylophone: Dry bright tone with short decay (1.0, 3.0, 6.0, 10.0) and attack transients (0.5-0.8s)

#### `vocoder`

Vocoder synthesis transfers spectral envelopes through a filter bank to create robot voices, vocal textures, and talking synths. The vocoder splits a carrier signal into frequency bands and applies time-varying amplitude envelopes (simulating formant patterns) to each band.

```json
{
  "type": "vocoder",
  "carrier_freq": 220.0,
  "carrier_type": "sawtooth",
  "num_bands": 16,
  "band_spacing": "logarithmic",
  "envelope_attack": 0.005,
  "envelope_release": 0.020,
  "formant_rate": 3.0,
  "bands": []
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `carrier_freq` | number | yes | Base frequency of carrier in Hz |
| `carrier_type` | string | yes | Carrier waveform: `sawtooth`, `pulse`, or `noise` |
| `num_bands` | integer | yes | Number of filter bands (8-32 typical) |
| `band_spacing` | string | yes | Band spacing: `linear` or `logarithmic` |
| `envelope_attack` | number | yes | Envelope attack time in seconds (how fast bands respond) |
| `envelope_release` | number | yes | Envelope release time in seconds (how fast bands decay) |
| `formant_rate` | number | no | Formant animation rate in Hz (default 2.0) |
| `bands` | array | no | Optional custom band configurations (overrides num_bands) |

**Carrier Types:**
- `sawtooth` — Rich in harmonics, classic vocoder sound
- `pulse` — Hollow, more synthetic sound
- `noise` — Whispery, unvoiced consonant-like sound

**Band Spacing:**
- `linear` — Equal Hz between band centers
- `logarithmic` — Equal ratio between bands (more perceptually uniform)

**Custom Bands:**
Each band object has:
- `center_freq` — Center frequency in Hz
- `bandwidth` — Q factor for the band filter
- `envelope_pattern` — Optional amplitude envelope values (0.0-1.0)

**Common Presets:**
- Robot Voice: `carrier_type: "sawtooth"`, 16 bands, logarithmic spacing, fast envelopes (5ms/20ms), `formant_rate: 3.0`
- Choir: `carrier_type: "noise"`, 24 bands, logarithmic spacing, slow envelopes (50ms/100ms), `formant_rate: 0.5`
- Strings Through Vocoder: `carrier_type: "pulse"`, 20 bands, logarithmic spacing, medium envelopes (30ms/80ms), `formant_rate: 1.0`

#### `formant`

Formant synthesis creates vowel and voice sounds using resonant filter banks tuned to formant frequencies. Human vowels are characterized by formant frequencies (F1, F2, F3, etc.) - resonant peaks in the spectrum. Ideal for vocal synthesis, creature sounds, and choir textures.

```json
{
  "type": "formant",
  "frequency": 110.0,
  "vowel": "a",
  "vowel_morph": "i",
  "morph_amount": 0.5,
  "breathiness": 0.15,
  "formants": []
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `frequency` | number | yes | Base pitch frequency of the voice in Hz |
| `formants` | array | no | Custom formant configurations (overrides vowel preset if provided) |
| `vowel` | string | no | Vowel preset: `a`, `i`, `u`, `e`, or `o` (default: `a` if formants not provided) |
| `vowel_morph` | string | no | Second vowel for morphing transitions |
| `morph_amount` | number | no | Morph blend between vowels (0.0 = first vowel, 1.0 = second vowel, default 0.0) |
| `breathiness` | number | no | Noise amount for breathiness (0.0-1.0, default 0.0) |

**Vowel Presets:**
- `a` — /a/ (ah) as in "father" (F1: 800 Hz, F2: 1200 Hz, F3: 2800 Hz)
- `i` — /i/ (ee) as in "feet" (F1: 280 Hz, F2: 2250 Hz, F3: 2890 Hz)
- `u` — /u/ (oo) as in "boot" (F1: 310 Hz, F2: 870 Hz, F3: 2250 Hz)
- `e` — /e/ (eh) as in "bed" (F1: 530 Hz, F2: 1840 Hz, F3: 2480 Hz)
- `o` — /o/ (oh) as in "boat" (F1: 500 Hz, F2: 1000 Hz, F3: 2800 Hz)

**Custom Formants:**
Each formant object has:
- `frequency` — Center frequency in Hz (20-20000)
- `amplitude` — Amplitude/gain (0.0-1.0)
- `bandwidth` — Bandwidth/Q factor (0.5-20.0)

```json
{
  "type": "formant",
  "frequency": 110.0,
  "formants": [
    { "frequency": 400.0, "amplitude": 1.0, "bandwidth": 5.0 },
    { "frequency": 1000.0, "amplitude": 0.7, "bandwidth": 6.0 },
    { "frequency": 2500.0, "amplitude": 0.5, "bandwidth": 7.0 }
  ]
}
```

**Common Presets:**
- Vowel A: `vowel: "a"`, `breathiness: 0.0` (pure vowel sound)
- Vowel I: `vowel: "i"`, `breathiness: 0.0` (bright, forward vowel)
- Choir Ah: `vowel: "a"`, `breathiness: 0.15` (warm, resonant choir vowel)
- Creature Growl: Custom low formants at 300/600/1200/2000 Hz, `breathiness: 0.4` (guttural growl)
- Vowel Morph: `vowel: "a"`, `vowel_morph: "i"`, `morph_amount: 0.5` (smooth vowel transition)

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

#### `vector`

Vector synthesis creates evolving textures by crossfading between 4 sources in 2D space. The sources are positioned at the corners of a square: top-left (0,0), top-right (1,0), bottom-left (0,1), and bottom-right (1,1). A position (position_x, position_y) moves within this space, smoothly blending between sources. Ideal for morphing pads, animated textures, and evolving soundscapes.

```json
{
  "type": "vector",
  "frequency": 220.0,
  "sources": [
    { "type": "sine" },
    { "type": "saw" },
    { "type": "square", "duty": 0.5 },
    { "type": "triangle" }
  ],
  "position_x": 0.5,
  "position_y": 0.5,
  "path": [
    { "x": 0.0, "y": 0.0, "duration": 1.0 },
    { "x": 1.0, "y": 0.0, "duration": 1.0 },
    { "x": 1.0, "y": 1.0, "duration": 1.0 },
    { "x": 0.0, "y": 1.0, "duration": 1.0 }
  ],
  "path_loop": true,
  "path_curve": "linear"
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `frequency` | number | yes | Base frequency in Hz |
| `sources` | array | yes | Array of exactly 4 source objects (see below) |
| `position_x` | number | no | X position in vector space (0.0-1.0, default 0.5). Only used if no path specified |
| `position_y` | number | no | Y position in vector space (0.0-1.0, default 0.5). Only used if no path specified |
| `path` | array | no | Array of path points for animating position over time |
| `path_loop` | boolean | no | Loop the path (default false) |
| `path_curve` | string | no | Interpolation curve: `linear`, `ease_in_out`, `exponential` (default linear) |

**Vector Sources:**
Each source is one of:
- `{ "type": "sine" }` — Pure sine wave
- `{ "type": "saw" }` — Sawtooth wave
- `{ "type": "square", "duty": 0.5 }` — Square/pulse wave with duty cycle (0.0-1.0)
- `{ "type": "triangle" }` — Triangle wave
- `{ "type": "noise", "noise_type": "white" }` — Noise source (white, pink, or brown)
- `{ "type": "wavetable", "table": "basic", "position": 0.5 }` — Wavetable source (same tables as wavetable synthesis)

**Path Points:**
Each path point has:
- `x` — X position (0.0-1.0)
- `y` — Y position (0.0-1.0)
- `duration` — Time to reach this point from previous point in seconds

**Common Presets:**
- Evolving Pad: Static sources (sine, saw, square, triangle), path sweeping all corners over 8 seconds with ease_in_out curve
- Morph Texture: Mix of tonal and noise sources (sine, noise, triangle, wavetable), medium-speed path with exponential curve
- Sweep Corners: 4 distinct wavetables at different positions, fast circular path with linear interpolation

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
