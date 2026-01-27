# SpecCade LLM Quick Reference (ARCHIVED)

> **DEPRECATED** — This file is a historical snapshot from RFC-0008 and contains **stale syntax and APIs**.
> Do NOT use this as a reference for current SpecCade development.
>
> Current authoritative sources:
> - `speccade stdlib dump --format json` (Starlark stdlib)
> - `speccade validate` / `docs/spec-reference/` (spec contract)
> - `docs/stdlib-reference.md` (stdlib index with domain file links)

---

## Core Concepts

SpecCade creates deterministic audio/texture/mesh assets from declarative specs.

**Pipeline**: `.star file → compile → JSON IR → validate → generate → .wav/.png/.glb`

**Key Principle**: Same spec + seed = identical output, always.

---

## Starlark Basics

```python
# Every spec needs: spec(), one or more outputs, output()
spec(name="my_sound", seed=42)

sound = audio_layer(
    synthesis = oscillator("sine", 440, 0.8),
    envelope = envelope(0.01, 0.1, 0.7, 0.3)
)

output("main", "audio", sound, duration=1.0, sample_rate=44100)
```

---

## Synthesis Types

| Type | Function | Key Params | Use For |
|------|----------|------------|---------|
| Basic | `oscillator(wave, freq, amp)` | wave: sine/square/sawtooth/triangle/noise/pulse | Simple tones, subs |
| FM | `fm_synth(carrier, mod, index)` | carrier_freq, mod_freq, mod_index | Bells, metallic, bass |
| Additive | `additive_synth(partials)` | list of {freq, amp, phase} | Organs, rich tones |
| Wavetable | `wavetable_synth(table, pos)` | table: [samples], position | Evolving pads |
| Karplus | `karplus_strong(freq, decay, ...)` | frequency, damping, feedback | Plucked strings |
| Modal | `modal_synth(modes)` | list of {freq, decay, amp} | Bells, wood, metal |
| Granular | `granular_synth(grain_size, ...)` | grain_size_ms, density, pitch_var | Textures, pads |
| Supersaw | `supersaw(freq, voices, detune)` | num_voices, detune_amount | Leads, trance |
| Physical | `physical_model(model, ...)` | model: string/membrane/... | Realistic instruments |

### Oscillator Waveforms

`sine` | `square` | `sawtooth` | `triangle` | `noise` | `pulse` (with pulse_width)

---

## Envelope

```python
envelope(attack, decay, sustain, release)
```

| Param | Range | Description |
|-------|-------|-------------|
| attack | 0.001 - 2.0 | Seconds to peak |
| decay | 0.01 - 5.0 | Seconds to sustain |
| sustain | 0.0 - 1.0 | Level (0=silent, 1=full) |
| release | 0.01 - 10.0 | Seconds to silence |

**Common shapes**:
- Pluck: `envelope(0.001, 0.15, 0.0, 0.2)`
- Pad: `envelope(0.5, 0.2, 0.8, 1.0)`
- Percussion: `envelope(0.001, 0.1, 0.0, 0.05)`

---

## Filters

```python
filter(type, cutoff, resonance)
```

| Type | Description |
|------|-------------|
| lowpass | Removes highs, warmth |
| highpass | Removes lows, thin |
| bandpass | Isolates frequency range |
| notch | Removes specific frequency |
| ladder | Classic analog character |
| state_variable | Flexible, morphable |

| Param | Range | Description |
|-------|-------|-------------|
| cutoff | 20 - 20000 | Frequency in Hz |
| resonance | 0.0 - 1.0 | Emphasis at cutoff |

---

## Effects

```python
effect(type, **params)
```

### Dynamics
| Effect | Key Params |
|--------|------------|
| compressor | threshold (-60 to 0 dB), ratio (1-20), attack_ms, release_ms |
| limiter | threshold, release_ms |
| gate | threshold, attack_ms, release_ms |

### Time-Based
| Effect | Key Params |
|--------|------------|
| delay | time_ms (1-2000), feedback (0-1), wet (0-1) |
| reverb | room_size (0-1), damping (0-1), wet (0-1) |
| chorus | rate_hz, depth, wet |
| flanger | rate_hz, depth, feedback |
| phaser | rate_hz, depth, stages (2-12) |

### Distortion
| Effect | Key Params |
|--------|------------|
| distortion | drive (0-1), tone (0-1) |
| bitcrush | bit_depth (1-16), sample_rate_reduction |
| saturation | amount (0-1), type: tape/tube/soft |

### EQ/Filter
| Effect | Key Params |
|--------|------------|
| eq | bands: [{freq, gain_db, q}] |
| lowpass/highpass/bandpass | cutoff, resonance |

---

## Modulation (LFO)

```python
lfo(shape, rate, depth, target)
```

| Param | Values |
|-------|--------|
| shape | sine, triangle, square, sawtooth, random |
| rate | 0.01 - 50.0 Hz |
| depth | 0.0 - 1.0 |
| target | filter_cutoff, amplitude, pitch, pan, effect_param |

---

## Layering

```python
audio_layer(
    synthesis = ...,
    envelope = ...,
    filter = ...,      # optional
    lfo = ...,         # optional
    effects = [...],   # optional list
    gain = 0.8,        # 0.0 - 1.0
    pan = 0.0          # -1.0 (left) to 1.0 (right)
)
```

Combine multiple layers:
```python
kick_body = audio_layer(synthesis=oscillator("sine", 55, 1.0), ...)
kick_click = audio_layer(synthesis=oscillator("noise", 0, 0.3), ...)

output("kick", "audio", [kick_body, kick_click], duration=0.5)
```

---

## Common Recipes

### Kick Drum
```python
audio_layer(
    synthesis = oscillator("sine", 55, 1.0),
    envelope = envelope(0.001, 0.15, 0.0, 0.1),
    effects = [effect("distortion", drive=0.2)]
)
```

### Snare
```python
body = audio_layer(
    synthesis = oscillator("triangle", 180, 0.7),
    envelope = envelope(0.001, 0.1, 0.0, 0.15)
)
noise = audio_layer(
    synthesis = oscillator("noise", 0, 0.5),
    envelope = envelope(0.001, 0.15, 0.0, 0.1),
    filter = filter("highpass", 2000, 0.3)
)
```

### Hi-Hat
```python
audio_layer(
    synthesis = oscillator("noise", 0, 0.4),
    envelope = envelope(0.001, 0.05, 0.0, 0.03),
    filter = filter("highpass", 8000, 0.4)
)
```

### Pad
```python
audio_layer(
    synthesis = supersaw(220, 5, 0.03),
    envelope = envelope(0.8, 0.3, 0.7, 1.5),
    filter = filter("lowpass", 2000, 0.3),
    lfo = lfo("sine", 0.2, 0.1, "filter_cutoff"),
    effects = [effect("reverb", room_size=0.8, wet=0.4)]
)
```

### Bass
```python
audio_layer(
    synthesis = oscillator("sawtooth", 55, 0.9),
    envelope = envelope(0.005, 0.2, 0.6, 0.15),
    filter = filter("lowpass", 800, 0.5),
    effects = [effect("distortion", drive=0.15)]
)
```

### Bell/FM
```python
audio_layer(
    synthesis = fm_synth(440, 880, 5.0),
    envelope = envelope(0.001, 0.5, 0.3, 2.0)
)
```

### Plucked String
```python
audio_layer(
    synthesis = karplus_strong(220, 0.996, 0.5),
    envelope = envelope(0.001, 0.3, 0.0, 0.5)
)
```

---

## Music/Tracker Basics

```python
spec(name="song", seed=1)

inst = instrument("synth_lead", audio_layer(...))
pat = pattern("verse", length=64, events=[
    emit(at=range_op(0, 16, 4), cell=cell(0, "C4", 0, 64))
])
song("main", tempo=120, patterns=[
    pattern_ref("verse", 0, 0)
])

output("song", "module", song_ref("main"), format="xm")
```

### Notes
Use: `C4`, `D#5`, `Gb3`, etc. (note + octave)

### Pattern Operations
- `emit(at, cell)` - Place note at tick(s)
- `range_op(start, end, step)` - Generate tick positions
- `cell(channel, note, instrument, volume)` - Note data

---

## Texture Basics

```python
spec(name="texture", seed=1)

n1 = noise_node("perlin", scale=4.0, octaves=4)
n2 = gradient_node("linear", direction="horizontal")
blend = blend_node(n1, n2, mode="multiply", factor=0.5)

output("tex", "texture", blend, width=256, height=256, format="png")
```

### Noise Types
`perlin` | `simplex` | `voronoi` | `worley` | `fbm` | `white`

### Blend Modes
`add` | `multiply` | `screen` | `overlay` | `difference`

---

## Mesh Basics

```python
spec(name="mesh", seed=1)

cube = mesh_primitive("cube", size=1.0)
smoothed = mesh_modifier(cube, "subdivide", levels=2)

output("model", "mesh", smoothed, format="glb")
```

### Primitives
`cube` | `sphere` | `cylinder` | `cone` | `torus` | `plane`

### Modifiers
`subdivide` | `decimate` | `smooth` | `mirror` | `array`

---

## Budget Profiles

| Profile | Max Layers | Max Effects | Max Duration | Use Case |
|---------|------------|-------------|--------------|----------|
| default | 16 | 8/layer | 60s | General |
| strict | 8 | 4/layer | 30s | Production |

Use: `speccade validate --budget strict`

---

## Common Errors

| Code | Meaning | Fix |
|------|---------|-----|
| E001 | Missing field | Add required param |
| E003 | Value out of range | Check param ranges |
| E006 | Unknown synthesis type | Use valid type name |
| S103 | Starlark range error | Check numeric ranges |
| S104 | Invalid enum | Use exact value from list |

---

## Quick CLI

```bash
# Compile .star to JSON
speccade eval --spec file.star --pretty

# Validate only
speccade validate --spec file.star

# Generate
speccade generate --spec file.star --out-root ./out

# With budget
speccade generate --spec file.star --out-root ./out --budget strict
```

---

*Reference v1.0 - SpecCade Starlark Branch*
