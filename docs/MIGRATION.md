# Migration Guide

This guide covers migrating from the legacy `.studio` spec system to SpecCade's canonical JSON format.

## Overview

SpecCade replaces executable Python specs (`.spec.py`) with declarative JSON specs. This migration eliminates code execution risk while providing better version control, diffing, and tool integration.

### What Changed?

| Legacy System | SpecCade |
|--------------|----------|
| `.spec.py` files with Python code | `.json` files with pure data |
| Dynamic execution at runtime | Static validation before generation |
| Per-category parsers | Unified spec contract + typed recipe kinds |
| Implicit randomness | Explicit seeds and deterministic RNG |
| Mixed security model (exec + static) | Safe by default (no code execution) |

## Migration Workflow

### Automated Migration

Use the `speccade migrate` command to automatically convert legacy specs:

```bash
# Dry run (shows what would be migrated)
speccade migrate --project ./my-game --dry-run

# Full migration with exec permission (required for .spec.py evaluation)
speccade migrate --project ./my-game --allow-exec-specs --out-root ./migrated-specs
```

The migration tool:

1. Scans `.studio/specs/` for `.spec.py` files
2. Executes each spec in a sandboxed Python environment (with `--allow-exec-specs`)
3. Extracts the spec dict (`SOUND`, `INSTRUMENT`, `SONG`, `TEXTURE`, `NORMAL`, `MESH`, `CHARACTER`, `ANIMATION`)
4. Maps legacy keys to SpecCade canonical schema
5. Writes JSON specs to `specs/{asset_type}/{asset_id}.json`
6. Generates a migration report with warnings and manual review items

### Security Note: `--allow-exec-specs`

The `--allow-exec-specs` flag is **required** to execute legacy `.spec.py` files. This executes Python code on your machine.

**Safety guidelines:**

- Only run on trusted local projects (never on untrusted repos)
- Review the migration report for unexpected code execution
- Consider running in a VM or container for untrusted sources
- Once migrated, the JSON specs are safe to review, diff, and share

**Alternative (static mode):** If you don't trust the specs, manually convert them by reading the Python dict definitions. The migration tool can parse static dicts without execution (experimental).

## Asset Type Mapping

Legacy specs are mapped to SpecCade asset types and recipe kinds:

| Legacy Category | SpecCade Asset Type | Recipe Kind |
|-----------------|---------------------|-------------|
| `sounds/` | `audio_sfx` | `audio_sfx.layered_synth_v1` |
| `instruments/` | `audio_instrument` | `audio_instrument.synth_patch_v1` |
| `music/` | `music` | `music.tracker_song_v1` |
| `textures/` | `texture_2d` | `texture_2d.material_maps_v1` |
| `normals/` | `texture_2d` | `texture_2d.normal_map_v1` |
| `meshes/` | `static_mesh` | `static_mesh.blender_primitives_v1` |
| `characters/` | `skeletal_mesh` | `skeletal_mesh.blender_rigged_mesh_v1` |
| `animations/` | `skeletal_animation` | `skeletal_animation.blender_clip_v1` |

## Key Mapping Examples

### Audio SFX (Sounds)

**Legacy:**

```python
SOUND = {
    "name": "laser_shot",
    "duration": 0.25,
    "sample_rate": 44100,
    "layers": [
        {
            "type": "fm_synth",
            "carrier_freq": 1200,
            "mod_ratio": 2.5,
            "mod_index": 8.0,
            "envelope": [0.001, 0.1, 0.3, 0.1],
            "volume": 0.9,
        }
    ],
}
```

**SpecCade:**

```json
{
  "spec_version": 1,
  "asset_id": "laser_shot",
  "asset_type": "audio_sfx",
  "license": "CC0-1.0",
  "seed": 1002,
  "outputs": [
    {
      "kind": "audio",
      "format": "wav",
      "path": "laser_shot.wav"
    }
  ],
  "recipe": {
    "kind": "audio_sfx.layered_synth_v1",
    "params": {
      "duration_seconds": 0.25,
      "sample_rate": 44100,
      "layers": [
        {
          "synthesis": {
            "type": "fm_synth",
            "carrier_freq": 1200,
            "mod_ratio": 2.5,
            "mod_index": 8.0
          },
          "amplitude": 0.9,
          "envelope": {
            "attack": 0.001,
            "decay": 0.1,
            "sustain": 0.3,
            "release": 0.1
          }
        }
      ]
    }
  }
}
```

**Changes:**

- `name` becomes `asset_id`
- Explicit `spec_version`, `asset_type`, `license`, `seed`
- `outputs[]` array declares expected artifacts
- `envelope` list becomes named ADSR object
- `volume` becomes `amplitude`
- Synthesis params nested under `synthesis` object

### Music (Tracker Songs)

**Legacy:**

```python
SONG = {
    "name": "battle_theme",
    "bpm": 140,
    "speed": 6,
    "channels": 4,
    "instruments": [
        {"type": "pulse", "duty": 0.5, "envelope": [0.01, 0.1, 0.6, 0.2]},
        {"type": "triangle", "envelope": [0.01, 0.05, 0.8, 0.1]},
    ],
    "patterns": {
        "intro": [...],
        "verse": [...],
    },
    "arrangement": ["intro", "verse", "verse"],
}
```

**SpecCade:**

```json
{
  "spec_version": 1,
  "asset_id": "battle_theme",
  "asset_type": "music",
  "license": "CC-BY-4.0",
  "seed": 12345,
  "outputs": [
    {
      "kind": "audio",
      "format": "xm",
      "path": "battle_theme.xm"
    }
  ],
  "recipe": {
    "kind": "music.tracker_song_v1",
    "params": {
      "format": "xm",
      "bpm": 140,
      "speed": 6,
      "channels": 4,
      "instruments": [
        {
          "name": "lead_pulse",
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
        }
      ],
      "patterns": {
        "intro": { "rows": 64, "data": [...] },
        "verse": { "rows": 64, "data": [...] }
      },
      "arrangement": [
        { "pattern": "intro", "repeat": 1 },
        { "pattern": "verse", "repeat": 2 }
      ]
    }
  }
}
```

**Changes:**

- `format` field added (`xm` or `it`)
- Instruments have explicit `name` field
- `duty` becomes `duty_cycle`
- Patterns are objects with `rows` and `data` fields
- Arrangement entries are objects with `pattern` and `repeat` fields

### Textures (Material Maps)

**Legacy:**

```python
TEXTURE = {
    "name": "metal_panel",
    "resolution": [1024, 1024],
    "tileable": True,
    "maps": ["albedo", "roughness", "metallic", "normal"],
    "base_color": [0.6, 0.6, 0.65],
    "roughness_range": [0.3, 0.6],
    "metallic": 1.0,
    "layers": [
        {
            "type": "noise",
            "scale": 8.0,
            "octaves": 4,
            "affects": ["roughness", "normal"],
        },
        {
            "type": "scratches",
            "density": 0.15,
            "length_range": [0.1, 0.4],
            "affects": ["albedo", "roughness"],
        },
    ],
}
```

**SpecCade:**

```json
{
  "spec_version": 1,
  "asset_id": "metal_panel",
  "asset_type": "texture_2d",
  "license": "CC0-1.0",
  "seed": 98765,
  "outputs": [
    {
      "kind": "map",
      "format": "png",
      "path": "metal_panel_albedo.png"
    },
    {
      "kind": "map",
      "format": "png",
      "path": "metal_panel_roughness.png"
    },
    {
      "kind": "map",
      "format": "png",
      "path": "metal_panel_metallic.png"
    },
    {
      "kind": "map",
      "format": "png",
      "path": "metal_panel_normal.png"
    }
  ],
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
          "affects": ["roughness", "normal"]
        },
        {
          "type": "scratches",
          "density": 0.15,
          "length_range": [0.1, 0.4],
          "affects": ["albedo", "roughness"]
        }
      ]
    }
  }
}
```

**Changes:**

- Each map gets its own `outputs[]` entry
- Base material params nested under `base_material`
- Noise layers have explicit `noise` sub-object with `algorithm` field
- Layer types renamed (e.g., `noise` becomes `noise_pattern`)

### Meshes (Static Meshes)

**Legacy:**

```python
MESH = {
    "name": "crate_wooden",
    "primitive": "cube",
    "dimensions": [1.0, 1.0, 1.0],
    "modifiers": [
        {"type": "bevel", "width": 0.02, "segments": 2},
        {"type": "edge_split", "angle": 30},
    ],
    "uv_projection": "box",
    "material": {"base_color": [0.4, 0.25, 0.1, 1.0]},
}
```

**SpecCade:**

```json
{
  "spec_version": 1,
  "asset_id": "crate_wooden",
  "asset_type": "static_mesh",
  "license": "CC0-1.0",
  "seed": 54321,
  "outputs": [
    {
      "kind": "mesh",
      "format": "glb",
      "path": "crate_wooden.glb"
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
      }
    }
  }
}
```

**Changes:**

- `primitive` becomes `base_primitive`
- `material` becomes `material_slots[]` array
- Explicit `export` settings for GLB output

## Common Migration Issues

### Issue: Implicit Randomness

**Problem:** Legacy specs may use `random.random()` or `np.random` without explicit seeding.

**Solution:** SpecCade requires explicit `seed` field. The migration tool assigns deterministic seeds based on asset_id hash. Review and adjust seeds as needed.

### Issue: Dynamic Expressions

**Problem:** Legacy specs may contain Python expressions like `random.choice([...])` or `np.linspace(...)`.

**Solution:** The migration tool evaluates expressions at migration time and bakes values into the JSON. Review `migration_notes[]` in the output for cases requiring manual attention.

### Issue: Complex Custom Logic

**Problem:** Legacy specs with custom functions, imports, or conditional logic.

**Solution:** SpecCade does not support arbitrary code execution. You'll need to:

1. Extract the intent of the custom logic
2. Represent it declaratively using recipe params
3. File a feature request if the recipe kind lacks the needed expressiveness
4. As a workaround, generate multiple specs for different variations

### Issue: Missing Output Declarations

**Problem:** Legacy specs infer output paths from `name` and category.

**Solution:** SpecCade requires explicit `outputs[]` declarations. The migration tool auto-generates them, but review paths to match your project structure.

### Issue: Unsupported Features

**Problem:** Legacy spec uses a feature not yet supported in SpecCade v1.

**Solution:** Check the migration report for unsupported features. Options:

1. Wait for SpecCade v1.1+ support
2. File a feature request with use case
3. Use legacy system for that asset until support arrives
4. Hand-author the asset and skip procedural generation

## Manual Migration Steps

If you prefer manual migration or need fine control:

1. **Read the legacy spec:** Open the `.spec.py` file and identify the dict (e.g., `SOUND`, `MESH`).

2. **Create the JSON structure:** Start with the canonical schema:

   ```json
   {
     "spec_version": 1,
     "asset_id": "your_asset_id",
     "asset_type": "audio_sfx",
     "license": "CC0-1.0",
     "seed": 42,
     "outputs": [...],
     "recipe": {
       "kind": "audio_sfx.layered_synth_v1",
       "params": {...}
     }
   }
   ```

3. **Map keys:** Use the mapping tables in this guide to translate legacy keys to SpecCade schema.

4. **Add outputs:** Declare expected output files in the `outputs[]` array.

5. **Validate:** Run `speccade validate --spec your_asset.json` to catch schema errors.

6. **Generate:** Run `speccade generate --spec your_asset.json` and compare output with legacy version.

7. **Iterate:** Adjust params, seed, or recipe kind until output matches your intent.

## Deprecation Timeline

SpecCade is designed to replace the legacy `.studio` system:

| Version | Milestone |
|---------|-----------|
| **v0.3** | Migration tool available |
| **v1.0** | Full parity with legacy feature set |
| **v1.1+** | Legacy system deprecated (security risk) |

**Recommendation:** Migrate projects to SpecCade before v1.1. The legacy system will not receive security updates after deprecation.

## Validation After Migration

After migration, validate your specs:

```bash
# Validate all migrated specs
find ./specs -name "*.json" -exec speccade validate --spec {} \;

# Generate all assets and compare
speccade generate --spec-dir ./specs --out-root ./output

# Compare with legacy outputs (manual inspection)
# SpecCade outputs should match intent, but byte-identity is not guaranteed vs legacy
```

## Getting Help

- **Migration issues:** [github.com/SpecCade/SpecCade/issues](https://github.com/SpecCade/SpecCade/issues)
- **Feature requests:** [github.com/SpecCade/SpecCade/discussions](https://github.com/SpecCade/SpecCade/discussions)
- **Schema questions:** See [SPEC_REFERENCE.md](SPEC_REFERENCE.md)

## Summary

Migration from legacy to SpecCade involves:

1. Run `speccade migrate --allow-exec-specs` on trusted projects
2. Review migration report for warnings and manual review items
3. Validate migrated specs with `speccade validate`
4. Generate assets and compare with legacy outputs
5. Commit JSON specs to version control
6. Decommission `.spec.py` files after validation

The result: safer, more portable, and easier-to-review asset specs with deterministic generation.
