# Spec Reference

This directory contains detailed reference documentation for SpecCade specs organized by asset type.

## SSOT

If a doc/example disagrees with validation, treat `speccade validate` + Rust types in `crates/speccade-spec/` as authoritative. See [`AGENTS.md`](../../AGENTS.md) for the full SSOT map.

## Quick Links

- [Texture Specs](texture.md) - Unified procedural texture graphs
- [Audio Specs](audio.md) - Sound effects and instrument samples
- [Music Specs](music.md) - Tracker module songs
- [Static Mesh Specs](mesh.md) - Blender primitive meshes with modifiers
- [Character Specs](character.md) - Skeletal meshes with armatures
- [Animation Specs](animation.md) - Skeletal animation clips
- [Structural Metrics](structural-metrics.md) - LLM-friendly geometric analysis for 3D assets

## Spec Structure

Every SpecCade spec is a JSON document with two logical sections:

1. **Contract** - Metadata and output declarations (required for all operations)
2. **Recipe** - Backend-specific generation parameters (required for `generate`)

### Minimal Valid Spec

```json
{
  "spec_version": 1,
  "asset_id": "my_asset",
  "asset_type": "audio",
  "license": "CC0-1.0",
  "seed": 42,
  "outputs": [
    {
      "kind": "primary",
      "format": "wav",
      "path": "my_sound.wav"
    }
  ],
  "recipe": {
    "kind": "audio_v1",
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
| `asset_type` | string | Asset type enum | See asset types below |
| `license` | string | License identifier | SPDX recommended (e.g., `"CC0-1.0"`) |
| `seed` | integer | RNG seed | Range: `0` to `4294967295` (2^32-1) |
| `outputs` | array | Expected artifacts | At least one entry required; textures require `primary` PNG outputs with `source` bindings |

### Optional Fields

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `description` | string | Human-readable description | omitted |
| `style_tags` | array | Semantic tags (e.g., `["retro", "8bit"]`) | omitted |
| `engine_targets` | array | Target engines: `"godot"`, `"unity"`, `"unreal"` | omitted |
| `migration_notes` | array | Informational notes from `speccade migrate` | omitted |
| `variants` | array | Variant specs for procedural variations | omitted |

## Seeds and Determinism

The `seed` field controls RNG initialization for procedural generation:

- Must be a non-negative integer less than 2^32 (0 to 4,294,967,295)
- Same seed + same spec = identical output (within documented tolerances)
- `variants` can be expanded by the CLI with `speccade generate --expand-variants`

```json
{
  "seed": 12345,
  "variants": [
    {"variant_id": "soft", "seed_offset": 0},
    {"variant_id": "hard", "seed_offset": 100}
  ]
}
```
When variant expansion is enabled, each variant is generated under `{out_root}/variants/{variant_id}/` using a derived seed.

### Determinism Tiers

- **Tier 1 (Audio, Music, Textures):** Byte-identical output per platform and backend version
- **Tier 2 (Meshes, Characters, Animations):** Metric validation (triangle count, bounds, bone count)

See [DETERMINISM.md](../DETERMINISM.md) for the complete determinism policy.

## Output Specification

Each entry in `outputs[]` declares an expected artifact:

```json
{
  "kind": "primary",
  "format": "wav",
  "path": "sounds/laser_blast.wav"
}
```

### Output Fields

| Field | Type | Description | Values |
|-------|------|-------------|--------|
| `kind` | string | Output category | See output kinds below |
| `format` | string | File format | `"wav"`, `"xm"`, `"it"`, `"png"`, `"glb"`, `"gltf"`, `"json"` |
| `path` | string | Relative output path | Must be safe (see constraints) |
| `source` | string | Optional output binding to a named node | Used by `texture.procedural_v1` |

### Output Kinds

Currently supported output kinds are:

- `primary` (most recipes)

`preview` is reserved and currently rejected by validation.

`metadata` is reserved by default, but some recipe kinds may explicitly allow it (e.g. `texture.trimsheet_v1`).

| Kind | Description | Use Case |
|------|-------------|----------|
| `primary` | Main asset output | The generated asset file |
| `preview` | Preview/thumbnail | Reserved (not declared in `outputs[]`) |
| `metadata` | Extra JSON output | Reserved by default; use `${asset_id}.report.json` unless a recipe explicitly allows metadata outputs |

### Path Constraints

- Must be relative (no leading `/`, `\`, or drive letter)
- Must use forward slashes (`/`) only
- Must not contain `..` segments (path traversal)
- Must end with extension matching `format`
- Must be unique within the spec

## Validation

### Common Validation Rules

| Rule | Error Code | Description |
|------|------------|-------------|
| `spec_version` must equal `1` | E001 | Unsupported spec version |
| `asset_id` must match `[a-z][a-z0-9_-]{2,63}` | E002 | Invalid asset_id format |
| `asset_type` must be known | E003 | Unknown asset type |
| `seed` must be in range `0..2^32-1` | E004 | Seed out of range |
| `outputs` must have at least one entry | E005 | No outputs declared |
| `outputs` must include at least one `primary` output | E006 | No primary output declared |
| `outputs[].path` must be unique | E007 | Duplicate output path |
| `outputs[].path` must be safe | E008 | Unsafe output path |
| `outputs[].path` extension must match format | E009 | Path/format mismatch |
| `recipe` required for `generate` | E010 | Missing recipe |
| `recipe.kind` must match `asset_type` | E011 | Recipe/asset type mismatch |

### Common Warnings

| Rule | Warning Code | Description |
|------|--------------|-------------|
| `license` is empty | W001 | Missing license information |
| `description` is empty | W002 | Missing description |
| Large seed near max value | W003 | Seed close to overflow |
| Unused recipe params | W004 | Recipe params not used by backend |

## Asset Types Overview

| Asset Type | Recipe Kinds | Output Formats | Documentation |
|------------|--------------|----------------|---------------|
| `audio` | `audio_v1` | WAV | [audio.md](audio.md) |
| `music` | `music.tracker_song_v1` (canonical), `music.tracker_song_compose_v1` (draft) | XM, IT | [music.md](music.md) |
| `texture` | `texture.procedural_v1`, `texture.trimsheet_v1`, `texture.decal_v1`, `texture.splat_set_v1`, `texture.matcap_v1`, `texture.material_preset_v1` | PNG / JSON | [texture.md](texture.md) |
| `sprite` | `sprite.sheet_v1`, `sprite.animation_v1`, `sprite.render_from_mesh_v1` | PNG / JSON | [sprite.md](sprite.md) |
| `vfx` | `vfx.flipbook_v1`, `vfx.particle_profile_v1` | PNG / JSON | [vfx.md](vfx.md) |
| `ui` | `ui.nine_slice_v1`, `ui.icon_set_v1`, `ui.item_card_v1`, `ui.damage_number_v1` | PNG / JSON | [ui.md](ui.md) |
| `font` | `font.bitmap_v1` | PNG / JSON | [font.md](font.md) |
| `static_mesh` | `static_mesh.blender_primitives_v1`, `static_mesh.modular_kit_v1`, `static_mesh.organic_sculpt_v1` | GLB | [mesh.md](mesh.md) |
| `skeletal_mesh` | `skeletal_mesh.blender_rigged_mesh_v1` | GLB | [character.md](character.md) |
| `skeletal_animation` | `skeletal_animation.blender_clip_v1`, `skeletal_animation.blender_rigged_v1`, `skeletal_animation.helpers_v1` | GLB | [animation.md](animation.md) |

## Recipe Structure

Every recipe has a `kind` and `params`. Most recipe kinds are `asset_type.recipe_name` (e.g. `texture.procedural_v1`), but some are underscore-delimited (e.g. `audio_v1`).

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

Recipe kinds must be compatible with the spec's `asset_type`. See type-specific documentation for available recipes and their parameters.

## Golden Corpus

Reference specs are available in the golden corpus:

```
golden/speccade/specs/
  audio/            # Audio specs (SFX and instrument samples)
  music/            # Tracker song specs
  texture/          # Texture specs (procedural graphs)
  static_mesh/      # Static mesh specs
  skeletal_mesh/    # Character mesh specs
  skeletal_animation/ # Animation clip specs
```

See `golden/speccade/specs/` for the current golden corpus layout.
