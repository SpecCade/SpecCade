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
| `post_fx_lfos` | array | no | `[]` | Post-FX LFO modulations (see Post-FX LFO Modulation section) |

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
| `depth` | number | yes | Modulation depth (0.0-1.0). Must be `> 0.0` when `lfo` is present. |
| `phase` | number | no | Initial phase offset (0.0-1.0) |

**Modulation Targets:**
- `{ "target": "pitch", "semitones": 2.0 }` — Vibrato (pitch deviation in semitones; effective range is `semitones * depth`)
- `{ "target": "volume", "amount": 1.0 }` — Tremolo (maximum amplitude reduction; effective strength is `amount * depth`)
- `{ "target": "filter_cutoff", "amount": 1000.0 }` — Filter sweep (Hz delta; effective strength is `amount * depth`)
- `{ "target": "pan", "amount": 1.0 }` — Auto-panning (max pan delta around `layer.pan`; effective strength is `amount * depth`)
- `{ "target": "pulse_width", "amount": 0.2 }` — Pulse width modulation (max duty delta around base duty; effective strength is `amount * depth`). Only valid for `oscillator` or `multi_oscillator` synthesis with `waveform: square` or `waveform: pulse`. Amount must be in range [0.0, 0.49]. Result duty is clamped to (0.01, 0.99).
- `{ "target": "fm_index", "amount": 4.0 }` — FM modulation index modulation (max index delta around base index; effective strength is `amount * depth`). Only valid for `fm_synth` synthesis. Amount must be > 0. Result index is clamped to >= 0.0.
- `{ "target": "grain_size", "amount_ms": 30.0 }` — Grain size modulation (max size delta in milliseconds; effective strength is `amount_ms * depth`). Only valid for `granular` synthesis. Amount must be > 0. Result size is clamped to [10.0, 500.0] ms. Modulation is applied per-grain at grain start time.
- `{ "target": "grain_density", "amount": 20.0 }` — Grain density modulation (max density delta in grains/sec; effective strength is `amount * depth`). Only valid for `granular` synthesis. Amount must be > 0. Result density is clamped to [1.0, 100.0] grains/sec. Modulation is applied per-grain at grain start time.

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

#### `feedback_fm`

Feedback FM synthesis with a self-modulating operator. A single oscillator modulates itself by feeding its output back into its own phase. Creates characteristic "screaming" or "gritty" timbres at high feedback values, similar to DX7 operator 1 self-feedback. Distinct from standard 2-operator FM because the output feeds back into itself rather than using a separate modulator.

```json
{
  "type": "feedback_fm",
  "frequency": 220.0,
  "feedback": 0.7,
  "modulation_index": 3.0,
  "freq_sweep": { "end_freq": 110.0, "curve": "exponential" }
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `frequency` | number | yes | Base frequency in Hz |
| `feedback` | number | yes | Self-modulation amount (0.0-1.0). Internally clamped to max 0.99 for stability |
| `modulation_index` | number | yes | Modulation depth controlling harmonic richness |
| `freq_sweep` | object | no | Optional frequency sweep |

**Algorithm:**
- Single oscillator with phase feedback: `output = sin(phase + feedback * prev_output * modulation_index)`
- Previous output sample is fed back into current sample's phase
- Feedback is clamped to 0.99 maximum to prevent runaway oscillation
- Higher feedback values produce increasingly harsh, screaming timbres

**Common Presets:**
- Mild Growl: feedback 0.3, modulation_index 2.0 (subtle harmonic enrichment)
- Gritty Lead: feedback 0.6, modulation_index 3.0 (aggressive but controlled)
- Screaming: feedback 0.9, modulation_index 4.0 (harsh, DX7-style feedback)
- Bass: feedback 0.5, modulation_index 2.5, frequency 80-150 Hz (rich bass with character)

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

#### `bowed_string`

Bowed string synthesis for violin/cello-like sounds. Uses a bidirectional delay-line waveguide with continuous bow excitation via a stick-slip friction model. Unlike plucked strings (Karplus-Strong), bowed strings have continuous excitation during the entire duration. Ideal for sustained string sounds.

```json
{
  "type": "bowed_string",
  "frequency": 440.0,
  "bow_pressure": 0.5,
  "bow_position": 0.12,
  "damping": 0.3
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `frequency` | number | yes | Base frequency in Hz |
| `bow_pressure` | number | yes | Bow pressure / force on string (0.0-1.0) |
| `bow_position` | number | yes | Bow position along string (0.0 = bridge, 1.0 = nut) |
| `damping` | number | yes | String damping / high-frequency absorption (0.0-1.0) |

**Algorithm:**
- Bidirectional delay line represents waves traveling in both directions on the string
- Bow position splits the delay line (0.0 = close to bridge, 1.0 = close to nut)
- Stick-slip friction model generates continuous excitation at the bow point
- Higher bow pressure increases friction force and harmonic content
- Damping applies lowpass filtering to simulate string and air losses

**Common Presets:**
- Violin: bow_pressure 0.5, bow_position 0.12, damping 0.3 (bright, expressive)
- Viola: bow_pressure 0.55, bow_position 0.15, damping 0.35 (warmer, fuller)
- Cello: bow_pressure 0.6, bow_position 0.18, damping 0.4 (rich, warm)
- Double Bass: bow_pressure 0.7, bow_position 0.2, damping 0.5 (deep, resonant)

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

#### `supersaw_unison`

Supersaw/Unison synthesis creates thick, detuned sawtooth stacks by layering multiple detuned oscillators with stereo spread. Classic sound for trance leads, EDM supersaws, and synthwave pads.

```json
{
  "type": "supersaw_unison",
  "frequency": 440.0,
  "voices": 7,
  "detune_cents": 25.0,
  "spread": 0.8,
  "detune_curve": "linear"
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `frequency` | number | yes | Base frequency in Hz |
| `voices` | integer | yes | Number of unison voices (1-16) |
| `detune_cents` | number | yes | Maximum detune amount in cents (100 cents = 1 semitone) |
| `spread` | number | yes | Stereo spread (0.0 = mono, 1.0 = full stereo spread) |
| `detune_curve` | string | no | Detune distribution: `linear` (default) or `exp2` |

**Detune Curves:**
- `linear` — Voices are evenly spaced in cents across the detune range
- `exp2` — Outer voices are detuned more aggressively (squared curve), creating a wider perceived spread

**Voice Distribution:**
For N voices, each voice has a normalized position x from -1 to +1:
- Voice 0: x = -1.0 (leftmost, max negative detune)
- Voice N/2: x = 0.0 (center, no detune)
- Voice N-1: x = +1.0 (rightmost, max positive detune)

**Implementation Notes:**
- Each voice is rendered as a virtual layer with its own pan and frequency
- Total expanded layers (sum of voices across all supersaw_unison layers) counts toward the 32-layer limit
- Volume is automatically scaled by 1/N to maintain consistent loudness

**Common Presets:**
- Classic Supersaw: 7 voices, 25 cents detune, 0.8 spread, linear curve
- Thick Lead: 5 voices, 15 cents detune, 0.6 spread, linear curve
- Wide Pad: 9 voices, 30 cents detune, 1.0 spread, exp2 curve
- Subtle Unison: 3 voices, 10 cents detune, 0.4 spread, linear curve

#### `waveguide`

Waveguide synthesis for wind/brass physical modeling. Uses a delay-line waveguide with filtered noise excitation to simulate the acoustics of wind instruments. The delay line represents the resonant air column, noise excitation simulates breath turbulence, damping models high-frequency absorption, and resonance controls feedback.

```json
{
  "type": "waveguide",
  "frequency": 440.0,
  "breath": 0.7,
  "noise": 0.3,
  "damping": 0.3,
  "resonance": 0.8
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `frequency` | number | yes | Base frequency in Hz |
| `breath` | number | yes | Breath/excitation strength (0.0-1.0) |
| `noise` | number | yes | Noise mix in excitation (0.0 = pure tone, 1.0 = pure noise) |
| `damping` | number | yes | Delay line damping / high-frequency absorption (0.0-1.0) |
| `resonance` | number | yes | Feedback/resonance amount (0.0-1.0) |

**Algorithm:**
- Delay line length = sample_rate / frequency
- Excitation = breath * (noise * random_noise + (1-noise) * sine_tone)
- One-pole lowpass filter for damping (higher damping = more HF absorption)
- Feedback with tanh saturation for stability
- Output is the filtered delay line signal

**Common Presets:**
- Flute: breath 0.6, noise 0.3, damping 0.2, resonance 0.7 (breathy, bright)
- Clarinet: breath 0.8, noise 0.15, damping 0.4, resonance 0.85 (warm, woody)
- Brass: breath 0.9, noise 0.1, damping 0.3, resonance 0.9 (bold, bright)
- Breathy/Airy: breath 0.5, noise 0.7, damping 0.5, resonance 0.5 (soft, ethereal)

#### `membrane_drum`

Membrane drum synthesis for toms, hand drums, congas, bongos, and other pitched percussion. Uses modal synthesis based on circular membrane mode frequencies derived from Bessel function zeros. Creates the characteristic pitched/tonal sound of drums distinct from simple noise-based synthesis.

```json
{
  "type": "membrane_drum",
  "frequency": 100.0,
  "decay": 0.5,
  "tone": 0.4,
  "strike": 0.7
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `frequency` | number | yes | Fundamental frequency in Hz |
| `decay` | number | yes | Decay rate (0.0-1.0). Higher values decay faster |
| `tone` | number | yes | Tone/brightness (0.0-1.0). Low = fundamental emphasis, high = more overtones |
| `strike` | number | yes | Strike strength (0.0-1.0). Affects attack transient intensity |

**Algorithm:**
- Uses 9 modes based on circular membrane Bessel function zeros
- Mode frequency ratios: 1.0, 1.593, 2.135, 2.295, 2.653, 2.917, 3.155, 3.5, 3.598
- Each mode is a decaying sine wave with amplitude weighted by tone parameter
- Higher modes decay faster (realistic membrane behavior)
- Strike parameter controls impulse/noise mix in excitation transient
- Output is normalized to [-1.0, 1.0]

**Common Presets:**
- Floor Tom: frequency 80 Hz, decay 0.4, tone 0.3, strike 0.7 (deep, full)
- Rack Tom: frequency 120 Hz, decay 0.5, tone 0.4, strike 0.8 (punchy, bright)
- Conga: frequency 200 Hz, decay 0.6, tone 0.5, strike 0.6 (warm, resonant)
- Bongo: frequency 300 Hz, decay 0.7, tone 0.6, strike 0.5 (crisp, short)
- Djembe: frequency 150 Hz, decay 0.5, tone 0.7, strike 0.8 (bright, articulate)
- Timpani: frequency 60 Hz, decay 0.3, tone 0.2, strike 0.6 (sustained, mellow)

#### `comb_filter_synth`

Comb filter synthesis for resonant metallic tones. Uses a delay-line comb filter with feedback to create pitched resonant sounds. The delay line length determines the pitch (sample_rate / frequency). An excitation signal is fed through the comb filter to produce metallic, resonant, and bell-like timbres.

Distinct from Karplus-Strong (which uses lowpass filtering in the feedback loop for plucked string sounds) and metallic synthesis (which uses inharmonic additive partials).

```json
{
  "type": "comb_filter_synth",
  "frequency": 440.0,
  "decay": 0.9,
  "excitation": "impulse"
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `frequency` | number | yes | Base frequency in Hz (determines delay line length) |
| `decay` | number | yes | Feedback decay amount (0.0-1.0). Higher values = longer resonance |
| `excitation` | string | no | Excitation type: `impulse` (default), `noise`, or `saw` |

**Algorithm:**
- Delay line length = sample_rate / frequency
- Excitation signal is generated based on type (one period length)
- Comb filter: output = input + decay * delayed_sample
- Decay is clamped to 0.999 for stability

**Excitation Types:**
- `impulse` — Single sample spike at start (sharp, bell-like attack)
- `noise` — Short burst of seeded random noise (richer, more complex)
- `saw` — Short sawtooth burst (harmonic content, brighter)

**Common Presets:**
- Bell: frequency 440 Hz, decay 0.95, excitation impulse (pure, ringing)
- Resonant: frequency 220 Hz, decay 0.9, excitation noise (complex, metallic)
- Harsh: frequency 330 Hz, decay 0.85, excitation saw (bright, aggressive)
- High Ping: frequency 880 Hz, decay 0.92, excitation impulse (short, bright ping)
- Low Drone: frequency 110 Hz, decay 0.98, excitation noise (sustained, deep)

#### `pulsar`

Pulsar synthesis generates sound using synchronized grain trains ("pulsarets"). Each pulsaret is a windowed waveform burst at a specified frequency, emitted at a fixed pulse rate. The result is a distinctive rhythmic tonal texture where both the fundamental frequency AND the pulse rate are heard as separate perceptual elements. Classic technique for granular/rhythmic synthesis.

```json
{
  "type": "pulsar",
  "frequency": 440.0,
  "pulse_rate": 20.0,
  "grain_size_ms": 30.0,
  "shape": "sine"
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `frequency` | number | yes | Fundamental frequency of each grain in Hz |
| `pulse_rate` | number | yes | Grains per second (pulsaret rate) |
| `grain_size_ms` | number | yes | Duration of each grain in milliseconds |
| `shape` | string | yes | Waveform shape: `sine`, `square`, `sawtooth`, `triangle`, or `pulse` |

**Algorithm:**
- Pulsarets (grain bursts) are emitted at regular intervals based on pulse_rate
- Each pulsaret is a waveform oscillating at the specified frequency
- A Hann window is applied to each pulsaret for smooth onset/offset
- All pulsarets are identical (deterministic, no randomization)

**Perceptual Effects:**
- Low pulse rates (1-10 Hz): Individual grains heard as rhythmic pulses
- Mid pulse rates (10-30 Hz): Transitional zone where rhythm becomes pitch
- High pulse rates (30+ Hz): Pulse rate becomes audible as a secondary pitch

**Common Presets:**
- Rhythmic Texture: frequency 220 Hz, pulse_rate 8.0, grain_size_ms 50.0, shape sine (clearly separated pulses)
- Tonal Grain: frequency 440 Hz, pulse_rate 30.0, grain_size_ms 20.0, shape sawtooth (grainy tonal sound)
- Buzzy Lead: frequency 330 Hz, pulse_rate 60.0, grain_size_ms 15.0, shape square (buzzy, harmonic-rich)
- Soft Pulses: frequency 880 Hz, pulse_rate 12.0, grain_size_ms 40.0, shape triangle (gentle, bell-like pulses)

#### `vosim`

VOSIM (Voice Simulation) synthesis generates formant-rich sounds using squared-sine pulse trains. Each fundamental period contains N pulses at the formant frequency with exponential decay, creating vowel-like and robotic timbres. This is efficient for speech synthesis because the formant is generated directly through the pulse rate rather than filtering.

```json
{
  "type": "vosim",
  "frequency": 110.0,
  "formant_freq": 800.0,
  "pulses": 5,
  "breathiness": 0.1
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `frequency` | number | yes | Fundamental frequency (pitch) in Hz |
| `formant_freq` | number | yes | Formant frequency (spectral peak) in Hz |
| `pulses` | integer | yes | Number of pulses per period (1-16) |
| `breathiness` | number | no | Noise amount for breathiness (0.0-1.0, default 0.0) |

**Algorithm:**
- Each fundamental period contains N squared-sine pulses at the formant frequency
- Formula: `output = sin^2(2 * PI * formant_freq * t) * decay(t)`
- Pulses decay exponentially within each period
- The rest of each period is silent (between pulse trains)
- Breathiness adds filtered noise around the formant frequency

**Formant Frequencies for Vowels:**
- Vowel "a" (/a/ as in "father"): formant_freq ~800 Hz
- Vowel "i" (/i/ as in "feet"): formant_freq ~300 Hz (F1) or ~2200 Hz (F2)
- Vowel "u" (/u/ as in "boot"): formant_freq ~300 Hz (F1) or ~800 Hz (F2)
- Vowel "e" (/e/ as in "bed"): formant_freq ~500 Hz
- Vowel "o" (/o/ as in "boat"): formant_freq ~500 Hz

**Common Presets:**
- Robot Voice: frequency 110 Hz, formant_freq 800 Hz, pulses 5, breathiness 0.0 (pure robotic vowel)
- Breathy Voice: frequency 110 Hz, formant_freq 800 Hz, pulses 4, breathiness 0.3 (softer, breathier)
- High Vowel: frequency 220 Hz, formant_freq 2000 Hz, pulses 8, breathiness 0.1 (bright, nasal)
- Bass Drone: frequency 55 Hz, formant_freq 400 Hz, pulses 3, breathiness 0.0 (deep, resonant)

#### `spectral_freeze`

Spectral freeze synthesis captures the spectral content of a short source signal and sustains it indefinitely, creating frozen, pad-like tones. Uses FFT to analyze a single frame of source material (noise or tone), stores the complex spectrum (magnitude and phase), and repeatedly synthesizes it via inverse FFT with overlap-add. Ideal for sustained drones, evolving pads, and frozen textures.

```json
{
  "type": "spectral_freeze",
  "source": { "type": "noise", "noise_type": "pink" }
}
```

```json
{
  "type": "spectral_freeze",
  "source": { "type": "tone", "waveform": "sawtooth", "frequency": 220.0 }
}
```

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `source` | object | yes | Source material for spectral capture (see below) |

**Spectral Sources:**
- `{ "type": "noise", "noise_type": "white" }` - White, pink, or brown noise
- `{ "type": "tone", "waveform": "sine", "frequency": 440.0 }` - Pitched waveform (sine, square, sawtooth, triangle, pulse)

**Algorithm:**
- Generates one deterministic source frame of 2048 samples from noise or tone
- Applies Hann window and computes FFT, storing the complex spectrum
- Renders full output by repeating:
  - Inverse FFT of stored spectrum
  - Hann window the result
  - Overlap-add at 512 sample intervals (75% overlap)
- Normalizes by overlap-add window sum for stable amplitude

**Characteristics:**
- FFT size: 2048 samples (fixed)
- Hop size: 512 samples (fixed, 75% overlap)
- Window: Hann
- Deterministic output from seed-derived source frame
- Creates sustained, frozen spectral content

**Common Presets:**
- Frozen Noise Pad: `source: { type: "noise", noise_type: "pink" }` (warm, sustained texture)
- Frozen Tone Drone: `source: { type: "tone", waveform: "sawtooth", frequency: 110.0 }` (rich harmonic drone)
- Ethereal Texture: `source: { type: "noise", noise_type: "brown" }` (dark, rumbling pad)
- Bright Freeze: `source: { type: "tone", waveform: "square", frequency: 440.0 }` (buzzy sustained tone)

### Filters

Filters are tagged unions with `type`:

- `lowpass`: `{ "type": "lowpass", "cutoff": 2000.0, "resonance": 0.7, "cutoff_end": 500.0 }`
- `highpass`: `{ "type": "highpass", "cutoff": 200.0, "resonance": 0.7, "cutoff_end": 2000.0 }`
- `bandpass`: `{ "type": "bandpass", "center": 800.0, "resonance": 0.7, "center_end": 1200.0 }`
- `notch`: `{ "type": "notch", "center": 800.0, "resonance": 0.7, "center_end": 1200.0 }`
- `allpass`: `{ "type": "allpass", "frequency": 1000.0, "resonance": 0.7, "frequency_end": 2000.0 }`
- `comb`: `{ "type": "comb", "delay_ms": 5.0, "feedback": 0.7, "wet": 0.5 }`
- `formant`: `{ "type": "formant", "vowel": "a", "intensity": 0.8 }`
- `ladder`: `{ "type": "ladder", "cutoff": 1000.0, "resonance": 0.7, "cutoff_end": 200.0 }`
- `shelf_low`: `{ "type": "shelf_low", "frequency": 200.0, "gain_db": 6.0 }`
- `shelf_high`: `{ "type": "shelf_high", "frequency": 4000.0, "gain_db": -3.0 }`

Sweep fields like `cutoff_end` / `center_end` / `frequency_end` are optional. The `comb`, `formant`, `shelf_low`, and `shelf_high` filters do not support sweep.

**Notch Filter:**
A notch (band-reject) filter removes a narrow band of frequencies around the center frequency while passing all others. Useful for removing unwanted resonances, hum removal, or creating comb-filter effects.

**Allpass Filter:**
An allpass filter passes all frequencies at equal magnitude but shifts the phase. The phase shift varies with frequency, centered around the specified frequency parameter. Useful for phaser effects, stereo widening, and building complex phase relationships. Unlike magnitude filters, allpass filters affect the timing relationships between frequencies rather than their loudness.

**Comb Filter:**
A comb filter uses a delay line with feedback to create resonant peaks at harmonics of the delay frequency (1/delay_time). The name comes from the comb-like appearance of the frequency response. Creates metallic, resonant coloration useful for:
- Flanging effects (very short delays, 1-5ms)
- Metallic/robotic tones (medium delays, 5-20ms)
- Resonant coloration (longer delays)

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `delay_ms` | number | yes | Delay time in milliseconds (determines resonant frequency) |
| `feedback` | number | yes | Feedback amount (0.0-0.99, clamped for stability) |
| `wet` | number | yes | Wet/dry mix (0.0-1.0) |

**Note:** The resonant frequency is approximately 1000/delay_ms Hz. For example, a 5ms delay creates resonances at 200 Hz and its harmonics.

**Formant Filter:**
A formant filter shapes the spectrum of the input signal to match the resonant characteristics of human vowel sounds. It uses a bank of parallel resonant bandpass filters tuned to formant frequencies (F1, F2, F3). This creates vowel-like coloration of any input signal.

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `vowel` | string | yes | Target vowel: `a`, `e`, `i`, `o`, or `u` |
| `intensity` | number | yes | Effect intensity (0.0 = dry, 1.0 = full vowel shape) |

**Vowel Formant Frequencies:**
- `a` (/a/ as in "father"): F1 ~800 Hz, F2 ~1200 Hz, F3 ~2600 Hz
- `e` (/e/ as in "bed"): F1 ~400 Hz, F2 ~2200 Hz, F3 ~2600 Hz
- `i` (/i/ as in "feet"): F1 ~300 Hz, F2 ~2300 Hz, F3 ~3000 Hz
- `o` (/o/ as in "boat"): F1 ~450 Hz, F2 ~800 Hz, F3 ~2600 Hz
- `u` (/u/ as in "boot"): F1 ~350 Hz, F2 ~700 Hz, F3 ~2600 Hz

Common uses include adding vocal quality to noise, creating talking synth effects, and shaping sounds to have vowel-like characteristics.

**Ladder Filter:**
A Moog-style 4-pole (24 dB/octave) lowpass filter with resonance feedback. This classic analog-style filter produces a warm, musical sound with a steep rolloff. At high resonance settings, the filter emphasizes frequencies near the cutoff, creating a characteristic "squelchy" sound. Uses tanh saturation internally for stability at high resonance.

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `cutoff` | number | yes | Cutoff frequency in Hz |
| `resonance` | number | yes | Resonance amount (0.0-1.0, maps internally to 0-4x feedback) |
| `cutoff_end` | number | no | Optional target cutoff frequency for sweep |

**Characteristics:**
- 24 dB/octave slope (steeper than biquad lowpass)
- Self-oscillation possible at maximum resonance
- Warm, musical character ideal for bass, leads, and pads
- Supports cutoff sweeps for classic filter envelopes

Common uses include bass synthesis, acid basslines, lead synths, and creating classic analog-style filter sweeps.

**Low Shelf Filter:**
A low shelf filter boosts or cuts all frequencies below a specified frequency. Unlike a lowpass filter which removes high frequencies, a shelf filter applies a constant gain change to the affected frequency range. Ideal for adding warmth (boost) or reducing muddiness (cut) in the bass region.

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `frequency` | number | yes | Shelf frequency in Hz (frequencies below this are affected) |
| `gain_db` | number | yes | Gain in dB (-24 to +24). Positive boosts bass, negative cuts bass |

**Characteristics:**
- Gentle slope at the transition frequency
- Frequencies below the shelf frequency are uniformly boosted/cut
- Frequencies above the shelf frequency pass unchanged
- No resonance parameter (smooth transition)

Common uses include bass enhancement, rumble removal, adding weight to drums, and tonal balance adjustment.

**High Shelf Filter:**
A high shelf filter boosts or cuts all frequencies above a specified frequency. Unlike a highpass filter which removes low frequencies, a shelf filter applies a constant gain change to the affected frequency range. Ideal for adding brightness (boost) or reducing harshness (cut) in the treble region.

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `frequency` | number | yes | Shelf frequency in Hz (frequencies above this are affected) |
| `gain_db` | number | yes | Gain in dB (-24 to +24). Positive boosts treble, negative cuts treble |

**Characteristics:**
- Gentle slope at the transition frequency
- Frequencies above the shelf frequency are uniformly boosted/cut
- Frequencies below the shelf frequency pass unchanged
- No resonance parameter (smooth transition)

Common uses include adding air/sparkle, taming harsh high frequencies, presence adjustment, and de-essing.

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

#### `multi_tap_delay`

Multi-tap delay with independent delay lines per tap. Each tap has its own delay time, feedback, stereo position, level, and optional lowpass filter. Useful for complex echo patterns and rhythmic delays.

```json
{
  "type": "multi_tap_delay",
  "taps": [
    { "time_ms": 100, "feedback": 0.3, "pan": -0.5, "level": 0.8, "filter_cutoff": 2000 },
    { "time_ms": 200, "feedback": 0.2, "pan": 0.5, "level": 0.6 },
    { "time_ms": 400, "feedback": 0.1, "pan": 0.0, "level": 0.4, "filter_cutoff": 1000 }
  ]
}
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `taps` | array | yes | — | Array of delay taps (see below) |

**Delay Tap Parameters:**

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `time_ms` | number | yes | — | Delay time in ms (1-2000) |
| `feedback` | number | yes | — | Feedback amount (0.0-0.99) |
| `pan` | number | yes | — | Stereo pan position (-1.0 to 1.0) |
| `level` | number | yes | — | Output level (0.0-1.0) |
| `filter_cutoff` | number | no | `0` | Lowpass filter cutoff in Hz (0 = no filter) |

**Characteristics:**
- Each tap has independent delay line with feedback
- Taps are processed in order (stable iteration)
- Input is mixed to mono before delay processing
- Constant-power panning for natural stereo placement
- Simple one-pole lowpass filter when filter_cutoff > 0
- Supports `delay_time` post-FX LFO modulation (affects all taps)

**Common Uses:**
- Rhythmic echo patterns (quarter note, dotted eighth, etc.)
- Stereo ping-pong with filtering (alternating pan)
- Dub-style delays with darkening feedback
- Complex multi-voice echoes for ambience

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

#### `tape_saturation`

Tape saturation effect with warmth, wow/flutter, and deterministic hiss. Provides analog tape-style warmth that differs from basic waveshaping through asymmetric soft clipping and pitch modulation.

```json
{ "type": "tape_saturation", "drive": 3.0, "bias": 0.05, "wow_rate": 0.8, "flutter_rate": 6.0, "hiss_level": 0.02 }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `drive` | number | yes | — | Drive/saturation amount (1.0-20.0) |
| `bias` | number | yes | — | DC bias before saturation (-0.5 to 0.5). Affects harmonic content |
| `wow_rate` | number | yes | — | Wow LFO rate in Hz (0.0-3.0). Low-frequency pitch modulation |
| `flutter_rate` | number | yes | — | Flutter LFO rate in Hz (0.0-20.0). Higher-frequency pitch modulation |
| `hiss_level` | number | yes | — | Tape hiss amount (0.0-0.1). Deterministic noise added to output |

**Characteristics:**
- Asymmetric soft clipping creates even harmonics typical of analog tape
- Wow provides slow pitch drift (0.5-2 Hz typical for tape machines)
- Flutter provides faster pitch variations (5-20 Hz)
- Hiss is deterministic (seed-driven) for bit-identical output
- Supports `distortion_drive` post-FX LFO modulation

**Common Uses:**
- Warming up digital sources with analog-style saturation
- Lo-fi/vintage effects on drums and synths
- Subtle tape emulation on master bus (low drive, minimal wow/flutter)
- Obvious tape effects (higher wow/flutter rates, audible hiss)

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

#### `flanger`

Flanger effect with modulated delay and feedback.

Flanger is similar to chorus but uses shorter base delays (1-20ms vs 20-40ms for chorus) and a feedback path to create comb filter resonance and the characteristic "jet" or "swoosh" sound.

```json
{ "type": "flanger", "rate": 0.5, "depth": 0.7, "feedback": 0.7, "delay_ms": 5.0, "wet": 0.5 }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `rate` | number | yes | — | LFO rate in Hz (0.1-10.0) |
| `depth` | number | yes | — | Modulation depth (0.0-1.0) |
| `feedback` | number | yes | — | Feedback amount (-0.99 to 0.99) |
| `delay_ms` | number | yes | — | Base delay time in ms (0.1-50.0, typically 1-20) |
| `wet` | number | yes | — | Wet/dry mix (0.0-1.0) |

**Characteristics:**
- Shorter base delays than chorus create comb filtering
- Feedback path adds resonance and metallic coloration
- Negative feedback creates a different harmonic character
- Stereo width from L/R LFO phase offset (quarter cycle)
- Supports `delay_time` post-FX LFO modulation

**Common Uses:**
- Classic jet/swoosh sound effects
- Metallic textures on guitars and synths
- Sci-fi and robotic effects (with high feedback)
- Subtle motion and shimmer (with low depth/feedback)

#### `parametric_eq`

Multi-band parametric equalizer using cascaded biquad filters.

```json
{
  "type": "parametric_eq",
  "bands": [
    { "frequency": 100.0, "gain_db": 3.0, "q": 1.0, "band_type": "lowshelf" },
    { "frequency": 1000.0, "gain_db": -2.0, "q": 2.0, "band_type": "peak" },
    { "frequency": 8000.0, "gain_db": 2.0, "q": 1.0, "band_type": "highshelf" }
  ]
}
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `bands` | array | yes | - | Array of EQ bands to apply in order |

**EQ Band Parameters:**

| Param | Type | Required | Notes |
|------:|------|:--------:|-------|
| `frequency` | number | yes | Center/corner frequency in Hz |
| `gain_db` | number | yes | Gain in dB (-24 to +24) |
| `q` | number | yes | Q factor (0.1 to 10). Higher Q = narrower band |
| `band_type` | string | yes | Band type: `lowshelf`, `highshelf`, `peak`, `notch` |

**Band Types:**
- `lowshelf` - Boosts/cuts frequencies below the frequency. Q is ignored.
- `highshelf` - Boosts/cuts frequencies above the frequency. Q is ignored.
- `peak` - Bell curve boost/cut around the frequency. Q controls bandwidth.
- `notch` - Removes frequencies at the center frequency. gain_db is ignored.

**Characteristics:**
- Bands are applied in listed order (cascaded processing)
- Same coefficients applied to both stereo channels
- Deterministic output for the same input
- Parameters are clamped to safe ranges internally

**Common Uses:**
- Tonal shaping and frequency balance
- Removing unwanted resonances (notch)
- Adding warmth (low shelf boost) or air (high shelf boost)
- Surgical frequency correction (narrow peak/cut)

#### `limiter`

Brick-wall limiter effect with lookahead.

A limiter prevents output from exceeding a ceiling level using an infinite compression ratio above the threshold. Unlike a compressor, a limiter uses instant attack (via lookahead) to catch transients before they exceed the ceiling.

```json
{ "type": "limiter", "threshold_db": -6, "release_ms": 100, "lookahead_ms": 5, "ceiling_db": -0.3 }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `threshold_db` | number | yes | - | Threshold in dB where limiting begins (-24 to 0) |
| `release_ms` | number | yes | - | Release time in ms for gain recovery (10-500) |
| `lookahead_ms` | number | yes | - | Lookahead time in ms for peak detection (1-10) |
| `ceiling_db` | number | yes | - | Maximum output level in dB (-6 to 0) |

**Characteristics:**
- Brick-wall limiting: output never exceeds ceiling (within numerical tolerance)
- Lookahead allows gain reduction before peaks occur
- Instant attack with smooth release for transparent limiting
- Deterministic output for the same input

**Common Uses:**
- Mastering: ensure output doesn't clip (ceiling_db: -0.1 to -0.3)
- Loudness maximization: threshold close to ceiling for dense sound
- Transient protection: prevent sudden peaks from distorting
- Broadcast compliance: ensure output stays within limits

#### `gate_expander`

Gate/expander effect for tightening drums and noise reduction.

A gate attenuates signals that fall below a threshold. An expander is a softer version that reduces gain proportionally based on the ratio. The hold parameter keeps the gate open briefly after the signal drops below threshold, preventing choppy artifacts on decaying sounds.

```json
{ "type": "gate_expander", "threshold_db": -30, "ratio": 4.0, "attack_ms": 1.0, "hold_ms": 50, "release_ms": 100, "range_db": -60 }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `threshold_db` | number | yes | - | Threshold in dB where gate opens (-60 to 0) |
| `ratio` | number | yes | - | Expansion ratio (1.0=off, >1.0=expander, infinity=hard gate) |
| `attack_ms` | number | yes | - | Attack time in ms to open gate (0.1-50) |
| `hold_ms` | number | yes | - | Hold time in ms to stay open after signal drops (0-500) |
| `release_ms` | number | yes | - | Release time in ms to close gate (10-2000) |
| `range_db` | number | yes | - | Maximum attenuation depth in dB (-80 to 0) |

**Characteristics:**
- Peak envelope detection for fast response
- Hold time prevents choppy gating on decaying sounds
- Range limits maximum attenuation (useful for subtle expansion)
- Smooth attack/release for natural sound

**Common Uses:**
- Drum tightening: remove bleed between hits (threshold: -30 to -20 dB, ratio: 4-10)
- Noise reduction: attenuate quiet sections (threshold: -50 to -40 dB, ratio: 2-4)
- De-noising recordings: reduce background noise (high range_db for subtle effect)
- Transient shaping: emphasize attack by gating sustain (short hold_ms)

#### `stereo_widener`

Stereo widener effect for enhancing stereo image with three processing modes.

```json
{ "type": "stereo_widener", "width": 1.5, "mode": "mid_side", "delay_ms": 10 }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `width` | number | yes | - | Stereo width (0.0 = mono, 1.0 = normal, >1.0 = wider). Range: 0.0-2.0 |
| `mode` | string | no | `simple` | Processing algorithm: `simple`, `haas`, or `mid_side` |
| `delay_ms` | number | no | `10` | Delay time in ms for Haas mode only (1-30 typical) |

**Processing Modes:**

1. **simple**: L/R crossmix
   - Adjusts the stereo difference (side) signal level
   - At width=0: mono output (side removed)
   - At width=1: unchanged stereo
   - At width>1: enhanced stereo (side amplified)

2. **haas**: Delay-based psychoacoustic widening
   - Delays one channel slightly to create width perception
   - Based on the Haas (precedence) effect
   - `delay_ms` controls the delay time (1-30ms typical)
   - Supports `delay_time` post-FX LFO modulation

3. **mid_side**: Classic M/S processing
   - Converts to mid/side domain, scales side, converts back
   - `mid = (L + R) / 2`, `side = (L - R) / 2`
   - `side *= width`
   - `L = mid + side`, `R = mid - side`

**Characteristics:**
- Width clamped to [0.0, 2.0] for stability
- All modes are deterministic
- Mono input (L=R) is not affected by mid_side or simple widening (no side signal exists)
- Haas mode can introduce comb filtering artifacts at extreme settings

**Common Uses:**
- Widening narrow stereo sources (width: 1.2-1.5)
- Converting stereo to mono (width: 0.0)
- Creating immersive stereo effects (Haas mode with 10-20ms delay)
- Subtle stereo enhancement (mid_side mode with width: 1.1-1.3)

#### `transient_shaper`

Transient shaper effect for controlling attack punch and sustain using dual envelope detection.

A transient shaper uses two envelope followers with different time constants to distinguish between transients (attack) and sustained signal. The fast envelope tracks rapid changes (transients), while the slow envelope tracks the overall signal level (sustain). The difference between them detects transients.

```json
{ "type": "transient_shaper", "attack": 0.5, "sustain": -0.3, "output_gain_db": 2.0 }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `attack` | number | yes | - | Attack enhancement (-1.0 to 1.0). Negative = softer transients, positive = more punch |
| `sustain` | number | yes | - | Sustain enhancement (-1.0 to 1.0). Negative = tighter, positive = fuller |
| `output_gain_db` | number | yes | - | Output makeup gain in dB (-12 to +12) |

**Algorithm:**
- Fast envelope: 1ms attack, 50ms release (detects transients)
- Slow envelope: 20ms attack, 200ms release (detects sustained level)
- Transient = max(fast_env - slow_env, 0) (positive transients only)
- Attack gain = 1 + attack * transient
- Sustain gain = 1 + sustain * slow_env
- Output = input * attack_gain * sustain_gain * db_to_linear(output_gain_db)

**Characteristics:**
- Dual envelope detection for accurate transient/sustain separation
- Attack enhancement adds punch without affecting sustain
- Sustain reduction tightens sound without killing attack
- Deterministic output for the same input

**Common Uses:**
- Drum punch: positive attack (0.3-0.7) adds snap to kicks and snares
- Tighter drums: negative sustain (-0.3 to -0.5) reduces ring and bleed
- Softer attacks: negative attack (-0.3 to -0.5) for smoother transients
- Fuller sustain: positive sustain (0.3-0.5) for rounder, warmer sound

#### `auto_filter`

Auto-filter / envelope follower effect for dynamic filter sweeps driven by signal level. Creates auto-wah effects where louder signals result in brighter (higher cutoff) filtering.

The effect uses an envelope follower to track the input signal level. This envelope modulates a lowpass filter's cutoff frequency from the base frequency up toward 20kHz based on signal amplitude. Higher sensitivity means stronger reaction to signal level; higher depth means wider frequency sweep range.

```json
{ "type": "auto_filter", "sensitivity": 0.8, "attack_ms": 5.0, "release_ms": 100.0, "depth": 0.7, "base_frequency": 500.0 }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `sensitivity` | number | yes | - | How much signal level affects filter (0.0-1.0) |
| `attack_ms` | number | yes | - | Envelope attack time in ms (0.1-100) |
| `release_ms` | number | yes | - | Envelope release time in ms (10-1000) |
| `depth` | number | yes | - | Filter sweep range (0.0-1.0) |
| `base_frequency` | number | yes | - | Base cutoff frequency when signal is quiet (100-8000 Hz) |

**Algorithm:**
- Envelope follower tracks peak signal level with attack/release smoothing
- Modulation = envelope * sensitivity * depth
- Cutoff = base_frequency + modulation * (20000 - base_frequency)
- Cutoff is clamped to [20, 20000] Hz
- Lowpass filter with moderate resonance (1.5) applied per-sample with dynamic cutoff

**Characteristics:**
- Envelope-driven filter creates dynamic "wah" effects
- Fast attack (1-10ms) creates quacky envelope response
- Slow release (100-500ms) creates smooth filter decay
- Higher base frequency means brighter quiet sections
- Moderate resonance adds musical emphasis without instability
- Deterministic output for the same input

**Common Uses:**
- Auto-wah on funky guitar/bass (sensitivity: 0.8-1.0, attack: 5-10ms, release: 100-200ms, base: 300-500Hz)
- Envelope filter on synths (sensitivity: 0.6-0.8, depth: 0.5-0.8, base: 200-400Hz)
- Dynamic brightness control (sensitivity: 0.5, depth: 0.3, base: 1000Hz for subtle effect)
- Quacky duck sounds (sensitivity: 1.0, attack: 1ms, release: 50ms, depth: 1.0, base: 200Hz)

#### `cabinet_sim`

Cabinet simulation effect using cascaded biquad filters to approximate the frequency response of various speaker cabinets. This is a procedural approach (no convolution/IR) for deterministic output.

```json
{ "type": "cabinet_sim", "cabinet_type": "guitar_4x12", "mic_position": 0.5 }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `cabinet_type` | string | yes | - | Cabinet type: `guitar_1x12`, `guitar_4x12`, `bass_1x15`, `radio`, `telephone` |
| `mic_position` | number | no | `0.0` | Mic position (0.0 = close/bright, 1.0 = far/dark) |

**Cabinet Types:**

1. **guitar_1x12**: Classic 1x12 combo amp
   - Highpass: 80 Hz
   - Peak boost: 3 kHz, +3 dB
   - Lowpass: 6 kHz
   - Character: Bright, focused

2. **guitar_4x12**: Big 4x12 stack
   - Highpass: 60 Hz
   - Low shelf boost: 120 Hz, +2 dB
   - Peak cut: 400 Hz, -2 dB (less mud)
   - Peak boost: 2.5 kHz, +3 dB
   - Lowpass: 5 kHz
   - Character: Full, warm

3. **bass_1x15**: Bass 1x15 cabinet
   - Highpass: 40 Hz
   - Low shelf boost: 80 Hz, +4 dB
   - Peak cut: 600 Hz, -3 dB
   - Lowpass: 4 kHz
   - Character: Deep, punchy

4. **radio**: AM radio lo-fi
   - Highpass: 300 Hz
   - Bandpass centered: 1.5 kHz
   - Lowpass: 3 kHz
   - Character: Bandlimited, vintage

5. **telephone**: Telephone line quality
   - Highpass: 300 Hz
   - Bandpass centered: 1.5 kHz, narrow Q
   - Lowpass: 3.4 kHz
   - Character: Narrow bandwidth, lo-fi

**Mic Position:**
- 0.0 = close mic (brighter, more direct, base cutoff)
- 1.0 = far mic (darker, more ambient, 30% lower cutoff)
- Implemented as additional lowpass rolloff: `cutoff = base * (1.0 - 0.3 * mic_position)`

**Characteristics:**
- Procedural filter-stack approximation (no convolution)
- Deterministic output for the same input
- Each cabinet type defines a stable EQ/filter curve
- All filters use biquad implementation

**Common Uses:**
- Guitar amp simulation: guitar_1x12 or guitar_4x12 for electric guitar tones
- Bass amp simulation: bass_1x15 for bass guitar or synth bass
- Lo-fi effects: radio or telephone for vintage/degraded sound
- Vocal processing: telephone for voice-over-phone effect
- Sound design: various cabinets for tonal shaping

#### `rotary_speaker`

Rotary speaker (Leslie) effect that simulates a rotating speaker cabinet. Combines amplitude modulation (tremolo), stereo pan modulation (circular panning with 90 degree phase offset), and Doppler pitch wobble from short modulated delays. Classic organ and psychedelic effect.

```json
{ "type": "rotary_speaker", "rate": 5.0, "depth": 0.7, "wet": 0.6 }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `rate` | number | yes | - | Rotation rate in Hz (0.5-10.0 typical) |
| `depth` | number | yes | - | Effect intensity (0.0-1.0) |
| `wet` | number | yes | - | Wet/dry mix (0.0-1.0) |

**Algorithm:**
- Amplitude modulation: `amp = 1.0 + depth * 0.3 * sin(2 * PI * rate * t)`
- Stereo: Left and right channels use 90 degree phase offset for circular motion
- Doppler: Short delay (~3ms base) modulated +/- 2ms by LFO for pitch wobble

**Speed Presets:**
- Slow/chorale: rate ~1.0 Hz (gentle, warm swirl)
- Fast/tremolo: rate ~6.0 Hz (classic fast Leslie)
- Variable: ramp between slow and fast for expressive playing

**Characteristics:**
- Deterministic output for the same input
- Stereo width from L/R phase offset (mono input becomes stereo)
- Doppler delay creates subtle pitch modulation
- Amplitude modulation provides tremolo character

**Common Uses:**
- Hammond organ simulation (classic B3 with Leslie)
- Psychedelic guitar effects (60s/70s swirling sound)
- Electric piano enhancement (Rhodes/Wurlitzer through Leslie)
- Vocal effects (warped, rotating vocal sound)
- Synth pads (adding motion and width)

#### `ring_modulator`

Ring modulator effect that multiplies audio with a carrier sine oscillator to produce sum and difference frequencies (sidebands). Creates metallic, robotic, and sci-fi timbres. Unlike the `ring_mod_synth` synthesis type which generates sound from scratch, this effect processes existing audio.

```json
{ "type": "ring_modulator", "frequency": 150.0, "mix": 0.8 }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `frequency` | number | yes | - | Carrier oscillator frequency in Hz (20-2000 typical) |
| `mix` | number | yes | - | Wet/dry mix (0.0 = dry input, 1.0 = full ring mod) |

**Algorithm:**
```
carrier = sin(2 * PI * frequency * time)
modulated = input * carrier
output = mix * modulated + (1 - mix) * input
```

**Characteristics:**
- Deterministic output for the same input
- Creates sum (input + carrier) and difference (input - carrier) frequencies
- Low carrier frequencies (20-100 Hz) create tremolo-like effects
- Mid carrier frequencies (100-500 Hz) create metallic timbres
- Higher carrier frequencies create more complex, bell-like sidebands
- Same carrier applied to both stereo channels (maintains stereo image)

**Common Uses:**
- Metallic textures: carrier ~150 Hz on pitched material for shimmering metallic tones
- Robotic voices: carrier ~80-150 Hz on speech for sci-fi robot effects
- Bell-like tones: higher carrier frequencies on sustained sounds
- Tremolo alternative: very low carrier (~5-20 Hz) for amplitude modulation effects
- Sound design: combine with other effects for complex transformations

#### `granular_delay`

Granular delay effect for shimmer and pitchy delays. Uses pitch-shifted grains read from a delay buffer to create ethereal, shimmering delay textures. Each grain is windowed with a Hann envelope and pitch-shifted via resampling. Overlapping grains at 50% overlap create smooth textures.

```json
{ "type": "granular_delay", "time_ms": 200.0, "feedback": 0.6, "grain_size_ms": 50.0, "pitch_semitones": 12.0, "wet": 0.5 }
```

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `time_ms` | number | yes | - | Delay time in milliseconds (10-2000) |
| `feedback` | number | yes | - | Feedback amount (0.0-0.95) |
| `grain_size_ms` | number | yes | - | Grain window size in milliseconds (10-200) |
| `pitch_semitones` | number | yes | - | Pitch shift per grain pass in semitones (-24 to +24) |
| `wet` | number | yes | - | Wet/dry mix (0.0-1.0) |

**Algorithm:**
```
1. Read from delay buffer at time_ms
2. Apply pitch shift by resampling grain (pitch_ratio = 2^(semitones/12))
3. Window grain with Hann envelope for smooth overlap
4. Sum overlapping grains (50% overlap)
5. Feed back output to delay buffer with feedback amount
6. Mix wet/dry
```

**Characteristics:**
- Seeded RNG controls grain timing jitter for deterministic output
- Grain timing has 20% jitter for natural variation
- Pitch shift via linear interpolation resampling
- Feedback clamped to 0.95 for stability
- Supports `delay_time` post-FX LFO modulation

**Common Uses:**
- Shimmer reverb: pitch_semitones: +12 (octave up) with high feedback for ethereal textures
- Pitch cascade: positive pitch_semitones creates rising cascades, negative creates falling
- Frozen textures: long delay time with high feedback for sustaining pad-like textures
- Granular ambience: moderate settings for subtle texture without obvious pitch shift

### Post-FX LFO Modulation

The `post_fx_lfos` array on `AudioV1Params` enables LFO modulation of effect chain parameters over time. Unlike layer LFOs which modulate synthesis parameters, post-FX LFOs modulate the effect chain itself.

```json
{
  "effects": [
    { "type": "delay", "time_ms": 250, "feedback": 0.4, "wet": 0.3 }
  ],
  "post_fx_lfos": [
    {
      "config": { "waveform": "sine", "rate": 0.5, "depth": 1.0 },
      "target": { "target": "delay_time", "amount_ms": 25.0 }
    }
  ]
}
```

**Post-FX LFO Targets:**

| Target | Valid Effects | Amount Field | Formula |
|--------|---------------|--------------|---------|
| `delay_time` | `delay`, `multi_tap_delay`, `flanger`, `granular_delay`, `stereo_widener` (haas mode) | `amount_ms` | `time_ms = clamp(base + bipolar * amount_ms, 1.0, 2000.0)` for delay/multi_tap_delay; `time_ms = clamp(base + bipolar * amount_ms, 10.0, 2000.0)` for granular_delay; `delay_ms = clamp(base + bipolar * amount_ms, 0.1, 50.0)` for flanger/stereo_widener |
| `reverb_size` | `reverb` | `amount` | `room_size = clamp(base + bipolar * amount, 0.0, 1.0)` |
| `distortion_drive` | `waveshaper`, `tape_saturation` | `amount` | `drive = clamp(base + bipolar * amount, 1.0, 100.0)` for waveshaper; `drive = clamp(base + bipolar * amount, 1.0, 20.0)` for tape_saturation |

Where `bipolar = (lfo_value - 0.5) * 2.0` converts the LFO range from [0.0, 1.0] to [-1.0, 1.0].

**Validation Rules:**
- Post-FX targets (`delay_time`, `reverb_size`, `distortion_drive`) are NOT valid on layer LFOs (`layers[].lfo`)
- Layer targets (`pitch`, `volume`, `filter_cutoff`, `pan`, `pulse_width`, `fm_index`, `grain_size`, `grain_density`) are NOT valid on post-FX LFOs
- Each target may appear at most once in `post_fx_lfos`
- A `delay_time` LFO requires at least one `delay`, `multi_tap_delay`, `flanger`, `granular_delay`, or `stereo_widener` (haas mode) effect in `effects[]`
- A `reverb_size` LFO requires at least one `reverb` effect in `effects[]`
- A `distortion_drive` LFO requires at least one `waveshaper` or `tape_saturation` effect in `effects[]`

**Determinism:** Post-FX LFO curves are generated once for the full render duration using a component-derived seed, ensuring time-alignment across all matching effects.

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
