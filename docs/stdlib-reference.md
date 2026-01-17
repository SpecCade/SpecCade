# SpecCade Starlark Standard Library Reference

The SpecCade stdlib provides helper functions for authoring asset specs in Starlark.
These functions emit canonical IR-compatible dictionaries, reducing boilerplate and
improving the authoring experience.

## Design Principles

1. **Flat, explicit parameters** - No hidden defaults; parameters are clear and explicit
2. **Composable** - Functions return dicts that can be modified or combined
3. **Deterministic** - No random, time, or IO functions
4. **Domain-prefixed** - Functions are grouped by domain (audio, texture, mesh)
5. **Minimal** - Core functions covering the most common use cases

## Error Codes

All stdlib errors use a stable S-series format:

| Code | Category | Description |
|------|----------|-------------|
| S001-S009 | Compiler | Syntax, runtime, timeout errors |
| S101 | Stdlib | Missing required argument |
| S102 | Stdlib | Type mismatch |
| S103 | Stdlib | Value out of range |
| S104 | Stdlib | Invalid enum value |

---

## Function Categories

### [Core Functions](stdlib-core.md)
Core spec and output functions for creating asset specifications.

**Functions:** `spec()`, `output()`

---

### [Audio Functions](stdlib-audio.md)
Audio synthesis, filtering, effects, modulation, and layer composition.

**Categories:**
- **Synthesis** - Oscillators, FM, AM, granular, physical modeling, and more
- **Filters** - Lowpass, highpass, bandpass, ladder, formant, and shelving filters
- **Effects** - Reverb, delay, chorus, phaser, compression, distortion, and specialty effects
- **Modulation** - LFOs, envelopes, and pitch modulation
- **Layers** - Audio layer composition

**Key Functions:** `oscillator()`, `fm_synth()`, `granular()`, `lowpass()`, `reverb()`, `delay()`, `lfo()`, `audio_layer()`

---

### [Texture Functions](stdlib-texture.md)
Node-based procedural texture generation.

**Categories:**
- **Node Functions** - Noise, gradients, patterns, operations, and color manipulation
- **Graph Functions** - Texture graph assembly

**Key Functions:** `noise_node()`, `gradient_node()`, `color_ramp_node()`, `texture_graph()`

---

### [Mesh Functions](stdlib-mesh.md)
Primitive mesh generation with modifiers.

**Categories:**
- **Primitive Functions** - Basic mesh shapes
- **Modifier Functions** - Bevel, subdivision, decimation, and other modifiers

**Key Functions:** `mesh_primitive()`, `mesh_recipe()`, `bevel_modifier()`, `subdivision_modifier()`

---

### [Music Functions](stdlib-music.md)
Tracker-style music composition with instruments, patterns, and arrangements.

**Categories:**
- **Instrument Functions** - Tracker instrument synthesis and configuration
- **Pattern Functions** - Note events and pattern definition
- **Song Functions** - Song structure, automation, and options

**Key Functions:** `tracker_instrument()`, `pattern_note()`, `tracker_pattern()`, `tracker_song()`, `music_spec()`

---

## Quick Reference

### Core Functions
| Function | Description |
|----------|-------------|
| `spec()` | Create a complete spec dictionary |
| `output()` | Create an output specification |

### Audio Synthesis
| Function | Description |
|----------|-------------|
| `envelope()` | ADSR envelope |
| `oscillator()` | Basic oscillator synthesis |
| `fm_synth()` | FM synthesis |
| `am_synth()` | AM synthesis |
| `noise_burst()` | Noise burst synthesis |
| `karplus_strong()` | Plucked string synthesis |
| `additive()` | Additive synthesis |
| `supersaw_unison()` | Supersaw/unison synthesis |
| `wavetable()` | Wavetable synthesis |
| `granular()` | Granular synthesis |
| `modal()` | Modal synthesis |
| `metallic()` | Metallic synthesis |
| `vocoder()` | Vocoder synthesis |
| `formant_synth()` | Formant/voice synthesis |
| `vector_synth()` | Vector synthesis |
| `waveguide()` | Wind instrument modeling |
| `bowed_string()` | Bowed string modeling |
| `pulsar()` | Pulsar synthesis |
| `vosim()` | VOSIM voice synthesis |
| `spectral_freeze()` | Spectral freeze synthesis |
| `pitched_body()` | Impact sound synthesis |

### Audio Filters
| Function | Description |
|----------|-------------|
| `lowpass()` | Lowpass filter |
| `highpass()` | Highpass filter |
| `bandpass()` | Bandpass filter |
| `notch()` | Notch filter |
| `allpass()` | Allpass filter |
| `comb_filter()` | Comb filter |
| `formant_filter()` | Formant filter |
| `ladder()` | Ladder filter (Moog-style) |
| `shelf_low()` | Low shelf filter |
| `shelf_high()` | High shelf filter |

### Audio Effects
| Function | Description |
|----------|-------------|
| `reverb()` | Reverb effect |
| `delay()` | Delay effect |
| `compressor()` | Compressor |
| `limiter()` | Limiter |
| `chorus()` | Chorus effect |
| `phaser()` | Phaser effect |
| `flanger()` | Flanger effect |
| `bitcrush()` | Bitcrusher |
| `waveshaper()` | Waveshaper distortion |
| `parametric_eq()` | Parametric EQ |
| `stereo_widener()` | Stereo width effect |
| `multi_tap_delay()` | Multi-tap delay |
| `tape_saturation()` | Tape saturation |
| `transient_shaper()` | Transient shaper |
| `auto_filter()` | Auto-filter/envelope follower |
| `cabinet_sim()` | Cabinet simulation |
| `rotary_speaker()` | Rotary speaker (Leslie) |
| `ring_modulator()` | Ring modulator |
| `granular_delay()` | Granular delay |

### Audio Modulation
| Function | Description |
|----------|-------------|
| `lfo()` | LFO configuration |
| `lfo_modulation()` | LFO modulation with target |
| `pitch_envelope()` | Pitch envelope |

### Audio Layers
| Function | Description |
|----------|-------------|
| `audio_layer()` | Complete audio synthesis layer |

### Texture Nodes
| Function | Description |
|----------|-------------|
| `noise_node()` | Noise texture node |
| `gradient_node()` | Gradient node |
| `constant_node()` | Constant value node |
| `threshold_node()` | Threshold operation |
| `invert_node()` | Invert operation |
| `color_ramp_node()` | Color ramp mapping |
| `add_node()` | Add blend |
| `multiply_node()` | Multiply blend |
| `lerp_node()` | Linear interpolation |
| `clamp_node()` | Clamp operation |
| `stripes_node()` | Stripes pattern |
| `checkerboard_node()` | Checkerboard pattern |
| `grayscale_node()` | Grayscale conversion |
| `palette_node()` | Palette quantization |
| `compose_rgba_node()` | RGBA composition |
| `normal_from_height_node()` | Normal map from height |

### Texture Graph
| Function | Description |
|----------|-------------|
| `texture_graph()` | Complete texture graph |

### Mesh Primitives
| Function | Description |
|----------|-------------|
| `mesh_primitive()` | Base mesh primitive |
| `mesh_recipe()` | Complete mesh recipe |

### Mesh Modifiers
| Function | Description |
|----------|-------------|
| `bevel_modifier()` | Bevel modifier |
| `subdivision_modifier()` | Subdivision surface |
| `decimate_modifier()` | Decimate modifier |
| `edge_split_modifier()` | Edge split |
| `mirror_modifier()` | Mirror modifier |
| `array_modifier()` | Array modifier |
| `solidify_modifier()` | Solidify modifier |

### Music Instruments
| Function | Description |
|----------|-------------|
| `instrument_synthesis()` | Tracker instrument synthesis |
| `tracker_instrument()` | Tracker instrument definition |

### Music Patterns
| Function | Description |
|----------|-------------|
| `pattern_note()` | Pattern note event |
| `tracker_pattern()` | Tracker pattern definition |
| `arrangement_entry()` | Arrangement entry |

### Music Song
| Function | Description |
|----------|-------------|
| `it_options()` | IT-specific options |
| `volume_fade()` | Volume fade automation |
| `tempo_change()` | Tempo change automation |
| `tracker_song()` | Complete tracker song |
| `music_spec()` | Music spec wrapper |

---

## Usage Examples

### Creating an Audio Spec
```python
spec(
    asset_id = "laser-blast-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/laser.wav", "wav")],
    recipe = {
        "backend": "audio.procedural_v1",
        "params": {
            "duration": 0.5,
            "layers": [
                audio_layer(
                    oscillator(880, "sawtooth", 220, "exponential"),
                    envelope(0.01, 0.1, 0.0, 0.1),
                    filter = lowpass(5000, 0.707, 500)
                )
            ]
        }
    }
)
```

### Creating a Texture Spec
```python
spec(
    asset_id = "noise-texture-01",
    asset_type = "texture",
    seed = 123,
    outputs = [output("textures/noise.png", "png")],
    recipe = {
        "backend": "texture.procedural_v1",
        "params": texture_graph(
            [256, 256],
            [
                noise_node("base", "perlin", 0.05, 6),
                threshold_node("mask", "base", 0.5),
                color_ramp_node("colored", "mask", ["#000000", "#ff6b35", "#ffffff"])
            ]
        )
    }
)
```

### Creating a Mesh Spec
```python
spec(
    asset_id = "rounded-cube-01",
    asset_type = "static_mesh",
    seed = 456,
    outputs = [output("meshes/cube.glb", "glb")],
    recipe = {
        "backend": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe(
            "cube",
            [2.0, 2.0, 2.0],
            [bevel_modifier(0.1, 3), subdivision_modifier(2)]
        )
    }
)
```

### Creating a Music Spec
```python
music_spec(
    asset_id = "test-song-01",
    seed = 789,
    output_path = "music/song.xm",
    format = "xm",
    bpm = 140,
    speed = 6,
    channels = 4,
    instruments = [
        tracker_instrument(
            name = "bass",
            synthesis = instrument_synthesis("sawtooth")
        )
    ],
    patterns = {
        "intro": tracker_pattern(64, notes = {
            "0": [
                pattern_note(0, "C3", 0),
                pattern_note(16, "E3", 0),
                pattern_note(32, "G3", 0)
            ]
        })
    },
    arrangement = [arrangement_entry("intro", 4)]
)
```

---

## See Also

- [Starlark Authoring Guide](starlark-authoring.md) - Guide to writing Starlark specs
- [ARCHITECTURE.md](../ARCHITECTURE.md) - System architecture overview
- [README.md](../README.md) - Project overview and usage
