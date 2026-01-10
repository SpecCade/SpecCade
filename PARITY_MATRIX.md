# Legacy Spec Parity Matrix

> Auto-generated inventory of ai-studio-core legacy `.spec.py` dict keys.
> Source: `ai-studio-core/ai_studio_core/templates/project/studio/parsers/`

---

## Table of Contents

1. [SOUND (audio_sfx)](#sound-audio_sfx)
2. [INSTRUMENT (audio_instrument)](#instrument-audio_instrument)
3. [SONG (music)](#song-music)
4. [TEXTURE (texture_2d)](#texture-texture_2d)
5. [NORMAL (normal_map)](#normal-normal_map)
6. [MESH (static_mesh)](#mesh-static_mesh)
7. [SPEC/CHARACTER (skeletal_mesh)](#speccharacter-skeletal_mesh)
8. [ANIMATION (skeletal_animation)](#animation-skeletal_animation)
9. [Randomness Audit](#randomness-audit)

---

## SOUND (audio_sfx)

**Wrapper key:** `sound` (or top-level dict if no wrapper)
**Legacy dict name:** `SOUND`
**Parser:** `sound.py`

### Top-Level Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `name` | Yes | `str` | No path separators (`/`, `\`) | - | Used for output filename | ✓ |
| `duration` | No | `float` | > 0 | `1.0` | Total duration in seconds | ✓ |
| `sample_rate` | No | `int` | Valid audio rate | `22050` | Output sample rate | ✓ |
| `layers` | No | `list[dict]` | - | `[{'type': 'sine', 'freq': 440}]` | Layer specs (see below) | ✓ |
| `envelope` | No | `dict` | ADSR params | - | Master envelope | ✓ |
| `master_filter` | No | `dict` | Filter spec | - | Master filter applied after mixing | ✓ |
| `normalize` | No | `bool` | - | `True` | Whether to normalize output | ✓ |
| `peak_db` | No | `float` | Typically negative | `-3.0` | Target peak level in dB | ✓ |

### Layer Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `type` | No | `str` | One of: `sine`, `square`, `saw`, `triangle`, `noise_burst`, `fm_synth`, `karplus`, `pitched_body`, `metallic`, `harmonics` | `'sine'` | Synthesis type | ✓ |
| `duration` | No | `float` | > 0 | Parent duration | Layer duration | ✓ |
| `amplitude` | No | `float` | 0-1 | `1.0` | Layer volume | ✓ |
| `delay` | No | `float` | >= 0 | `0.0` | Time offset in seconds | ✓ |
| `freq` | No | `float` | > 0 | `440` | Base frequency (sine, square, saw, triangle, karplus) | ✓ |
| `freq_end` | No | `float` | > 0 | - | End frequency for sweep | ✓ |
| `duty` | No | `float` | 0-1 | `0.5` | Square wave duty cycle | ✓ |
| `color` | No | `str` | `white`, `pink`, `brown` | `'white'` | Noise color | ✓ |
| `carrier_freq` | No | `float` | > 0 | `440` | FM carrier frequency | ✓ |
| `mod_ratio` | No | `float` | > 0 | `1.0` | FM modulator ratio | ✓ |
| `mod_index` | No | `float` | >= 0 | `2.0` | FM modulation index | ✓ |
| `index_decay` | No | `float` | >= 0 | `5.0` | FM index decay rate | ✓ |
| `damping` | No | `float` | 0-1 | `0.996` | Karplus-Strong damping | ✓ |
| `brightness` | No | `float` | 0-1 | `0.7` | Karplus-Strong brightness | ✓ |
| `start_freq` | No | `float` | > 0 | `200` | Pitched body start freq | ✓ |
| `end_freq` | No | `float` | > 0 | `50` | Pitched body end freq | ✓ |
| `base_freq` | No | `float` | > 0 | `800` | Metallic base frequency | ✓ |
| `num_partials` | No | `int` | >= 1 | `6` | Metallic partial count | ✓ |
| `inharmonicity` | No | `float` | > 0 | `1.414` | Metallic inharmonicity factor | ✓ |
| `freqs` | No | `list[float]` | - | `[440]` | Harmonics frequencies | ✓ |
| `amplitudes` | No | `list[float]` | - | `[1.0]` | Harmonics amplitudes | ✓ |
| `envelope` | No | `dict` | ADSR | - | Per-layer envelope | ✓ |
| `filter` | No | `dict` | Filter spec | - | Per-layer filter | ✓ |

### Envelope Keys (ADSR)

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `attack` | No | `float` | >= 0 | `0.01` | Attack time in seconds | ✓ |
| `decay` | No | `float` | >= 0 | `0.1` | Decay time in seconds | ✓ |
| `sustain` | No | `float` | 0-1 | `0.7` | Sustain level | ✓ |
| `release` | No | `float` | >= 0 | `0.2` | Release time in seconds | ✓ |

### Filter Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `type` | No | `str` | `lowpass`, `highpass`, `bandpass` | `'lowpass'` | Filter type | ✓ |
| `cutoff` | No | `float` | 10 to Nyquist | `2000` | Cutoff frequency | ✓ |
| `cutoff_end` | No | `float` | 10 to Nyquist | - | End cutoff for sweep | ✓ |
| `q` | No | `float` | > 0 | `0.707` | Filter resonance | ✓ |
| `cutoff_low` | No | `float` | > 0 | `cutoff * 0.5` | Bandpass low cutoff | ✓ |
| `cutoff_high` | No | `float` | > 0 | `cutoff * 2.0` | Bandpass high cutoff | ✓ |

### Implementation Status

| Status | Count | Percentage |
|--------|-------|------------|
| Implemented (✓) | 38 | 100% |
| Partial (~) | 0 | 0% |
| Not Implemented (✗) | 0 | 0% |
| Deprecated (-) | 0 | 0% |

---

## INSTRUMENT (audio_instrument)

**Wrapper key:** `instrument` (or top-level dict if no wrapper)
**Legacy dict name:** `INSTRUMENT`
**Parser:** `sound.py`

### Top-Level Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `name` | Yes | `str` | No path separators | - | Output filename | ✓ |
| `base_note` | No | `str` or `int` | Note name (e.g., `'C4'`) or MIDI number | `'C4'` | Reference pitch | ✓ |
| `sample_rate` | No | `int` | Valid audio rate | `22050` | Output sample rate | ✓ |
| `synthesis` | No | `dict` | See below | `{'type': 'karplus_strong'}` | Synthesis parameters | ✓ |
| `envelope` | No | `dict` | ADSR | - | Amplitude envelope | ✓ |
| `pitch_envelope` | No | `dict` | - | - | Pitch envelope with depth in semitones | ✓ |
| `output` | No | `dict` | See below | - | Output options | ✓ |

### Synthesis Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `type` | No | `str` | `karplus_strong`, `fm`, `subtractive`, `additive` | `'karplus_strong'` | Synthesis type | ✓ |
| `damping` | No | `float` | 0-1 | `0.996` | KS damping | ✓ |
| `brightness` | No | `float` | 0-1 | `0.7` | KS brightness | ✓ |
| `operators` | No | `list[dict]` | - | - | FM operators | ✓ |
| `index` | No | `float` | >= 0 | `2.0` | FM modulation index | ✓ |
| `index_decay` | No | `float` | >= 0 | `5.0` | FM index decay | ✓ |
| `oscillators` | No | `list[dict]` | - | `[{'waveform': 'saw'}]` | Subtractive oscillators | ✓ |
| `filter` | No | `dict` | Filter spec | - | Subtractive filter | ✓ |
| `partials` | No | `list[tuple]` | `(ratio, amplitude)` | `[(1.0, 1.0)]` | Additive partials | ✓ |

### Oscillator Keys (Subtractive)

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `waveform` | No | `str` | `saw`, `square`, `sine`, `triangle` | `'saw'` | Waveform type | ✓ |
| `detune` | No | `float` | In cents | `0` | Detune amount | ✓ |
| `duty` | No | `float` | 0-1 | `0.5` | Square duty cycle | ✓ |

### Output Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `duration` | No | `float` | > 0 | `1.0` | Sample duration | ✓ |
| `bit_depth` | No | `int` | 8 or 16 | `16` | Output bit depth | ✓ |

### Implementation Status

| Status | Count | Percentage |
|--------|-------|------------|
| Implemented (✓) | 22 | 100% |
| Partial (~) | 0 | 0% |
| Not Implemented (✗) | 0 | 0% |
| Deprecated (-) | 0 | 0% |

---

## SONG (music)

**Wrapper key:** `song` (or top-level dict if no wrapper)
**Legacy dict name:** `SONG`
**Parser:** `music.py`

### Top-Level Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `name` | Yes | `str` | No path separators | - | Song title / output filename | ✓ |
| `title` | No | `str` | - | - | Alternative to `name` for title | ✓ |
| `format` | No | `str` | `'xm'` or `'it'` | `'xm'` | Output format | ✓ |
| `bpm` | No | `int` | 32-255 | `125` | Tempo in BPM | ✓ |
| `speed` | No | `int` | 1-31 | `6` | Ticks per row | ✓ |
| `channels` | No | `int` | 1-64 | `8` | Number of channels | ✓ |
| `restart_position` | No | `int` | >= 0 | `0` | Loop restart position | ✓ |
| `instruments` | No | `list[dict]` | - | `[]` | Instrument definitions | ✓ |
| `patterns` | No | `dict[str, dict]` | - | `{}` | Pattern definitions | ✓ |
| `arrangement` | No | `list[dict]` | - | `[]` | Pattern order | ✓ |
| `automation` | No | `list[dict]` | - | `[]` | Volume/tempo automation | ✓ |
| `it_options` | No | `dict` | - | - | IT-specific options | ✓ |

### Instrument Keys (inline or ref)

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `name` | No | `str` | - | `'instrument'` or filename | Instrument name | ✓ |
| `ref` | No | `str` | Path | - | Reference to external .spec.py | ✓ |
| `wav` | No | `str` | Path | - | Pre-recorded WAV file | ✓ |
| `base_note` | No | `str` | Note name | Format-dependent (`'C4'` for XM, `'C5'` for IT) | Sample pitch | ✓ |
| `synthesis` | No | `dict` | Same as INSTRUMENT | - | Inline synthesis | ✓ |
| `sample_rate` | No | `int` | - | `22050` | Sample rate | ✓ |
| `envelope` | No | `dict` | ADSR | - | Amplitude envelope | ✓ |

### Pattern Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `rows` | No | `int` | 1-256 | `64` | Number of rows | ✓ |
| `notes` | No | `dict[str, list]` | Channel -> note list | `{}` | Note data per channel | ✓ |

### Note Keys (within pattern)

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `row` | No | `int` | 0 to rows-1 | `0` | Row index | ✓ |
| `note` | No | `str` or `int` | Note name or XM/IT value | - | Note (e.g., `'C4'`, `'OFF'`, `97`) | ✓ |
| `inst` | No | `int` | 0-indexed | - | Instrument index (0-indexed, converted to 1-indexed) | ✓ |
| `vol` | No | `int` | 0-64 | `0` | Volume | ✓ |
| `effect` | No | `int` | 0-255 | `0` | Effect code | ✓ |
| `param` | No | `int` | 0-255 | `0` | Effect parameter | ✓ |
| `effect_name` | No | `str` | Named effect | - | Named effect (converted to code) | ✓ |
| `effect_xy` | No | `tuple[int, int]` | - | - | Effect X/Y nibbles | ✓ |

### Arrangement Entry Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `pattern` | Yes | `str` | Must exist in patterns | - | Pattern name | ✓ |
| `repeat` | No | `int` | >= 1 | `1` | Repeat count | ✓ |

### Automation Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `type` | Yes | `str` | `volume_fade`, `tempo_change` | - | Automation type | ✓ |
| `pattern` | Yes | `str` | Pattern name | - | Target pattern | ✓ |
| `channel` | No | `int` | 0-indexed | `0` | Target channel | ✓ |
| `start_row` | No | `int` | >= 0 | `0` | Start row | ✓ |
| `end_row` | No | `int` | >= start_row | `63` | End row | ✓ |
| `start_vol` | No | `int` | 0-64 | `0` | Start volume | ✓ |
| `end_vol` | No | `int` | 0-64 | `64` | End volume | ✓ |
| `row` | No | `int` | >= 0 | `0` | Row for tempo change | ✓ |
| `bpm` | No | `int` | 32-255 | `125` | New BPM | ✓ |

### IT Options Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `stereo` | No | `bool` | - | `True` | Stereo flag | ✓ |
| `global_volume` | No | `int` | 0-128 | `128` | Global volume | ✓ |
| `mix_volume` | No | `int` | 0-128 | `48` | Mix volume | ✓ |

### Implementation Status

| Status | Count | Percentage |
|--------|-------|------------|
| Implemented (✓) | 43 | 100% |
| Partial (~) | 0 | 0% |
| Not Implemented (✗) | 0 | 0% |
| Deprecated (-) | 0 | 0% |

---

## TEXTURE (texture_2d)

**Wrapper key:** `texture` (or top-level dict if no wrapper)
**Legacy dict name:** `TEXTURE`
**Parser:** `texture.py`

### Top-Level Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `name` | Yes | `str` | No path separators | - | Output filename | ✓ |
| `size` | No | `list[int, int]` | - | `[256, 256]` | Output dimensions [width, height] | ✓ |
| `output` | No | `dict` | - | - | Alternative size spec | ✓ |
| `output.size` | No | `list[int, int]` | - | `[256, 256]` | Output dimensions | ✓ |
| `layers` | No | `list[dict]` | - | `[{'type': 'noise', 'noise_type': 'perlin'}]` | Layer definitions | ✓ |
| `palette` | No | `list[str]` | Hex colors | - | Discrete color palette | ✓ |
| `color_ramp` | No | `list[str]` | Hex colors | - | Interpolated color ramp | ✓ |

### Layer Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `type` | No | `str` | `solid`, `noise`, `gradient`, `checkerboard`, `stripes`, `wood_grain`, `brick` | `'solid'` | Layer type | ✓ |
| `blend` | No | `str` | `normal`, `multiply`, `add`, `screen`, `overlay`, `soft_light` | `'normal'` | Blend mode | ✓ |
| `opacity` | No | `float` | 0-1 | `1.0` | Layer opacity | ✓ |

#### Solid Layer

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `color` | No | `float` or `str` | 0-1 or hex | `1.0` | Fill color/value | ✓ |

#### Noise Layer

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `noise_type` | No | `str` | `perlin`, `simplex`, `voronoi` | `'perlin'` | Noise algorithm | ✓ |
| `scale` | No | `float` | > 0 | `0.1` | Noise scale/frequency | ✓ |
| `octaves` | No | `int` | >= 1 | `4` | Fractal octaves | ✓ |
| `seed` | No | `int` | - | `42` | RNG seed | ✓ |
| `jitter` | No | `float` | 0-1 | `1.0` | Voronoi cell jitter | ✓ |

#### Gradient Layer

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `direction` | No | `str` | `vertical`, `horizontal`, `diagonal`, `radial` | `'vertical'` | Gradient direction | ✓ |
| `start` | No | `float` | 0-1 | `0.0` | Start value (linear) | ✓ |
| `end` | No | `float` | 0-1 | `1.0` | End value (linear) | ✓ |
| `center` | No | `tuple[float, float]` | 0-1 normalized | `(0.5, 0.5)` | Radial center | ✓ |
| `inner` | No | `float` | 0-1 | `1.0` | Radial inner value | ✓ |
| `outer` | No | `float` | 0-1 | `0.0` | Radial outer value | ✓ |

#### Checkerboard Layer

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `tile_size` | No | `int` | > 0 | `32` | Tile size in pixels | ✓ |
| `color1` | No | `float` | 0-1 | `0.0` | First color | ✓ |
| `color2` | No | `float` | 0-1 | `1.0` | Second color | ✓ |

#### Stripes Layer

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `direction` | No | `str` | `vertical`, `horizontal` | `'vertical'` | Stripe direction | ✓ |
| `stripe_width` | No | `int` | > 0 | `16` | Stripe width in pixels | ✓ |
| `color1` | No | `float` | 0-1 | `0.0` | First color | ✓ |
| `color2` | No | `float` | 0-1 | `1.0` | Second color | ✓ |

#### Wood Grain Layer

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `ring_count` | No | `int` | >= 1 | `8` | Number of rings | ✓ |
| `distortion` | No | `float` | >= 0 | `0.3` | Ring distortion amount | ✓ |
| `seed` | No | `int` | - | `42` | RNG seed | ✓ |

#### Brick Layer

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `brick_width` | No | `int` | > 0 | `64` | Brick width | ✓ |
| `brick_height` | No | `int` | > 0 | `32` | Brick height | ✓ |
| `mortar_width` | No | `int` | >= 0 | `4` | Mortar width | ✓ |
| `mortar_color` | No | `float` | 0-1 | `0.3` | Mortar color | ✓ |
| `brick_color` | No | `float` | 0-1 | `0.7` | Brick color | ✓ |
| `variation` | No | `float` | >= 0 | `0.1` | Per-brick variation | ✓ |
| `seed` | No | `int` | - | `42` | RNG seed | ✓ |

### Implementation Status

| Status | Count | Percentage |
|--------|-------|------------|
| Implemented (✓) | 47 | 100% |
| Partial (~) | 0 | 0% |
| Not Implemented (✗) | 0 | 0% |
| Deprecated (-) | 0 | 0% |

---

## NORMAL (normal_map)

**Wrapper key:** `normal` (or top-level dict if no wrapper)
**Legacy dict name:** `NORMAL`
**Parser:** `normal.py`

### Top-Level Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `name` | Yes | `str` | No path separators | - | Output filename | ✓ |
| `size` | No | `list[int, int]` | - | `[256, 256]` | Output dimensions | ✓ |
| `method` | No | `str` | `from_pattern` | `'from_pattern'` | Generation method | ✓ |
| `pattern` | No | `dict` | - | `{'type': 'noise'}` | Pattern specification | ✓ |
| `processing` | No | `dict` | - | - | Post-processing options | ✓ |

### Processing Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `strength` | No | `float` | > 0 | `1.0` | Normal strength multiplier | ✓ |
| `blur` | No | `float` | >= 0 | `0.0` | Gaussian blur sigma | ✓ |
| `invert` | No | `bool` | - | `True` | Invert height map | ✓ |

### Pattern Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `type` | No | `str` | `bricks`, `tiles`, `hexagons`, `noise`, `scratches`, `rivets`, `weave` | `'noise'` | Pattern type | ✓ |

#### Bricks Pattern

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `brick_width` | No | `int` | > 0 | `64` | Brick width | ✓ |
| `brick_height` | No | `int` | > 0 | `32` | Brick height | ✓ |
| `brick_size` | No | `list[int, int]` | - | - | Alternative: `[width, height]` | ✓ |
| `mortar_width` | No | `int` | >= 0 | `4` | Mortar width | ✓ |
| `mortar_depth` | No | `float` | 0-1 | `0.3` | Mortar depth | ✓ |
| `brick_variation` | No | `float` | >= 0 | `0.1` | Height variation | ✓ |
| `seed` | No | `int` | - | `42` | RNG seed | ✓ |

#### Tiles Pattern

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `tile_size` | No | `int` | > 0 | `64` | Tile size | ✓ |
| `gap_width` | No | `int` | >= 0 | `4` | Gap width | ✓ |
| `gap_depth` | No | `float` | 0-1 | `0.3` | Gap depth | ✓ |
| `seed` | No | `int` | - | `42` | RNG seed | ✓ |

#### Hexagons Pattern

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `hex_size` | No | `int` | > 0 | `32` | Hexagon size | ✓ |
| `gap_width` | No | `int` | >= 0 | `3` | Gap width | ✓ |
| `gap_depth` | No | `float` | 0-1 | `0.25` | Gap depth | ✓ |
| `seed` | No | `int` | - | `42` | RNG seed | ✓ |

#### Noise Pattern

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `scale` | No | `float` | > 0 | `0.1` | Noise scale | ✓ |
| `octaves` | No | `int` | >= 1 | `4` | Octaves | ✓ |
| `height_range` | No | `tuple[float, float]` | - | `(0.0, 1.0)` | Height range | ✓ |
| `seed` | No | `int` | - | `42` | RNG seed | ✓ |

#### Scratches Pattern

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `density` | No | `int` | >= 1 | `50` | Number of scratches | ✓ |
| `length_range` | No | `tuple[int, int]` | - | `(10, 40)` | Scratch length range | ✓ |
| `depth` | No | `float` | 0-1 | `0.15` | Scratch depth | ✓ |
| `seed` | No | `int` | - | `42` | RNG seed | ✓ |

#### Rivets Pattern

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `spacing` | No | `int` | > 0 | `32` | Rivet spacing | ✓ |
| `radius` | No | `int` | > 0 | `4` | Rivet radius | ✓ |
| `height` | No | `float` | >= 0 | `0.2` | Rivet height | ✓ |
| `seed` | No | `int` | - | `42` | RNG seed | ✓ |

#### Weave Pattern

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `thread_width` | No | `int` | > 0 | `8` | Thread width | ✓ |
| `gap` | No | `int` | >= 0 | `2` | Gap between threads | ✓ |
| `depth` | No | `float` | >= 0 | `0.15` | Weave depth | ✓ |

### Implementation Status

| Status | Count | Percentage |
|--------|-------|------------|
| Implemented (✓) | 44 | 100% |
| Partial (~) | 0 | 0% |
| Not Implemented (✗) | 0 | 0% |
| Deprecated (-) | 0 | 0% |

---

## MESH (static_mesh)

**Wrapper key:** `mesh` (or top-level dict if no wrapper)
**Legacy dict name:** `MESH`
**Parser:** `mesh.py`

### Top-Level Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `name` | Yes | `str` | No path separators | - | Output filename | ✓ |
| `primitive` | No | `str` | `cube`, `cylinder`, `sphere`, `icosphere`, `cone`, `torus` | `'cube'` | Primitive type | ✓ |
| `params` | No | `dict` | - | `{}` | Primitive parameters | ✓ |
| `location` | No | `tuple[float, float, float]` | - | `(0, 0, 0)` | Object location | ✓ |
| `rotation` | No | `tuple[float, float, float]` | Radians | `(0, 0, 0)` | Object rotation | ✓ |
| `scale` | No | `tuple[float, float, float]` | - | `(1, 1, 1)` | Object scale | ✓ |
| `shade` | No | `str` | `smooth`, `flat`, `None` | `'smooth'` | Shading mode | ✓ |
| `modifiers` | No | `list[dict]` | - | `[]` | Modifiers to apply | ✓ |
| `uv` | No | `dict` | - | `{}` | UV unwrap options | ✓ |
| `export` | No | `dict` | - | `{}` | Export options | ✓ |

### Primitive Params

#### Cube

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `size` | No | `float` | > 0 | `1.0` | Cube size | ✓ |

#### Cylinder

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `radius` | No | `float` | > 0 | `0.5` | Radius | ✓ |
| `depth` | No | `float` | > 0 | `1.0` | Height | ✓ |
| `vertices` | No | `int` | >= 3 | `24` | Vertex count | ✓ |

#### Sphere (UV)

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `radius` | No | `float` | > 0 | `0.5` | Radius | ✓ |
| `segments` | No | `int` | >= 3 | `32` | Horizontal segments | ✓ |
| `rings` | No | `int` | >= 2 | `16` | Vertical rings | ✓ |

#### Icosphere

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `radius` | No | `float` | > 0 | `0.5` | Radius | ✓ |
| `subdivisions` | No | `int` | >= 1 | `2` | Subdivision level | ✓ |

#### Cone

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `radius1` | No | `float` | >= 0 | `0.5` | Base radius | ✓ |
| `radius2` | No | `float` | >= 0 | `0.0` | Top radius | ✓ |
| `depth` | No | `float` | > 0 | `1.0` | Height | ✓ |
| `vertices` | No | `int` | >= 3 | `24` | Vertex count | ✓ |

#### Torus

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `major_radius` | No | `float` | > 0 | `0.75` | Major radius | ✓ |
| `minor_radius` | No | `float` | > 0 | `0.25` | Minor radius | ✓ |
| `major_segments` | No | `int` | >= 3 | `48` | Major segments | ✓ |
| `minor_segments` | No | `int` | >= 3 | `16` | Minor segments | ✓ |

### Modifier Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `type` | Yes | `str` | `bevel`, `decimate`, `triangulate` | - | Modifier type | ✓ |

#### Bevel Modifier

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `width` | No | `float` | > 0 | `0.02` | Bevel width | ✓ |
| `segments` | No | `int` | >= 1 | `2` | Bevel segments | ✓ |
| `angle_limit` | No | `float` | Radians | `0.785398` (~45 deg) | Angle limit | ✓ |

#### Decimate Modifier

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `ratio` | No | `float` | 0-1 | `0.5` | Decimation ratio | ✓ |

### UV Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `mode` | No | `str` | `smart_project`, `cube_project`, `None` | `'smart_project'` | UV mode | ✓ |
| `angle_limit` | No | `float` | Degrees | `66.0` | Smart project angle | ✓ |
| `cube_size` | No | `float` | > 0 | `1.0` | Cube project size | ✓ |

### Export Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `tangents` | No | `bool` | - | `False` | Export tangents | ✓ |

### Implementation Status

| Status | Count | Percentage |
|--------|-------|------------|
| Implemented (✓) | 40 | 100% |
| Partial (~) | 0 | 0% |
| Not Implemented (✗) | 0 | 0% |
| Deprecated (-) | 0 | 0% |

---

## SPEC/CHARACTER (skeletal_mesh)

**Wrapper key:** `character` (or top-level dict if no wrapper)
**Legacy dict name:** `SPEC`
**Parser:** `character.py`

### Top-Level Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `name` | Yes | `str` | No path separators | - | Output filename | ✓ |
| `tri_budget` | No | `int` | > 0 | `500` | Triangle budget (validation) | ✓ |
| `skeleton` | Yes | `list[dict]` | - | - | Bone definitions | ✓ |
| `parts` | Yes | `dict[str, dict]` | - | - | Body part definitions | ✓ |
| `texturing` | No | `dict` | - | - | UV/texturing options | ✓ |

### Skeleton Bone Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `bone` | Yes | `str` | Unique name | - | Bone name | ✓ |
| `head` | Yes (or mirror) | `list[float, float, float]` | - | - | Bone head position | ✓ |
| `tail` | Yes (or mirror) | `list[float, float, float]` | - | - | Bone tail position | ✓ |
| `parent` | No | `str` | Must exist | - | Parent bone name | ✓ |
| `mirror` | No | `str` | Existing bone name | - | Mirror from another bone (L->R) | ✓ |

### Part Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `bone` | Yes | `str` | Must exist in skeleton | - | Associated bone | ✓ |
| `base` | Yes | `str` | e.g., `'hexagon(6)'` | - | Base shape | ✓ |
| `base_radius` | Yes | `float` or `list[float, float]` | > 0 | - | Base radius or [bottom, top] | ✓ |
| `steps` | Yes | `list[dict or str]` | - | - | Extrusion steps | ✓ |
| `mirror` | No | `str` | Part name | - | Mirror from another part | ✓ |
| `offset` | No | `list[float, float, float]` | - | `[0, 0, 0]` | Position offset | ✓ |
| `rotation` | No | `list[float, float, float]` | Degrees | - | Initial rotation | ✓ |
| `cap_start` | No | `bool` | - | `True` | Cap bottom face | ✓ |
| `cap_end` | No | `bool` | - | `True` | Cap top face | ✓ |
| `skinning_type` | No | `str` | `soft`, `rigid` | `'soft'` | Skinning mode | ✓ |
| `thumb` | No | `dict` or `list[dict]` | - | - | Thumb sub-parts | ✓ |
| `fingers` | No | `list[dict]` | - | - | Finger sub-parts | ✓ |
| `instances` | No | `list[dict]` | - | - | Instanced copies | ✓ |

### Step Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `extrude` | No | `float` | - | `0.1` | Extrusion distance | ✓ |
| `scale` | No | `float` or `list[float]` | - | `1.0` | Scale factor(s) | ✓ |
| `translate` | No | `list[float, float, float]` | - | - | Translation offset | ✓ |
| `rotate` | No | `float` | Degrees | - | Z-axis rotation | ✓ |
| `bulge` | No | `float` or `list[float, float]` | - | - | Asymmetric bulge [side, forward_back] | ✓ |
| `tilt` | No | `float` or `list[float, float]` | Degrees | - | Tilt [x, y] | ✓ |

### Instance Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `position` | No | `list[float, float, float]` | - | `[0, 0, 0]` | Instance position | ✓ |
| `rotation` | No | `list[float, float, float]` | Degrees | `[0, 0, 0]` | Instance rotation | ✓ |

### Texturing Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `uv_mode` | No | `str` | `smart_project`, `region_based` | `'smart_project'` | UV mode | ✓ |
| `regions` | No | `dict[str, dict]` | - | - | Region definitions | ✓ |

### Implementation Status

| Status | Count | Percentage |
|--------|-------|------------|
| Implemented (✓) | 35 | 100% |
| Partial (~) | 0 | 0% |
| Not Implemented (✗) | 0 | 0% |
| Deprecated (-) | 0 | 0% |

---

## ANIMATION (skeletal_animation)

**Wrapper key:** `animation` (or top-level dict if no wrapper)
**Legacy dict name:** `ANIMATION`
**Parser:** `animation.py`

### Top-Level Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `name` | Yes | `str` | No path separators | - | Animation clip name | ✓ |
| `input_armature` | No | `str` | Path | - | Input GLB with armature | ✓ |
| `character` | No | `str` | Character name | - | Alternative: derive from input | ✓ |
| `duration_frames` | Yes | `int` | > 0 | - | Total frames | ✓ |
| `fps` | No | `int` | > 0 | `30` | Frames per second | ✓ |
| `loop` | No | `bool` | - | `False` | Looping animation | ✓ |
| `ground_offset` | No | `float` | - | `0.0` | Vertical offset | ✓ |
| `poses` | No | `dict[str, dict]` | - | `{}` | Named pose definitions | ✓ |
| `phases` | No | `list[dict]` | - | `[]` | Animation phases | ✓ |
| `procedural_layers` | No | `list[dict]` | - | `[]` | Procedural overlays | ✓ |
| `rig_setup` | No | `dict` | - | - | IK/constraint setup | ✓ |
| `ik_hints` | No | `dict` | - | - | Legacy IK hints (deprecated) | - |
| `save_blend` | No | `bool` | - | `False` | Save .blend file | ✓ |
| `animator_rig` | No | `dict` | - | - | Animator rig config | ✓ |
| `conventions` | No | `dict` | - | - | Validation conventions | ✓ |

### Pose Keys (per bone)

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `pitch` | No | `float` | Degrees | `0` | X rotation (pitch) | ✓ |
| `yaw` | No | `float` | Degrees | `0` | Y rotation (yaw) | ✓ |
| `roll` | No | `float` | Degrees | `0` | Z rotation (roll) | ✓ |
| `location` | No | `list[float, float, float]` | - | - | Bone location offset | ✓ |

### Phase Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `name` | No | `str` | - | - | Phase name | ✓ |
| `frames` | Yes | `list[int, int]` | - | - | [start, end] frame range | ✓ |
| `pose` | No | `str` | Must exist in poses | - | Pose name to apply | ✓ |
| `timing_curve` | No | `str` | `linear`, `ease_in`, `ease_out`, `ease_in_out`, `exponential_in`, `exponential_out`, `constant` | `'linear'` | Interpolation curve | ✓ |
| `ik_targets` | No | `dict[str, list]` | - | - | IK target keyframes | ✓ |
| `ikfk_blend` | No | `dict[str, list]` | - | - | IK/FK blend keyframes | ✓ |

### IK Target Keyframe Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `frame` | Yes | `int` | >= 0 | - | Frame number | ✓ |
| `location` | Yes | `list[float, float, float]` | - | - | World position | ✓ |
| `ikfk` | No | `float` | 0-1 | - | IK/FK blend (0=FK, 1=IK) | ✓ |

### Procedural Layer Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `type` | Yes | `str` | `breathing`, `sway`, `bob`, `noise` | - | Layer type | ✓ |
| `target` | Yes | `str` | Bone name | - | Target bone | ✓ |
| `axis` | No | `str` | `pitch`, `yaw`, `roll` | `'pitch'` | Rotation axis | ✓ |
| `period_frames` | No | `int` | > 0 | `60` | Period for sine-based | ✓ |
| `amplitude` | No | `float` | - | `0.01` | Amplitude (radians for sine, degrees for noise) | ✓ |
| `phase_offset` | No | `float` | - | `0` | Phase offset | ✓ |
| `frequency` | No | `float` | > 0 | `0.3` | Noise frequency | ✓ |

### Rig Setup Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `presets` | No | `dict[str, bool]` | - | `{}` | Preset enables | ✓ |
| `ik_chains` | No | `list[dict]` | - | `[]` | Custom IK chains | ✓ |
| `foot_systems` | No | `list[dict]` | - | `[]` | Foot roll systems | ✓ |
| `aim_constraints` | No | `list[dict]` | - | `[]` | Aim/look-at constraints | ✓ |
| `constraints` | No | `list[dict]` | - | `[]` | Generic constraints | ✓ |
| `twist_bones` | No | `list[dict]` | - | `[]` | Twist bone setups | ✓ |
| `stretch` | No | `dict` | - | - | Stretch settings | ✓ |
| `bake` | No | `dict` | - | - | Bake settings | ✓ |

### Preset Names

| Preset | Required Bones | Creates |
|--------|---------------|---------|
| `humanoid_legs` | `leg_upper_L/R`, `leg_lower_L/R`, `foot_L/R` | `ik_foot_L/R`, `pole_knee_L/R` |
| `humanoid_arms` | `arm_upper_L/R`, `arm_lower_L/R` | `ik_hand_L/R`, `pole_elbow_L/R` |
| `spider_legs` | `leg_{front|mid_front|mid_back|back}_{upper|lower}_{L|R}` | IK targets and poles |
| `quadruped_legs` | `leg_{front|back}_{upper|lower}_{L|R}` | IK targets and poles |
| `basic_spine` | - (auto-detects spine bones) | `ik_spine_tip` |
| `head_look` | `head` | `look_target` |

### IK Chain Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `name` | Yes | `str` | - | - | Chain name | ✓ |
| `bones` | Yes | `list[str]` | - | - | Bone list (root to tip) | ✓ |
| `target` | No | `dict` | - | Auto-generated | IK target config | ✓ |
| `target.name` | No | `str` | - | `ik_{chain_name}` | Target bone name | ✓ |
| `target.at` | No | `str` or `list` | `tip`, `head`, `[x,y,z]` | `'tip'` | Target position | ✓ |
| `pole` | No | `dict` | - | - | Pole target config | ✓ |
| `pole.name` | No | `str` | - | `pole_{chain_name}` | Pole bone name | ✓ |
| `pole.offset` | No | `list[float, float, float]` | - | `[0, 0.3, 0]` | Pole offset | ✓ |
| `pole.auto_place` | No | `bool` | - | `False` | Auto-place pole | ✓ |
| `pole.angle` | No | `float` or `'auto'` | Radians | `0` | Pole angle | ✓ |
| `stretch` | No | `dict` | - | - | Stretch settings | ✓ |
| `stretch.enabled` | No | `bool` | - | `False` | Enable stretch | ✓ |
| `rotation_limits` | No | `dict` | - | - | Per-axis limits | ✓ |

### Constraint Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `bone` | Yes | `str` | - | - | Target bone | ✓ |
| `type` | Yes | `str` | `hinge`, `ball`, `planar`, `soft` | - | Constraint type | ✓ |
| `axis` | No | `str` | `X`, `Y`, `Z` | `'X'` | Hinge axis | ✓ |
| `limits` | No | `list[float, float]` or `dict` | Degrees | `[0, 160]` | Rotation limits | ✓ |

### Twist Bone Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `name` | No | `str` | - | - | Twist name | ✓ |
| `source` | Yes | `str` | - | - | Source bone | ✓ |
| `target` | Yes | `str` | - | - | Twist bone | ✓ |
| `axis` | No | `str` | `X`, `Y`, `Z` | `'Y'` | Twist axis | ✓ |
| `influence` | No | `float` | 0-1 | `0.5` | Copy influence | ✓ |

### Bake Settings Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `simplify` | No | `bool` | - | `True` | Simplify curves after baking | ✓ |
| `start_frame` | No | `int` | >= 0 | - | Bake start frame | ✓ |
| `end_frame` | No | `int` | >= start | - | Bake end frame | ✓ |
| `visual_keying` | No | `bool` | - | `True` | Use visual transforms | ✓ |
| `clear_constraints` | No | `bool` | - | `True` | Clear constraints after bake | ✓ |
| `frame_step` | No | `int` | >= 1 | `1` | Bake step | ✓ |
| `remove_ik_bones` | No | `bool` | - | `True` | Remove IK control bones | ✓ |
| `tolerance` | No | `float` | > 0 | `0.001` | Clean tolerance | ✓ |

### Animator Rig Config Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `collections` | No | `bool` | - | `True` | Organize bone collections | ✓ |
| `shapes` | No | `bool` | - | `True` | Add custom bone shapes | ✓ |
| `colors` | No | `bool` | - | `True` | Color-code bones | ✓ |
| `display` | No | `str` | `OCTAHEDRAL`, `STICK`, `BBONE`, etc. | `'OCTAHEDRAL'` | Armature display | ✓ |

### Conventions Keys

| Key | Required | Type | Constraints | Default | Notes | Status |
|-----|----------|------|-------------|---------|-------|--------|
| `strict` | No | `bool` | - | `False` | Fail on validation errors | ✓ |

### Implementation Status

| Status | Count | Percentage |
|--------|-------|------------|
| Implemented (✓) | 74 | 100% |
| Partial (~) | 0 | 0% |
| Not Implemented (✗) | 0 | 0% |
| Deprecated (-) | 1 | - |

---

## Randomness Audit

This section documents all sources of randomness in the legacy parsers and whether they are seeded.

### sound.py

| Function | Randomness Source | Seeded | Notes |
|----------|-------------------|--------|-------|
| `white_noise()` | `np.random.randn()` | No | Uses global numpy state |
| `pink_noise()` | `np.random.randn()` | No | Uses global numpy state |
| `brown_noise()` | `np.random.randn()` | No | Uses global numpy state |
| `karplus_strong()` | `np.random.randn()` | No | Initial noise burst |

**Recommendation:** All noise functions should accept a `seed` parameter and use a seeded RNG.

### texture.py

| Function | Randomness Source | Seeded | Notes |
|----------|-------------------|--------|-------|
| `perlin_noise()` | `np.random.seed(seed)` | Yes | Uses passed seed |
| `voronoi_noise()` | `np.random.seed(seed)` | Yes | Uses passed seed |
| `wood_grain()` | `np.random.seed(seed)` | Yes | Via perlin_noise |
| `brick_pattern()` | `np.random.seed(seed)` | Yes | Per-brick seeding |

**Status:** Properly seeded with `seed` parameter.

### normal.py

| Function | Randomness Source | Seeded | Notes |
|----------|-------------------|--------|-------|
| `pattern_bricks()` | `np.random.seed(seed)` | Yes | Uses passed seed |
| `pattern_tiles()` | `np.random.seed(seed)` | Yes | Uses passed seed |
| `pattern_hexagons()` | `np.random.seed(seed)` | Yes | Uses passed seed |
| `pattern_noise()` | `np.random.seed(seed)` | Yes | Uses passed seed |
| `pattern_scratches()` | `np.random.seed(seed)` | Yes | Uses passed seed |
| `pattern_rivets()` | `np.random.seed(seed)` | Yes | Uses passed seed |

**Status:** Properly seeded with `seed` parameter.

### character.py

| Function | Randomness Source | Seeded | Notes |
|----------|-------------------|--------|-------|
| N/A | N/A | N/A | No randomness used |

**Status:** Deterministic.

### animation.py

| Function | Randomness Source | Seeded | Notes |
|----------|-------------------|--------|-------|
| `bake_noise_layer()` | `random.seed(seed)` | Yes | Seeded from bone name hash |

**Status:** Seeded, but seed derivation is implicit (bone name hash).

### music.py

| Function | Randomness Source | Seeded | Notes |
|----------|-------------------|--------|-------|
| N/A | N/A | N/A | No randomness used |

**Status:** Deterministic (patterns are explicit note data).

### mesh.py

| Function | Randomness Source | Seeded | Notes |
|----------|-------------------|--------|-------|
| N/A | N/A | N/A | No randomness used |

**Status:** Deterministic.

---

## Summary

| Asset Type | Legacy Dict | Wrapper Key | Required Keys | Has Randomness | Properly Seeded |
|------------|-------------|-------------|---------------|----------------|-----------------|
| audio_sfx | `SOUND` | `sound` | `name` | Yes (noise) | No - needs fix |
| audio_instrument | `INSTRUMENT` | `instrument` | `name` | Yes (noise) | No - needs fix |
| music | `SONG` | `song` | `name` | No | N/A |
| texture_2d | `TEXTURE` | `texture` | `name` | Yes | Yes |
| normal_map | `NORMAL` | `normal` | `name` | Yes | Yes |
| static_mesh | `MESH` | `mesh` | `name` | No | N/A |
| skeletal_mesh | `SPEC` | `character` | `name`, `skeleton`, `parts` | No | N/A |
| skeletal_animation | `ANIMATION` | `animation` | `name`, `input_armature`, `duration_frames` | Yes (noise layer) | Yes |

---

## Overall Implementation Status

| Asset Type | Implemented (✓) | Partial (~) | Not Implemented (✗) | Deprecated (-) | Total | % Implemented |
|------------|-----------------|-------------|---------------------|----------------|-------|---------------|
| SOUND | 38 | 0 | 0 | 0 | 38 | 100% |
| INSTRUMENT | 22 | 0 | 0 | 0 | 22 | 100% |
| SONG | 43 | 0 | 0 | 0 | 43 | 100% |
| TEXTURE | 47 | 0 | 0 | 0 | 47 | 100% |
| NORMAL | 44 | 0 | 0 | 0 | 44 | 100% |
| MESH | 40 | 0 | 0 | 0 | 40 | 100% |
| CHARACTER | 35 | 0 | 0 | 0 | 35 | 100% |
| ANIMATION | 74 | 0 | 0 | 1 | 75 | 100% |
| **TOTAL** | **343** | **0** | **0** | **1** | **344** | **100%** |

---

*Generated for SpecCade Phase 0 Task 0.1*
