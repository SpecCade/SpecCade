# SpecCade Parity Matrix

This document tracks **canonical** SpecCade generator coverage and planned generators.

For the legacy `.studio/specs/**.spec.py` key taxonomy used by `speccade migrate --audit`, see:
`docs/legacy/PARITY_MATRIX_LEGACY_SPEC_PY.md`.

---

## Canonical Generators (Implemented)

| Recipe kind | Asset type | Backend crate | Tier | Primary outputs | Status | Notes |
|------------|------------|--------------|------|-----------------|--------|-------|
| `audio_v1` | `audio` | `speccade-backend-audio` | 1 | `wav` | Implemented | Unified audio synthesis (SFX + instruments) |
| `music.tracker_song_v1` | `music` | `speccade-backend-music` | 1 | `xm` / `it` | Implemented | Deterministic tracker module writer |
| `texture.procedural_v1` | `texture` | `speccade-backend-texture` | 1 | `png` | Implemented | Unified procedural DAG with named outputs |
| `texture.matcap_v1` | `texture` | `speccade-backend-texture` | 1 | `png` | Implemented | Matcap for stylized NPR shading (toon, rim, metallic, etc.) |
| `texture.material_preset_v1` | `texture` | `speccade-backend-texture` | 1 | `png` (x4) + `json` | Implemented | PBR material presets (albedo, roughness, metallic, normal) |
| `vfx.particle_profile_v1` | `vfx` | `speccade-backend-texture` | 1 | `json` | Implemented | Particle rendering profile presets (metadata-only: blend modes, tint, distortion) |
| `ui.nine_slice_v1` | `ui` | `speccade-backend-texture` | 1 | `png` + `json` | Implemented | Nine-slice panel generation with corner/edge/center regions |
| `ui.icon_set_v1` | `ui` | `speccade-backend-texture` | 1 | `png` + `json` | Implemented | Icon pack assembly with sprite frames |
| `ui.item_card_v1` | `ui` | `speccade-backend-texture` | 1 | `png` + `json` | Implemented | Item card templates with rarity variants and customizable slots |
| `ui.damage_number_v1` | `ui` | `speccade-backend-texture` | 1 | `png` + `json` | Implemented | Damage number sprites with style variants (normal, critical, healing) |
| `font.bitmap_v1` | `font` | `speccade-backend-texture` | 1 | `png` + `json` | Implemented | Bitmap pixel font with glyph atlas and metrics |
| `static_mesh.blender_primitives_v1` | `static_mesh` | `speccade-backend-blender` | 2 | `glb` | Implemented | Blender-driven primitives |
| `static_mesh.modular_kit_v1` | `static_mesh` | `speccade-backend-blender` | 2 | `glb` | Implemented | Modular kit generators (walls, pipes, doors) |
| `skeletal_mesh.blender_rigged_mesh_v1` | `skeletal_mesh` | `speccade-backend-blender` | 2 | `glb` | Implemented | Rigged mesh export |
| `skeletal_animation.blender_clip_v1` | `skeletal_animation` | `speccade-backend-blender` | 2 | `glb` | Implemented | Simple keyframed clip |
| `skeletal_animation.blender_rigged_v1` | `skeletal_animation` | `speccade-backend-blender` | 2 | `glb` | Implemented | IK/rig-aware animation export |
| `sprite.render_from_mesh_v1` | `sprite` | `speccade-backend-blender` | 2 | `png` + `json` | Implemented | Render 3D mesh to sprite atlas from multiple angles |

---

## Planned / Proposed Generators

These entries are **design targets** (not yet implemented). Details may change as the schema evolves.

| Proposed recipe kind | Proposed asset type | Tier | Expected outputs | Status | Notes / keywords |
|----------------------|---------------------|------|------------------|--------|------------------|
| `sprite.sheet_v1` | `sprite` | 1 | `png` + `json` (metadata) | Planned | Spritesheet generator, palette control, outlines, lighting ramps |
| `sprite.animation_v1` | `sprite_animation` | 1 | `png` + `json` (timeline) + optional `webp/gif` preview | Planned | Sprite-based animation clips (loops, events, directional sets) |

---

## Production-Readiness Targets

- Tier 1: deterministic-by-contract, golden hashes, stable schemas, strict validation.
- Tier 2: external-tool backends (e.g., Blender) validated by metrics + pinned toolchain + CI invariants.

For longer-term generator expansion ideas, see `docs/ROADMAP.md`.
