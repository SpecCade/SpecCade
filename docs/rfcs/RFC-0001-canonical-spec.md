# RFC-0001: Canonical Spec Architecture

- **Status:** Implemented
- **Author:** SpecCade Team
- **Created:** 2026-01-10
- **Target Version:** SpecCade v1.0
- **Last reviewed:** 2026-01-12

## Summary

This RFC defines the canonical spec format for SpecCade, a declarative asset generation system that transforms JSON specifications into game-ready assets (WAV, XM/IT, PNG, GLB). The spec format establishes a stable contract between spec authors and asset backends, enabling deterministic, reproducible asset generation.

**Design principles:**

- **Declarative:** Specs describe *what* to produce, not *how* to produce it
- **Deterministic:** Same spec + seed = identical output (within documented tolerances)
- **Portable:** JSON specs are version-control friendly and tool-agnostic
- **Safe:** Pure data specs with no code execution

---

## 1. Spec Structure

A SpecCade spec is a JSON document with two logical sections:

1. **Contract** — metadata and output declarations (required for all operations)
2. **Recipe** — backend-specific generation parameters (required for `generate`)

### 1.1 Contract Fields (Required)

| Field | Type | Description |
|-------|------|-------------|
| `spec_version` | integer | Schema version; must be `1` for this RFC |
| `asset_id` | string | Stable identifier; format: `[a-z][a-z0-9_-]{2,63}` |
| `asset_type` | enum | One of the defined asset types (see Section 2) |
| `license` | string | License identifier (SPDX recommended, e.g., `"CC0-1.0"`) |
| `seed` | integer | RNG seed; range `0` to `2^32-1` (4294967295) |
| `outputs` | array | Expected artifacts (see Section 1.3) |

### 1.2 Contract Fields (Optional)

| Field | Type | Description |
|-------|------|-------------|
| `description` | string | Human-readable description of the asset |
| `style_tags` | array of strings | Semantic tags for filtering/search (e.g., `["retro", "8bit"]`) |
| `engine_targets` | array of enums | Target engines: `"godot"`, `"unity"`, `"unreal"` |
| `variants` | array | Variant specs for procedural variations (see Section 1.4) |

### 1.3 Output Specification

Each entry in `outputs[]` declares an expected artifact:

```json
{
  "kind": "primary",
  "format": "wav",
  "path": "sounds/laser_blast.wav"
}
```

| Field | Type | Values |
|-------|------|--------|
| `kind` | enum | `"primary"`, `"metadata"`, `"preview"` |
| `format` | enum | `"wav"`, `"xm"`, `"it"`, `"png"`, `"glb"`, `"gltf"`, `"json"` |
| `path` | string | Relative path under output root |

**Path constraints:**

- Must be relative (no leading `/` or drive letter)
- Must use forward slashes (`/`) only
- Must not contain `..` segments
- Must end with extension matching `format`
- Must be unique within the spec

**At least one output with `kind: "primary"` is required.**

**Reserved kinds:** `metadata` and `preview` are reserved for future use and are **invalid** in v1.
Validation rejects them; use the `${asset_id}.report.json` sibling file instead.

### 1.4 Variant Specification

Variants are a contract field intended to allow a single spec to produce multiple related outputs with derived seeds.

**Implementation note:** The CLI can expand `variants` during generation with `speccade generate --expand-variants`.

```json
{
  "variants": [
    { "variant_id": "soft", "seed_offset": 0 },
    { "variant_id": "hard", "seed_offset": 100 }
  ]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `variant_id` | string | Identifier for the variant |
| `seed_offset` | integer | Offset added to base seed for this variant |

### 1.5 Recipe Structure

The recipe defines how to generate the asset:

```json
{
  "recipe": {
    "kind": "audio_v1",
    "params": {
      "duration_seconds": 0.5,
      "sample_rate": 44100,
      "layers": [...]
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `recipe.kind` | string | Recipe kind identifier (see Section 2) |
| `recipe.params` | object | Backend-specific parameters |

**Constraint:** `recipe.kind` must be compatible with `asset_type` based on the kind prefix (e.g. `texture.*` requires `asset_type: "texture"`, and `audio_v1` requires `asset_type: "audio"`).

---

## 2. Asset Types and Recipe Kinds

### 2.1 Asset Types (v1)

| Asset Type | Description | Primary Formats |
|------------|-------------|-----------------|
| `audio` | One-shot sound effects and pitched samples | WAV |
| `music` | Tracker modules | XM, IT |
| `texture` | 2D texture maps | PNG |
| `static_mesh` | Non-skinned 3D meshes | GLB |
| `skeletal_mesh` | Skinned meshes with skeleton | GLB |
| `skeletal_animation` | Animation clips | GLB |

**Planned (post-v1):**

| Asset Type | Description | Primary Formats |
|------------|-------------|-----------------|
| `sprite_2d` | Sprite sheets | PNG |

### 2.2 Recipe Kinds (v1)

| Recipe Kind | Asset Type | Backend | Description |
|-------------|------------|---------|-------------|
| `audio_v1` | `audio` | Rust | Unified layered synthesis (SFX + samples) |
| `music.tracker_song_v1` | `music` | Rust | XM/IT tracker module generation |
| `texture.procedural_v1` | `texture` | Rust | Unified procedural DAG (named outputs) |
| `static_mesh.blender_primitives_v1` | `static_mesh` | Blender | Primitive-based mesh generation |
| `skeletal_mesh.blender_rigged_mesh_v1` | `skeletal_mesh` | Blender | Rigged character/prop meshes |
| `skeletal_animation.blender_clip_v1` | `skeletal_animation` | Blender | Animation clips for armatures |

---

## 3. Determinism Policy

### 3.1 Overview

SpecCade guarantees deterministic output within documented tolerances. Determinism requirements differ by backend tier:

| Tier | Backends | Guarantee |
|------|----------|-----------|
| **Tier 1** | Rust (audio, music, texture) | Byte-identical output per `(target_triple, backend_version)` |
| **Tier 2** | Blender (mesh, animation) | Metric validation only (not byte-identical) |

### 3.2 Tier 1: Deterministic Hash Guarantee

For Tier 1 backends, the same spec must produce byte-identical output when:

- Target triple matches (e.g., `x86_64-pc-windows-msvc`)
- Backend version matches (e.g., `speccade-backend-audio v0.1.0`)
- Spec is identical (same `spec_hash`)

**Cross-platform determinism is NOT guaranteed** unless explicitly documented. Floating-point operations may differ across CPU architectures.

### 3.3 Tier 2: Metric Validation

For Tier 2 backends (Blender-based), determinism is validated via metrics rather than file hashes:

| Metric | Tolerance |
|--------|-----------|
| Triangle count | Exact match (0% variance) |
| Bounding box (min/max XYZ) | +/- 0.001 units |
| UV island count | Exact match |
| Bone count | Exact match |
| Material slot count | Exact match |
| Animation frame count | Exact match |
| Animation duration | +/- 0.001 seconds |

### 3.4 RNG Algorithm

All Rust backends MUST use **PCG32** (Permuted Congruential Generator, 32-bit output) for random number generation.

**Seed derivation:**

```
base_seed = spec.seed

# Per-layer seed (for audio layers, texture layers, etc.)
layer_seed = truncate_u32(BLAKE3(base_seed.to_le_bytes() || layer_index.to_le_bytes()))

# Per-variant seed
variant_seed = truncate_u32(BLAKE3(base_seed.to_le_bytes() || variant_id.as_bytes()))
```

### 3.5 Spec Hashing

Spec hashes are computed for caching, reporting, and verification:

```
spec_hash = hex(BLAKE3(JCS(spec_json)))
```

Where JCS is JSON Canonicalization Scheme per [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785).

### 3.6 Artifact Comparison

| Format | Comparison Method |
|--------|-------------------|
| WAV | Hash PCM sample data only (ignore RIFF header metadata fields) |
| XM/IT | Hash full file bytes |
| PNG | Hash full file bytes (requires deterministic encoder settings) |
| GLB | Metric validation (see Section 3.3) |

---

## 4. Report Schema

Every `speccade generate` or `speccade validate` invocation produces a report:

### 4.1 Report Structure

```json
{
  "report_version": 1,
  "spec_hash": "a1b2c3d4e5f6...",
  "ok": true,
  "errors": [],
  "warnings": [],
  "outputs": [...],
  "duration_ms": 1234,
  "backend_version": "speccade-backend-audio v0.1.0",
  "target_triple": "x86_64-pc-windows-msvc"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `report_version` | integer | Report schema version; must be `1` |
| `spec_hash` | string | Hex-encoded BLAKE3 hash of canonicalized spec |
| `ok` | boolean | `true` if generation succeeded without errors |
| `errors` | array | Error entries (see below) |
| `warnings` | array | Warning entries (see below) |
| `outputs` | array | Output result entries (see below) |
| `duration_ms` | integer | Total execution time in milliseconds |
| `backend_version` | string | Backend identifier and version |
| `target_triple` | string | Rust target triple (e.g., `x86_64-unknown-linux-gnu`) |

### 4.2 Error/Warning Entry

```json
{
  "code": "E001",
  "message": "Invalid asset_id format",
  "path": "asset_id"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `code` | string | Error/warning code (e.g., `"E001"`, `"W001"`) |
| `message` | string | Human-readable description |
| `path` | string or null | JSON path to the problematic field |

### 4.3 Output Result Entry

```json
{
  "kind": "primary",
  "format": "wav",
  "path": "sounds/laser_blast.wav",
  "hash": "deadbeef...",
  "metrics": null
}
```

| Field | Type | Description |
|-------|------|-------------|
| `kind` | enum | Output kind from spec |
| `format` | enum | Output format from spec |
| `path` | string | Relative path where artifact was written |
| `hash` | string or null | Hex-encoded BLAKE3 hash (Tier 1 only) |
| `metrics` | object or null | Validation metrics (Tier 2 only) |

### 4.4 Metrics Object (Tier 2)

```json
{
  "triangle_count": 1234,
  "bounding_box": {
    "min": [-1.0, 0.0, -1.0],
    "max": [1.0, 2.0, 1.0]
  },
  "uv_island_count": 4,
  "bone_count": 22,
  "material_slot_count": 2,
  "max_bone_influences": 4,
  "animation_frame_count": 60,
  "animation_duration_seconds": 2.0
}
```

---

## 5. Deprecation Timeline

| Version | Milestone | Description |
|---------|-----------|-------------|
| **v0.1** | MVP | `validate` + `generate` for at least 1 asset type |
| **v0.2** | Full Suite | All legacy categories covered (audio, music, texture, mesh, animation) |
| **v0.3** | Migration | `speccade migrate` tool for legacy `.spec.py` conversion |
| **v1.0** | Stable Contract | Spec v1 frozen; breaking changes require spec_version bump |
| **v1.1+** | Extensions | New recipe kinds, asset types; backwards compatible |

### 5.1 Legacy Migration

The `speccade migrate` command (v0.3+) converts legacy `.spec.py` files to canonical JSON:

| Legacy Category | SpecCade Asset Type | Recipe Kind |
|-----------------|---------------------|-------------|
| `sounds/` | `audio` | `audio_v1` |
| `instruments/` | `audio` | `audio_v1` |
| `music/` | `music` | `music.tracker_song_v1` |
| `textures/` | `texture` | `texture.procedural_v1` |
| `normals/` | `texture` | `texture.procedural_v1` |
| `meshes/` | `static_mesh` | `static_mesh.blender_primitives_v1` |
| `characters/` | `skeletal_mesh` | `skeletal_mesh.blender_rigged_mesh_v1` |
| `animations/` | `skeletal_animation` | `skeletal_animation.blender_clip_v1` |

---

## 6. Example Specs

### 6.1 Audio (`audio_v1`)

```json
{
  "spec_version": 1,
  "asset_id": "laser-blast-01",
  "asset_type": "audio",
  "license": "CC0-1.0",
  "seed": 42,
  "description": "Sci-fi laser blast sound effect",
  "style_tags": ["retro", "scifi", "action"],
  "outputs": [
    {
      "kind": "primary",
      "format": "wav",
      "path": "sounds/laser_blast_01.wav"
    }
  ],
  "recipe": {
    "kind": "audio_v1",
    "params": {
      "duration_seconds": 0.3,
      "sample_rate": 44100,
      "layers": [
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
          },
          "envelope": {
            "attack": 0.01,
            "decay": 0.05,
            "sustain": 0.3,
            "release": 0.15
          },
          "volume": 0.8,
          "pan": 0.0
        },
        {
          "synthesis": {
            "type": "noise_burst",
            "noise_type": "white",
            "filter": {
              "type": "highpass",
              "cutoff": 2000.0,
              "resonance": 0.5
            }
          },
          "envelope": {
            "attack": 0.001,
            "decay": 0.02,
            "sustain": 0.0,
            "release": 0.08
          },
          "volume": 0.4,
          "pan": 0.0
        }
      ]
    }
  }
}
```

### 6.2 Music (Tracker Song)

```json
{
  "spec_version": 1,
  "asset_id": "battle-theme-01",
  "asset_type": "music",
  "license": "CC-BY-4.0",
  "seed": 12345,
  "description": "Upbeat 8-bit battle theme",
  "style_tags": ["chiptune", "action", "loop"],
  "outputs": [
    {
      "kind": "primary",
      "format": "xm",
      "path": "music/battle_theme_01.xm"
    }
  ],
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
        },
        {
          "name": "bass_triangle",
          "synthesis": {
            "type": "triangle"
          },
          "envelope": {
            "attack": 0.01,
            "decay": 0.05,
            "sustain": 0.8,
            "release": 0.1
          }
        },
        {
          "name": "drums_noise",
          "synthesis": {
            "type": "noise",
            "periodic": true
          },
          "envelope": {
            "attack": 0.001,
            "decay": 0.1,
            "sustain": 0.0,
            "release": 0.05
          }
        }
      ],
      "patterns": {
        "intro": {
          "rows": 64,
          "data": [
            { "row": 0, "channel": 0, "note": "C4", "instrument": 0, "volume": 64 },
            { "row": 0, "channel": 1, "note": "C2", "instrument": 1, "volume": 48 },
            { "row": 0, "channel": 3, "note": "C4", "instrument": 2, "volume": 32 }
          ]
        },
        "verse": {
          "rows": 64,
          "data": []
        },
        "chorus": {
          "rows": 64,
          "data": []
        }
      },
      "arrangement": [
        { "pattern": "intro", "repeat": 1 },
        { "pattern": "verse", "repeat": 2 },
        { "pattern": "chorus", "repeat": 2 },
        { "pattern": "verse", "repeat": 1 },
        { "pattern": "chorus", "repeat": 2 }
      ]
    }
  }
}
```

### 6.3 Texture (Material Maps)

```json
{
  "spec_version": 1,
  "asset_id": "metal-panel-01",
  "asset_type": "texture",
  "license": "CC0-1.0",
  "seed": 98765,
  "description": "Worn metal panel texture with scratches",
  "style_tags": ["metal", "industrial", "scifi"],
  "outputs": [
    {
      "kind": "primary",
      "format": "png",
      "path": "textures/metal_panel_01_albedo.png",
      "source": "albedo"
    },
    {
      "kind": "primary",
      "format": "png",
      "path": "textures/metal_panel_01_normal.png",
      "source": "normal"
    },
    {
      "kind": "primary",
      "format": "png",
      "path": "textures/metal_panel_01_roughness.png",
      "source": "roughness"
    },
    {
      "kind": "primary",
      "format": "png",
      "path": "textures/metal_panel_01_metallic.png",
      "source": "metallic"
    }
  ],
  "recipe": {
    "kind": "texture.procedural_v1",
    "params": {
      "resolution": [1024, 1024],
      "tileable": true,
      "nodes": [
        {
          "id": "height",
          "type": "noise",
          "noise": {
            "algorithm": "simplex",
            "scale": 0.08,
            "octaves": 4,
            "persistence": 0.5,
            "lacunarity": 2.0
          }
        },
        {
          "id": "albedo",
          "type": "color_ramp",
          "input": "height",
          "ramp": ["#6a6a70", "#9aa0a6", "#c8cdd2"]
        },
        {
          "id": "roughness",
          "type": "invert",
          "input": "height"
        },
        {
          "id": "metallic",
          "type": "constant",
          "value": 1.0
        },
        {
          "id": "normal",
          "type": "normal_from_height",
          "input": "height",
          "strength": 1.2
        }
      ]
    }
  }
}
```

### 6.4 Static Mesh (Blender Primitives)

```json
{
  "spec_version": 1,
  "asset_id": "crate-wooden-01",
  "asset_type": "static_mesh",
  "license": "CC0-1.0",
  "seed": 54321,
  "description": "Simple wooden storage crate",
  "style_tags": ["prop", "container", "lowpoly"],
  "engine_targets": ["godot", "unity"],
  "outputs": [
    {
      "kind": "primary",
      "format": "glb",
      "path": "meshes/crate_wooden_01.glb"
    }
  ],
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
        },
        {
          "type": "edge_split",
          "angle": 30
        }
      ],
      "uv_projection": "box",
      "material_slots": [
        {
          "name": "wood_body",
          "base_color": [0.4, 0.25, 0.1, 1.0]
        }
      ],
      "export": {
        "apply_modifiers": true,
        "triangulate": true,
        "include_normals": true,
        "include_uvs": true
      },
      "constraints": {
        "max_triangles": 500,
        "max_materials": 2
      }
    }
  }
}
```

### 6.5 Skeletal Mesh (Rigged Character)

```json
{
  "spec_version": 1,
  "asset_id": "robot-basic-01",
  "asset_type": "skeletal_mesh",
  "license": "CC-BY-4.0",
  "seed": 777888,
  "description": "Basic humanoid robot character",
  "style_tags": ["character", "robot", "scifi", "lowpoly"],
  "engine_targets": ["godot"],
  "outputs": [
    {
      "kind": "primary",
      "format": "glb",
      "path": "characters/robot_basic_01.glb"
    }
  ],
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
        },
        {
          "bone": "upper_arm_l",
          "mesh": {
            "primitive": "cylinder",
            "dimensions": [0.08, 0.25, 0.08],
            "segments": 6
          }
        },
        {
          "bone": "upper_arm_r",
          "mesh": {
            "primitive": "cylinder",
            "dimensions": [0.08, 0.25, 0.08],
            "segments": 6
          }
        },
        {
          "bone": "lower_arm_l",
          "mesh": {
            "primitive": "cylinder",
            "dimensions": [0.06, 0.22, 0.06],
            "segments": 6
          }
        },
        {
          "bone": "lower_arm_r",
          "mesh": {
            "primitive": "cylinder",
            "dimensions": [0.06, 0.22, 0.06],
            "segments": 6
          }
        },
        {
          "bone": "upper_leg_l",
          "mesh": {
            "primitive": "cylinder",
            "dimensions": [0.1, 0.35, 0.1],
            "segments": 6
          }
        },
        {
          "bone": "upper_leg_r",
          "mesh": {
            "primitive": "cylinder",
            "dimensions": [0.1, 0.35, 0.1],
            "segments": 6
          }
        },
        {
          "bone": "lower_leg_l",
          "mesh": {
            "primitive": "cylinder",
            "dimensions": [0.08, 0.3, 0.08],
            "segments": 6
          }
        },
        {
          "bone": "lower_leg_r",
          "mesh": {
            "primitive": "cylinder",
            "dimensions": [0.08, 0.3, 0.08],
            "segments": 6
          }
        }
      ],
      "material_slots": [
        {
          "name": "metal_body",
          "base_color": [0.5, 0.5, 0.55, 1.0],
          "metallic": 0.8,
          "roughness": 0.4
        }
      ],
      "skinning": {
        "max_bone_influences": 4,
        "auto_weights": true
      },
      "export": {
        "include_armature": true,
        "include_normals": true,
        "include_uvs": true,
        "triangulate": true
      },
      "constraints": {
        "max_triangles": 5000,
        "max_bones": 64,
        "max_materials": 4
      }
    }
  }
}
```

### 6.6 Skeletal Animation (Animation Clip)

```json
{
  "spec_version": 1,
  "asset_id": "robot-walk-01",
  "asset_type": "skeletal_animation",
  "license": "CC-BY-4.0",
  "seed": 111222,
  "description": "Basic walk cycle for humanoid robot",
  "style_tags": ["animation", "walk", "loop"],
  "outputs": [
    {
      "kind": "primary",
      "format": "glb",
      "path": "animations/robot_walk_01.glb"
    }
  ],
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
            "upper_leg_r": { "rotation": [-15, 0, 0] },
            "lower_leg_l": { "rotation": [-10, 0, 0] },
            "lower_leg_r": { "rotation": [5, 0, 0] },
            "upper_arm_l": { "rotation": [-10, 0, 0] },
            "upper_arm_r": { "rotation": [10, 0, 0] }
          }
        },
        {
          "time": 0.25,
          "bones": {
            "upper_leg_l": { "rotation": [0, 0, 0] },
            "upper_leg_r": { "rotation": [0, 0, 0] },
            "lower_leg_l": { "rotation": [0, 0, 0] },
            "lower_leg_r": { "rotation": [0, 0, 0] },
            "hips": { "position": [0, 0.02, 0] }
          }
        },
        {
          "time": 0.5,
          "bones": {
            "upper_leg_l": { "rotation": [-15, 0, 0] },
            "upper_leg_r": { "rotation": [15, 0, 0] },
            "lower_leg_l": { "rotation": [5, 0, 0] },
            "lower_leg_r": { "rotation": [-10, 0, 0] },
            "upper_arm_l": { "rotation": [10, 0, 0] },
            "upper_arm_r": { "rotation": [-10, 0, 0] }
          }
        },
        {
          "time": 0.75,
          "bones": {
            "upper_leg_l": { "rotation": [0, 0, 0] },
            "upper_leg_r": { "rotation": [0, 0, 0] },
            "lower_leg_l": { "rotation": [0, 0, 0] },
            "lower_leg_r": { "rotation": [0, 0, 0] },
            "hips": { "position": [0, 0.02, 0] }
          }
        },
        {
          "time": 1.0,
          "bones": {
            "upper_leg_l": { "rotation": [15, 0, 0] },
            "upper_leg_r": { "rotation": [-15, 0, 0] },
            "lower_leg_l": { "rotation": [-10, 0, 0] },
            "lower_leg_r": { "rotation": [5, 0, 0] },
            "upper_arm_l": { "rotation": [-10, 0, 0] },
            "upper_arm_r": { "rotation": [10, 0, 0] }
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

---

## 7. Validation Rules

### 7.1 Contract Validation

| Rule | Error Code | Description |
|------|------------|-------------|
| `spec_version` must equal `1` | E001 | Unsupported spec version |
| `asset_id` must match `[a-z][a-z0-9_-]{2,63}` | E002 | Invalid asset_id format |
| `asset_type` must be a known type | E003 | Unknown asset type |
| `seed` must be in range `0..2^32-1` | E004 | Seed out of range |
| `outputs` must have at least one entry | E005 | No outputs declared |
| `outputs` must have at least one `kind: "primary"` | E006 | No primary output |
| `outputs[].path` must be unique | E007 | Duplicate output path |
| `outputs[].path` must be safe (relative, no `..`) | E008 | Unsafe output path |
| `outputs[].path` extension must match format | E009 | Path/format mismatch |
| `outputs[].kind` must not be `metadata` or `preview` (reserved) | E015 | Output validation failed |

### 7.2 Recipe Validation

| Rule | Error Code | Description |
|------|------------|-------------|
| `recipe` required for `generate` command | E010 | Missing recipe |
| `recipe.kind` must be compatible with `asset_type` | E011 | Recipe/asset type mismatch |
| `recipe.params` must be valid for `recipe.kind` | E012 | Invalid recipe params |

### 7.3 Warnings

| Rule | Warning Code | Description |
|------|--------------|-------------|
| `license` is empty or missing | W001 | Missing license information |
| `description` is empty | W002 | Missing description |
| Large seed values near max | W003 | Seed close to overflow boundary |
| Unused recipe params | W004 | Recipe params not used by backend |

---

## 8. Security Considerations

### 8.1 Path Traversal Prevention

All `outputs[].path` values are validated to prevent path traversal attacks:

1. Must be relative (no leading `/`, `\`, or drive letters)
2. Must use forward slashes only
3. Must not contain `..` segments
4. Must be normalized before use
5. Must resolve within the output root directory

### 8.2 No Code Execution

SpecCade specs are pure JSON data. The `generate` command MUST NOT:

- Execute any code embedded in specs
- Evaluate expressions or templates
- Load external files referenced in specs (except for declared dependencies)

### 8.3 Resource Limits

Backends SHOULD enforce resource limits:

| Resource | Recommended Limit |
|----------|-------------------|
| Max spec file size | 10 MB |
| Max output file size | 500 MB |
| Max generation time | 5 minutes |
| Max memory usage | 4 GB |

---

## 9. Future Extensions

### 9.1 Planned Recipe Kinds

| Recipe Kind | Asset Type | Description |
|-------------|------------|-------------|
| `sprite_2d.spritesheet_v1` | `sprite_2d` | Animated sprite sheet generation |
| `audio.sample_chain_v1` | `audio` | Sample-based sound effects (proposed; not implemented) |
| `texture.heightmap_v1` | `texture` | Heightmap terrain textures (proposed; not implemented) |

### 9.2 Spec Version Evolution

When breaking changes are required:

1. Increment `spec_version` to `2`
2. Backends MUST support both v1 and v2 during transition
3. Migration tooling MUST be provided
4. Deprecation timeline: minimum 6 months notice

---

## 10. References

- [RFC 8785 - JSON Canonicalization Scheme (JCS)](https://www.rfc-editor.org/rfc/rfc8785)
- [BLAKE3 Hash Function](https://github.com/BLAKE3-team/BLAKE3)
- [PCG Random Number Generator](https://www.pcg-random.org/)
- [SPDX License List](https://spdx.org/licenses/)
- [glTF 2.0 Specification](https://www.khronos.org/gltf/)
- [XM Module Format](https://github.com/milkytracker/MilkyTracker/wiki/XM)
- [IT Module Format](https://github.com/schismtracker/schismtracker/wiki/ITTECH.TXT)

---

## Appendix A: Skeleton Presets

### humanoid_basic_v1

Standard humanoid skeleton with 22 bones:

```
root
  hips
    spine
      chest
        neck
          head
        shoulder_l
          upper_arm_l
            lower_arm_l
              hand_l
        shoulder_r
          upper_arm_r
            lower_arm_r
              hand_r
    upper_leg_l
      lower_leg_l
        foot_l
    upper_leg_r
      lower_leg_r
        foot_r
```

Bone naming follows Blender/glTF conventions with `_l`/`_r` suffixes for left/right.

---

## Appendix B: Error Code Reference

| Code | Category | Message |
|------|----------|---------|
| E001 | Contract | Unsupported spec_version |
| E002 | Contract | Invalid asset_id format |
| E003 | Contract | Unknown asset_type |
| E004 | Contract | Seed out of valid range |
| E005 | Contract | No outputs declared |
| E006 | Contract | No primary output declared |
| E007 | Contract | Duplicate output path |
| E008 | Contract | Unsafe output path (traversal) |
| E009 | Contract | Output path extension does not match format |
| E010 | Recipe | Recipe required for generate command |
| E011 | Recipe | Recipe kind incompatible with asset type |
| E012 | Recipe | Invalid recipe params |
| E013 | Backend | Backend not available |
| E014 | Backend | Backend execution failed |
| E015 | Backend | Output validation failed |
| W001 | Contract | Missing license information |
| W002 | Contract | Missing description |
| W003 | Contract | Seed near overflow boundary |
| W004 | Recipe | Unused recipe params |
