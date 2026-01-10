# Migration Guide

This guide covers migrating from the legacy `.studio/specs/*.spec.py` system to SpecCade JSON specs.

## TL;DR

- `speccade migrate` converts legacy `.spec.py` files into **canonical SpecCade spec _envelopes_** (contract + recipe kind).
- **Recipe param mapping is not fully implemented yet.** Migrated specs will usually require manual edits before `speccade generate` will succeed.

## Commands

### Migrate

```bash
speccade migrate --project /path/to/project
```

By default, the migrator uses a conservative parser and does **not** execute Python.

### Migrate (allow exec)

```bash
speccade migrate --project /path/to/project --allow-exec-specs
```

`--allow-exec-specs` will execute legacy `.spec.py` code. Only use this with trusted local projects.

### Audit (no migration)

```bash
speccade migrate --project /path/to/project --audit
speccade migrate --project /path/to/project --audit --audit-threshold 0.90
```

Audit mode parses specs, classifies legacy keys against `PARITY_MATRIX.md`, and prints an aggregate completeness report.

## What the Migrator Does Today (Accurate Behavior)

For each legacy file under:

```
<project>/.studio/specs/**/<name>.spec.py
```

the migrator will:

1. Parse the legacy dict (`SOUND`, `INSTRUMENT`, `SONG`, `TEXTURE`, `NORMAL`, `MESH`, `CHARACTER`, `ANIMATION`)
2. Choose a canonical `asset_type` and `recipe.kind` (see mapping below)
3. Create a SpecCade JSON spec at:
   ```
   <project>/specs/<asset_type>/<asset_id>.json
   ```
4. Generate:
   - `asset_id` from filename (lowercased, `_` → `-`)
   - `seed` from a deterministic hash of the filename
   - a placeholder `license: "UNKNOWN"`
5. Add `migration_notes` and (when applicable) warnings for manual review

### Important Limitation: Params Mapping Is TODO

The migrator currently **passes through** the legacy dict contents into `recipe.params` (minus the legacy `name` field). Because SpecCade recipe params are strict (`deny_unknown_fields`), migrated specs are not guaranteed to validate or generate until you translate legacy keys into the canonical recipe schema.

Use:

```bash
speccade validate --spec <migrated.json>
```

to see which parts need manual conversion.

## Category → Canonical Mapping

| Legacy Category | Canonical `asset_type` | Canonical `recipe.kind` |
|----------------|-------------------------|--------------------------|
| `sounds/` | `audio` | `audio_v1` |
| `instruments/` | `audio` | `audio_v1` |
| `music/` | `music` | `music.tracker_song_v1` |
| `textures/` | `texture` | `texture.material_v1` |
| `normals/` | `texture` | `texture.normal_v1` |
| `meshes/` | `static_mesh` | `static_mesh.blender_primitives_v1` |
| `characters/` | `skeletal_mesh` | `skeletal_mesh.blender_rigged_mesh_v1` |
| `animations/` | `skeletal_animation` | `skeletal_animation.blender_clip_v1` |

## Manual Conversion Pointers

After migrating, use the spec reference docs to translate legacy dict keys into canonical recipe params:

- `docs/spec-reference/audio.md`
- `docs/spec-reference/music.md`
- `docs/spec-reference/texture.md`

The migration parity inventory (legacy key taxonomy) is in `PARITY_MATRIX.md`.
