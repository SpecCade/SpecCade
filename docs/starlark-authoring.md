# Starlark Authoring Guide for SpecCade

This guide covers how to author asset specs using Starlark (`.star` files) with
the SpecCade standard library.

## LLM Convention

> **Starlark is the canonical output format for LLM-generated specs.** Prefer stdlib
> functions over raw JSON for validation, composability, and schema resilience.
> See also the [Claude plugin skill](../claude-plugin/skills/speccade-authoring/SKILL.md).

## Overview

SpecCade supports two input formats:
- **JSON** (`.json`) - Static spec files
- **Starlark** (`.star`) - Programmable specs with variables, functions, and stdlib helpers

Starlark provides a Python-like syntax for creating specs with less boilerplate,
better abstraction, and improved ergonomics.

## Basic Structure

A Starlark spec is a `.star` file that evaluates to a dictionary matching the
SpecCade IR format. The simplest spec looks like this:

```starlark
# Using stdlib helpers (recommended)
spec(
    asset_id = "my-asset-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/my-sound.wav", "wav")]
)
```

Or using raw dictionaries:

```starlark
# Equivalent raw dictionary
{
    "spec_version": 1,
    "asset_id": "my-asset-01",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [
        {
            "kind": "primary",
            "format": "wav",
            "path": "sounds/my-sound.wav"
        }
    ]
}
```

## Using the Standard Library

The stdlib provides helper functions that create correctly-structured dictionaries.
These are preferred over raw dictionaries for:

1. **Reduced boilerplate** - Default values are handled automatically
2. **Validation** - Errors are caught with clear error messages
3. **Type safety** - Parameters are validated at evaluation time
4. **Composability** - Functions can be combined naturally

### Example: Audio with Synthesis

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
                    synthesis = oscillator(440, "sine", 220, "exponential"),
                    envelope = envelope(0.01, 0.1, 0.0, 0.2),
                    volume = 0.8,
                    filter = lowpass(2000, 0.707, 500)
                )
            ],
            "effects": [reverb(0.3, 0.2)]
        }
    }
)
```

### Example: Texture with Node Graph

```starlark
spec(
    asset_id = "noise-pattern-01",
    asset_type = "texture",
    seed = 42,
    outputs = [output("textures/pattern.png", "png")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [128, 128],
            [
                noise_node("n", "simplex", 0.1, 4),
                color_ramp_node("colored", "n", ["#000000", "#0066ff", "#ffffff"])
            ]
        )
    }
)
```

### Example: Mesh with Modifiers

```starlark
spec(
    asset_id = "smooth-cube-01",
    asset_type = "static_mesh",
    seed = 42,
    outputs = [output("meshes/cube.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe(
            "cube",
            [1.0, 1.0, 1.0],
            [
                bevel_modifier(0.02, 2),
                subdivision_modifier(2)
            ]
        )
    }
)
```

## Language Features

### Variables

```starlark
base_freq = 440
duration = 0.5

spec(
    asset_id = "variable-example-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/test.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": duration,
            "sample_rate": 44100,
            "layers": [
                audio_layer(oscillator(base_freq))
            ]
        }
    }
)
```

### Functions

```starlark
def make_output(name):
    return output("sounds/" + name + ".wav", "wav")

def make_layer(freq, volume):
    return audio_layer(
        oscillator(freq, "sine"),
        envelope(0.01, 0.1, 0.5, 0.2),
        volume
    )

spec(
    asset_id = "function-example-01",
    asset_type = "audio",
    seed = 42,
    outputs = [make_output("test")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [
                make_layer(440, 0.5),
                make_layer(880, 0.3)
            ]
        }
    }
)
```

### List Comprehensions

```starlark
frequencies = [220, 440, 660, 880]

layers = [
    audio_layer(oscillator(freq), volume = 0.8 / (i + 1))
    for i, freq in enumerate(frequencies)
]

spec(
    asset_id = "comprehension-example-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/chord.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 2.0,
            "sample_rate": 44100,
            "layers": layers
        }
    }
)
```

## Error Handling

Stdlib functions validate their inputs and return clear error messages with
stable S-series error codes:

| Code | Description |
|------|-------------|
| S101 | Missing required argument |
| S102 | Type mismatch (e.g., expected float, got string) |
| S103 | Value out of range (e.g., negative frequency) |
| S104 | Invalid enum value (e.g., unknown waveform type) |

**Example error:**

```
S103: oscillator(): 'frequency' must be positive, got -440
```

```
S104: oscillator(): 'waveform' must be one of: sine, square, sawtooth, triangle, pulse. Did you mean 'sine'?
```

## Best Practices

### 1. Use the stdlib helpers

Prefer `spec()`, `output()`, and domain-specific helpers over raw dictionaries.
They provide validation and sensible defaults.

### 2. Extract common patterns into functions

```starlark
def sfx_spec(name, layers, duration = 0.5):
    return spec(
        asset_id = "sfx-" + name + "-01",
        asset_type = "audio",
        seed = 42,
        outputs = [output("sounds/" + name + ".wav", "wav")],
        recipe = {
            "kind": "audio_v1",
            "params": {
                "duration_seconds": duration,
                "sample_rate": 44100,
                "layers": layers
            }
        }
    )
```

### 3. Use meaningful variable names

```starlark
# Good
base_frequency = 440
attack_time = 0.01
filter_cutoff = 2000

# Less clear
f = 440
a = 0.01
c = 2000
```

### 4. Comment complex recipes

```starlark
# Layered laser sound:
# - Primary body: swept sine wave
# - Attack transient: filtered noise burst
# - Effect: slight reverb for space
```

## Placeholder Humanoid

A common LLM task is generating a humanoid character with blank materials as a
starting point for texturing. Use `skeletal_mesh_spec()` with a single white
material and `skeleton_preset = "humanoid_basic_v1"`:

```starlark
skeletal_mesh_spec(
    asset_id = "my-humanoid-01",
    seed = 42,
    output_path = "characters/my_humanoid.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    description = "Blank humanoid placeholder",
    body_parts = [
        body_part(bone = "spine",       primitive = "cylinder", dimensions = [0.25, 0.4, 0.25],  segments = 8,  offset = [0,0,0.3],  material_index = 0),
        body_part(bone = "chest",       primitive = "cylinder", dimensions = [0.3, 0.3, 0.28],   segments = 8,  offset = [0,0,0.6],  material_index = 0),
        body_part(bone = "head",        primitive = "sphere",   dimensions = [0.15, 0.18, 0.15], segments = 12, offset = [0,0,0.95], material_index = 0),
        body_part(bone = "upper_arm_l", primitive = "cylinder", dimensions = [0.06, 0.25, 0.06], segments = 6,  rotation = [0,0,90],  material_index = 0),
        body_part(bone = "upper_arm_r", primitive = "cylinder", dimensions = [0.06, 0.25, 0.06], segments = 6,  rotation = [0,0,-90], material_index = 0),
        body_part(bone = "upper_leg_l", primitive = "cylinder", dimensions = [0.08, 0.35, 0.08], segments = 6,  rotation = [180,0,0], material_index = 0),
        body_part(bone = "upper_leg_r", primitive = "cylinder", dimensions = [0.08, 0.35, 0.08], segments = 6,  rotation = [180,0,0], material_index = 0),
    ],
    material_slots = [
        material_slot(name = "blank_white", base_color = [1.0, 1.0, 1.0, 1.0]),
    ],
    skinning = skinning_config(max_bone_influences = 4, auto_weights = True),
    export = skeletal_export_settings(triangulate = True, include_skin_weights = True),
    constraints = skeletal_constraints(max_triangles = 5000, max_bones = 64, max_materials = 4),
    texturing = skeletal_texturing(uv_mode = "cylinder_project")
)
```

See [golden/starlark/character_humanoid_blank.star](../golden/starlark/character_humanoid_blank.star) for the full example.

## Limitations

1. **No `load()` statements** - For security, external file loading is disabled
2. **No recursion** - Recursive functions are disabled to prevent infinite loops
3. **Timeout** - Evaluation has a configurable timeout (default: 30 seconds)
4. **Deterministic** - No random, time, or IO functions are available

## See Also

- [stdlib-reference.md](./stdlib-reference.md) - Complete function reference
- [golden/starlark/](../golden/starlark/) - Example specs
