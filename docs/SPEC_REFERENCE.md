# Spec Reference

Complete reference for SpecCade v1 spec schema, recipe kinds, and validation rules.

## Table of Contents

- [Spec Structure](#spec-structure)
- [Contract Fields](#contract-fields)
- [Output Specification](#output-specification)
- [Recipe Kinds](#recipe-kinds)
- [Asset Types](#asset-types)
- [Validation Rules](#validation-rules)
- [Examples](#examples)

## Spec Structure

A SpecCade spec is a JSON document with two logical sections:

1. **Contract** — metadata and output declarations (required for all operations)
2. **Recipe** — backend-specific generation parameters (required for `generate`)

### Minimal Valid Spec

```json
{
  "spec_version": 1,
  "asset_id": "my_sound",
  "asset_type": "audio_sfx",
  "license": "CC0-1.0",
  "seed": 42,
  "outputs": [
    {
      "kind": "audio",
      "format": "wav",
      "path": "my_sound.wav"
    }
  ],
  "recipe": {
    "kind": "audio_sfx.layered_synth_v1",
    "params": {
      "duration_seconds": 1.0,
      "sample_rate": 44100,
      "layers": []
    }
  }
}
```

## Contract Fields

### Required Fields

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `spec_version` | integer | Schema version | Must be `1` |
| `asset_id` | string | Stable identifier | `[a-z][a-z0-9_-]{2,63}` |
| `asset_type` | string | Asset type enum | See [Asset Types](#asset-types) |
| `license` | string | License identifier | SPDX recommended (e.g., `"CC0-1.0"`) |
| `seed` | integer | RNG seed | Range: `0` to `4294967295` (2^32-1) |
| `outputs` | array | Expected artifacts | At least one entry required |

### Optional Fields

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `description` | string | Human-readable description | `""` |
| `style_tags` | array of strings | Semantic tags (e.g., `["retro", "8bit"]`) | `[]` |
| `engine_targets` | array of strings | Target engines: `"godot"`, `"unity"`, `"unreal"` | `[]` |
| `variants` | array | Variant specs for procedural variations | `[]` |

### Validation Rules

- `asset_id` must start with lowercase letter, contain only lowercase letters, digits, hyphens, and underscores, length 3-64 chars
- `seed` must be a non-negative integer less than 2^32
- `outputs` must contain at least one entry with `kind: "audio"`, `"map"`, `"mesh"`, or `"animation"`
- `recipe.kind` must be compatible with `asset_type`

## Output Specification

Each entry in `outputs[]` declares an expected artifact:

```json
{
  "kind": "audio",
  "format": "wav",
  "path": "sounds/laser_blast.wav"
}
```

### Output Fields

| Field | Type | Description | Values |
|-------|------|-------------|--------|
| `kind` | string | Output category | `"audio"`, `"map"`, `"mesh"`, `"animation"`, `"metadata"` |
| `format` | string | File format | `"wav"`, `"ogg"`, `"xm"`, `"it"`, `"png"`, `"glb"`, `"gltf"`, `"json"` |
| `path` | string | Relative output path | Must be safe (see below) |

### Path Constraints

- Must be relative (no leading `/`, `\`, or drive letter)
- Must use forward slashes (`/`) only
- Must not contain `..` segments (path traversal)
- Must end with extension matching `format`
- Must be unique within the spec

### Output Kind by Asset Type

| Asset Type | Valid Output Kinds |
|------------|-------------------|
| `audio_sfx` | `audio`, `metadata` |
| `audio_instrument` | `audio`, `metadata` |
| `music` | `audio`, `metadata` |
| `texture_2d` | `map`, `metadata` |
| `static_mesh` | `mesh`, `metadata` |
| `skeletal_mesh` | `mesh`, `metadata` |
| `skeletal_animation` | `animation`, `metadata` |

## Recipe Kinds

A recipe defines how to generate the asset. The `recipe.kind` must be compatible with `asset_type`.

### Recipe Structure

```json
{
  "recipe": {
    "kind": "audio_sfx.layered_synth_v1",
    "params": {
      "duration_seconds": 0.5,
      "sample_rate": 44100,
      "layers": [...]
    }
  }
}
```

## Asset Types

### Audio SFX

**Asset Type:** `audio_sfx`
**Recipe Kinds:** `audio_sfx.layered_synth_v1`
**Output Formats:** `wav`, `ogg`

One-shot sound effects using layered synthesis.

#### Recipe: `audio_sfx.layered_synth_v1`

| Param | Type | Description | Default |
|-------|------|-------------|---------|
| `duration_seconds` | float | Total duration in seconds | Required |
| `sample_rate` | integer | Sample rate in Hz | `44100` |
| `normalize` | boolean | Normalize to peak amplitude | `true` |
| `peak_db` | float | Target peak level in dB | `-1.0` |
| `layers` | array | Synthesis layers | `[]` |

**Layer Spec:**

| Field | Type | Description |
|-------|------|-------------|
| `synthesis` | object | Synthesis configuration |
| `amplitude` | float | Layer amplitude (0.0-1.0) |
| `envelope` | object | ADSR envelope |
| `pan` | float | Stereo pan (-1.0 to 1.0) |

**Synthesis Types:**

- `fm_synth`: FM synthesis with carrier and modulator
- `karplus_strong`: Plucked string simulation
- `noise_burst`: Filtered noise
- `pitched_body`: Frequency sweep oscillator
- `additive`: Harmonic additive synthesis

**Example:**

```json
{
  "synthesis": {
    "type": "fm_synth",
    "carrier_freq": 440.0,
    "mod_ratio": 2.0,
    "mod_index": 5.0,
    "index_decay": 10.0
  },
  "amplitude": 0.8,
  "envelope": {
    "attack": 0.01,
    "decay": 0.1,
    "sustain": 0.5,
    "release": 0.2
  },
  "pan": 0.0
}
```

---

### Audio Instrument

**Asset Type:** `audio_instrument`
**Recipe Kinds:** `audio_instrument.synth_patch_v1`
**Output Formats:** `wav`

Single-note instrument samples for tracker modules.

#### Recipe: `audio_instrument.synth_patch_v1`

| Param | Type | Description | Default |
|-------|------|-------------|---------|
| `note` | string or integer | MIDI note or frequency | `"C4"` |
| `duration_seconds` | float | Sample duration | `1.0` |
| `sample_rate` | integer | Sample rate in Hz | `44100` |
| `synthesis` | object | Synthesis configuration | Required |
| `envelope` | object | ADSR envelope | Required |

**Example:**

```json
{
  "recipe": {
    "kind": "audio_instrument.synth_patch_v1",
    "params": {
      "note": "C4",
      "duration_seconds": 1.0,
      "sample_rate": 44100,
      "synthesis": {
        "type": "pulse",
        "duty_cycle": 0.5
      },
      "envelope": {
        "attack": 0.01,
        "decay": 0.1,
        "sustain": 0.7,
        "release": 0.2
      }
    }
  }
}
```

---

### Music

**Asset Type:** `music`
**Recipe Kinds:** `music.tracker_song_v1`
**Output Formats:** `xm`, `it`

Tracker module songs with instruments, patterns, and arrangement.

#### Recipe: `music.tracker_song_v1`

| Param | Type | Description | Default |
|-------|------|-------------|---------|
| `format` | string | Module format (`"xm"` or `"it"`) | Required |
| `bpm` | integer | Tempo in beats per minute | `125` |
| `speed` | integer | Tracker speed (ticks per row) | `6` |
| `channels` | integer | Number of channels | `4` |
| `loop` | boolean | Loop song | `true` |
| `instruments` | array | Instrument definitions | `[]` |
| `patterns` | object | Pattern definitions | `{}` |
| `arrangement` | array | Pattern order | `[]` |

**Instrument Spec:**

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Instrument name |
| `synthesis` | object | Synthesis config (same as audio layers) |
| `envelope` | object | ADSR envelope |

**Pattern Spec:**

| Field | Type | Description |
|-------|------|-------------|
| `rows` | integer | Number of rows (typically 64) |
| `data` | array | Note events |

**Note Event:**

| Field | Type | Description |
|-------|------|-------------|
| `row` | integer | Row index (0-based) |
| `channel` | integer | Channel index (0-based) |
| `note` | string | Note name (e.g., `"C4"`, `"OFF"`) |
| `instrument` | integer | Instrument index |
| `volume` | integer | Volume (0-64) |
| `effect` | string | Effect code (optional) |
| `effect_param` | integer | Effect parameter (optional) |

**Arrangement Entry:**

| Field | Type | Description |
|-------|------|-------------|
| `pattern` | string | Pattern name |
| `repeat` | integer | Repeat count |

**Example:**

```json
{
  "recipe": {
    "kind": "music.tracker_song_v1",
    "params": {
      "format": "xm",
      "bpm": 140,
      "speed": 6,
      "channels": 4,
      "loop": true,
      "instruments": [
        {
          "name": "lead_square",
          "synthesis": {
            "type": "pulse",
            "duty_cycle": 0.5
          },
          "envelope": {
            "attack": 0.01,
            "decay": 0.1,
            "sustain": 0.6,
            "release": 0.2
          }
        }
      ],
      "patterns": {
        "intro": {
          "rows": 64,
          "data": [
            { "row": 0, "channel": 0, "note": "C4", "instrument": 0, "volume": 64 },
            { "row": 16, "channel": 0, "note": "E4", "instrument": 0, "volume": 64 }
          ]
        }
      },
      "arrangement": [
        { "pattern": "intro", "repeat": 2 }
      ]
    }
  }
}
```

---

### Texture 2D

**Asset Type:** `texture_2d`
**Recipe Kinds:** `texture_2d.material_maps_v1`, `texture_2d.normal_map_v1`
**Output Formats:** `png`

2D texture maps with coherent multi-layer generation.

#### Recipe: `texture_2d.material_maps_v1`

Generates coherent PBR material maps (albedo, roughness, metallic, normal, AO, emissive).

| Param | Type | Description | Default |
|-------|------|-------------|---------|
| `resolution` | array | Width and height `[w, h]` | `[1024, 1024]` |
| `tileable` | boolean | Generate tileable texture | `false` |
| `maps` | array | Maps to generate | `["albedo"]` |
| `base_material` | object | Base material properties | Required |
| `layers` | array | Procedural layers | `[]` |

**Base Material:**

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | Material type: `"metal"`, `"dielectric"`, `"plastic"` |
| `base_color` | array | RGB color `[r, g, b]` (0.0-1.0) |
| `roughness_range` | array | Min/max roughness `[min, max]` |
| `metallic` | float | Metallic value (0.0-1.0) |

**Layer Types:**

- `noise_pattern`: Noise-based variation
- `scratches`: Scratch/wear effects
- `edge_wear`: Edge damage
- `grunge`: Dirt/grime overlay

**Example:**

```json
{
  "recipe": {
    "kind": "texture_2d.material_maps_v1",
    "params": {
      "resolution": [1024, 1024],
      "tileable": true,
      "maps": ["albedo", "roughness", "metallic", "normal"],
      "base_material": {
        "type": "metal",
        "base_color": [0.6, 0.6, 0.65],
        "roughness_range": [0.3, 0.6],
        "metallic": 1.0
      },
      "layers": [
        {
          "type": "noise_pattern",
          "noise": {
            "algorithm": "simplex",
            "scale": 8.0,
            "octaves": 4
          },
          "affects": ["roughness", "normal"],
          "strength": 0.3
        },
        {
          "type": "scratches",
          "density": 0.15,
          "length_range": [0.1, 0.4],
          "width": 0.002,
          "affects": ["albedo", "roughness"],
          "strength": 0.5
        }
      ]
    }
  }
}
```

#### Recipe: `texture_2d.normal_map_v1`

Generates normal maps from parameterized patterns.

| Param | Type | Description | Default |
|-------|------|-------------|---------|
| `resolution` | array | Width and height `[w, h]` | `[1024, 1024]` |
| `tileable` | boolean | Generate tileable texture | `false` |
| `pattern` | string | Pattern type: `"bricks"`, `"tiles"`, `"hexagons"` | Required |
| `depth` | float | Normal depth (0.0-1.0) | `0.5` |
| `pattern_params` | object | Pattern-specific params | `{}` |

---

### Static Mesh

**Asset Type:** `static_mesh`
**Recipe Kinds:** `static_mesh.blender_primitives_v1`
**Output Formats:** `glb`, `gltf`

Non-skinned 3D meshes generated from primitives and modifiers.

#### Recipe: `static_mesh.blender_primitives_v1`

| Param | Type | Description | Default |
|-------|------|-------------|---------|
| `base_primitive` | string | Primitive type: `"cube"`, `"sphere"`, `"cylinder"`, `"cone"`, `"torus"` | Required |
| `dimensions` | array | XYZ dimensions `[x, y, z]` | `[1.0, 1.0, 1.0]` |
| `modifiers` | array | Blender modifiers | `[]` |
| `uv_projection` | string | UV method: `"box"`, `"sphere"`, `"cylinder"`, `"smart"` | `"smart"` |
| `material_slots` | array | Material definitions | `[]` |
| `export` | object | Export settings | See below |
| `constraints` | object | Validation constraints | `{}` |

**Modifier Types:**

- `bevel`: Bevel edges
- `subdivision`: Subdivision surface
- `edge_split`: Split sharp edges
- `solidify`: Add thickness
- `array`: Array modifier

**Export Settings:**

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `apply_modifiers` | boolean | Apply modifiers before export | `true` |
| `triangulate` | boolean | Triangulate faces | `true` |
| `include_normals` | boolean | Export normals | `true` |
| `include_uvs` | boolean | Export UVs | `true` |

**Example:**

```json
{
  "recipe": {
    "kind": "static_mesh.blender_primitives_v1",
    "params": {
      "base_primitive": "cube",
      "dimensions": [1.0, 1.0, 1.0],
      "modifiers": [
        {
          "type": "bevel",
          "width": 0.02,
          "segments": 2
        }
      ],
      "uv_projection": "box",
      "material_slots": [
        {
          "name": "main",
          "base_color": [0.8, 0.8, 0.8, 1.0]
        }
      ],
      "export": {
        "apply_modifiers": true,
        "triangulate": true
      }
    }
  }
}
```

---

### Skeletal Mesh

**Asset Type:** `skeletal_mesh`
**Recipe Kinds:** `skeletal_mesh.blender_rigged_mesh_v1`
**Output Formats:** `glb`, `gltf`

Skinned meshes with skeleton and automatic weights.

#### Recipe: `skeletal_mesh.blender_rigged_mesh_v1`

| Param | Type | Description | Default |
|-------|------|-------------|---------|
| `skeleton_preset` | string | Skeleton type: `"humanoid_basic_v1"` | Required |
| `body_parts` | array | Per-bone mesh definitions | `[]` |
| `material_slots` | array | Material definitions | `[]` |
| `skinning` | object | Skinning settings | See below |
| `export` | object | Export settings | See below |
| `constraints` | object | Validation constraints | `{}` |

**Body Part Spec:**

| Field | Type | Description |
|-------|------|-------------|
| `bone` | string | Target bone name |
| `mesh` | object | Mesh definition (primitive + dimensions) |

**Skinning Settings:**

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `max_bone_influences` | integer | Max bones per vertex | `4` |
| `auto_weights` | boolean | Auto-compute weights | `true` |

**Example:**

```json
{
  "recipe": {
    "kind": "skeletal_mesh.blender_rigged_mesh_v1",
    "params": {
      "skeleton_preset": "humanoid_basic_v1",
      "body_parts": [
        {
          "bone": "spine",
          "mesh": {
            "primitive": "cylinder",
            "dimensions": [0.3, 0.5, 0.3],
            "segments": 8
          }
        },
        {
          "bone": "head",
          "mesh": {
            "primitive": "cube",
            "dimensions": [0.25, 0.3, 0.25]
          }
        }
      ],
      "material_slots": [
        {
          "name": "body",
          "base_color": [0.8, 0.7, 0.6, 1.0]
        }
      ],
      "skinning": {
        "max_bone_influences": 4,
        "auto_weights": true
      },
      "export": {
        "include_armature": true,
        "triangulate": true
      }
    }
  }
}
```

---

### Skeletal Animation

**Asset Type:** `skeletal_animation`
**Recipe Kinds:** `skeletal_animation.blender_clip_v1`
**Output Formats:** `glb`, `gltf`

Animation clips targeting a skeleton.

#### Recipe: `skeletal_animation.blender_clip_v1`

| Param | Type | Description | Default |
|-------|------|-------------|---------|
| `skeleton_preset` | string | Skeleton type: `"humanoid_basic_v1"` | Required |
| `clip_name` | string | Animation clip name | Required |
| `duration_seconds` | float | Clip duration | Required |
| `fps` | integer | Frames per second | `30` |
| `loop` | boolean | Loop animation | `true` |
| `keyframes` | array | Keyframe definitions | `[]` |
| `interpolation` | string | Interpolation: `"linear"`, `"bezier"`, `"constant"` | `"linear"` |
| `export` | object | Export settings | See below |

**Keyframe Spec:**

| Field | Type | Description |
|-------|------|-------------|
| `time` | float | Time in seconds |
| `bones` | object | Bone transforms (keyed by bone name) |

**Bone Transform:**

| Field | Type | Description |
|-------|------|-------------|
| `position` | array | XYZ translation `[x, y, z]` |
| `rotation` | array | Euler angles `[x, y, z]` (degrees) |
| `scale` | array | XYZ scale `[x, y, z]` |

**Example:**

```json
{
  "recipe": {
    "kind": "skeletal_animation.blender_clip_v1",
    "params": {
      "skeleton_preset": "humanoid_basic_v1",
      "clip_name": "walk",
      "duration_seconds": 1.0,
      "fps": 30,
      "loop": true,
      "keyframes": [
        {
          "time": 0.0,
          "bones": {
            "upper_leg_l": { "rotation": [15, 0, 0] },
            "upper_leg_r": { "rotation": [-15, 0, 0] }
          }
        },
        {
          "time": 0.5,
          "bones": {
            "upper_leg_l": { "rotation": [-15, 0, 0] },
            "upper_leg_r": { "rotation": [15, 0, 0] }
          }
        },
        {
          "time": 1.0,
          "bones": {
            "upper_leg_l": { "rotation": [15, 0, 0] },
            "upper_leg_r": { "rotation": [-15, 0, 0] }
          }
        }
      ],
      "interpolation": "linear",
      "export": {
        "bake_transforms": true,
        "optimize_keyframes": false
      }
    }
  }
}
```

## Validation Rules

### Contract Validation

| Rule | Error Code | Description |
|------|------------|-------------|
| `spec_version` must equal `1` | `E001` | Unsupported spec version |
| `asset_id` must match `[a-z][a-z0-9_-]{2,63}` | `E002` | Invalid asset_id format |
| `asset_type` must be known | `E003` | Unknown asset type |
| `seed` must be in range `0..2^32-1` | `E004` | Seed out of range |
| `outputs` must have at least one entry | `E005` | No outputs declared |
| `outputs[].path` must be unique | `E007` | Duplicate output path |
| `outputs[].path` must be safe (relative, no `..`) | `E008` | Unsafe output path |
| `outputs[].path` extension must match format | `E009` | Path/format mismatch |

### Recipe Validation

| Rule | Error Code | Description |
|------|------------|-------------|
| `recipe` required for `generate` | `E010` | Missing recipe |
| `recipe.kind` must match `asset_type` | `E011` | Recipe/asset type mismatch |
| `recipe.params` must be valid | `E012` | Invalid recipe params |

### Warnings

| Rule | Warning Code | Description |
|------|--------------|-------------|
| `license` is empty | `W001` | Missing license information |
| `description` is empty | `W002` | Missing description |
| Large seed near max value | `W003` | Seed close to overflow |
| Unused recipe params | `W004` | Params not used by backend |

## Examples

See [RFC-0001](rfcs/RFC-0001-canonical-spec.md) for complete examples of all asset types.

Quick links to golden corpus specs:

- Audio SFX: `golden/speccade/specs/audio_sfx/`
- Instruments: `golden/speccade/specs/audio_instrument/`
- Music: `golden/speccade/specs/music/`
- Textures: `golden/speccade/specs/texture_2d/`
- Meshes: `golden/speccade/specs/static_mesh/`
- Characters: `golden/speccade/specs/skeletal_mesh/`
- Animations: `golden/speccade/specs/skeletal_animation/`

## Schema Evolution

When breaking changes are required, `spec_version` will increment. SpecCade maintains backward compatibility:

- v1 specs will always validate and generate on v1+ tooling
- Migration tools provided for major version bumps
- Minimum 6-month deprecation notice

See [RFC-0001](rfcs/RFC-0001-canonical-spec.md) for the complete specification and design rationale.
