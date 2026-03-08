# Sprite Spec Reference

This document covers sprite generation in SpecCade.

| Property | Value |
|----------|-------|
| Asset Type | `sprite` |
| Recipe Kinds | `sprite.sheet_v1`, `sprite.animation_v1`, `sprite.render_from_mesh_v1` |
| Output Formats | `png`, `json` |
| Determinism | Tier 1 for `sheet_v1` and `animation_v1`; Tier 2 for `render_from_mesh_v1` |

## SSOT (Source Of Truth)

- Rust types: `crates/speccade-spec/src/recipe/sprite/`
- Starlark specs: `specs/sprite/`
- Blender handler (Tier 2): `blender/speccade/handlers_render.py`

## Recipe Kind Selection

| Recipe Kind | Use Case | Notes |
|-------------|----------|-------|
| `sprite.sheet_v1` | Packed sprite atlases | Packs explicit frames into a deterministic atlas |
| `sprite.animation_v1` | Sprite clip metadata | Defines animation clips over packed frames |
| `sprite.render_from_mesh_v1` | 3D-to-2D rendering | Renders a mesh from multiple angles through Blender |

## Canonical Examples

- `specs/sprite/sprite_sheet_basic.star` for `sprite.sheet_v1`
- `specs/sprite/sprite_animation_basic.star` for `sprite.animation_v1`
- `specs/sprite/sprite_render_from_mesh.star` for `sprite.render_from_mesh_v1`

## Notes

- `sprite.sheet_v1` and `sprite.animation_v1` are Tier 1 Rust backends and should be byte-identical for the same validated spec and seed.
- `sprite.render_from_mesh_v1` is Tier 2 because the rendered output depends on Blender and is validated through report metrics and lint, not byte identity.
