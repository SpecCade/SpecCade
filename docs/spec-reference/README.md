# Spec Reference

This directory contains detailed reference documentation for SpecCade specs organized by asset type.

## SSOT (Source Of Truth)

- Contract fields, output kinds/formats: `schemas/speccade-spec-v1.schema.json` + `speccade validate`
- Recipe params (exact fields + enums): `crates/speccade-spec/src/recipe/**`
- Generator behavior: backend crates (e.g., `crates/speccade-backend-audio`)

If a doc/example disagrees with validation, treat `speccade validate` + Rust types as authoritative.

## Quick Links

- [Texture Specs](texture.md) - Material maps, normal maps, and packed textures
- [Audio Specs](audio.md) - Sound effects and instrument samples
- [Music Specs](music.md) - Tracker module songs
- [Game Music Genre Kits (Draft)](../music-genre-kits-master-list.md) - Target kit inventory + instrument roles
- [Game Music Genre Kits Audit (Draft)](../music-genre-kits-audit.md) - Coverage checklist + gap list
- [Audio Preset Library (Draft)](../audio-preset-library-master-list.md) - Target `audio_v1` preset inventory for music kits

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
| `outputs` | array | Expected artifacts | At least one entry required; most recipes require a `primary` output (textures can also use `packed`) |

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
- `variants` is currently reserved metadata (the CLI does not expand variants during generation yet)

```json
{
  "seed": 12345,
  "variants": [
    {"variant_id": "soft", "seed_offset": 0},
    {"variant_id": "hard", "seed_offset": 100}
  ]
}
```
When variant expansion is implemented, `seed_offset` will be used to derive related seeds deterministically.

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
| `format` | string | File format | `"wav"`, `"ogg"`, `"xm"`, `"it"`, `"png"`, `"glb"`, `"gltf"`, `"json"`, `"blend"` |
| `path` | string | Relative output path | Must be safe (see constraints) |

### Output Kinds

Currently supported output kinds are:

- `primary` (most recipes)
- `packed` (`texture.packed_v1` only)

`preview` and `metadata` are reserved and currently rejected by validation.

| Kind | Description | Use Case |
|------|-------------|----------|
| `primary` | Main asset output | The generated asset file |
| `preview` | Preview/thumbnail | Reserved (currently rejected by validation) |
| `metadata` | Generation metadata | Reserved (currently rejected; use `${asset_id}.report.json`) |
| `packed` | Channel-packed texture | Multiple maps in one file (see texture docs) |

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
| `texture` | `texture.material_v1`, `texture.normal_v1`, `texture.packed_v1` | PNG | [texture.md](texture.md) |
| `static_mesh` | `static_mesh.blender_primitives_v1` | GLB | See `docs/SPEC_REFERENCE.md` |
| `skeletal_mesh` | `skeletal_mesh.blender_rigged_mesh_v1` | GLB | See `docs/SPEC_REFERENCE.md` |
| `skeletal_animation` | `skeletal_animation.blender_clip_v1` | GLB | See `docs/SPEC_REFERENCE.md` |

## Recipe Structure

Every recipe has a `kind` and `params`. Most recipe kinds are `asset_type.recipe_name` (e.g. `texture.material_v1`), but some are underscore-delimited (e.g. `audio_v1`).

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
  texture/          # Texture specs (material, normal, packed)
  static_mesh/      # Static mesh specs
  skeletal_mesh/    # Character mesh specs
  skeletal_animation/ # Animation clip specs
```

See `golden/speccade/specs/` for the current golden corpus layout.
