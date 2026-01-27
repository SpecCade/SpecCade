# SpecCade Starlark Standard Library Reference

The SpecCade stdlib provides helper functions for authoring asset specs in Starlark.
These functions emit canonical IR-compatible dictionaries, reducing boilerplate and
improving the authoring experience.

SSOT: `speccade stdlib dump --format json` (this doc is a convenience view and may lag).

## Design Principles

1. **Flat, explicit parameters** - No hidden defaults; parameters are clear and explicit
2. **Composable** - Functions return dicts that can be modified or combined
3. **Deterministic** - No random, time, or IO functions
4. **Domain-prefixed** - Functions are grouped by domain (audio, texture, mesh)
5. **Minimal** - Core functions covering the most common use cases

## Coordinate System

See [Coordinate System Conventions](conventions/coordinate-system.md).

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
Node-based procedural texture generation and specialized texture recipes.

**Categories:**
- **Node Functions** - Noise, gradients, patterns, operations, and color manipulation
- **Graph Functions** - Texture graph assembly
- **Recipe Functions** - Specialized texture generators (matcaps, decals, trimsheets)

**Key Functions:** `noise_node()`, `gradient_node()`, `color_ramp_node()`, `texture_graph()`, `matcap_v1()`, `material_preset_v1()`, `decal_spec()`, `trimsheet_spec()`

---

### [Mesh Functions](stdlib-mesh.md)
Primitive mesh generation with modifiers.

**Categories:**
- **Primitive Functions** - Basic mesh shapes
- **Modifier Functions** - Bevel, subdivision, decimation, and other modifiers
- **Baking Functions** - Texture map baking settings

**Key Functions:** `mesh_primitive()`, `mesh_recipe()`, `bevel_modifier()`, `subdivision_modifier()`, `baking_settings()`

---

### Character Functions
Skeletal mesh generation with armatures, body parts, and skinning.

**Categories:**
- **Body Part Functions** - Attach mesh primitives to bones
- **Material Functions** - Material slot definitions
- **Skinning Functions** - Weight painting and bone influence settings
- **Spec Functions** - Complete skeletal mesh specs

**Key Functions:** `body_part()`, `material_slot()`, `skinning_config()`, `skeletal_mesh_spec()`

---

### Animation Functions
Skeletal animation with keyframes, bone transforms, and export settings.

**Categories:**
- **Transform Functions** - Bone and IK target transforms
- **Keyframe Functions** - Animation keyframes with bone data
- **Export Functions** - Animation export settings
- **Spec Functions** - Complete skeletal animation specs

**Key Functions:** `bone_transform()`, `animation_keyframe()`, `animation_export_settings()`, `skeletal_animation_spec()`

---

### [Music Functions](stdlib-music.md)
Tracker-style music composition with instruments, patterns, and arrangements.

**Categories:**
- **Instrument Functions** - Tracker instrument synthesis and configuration
- **Pattern Functions** - Note events and pattern definition
- **Song Functions** - Song structure, automation, and options

**Key Functions:** `tracker_instrument()`, `pattern_note()`, `tracker_pattern()`, `tracker_song()`, `music_spec()`

---

## Usage Examples

### Creating an Audio Spec
```starlark
spec(
    asset_id = "laser-blast-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/laser.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    oscillator(880, "sawtooth", 220, "exponential"),
                    envelope(0.01, 0.1, 0.0, 0.1),
                    filter = lowpass(5000, 0.707, 500)
                )
            ],
            "effects": [reverb()]
        }
    }
)
```

### Creating a Texture Spec
```starlark
spec(
    asset_id = "noise-texture-01",
    asset_type = "texture",
    seed = 123,
    outputs = [output("textures/noise.png", "png")],
    recipe = {
        "kind": "texture.procedural_v1",
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
```starlark
spec(
    asset_id = "rounded-cube-01",
    asset_type = "static_mesh",
    seed = 456,
    outputs = [output("meshes/cube.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe(
            "cube",
            [2.0, 2.0, 2.0],
            [bevel_modifier(0.1, 3), subdivision_modifier(2)]
        )
    }
)
```

### Creating a Mesh Spec with Baking
```starlark
spec(
    asset_id = "baked-prop-01",
    asset_type = "static_mesh",
    seed = 789,
    outputs = [output("meshes/prop.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": {
            "base_primitive": "cube",
            "dimensions": [1.0, 1.0, 1.0],
            "modifiers": [bevel_modifier(0.05, 3)],
            "uv_projection": "smart",
            "baking": baking_settings(
                ["normal", "ao"],
                ray_distance = 0.1,
                margin = 16,
                resolution = [1024, 1024]
            )
        }
    }
)
```

### Creating a Character Spec
```starlark
skeletal_mesh_spec(
    asset_id = "humanoid-01",
    seed = 42,
    output_path = "characters/humanoid.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    body_parts = [
        body_part(
            bone = "chest",
            primitive = "cylinder",
            dimensions = [0.3, 0.3, 0.28],
            segments = 8,
            offset = [0, 0, 0.6],
            material_index = 0
        ),
        body_part(
            bone = "head",
            primitive = "sphere",
            dimensions = [0.15, 0.18, 0.15],
            segments = 12,
            material_index = 1
        )
    ],
    material_slots = [
        material_slot(name = "body", base_color = [0.8, 0.6, 0.5, 1.0]),
        material_slot(name = "head", base_color = [0.9, 0.7, 0.6, 1.0])
    ],
    skinning = skinning_config(max_bone_influences = 4),
    constraints = skeletal_constraints(max_triangles = 5000, max_bones = 64)
)
```

### Creating an Animation Spec
```starlark
skeletal_animation_spec(
    asset_id = "walk-cycle-01",
    seed = 42,
    output_path = "animations/walk.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "walk",
    duration_seconds = 1.0,
    fps = 24,
    loop = True,
    keyframes = [
        animation_keyframe(
            time = 0.0,
            bones = {
                "upper_leg_l": bone_transform(rotation = [25.0, 0.0, 0.0]),
                "upper_leg_r": bone_transform(rotation = [-25.0, 0.0, 0.0])
            }
        ),
        animation_keyframe(
            time = 0.5,
            bones = {
                "upper_leg_l": bone_transform(rotation = [-25.0, 0.0, 0.0]),
                "upper_leg_r": bone_transform(rotation = [25.0, 0.0, 0.0])
            }
        )
    ],
    interpolation = "linear",
    export = animation_export_settings(bake_transforms = True)
)
```

### Creating a Music Spec
```starlark
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
