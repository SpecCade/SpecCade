# Spec Format Reference

Complete reference for SpecCade JSON spec structure, validation rules, and output formats.

## Top-Level Structure

Every spec requires these fields:

```json
{
  "spec_version": 1,
  "asset_id": "my_asset",
  "asset_type": "audio",
  "license": "CC0-1.0",
  "seed": 12345,
  "outputs": [...],
  "recipe": {...}
}
```

## Required Fields

### spec_version
Always `1` for current format.

```json
"spec_version": 1
```

### asset_id
Unique identifier. Pattern: `^[a-z][a-z0-9_-]{2,63}$`

**Rules:**
- 3-64 characters
- Start with lowercase letter
- Only lowercase, numbers, underscore, hyphen
- No spaces or special characters

**Valid:**
- `kick_drum`
- `bass-synth-01`
- `metallic_texture`

**Invalid:**
- `Kick_Drum` (uppercase)
- `1_kick` (starts with number)
- `kick drum` (space)
- `ki` (too short)

### asset_type
One of: `audio`, `texture`, `music`, `static_mesh`, `skeletal_mesh`, `skeletal_animation`

### license
SPDX license identifier. Common values:
- `CC0-1.0` - Public domain
- `CC-BY-4.0` - Attribution required
- `MIT` - MIT License
- `Apache-2.0` - Apache License

### seed
Unsigned 32-bit integer (0 to 4294967295).

Drives deterministic generation - same seed + spec = same output.

**Tips:**
- Use different seeds for variants
- Avoid near `u32::MAX` (overflow warning)
- Document memorable seeds for good results

### outputs
Array of output specifications. At least one `primary` required.

```json
"outputs": [
  {
    "kind": "primary",
    "format": "wav",
    "path": "output/my_sound.wav"
  },
  {
    "kind": "metadata",
    "format": "json",
    "path": "output/my_sound.meta.json"
  }
]
```

**Output kinds:**
- `primary` - Main asset output (required)
- `metadata` - Additional metadata
- `debug` - Debug information

**Output formats by asset type:**
| Asset Type | Primary Format | Optional |
|------------|----------------|----------|
| audio | wav | - |
| texture | png | json (metadata) |
| music | xm, it | - |
| static_mesh | glb | - |
| skeletal_mesh | glb | - |
| skeletal_animation | glb | - |

**Path rules:**
- Relative paths only
- No `..` (path traversal)
- Forward slashes preferred
- Safe characters only

### recipe
Generation parameters with `kind` and `params`.

```json
"recipe": {
  "kind": "audio_v1",
  "params": {
    "duration_seconds": 1.0,
    "sample_rate": 44100,
    "layers": [...]
  }
}
```

**Recipe kinds:**
- `audio_v1`
- `texture.procedural_v1`
- `music.tracker_song_v1`
- `music.tracker_song_compose_v1`
- `static_mesh.blender_primitives_v1`
- `skeletal_mesh.armature_driven_v1`
- `skeletal_mesh.skinned_mesh_v1`
- `skeletal_animation.blender_clip_v1`
- `skeletal_animation.blender_rigged_v1`

## Optional Fields

### description
Human-readable description.

```json
"description": "Punchy electronic kick drum with sub bass"
```

### style_tags
Categorization tags.

```json
"style_tags": ["drum", "kick", "electronic", "punchy"]
```

### engine_targets
Target game engines.

```json
"engine_targets": ["godot", "unity", "unreal"]
```

### migration_notes
Notes about spec changes/updates.

```json
"migration_notes": [
  "v1.1: Increased attack time for smoother onset"
]
```

### variants
Procedural variations of base asset.

```json
"variants": [
  {
    "variant_id": "soft",
    "seed_offset": 100,
    "param_overrides": {
      "layers[0].volume": 0.5
    }
  },
  {
    "variant_id": "hard",
    "seed_offset": 200,
    "param_overrides": {
      "effects[0].drive": 3.0
    }
  }
]
```

Generate with `--expand-variants` flag:
```bash
speccade generate --spec kick.json --expand-variants --out-root ./output
```

## Validation Rules

### Error Codes

| Code | Category | Description |
|------|----------|-------------|
| E001 | Format | Invalid asset_id format |
| E002 | Required | Missing required field |
| E003 | Type | Recipe/asset type mismatch |
| E004 | Range | Value out of valid range |
| E005 | Path | Invalid output path |
| E010 | Audio | Invalid synthesis params |
| E011 | Audio | Invalid effect params |
| E012 | Audio | Invalid envelope params |
| E020 | Texture | Invalid node graph |
| E021 | Texture | Missing node input |
| E030 | Music | Invalid pattern data |
| E031 | Music | Invalid instrument |
| E040 | Mesh | Invalid primitive params |
| E041 | Mesh | Invalid bone structure |

### Warnings

| Code | Description |
|------|-------------|
| W001 | Seed near u32::MAX (overflow risk) |
| W002 | Duration very long (>60s) |
| W003 | High sample rate (>48000) |
| W004 | Many layers (>16) |
| W005 | Deep effect chain (>8) |

## Output Report

Every generation produces a JSON report:

```json
{
  "asset_id": "my_sound",
  "spec_hash": "abc123...",
  "recipe_hash": "def456...",
  "seed": 12345,
  "backend": "audio_v1",
  "backend_version": "1.0.0",
  "tier": 1,
  "success": true,
  "outputs": [
    {
      "path": "output/my_sound.wav",
      "format": "wav",
      "hash": "789xyz...",
      "size_bytes": 176444
    }
  ],
  "duration_ms": 45,
  "git_commit": "abc1234",
  "validation": {
    "errors": [],
    "warnings": []
  }
}
```

## Determinism

### Tier 1 (Byte-Identical)
Audio, music, texture backends.

Same spec + seed = identical file hash.

```bash
# Generate twice
speccade generate --spec sound.json --out-root ./a
speccade generate --spec sound.json --out-root ./b

# Files should match
diff ./a/sound.wav ./b/sound.wav  # No difference
```

### Tier 2 (Metric-Validated)
Blender backends (mesh, animation).

Same spec = similar metrics (triangle count, bounds).

Output may differ by Blender version.

## CLI Quick Reference

```bash
# Validate spec
speccade validate --spec FILE

# Validate with full recipe check
speccade validate --spec FILE --artifacts

# Generate single asset
speccade generate --spec FILE --out-root DIR

# Generate with variants
speccade generate --spec FILE --expand-variants --out-root DIR

# Batch generate
speccade generate-all --spec-dir DIR --out-root DIR

# Skip Blender assets
speccade generate-all --spec-dir DIR --out-root DIR --skip-blender

# Format spec (canonical JSON)
speccade fmt --spec FILE

# Expand compose spec
speccade expand --spec FILE > expanded.json

# Check dependencies
speccade doctor

# List templates
speccade template list --asset-type audio

# Copy template
speccade template copy preset_kick --to ./my_kick.json
```

## JSON Schema

Full schema available at: `speccade/schemas/speccade-spec-v1.schema.json`

Validate with any JSON Schema validator:

```bash
# Using ajv-cli
ajv validate -s schemas/speccade-spec-v1.schema.json -d my_spec.json
```
