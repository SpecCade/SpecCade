# Spec Reference (Index)

The canonical, detailed spec documentation lives in `docs/spec-reference/`:

- `docs/spec-reference/README.md` (contract + common rules)
- `docs/spec-reference/audio.md`
- `docs/spec-reference/music.md`
- `docs/spec-reference/texture.md`

For authoritative validation, use `speccade validate` / `speccade generate` (these run the same Rust validation used by the CLI).

## Canonical Asset Types and Recipe Kinds

| Asset Type | Recipe Kind | Notes |
|------------|------------|------|
| `audio` | `audio_v1` | WAV output |
| `music` | `music.tracker_song_v1` | XM/IT output |
| `texture` | `texture.material_v1` | Material maps (multiple PNG outputs) |
| `texture` | `texture.normal_v1` | Normal map (single PNG output) |
| `texture` | `texture.packed_v1` | Packed RGBA PNG outputs |
| `texture` | `texture.graph_v1` | Map-agnostic node graph IR (multiple PNG outputs) |
| `static_mesh` | `static_mesh.blender_primitives_v1` | Blender-backed (Tier 2 metrics) |
| `skeletal_mesh` | `skeletal_mesh.blender_rigged_mesh_v1` | Blender-backed (Tier 2 metrics) |
| `skeletal_animation` | `skeletal_animation.blender_clip_v1` | Blender-backed (Tier 2 metrics) |

## Blender-Backed Specs (High-Level)

These recipes are validated and dispatched by the CLI, but the parameter surface area is large. The SSOT for exact fields is the `speccade-spec` Rust types:

- Static mesh params: `crates/speccade-spec/src/recipe/mesh/static_mesh.rs`
- Skeletal mesh params: `crates/speccade-spec/src/recipe/character/mod.rs`
- Animation clip params: `crates/speccade-spec/src/recipe/animation/clip.rs`

### `static_mesh.blender_primitives_v1`

Minimal params:

- `base_primitive`: a Blender primitive (e.g. `"cube"`, `"sphere"`, `"cylinder"`)
- `dimensions`: `[x, y, z]`

### `skeletal_mesh.blender_rigged_mesh_v1`

Top-level params support both a modern structured form (`skeleton_preset` / `skeleton` / `body_parts`) and a legacy dict form (`parts`). Both are currently part of the public schema.

### `skeletal_animation.blender_clip_v1`

Key fields:

- `skeleton_preset`
- `clip_name`
- `duration_seconds`
- `fps`
- `keyframes[]` (per-bone transforms keyed by bone name)

## Reports

`speccade validate` and `speccade generate` write an `${asset_id}.report.json` file next to the spec file. This is the primary structured output for hashes, metrics, and validation messages.
