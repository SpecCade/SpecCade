# SpecCade Starlark Standard Library Reference

> **SSOT:** `speccade stdlib dump --format json` (this doc is a convenience overview).

The stdlib provides Starlark helper functions that emit IR-compatible dicts for asset specs.

## Design Principles

1. **Flat, explicit parameters** — no hidden defaults
2. **Composable** — functions return dicts that can be modified or combined
3. **Deterministic** — no random, time, or IO functions
4. **Domain-prefixed** — grouped by domain (audio, texture, mesh, music)

## Coordinate System

See [Coordinate System Conventions](conventions/coordinate-system.md).

## Function Categories

### [Core Functions](stdlib-core.md)
`spec()`, `output()` — create asset specifications.

### [Audio Functions](stdlib-audio.md)
Synthesis, filters, effects, modulation, layers.

### [Texture Functions](stdlib-texture.md)
Node-based procedural textures, trimsheets, decals, splat sets, matcaps, material presets.

### [Mesh Functions](stdlib-mesh.md)
Primitives and modifiers for static meshes.

### Character Functions
`body_part()`, `material_slot()`, `skinning_config()`, `skeletal_mesh_spec()`

### Animation Functions
`bone_transform()`, `animation_keyframe()`, `animation_export_settings()`, `skeletal_animation_spec()`

### [Music Functions](stdlib-music.md)
Tracker-style instruments, patterns, songs, cue templates.

## Quick Examples

### Audio
```starlark
spec(
    asset_id = "laser-01", asset_type = "audio", seed = 42,
    outputs = [output("sounds/laser.wav", "wav")],
    recipe = {"kind": "audio_v1", "params": {
        "duration_seconds": 0.5, "sample_rate": 44100,
        "layers": [audio_layer(oscillator(880, "sawtooth", 220, "exponential"),
                               envelope(0.01, 0.1, 0.0, 0.1),
                               filter = lowpass(5000, 0.707, 500))],
        "effects": [reverb()]
    }}
)
```

### Texture
```starlark
spec(
    asset_id = "noise-01", asset_type = "texture", seed = 123,
    outputs = [output("textures/noise.png", "png")],
    recipe = {"kind": "texture.procedural_v1", "params": texture_graph(
        [256, 256], [noise_node("b", "perlin", 0.05, 6),
                     color_ramp_node("c", "b", ["#000000", "#ff6b35", "#ffffff"])]
    )}
)
```

### Mesh
```starlark
spec(
    asset_id = "cube-01", asset_type = "static_mesh", seed = 456,
    outputs = [output("meshes/cube.glb", "glb")],
    recipe = {"kind": "static_mesh.blender_primitives_v1", "params": mesh_recipe(
        "cube", [2.0, 2.0, 2.0], [bevel_modifier(0.1, 3), subdivision_modifier(2)]
    )}
)
```

### Music
```starlark
music_spec(
    asset_id = "song-01", seed = 789, output_path = "music/song.xm",
    format = "xm", bpm = 140, speed = 6, channels = 4,
    instruments = [tracker_instrument(name = "bass", synthesis = instrument_synthesis("sawtooth"))],
    patterns = {"intro": tracker_pattern(64, notes = {"0": [pattern_note(0, "C3", 0)]})},
    arrangement = [arrangement_entry("intro", 4)]
)
```

## See Also

- [Starlark Authoring Guide](starlark-authoring.md)
- [ARCHITECTURE.md](../ARCHITECTURE.md)
