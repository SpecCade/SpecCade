# SpecCade Authoring

This skill provides knowledge for authoring SpecCade specs - the declarative spec format (Starlark preferred, JSON accepted) that drives deterministic asset generation for audio, textures, music, and 3D meshes.

## When to Use

Use this skill when users ask about creating, editing, or understanding SpecCade specs. Triggered by questions like:
- "How do I create a sound/texture/asset?"
- "What synthesis types are available?"
- "How does the spec format work?"
- "What effects can I apply?"
- "How do I write tracker music?"
- "What is the compose IR / pattern IR?"
- "How do emit, stack, and repeat work?"

## Core Concepts

### Spec Structure

Every SpecCade spec is a JSON file with these required fields:

```json
{
  "spec_version": 1,
  "asset_id": "my_asset_name",
  "asset_type": "audio",
  "license": "CC0-1.0",
  "seed": 12345,
  "outputs": [{ "kind": "primary", "format": "wav", "path": "out/my_asset.wav" }],
  "recipe": {
    "kind": "audio_v1",
    "params": { ... }
  }
}
```

**Key rules:**
- `asset_id`: lowercase, 3-64 chars, pattern `^[a-z][a-z0-9_-]{2,63}$`
- `seed`: u32 (0 to 4294967295) - drives deterministic generation
- `outputs`: at least one with `kind: "primary"`
- `recipe.kind`: must match asset type prefix

## Preferred Format

LLMs should always output Starlark (`.star`) specs using stdlib functions unless the user explicitly requests JSON. Starlark provides validation, composability, and schema resilience over raw JSON. See [`docs/starlark-authoring.md`](../../../docs/starlark-authoring.md) for full guidance.

### Asset Types

| Type | Recipe Kind | Output | Backend |
|------|-------------|--------|---------|
| audio | `audio_v1` | WAV | Rust (Tier 1) |
| texture | `texture.procedural_v1` | PNG | Rust (Tier 1) |
| music | `music.tracker_song_v1` | XM/IT | Rust (Tier 1) |
| static_mesh | `static_mesh.blender_primitives_v1` | GLB | Blender (Tier 2) |
| skeletal_mesh | `skeletal_mesh.blender_rigged_mesh_v1` | GLB | Blender (Tier 2) |
| skeletal_animation | `skeletal_animation.blender_clip_v1` | GLB | Blender (Tier 2) |

**Tier 1** = byte-identical output (same spec + seed = same file)
**Tier 2** = metric-validated (depends on Blender version)

### Workflow

1. **Write spec**: Create Starlark (`.star`) using stdlib functions
2. **Validate**: `speccade validate --spec my_spec.json`
3. **Generate**: `speccade generate --spec my_spec.json --out-root ./output`
4. **Iterate**: Adjust params, regenerate

### CLI Commands

```bash
speccade validate --spec FILE          # Check spec validity
speccade generate --spec FILE          # Generate asset
speccade generate-all --spec-dir DIR   # Batch generate
speccade doctor                        # Check dependencies
speccade fmt --spec FILE               # Reformat spec
```

## Audio Specs

Audio specs use `audio_v1` recipe with layers, effects, and envelopes.

**Minimal structure:**
```json
{
  "recipe": {
    "kind": "audio_v1",
    "params": {
      "duration_seconds": 1.0,
      "sample_rate": 44100,
      "layers": [{
        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
        "envelope": { "attack": 0.01, "decay": 0.1, "sustain": 0.5, "release": 0.3 },
        "volume": 0.8
      }]
    }
  }
}
```

**16+ synthesis types available** - see `references/audio-synthesis.md`
**7 effects** - see `references/audio-effects.md`

## Texture Specs

Texture specs use procedural node graphs:

```json
{
  "recipe": {
    "kind": "texture.procedural_v1",
    "params": {
      "resolution": [512, 512],
      "tileable": true,
      "nodes": [
        { "id": "base", "type": "noise", "noise": { "type": "perlin", "scale": 4.0 } },
        { "id": "color", "type": "color_ramp", "input": "base", "stops": [...] }
      ]
    }
  }
}
```

**Node types** - see `references/texture-nodes.md`

## Music Specs

Music uses tracker format (XM/IT) with patterns and instruments.

### Basic Tracker Song (`music.tracker_song_v1`)

```json
{
  "recipe": {
    "kind": "music.tracker_song_v1",
    "params": {
      "format": "xm",
      "tempo": 125,
      "channels": 8,
      "patterns": [...],
      "instruments": [...]
    }
  }
}
```

### Compose IR (`music.tracker_song_compose_v1`)

For compact, operator-based authoring, use the Pattern IR format:

```json
{
  "recipe": {
    "kind": "music.tracker_song_compose_v1",
    "params": {
      "format": "xm",
      "bpm": 150,
      "channels": 8,
      "instruments": [...],
      "defs": { "kick_4": { "op": "emit", "at": {...}, "cell": {...} } },
      "patterns": { "verse": { "rows": 64, "program": { "op": "stack", "parts": [...] } } },
      "arrangement": [{ "pattern": "verse", "repeat": 4 }]
    }
  }
}
```

Key operators: `stack`, `emit`, `emit_seq`, `repeat`, `concat`, `ref`, `prob`, `choose`

**Tracker format** - see `references/music-tracker.md`
**Compose IR** - see `references/music-compose-ir.md`

## Mesh Specs

3D meshes require Blender (Tier 2):

```json
{
  "recipe": {
    "kind": "static_mesh.blender_primitives_v1",
    "params": {
      "primitive": "cube",
      "size": [1.0, 1.0, 1.0]
    }
  }
}
```

**Blender backends** - see `references/mesh-blender.md`

## Common Patterns

### Kick Drum
```json
"layers": [{
  "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 150.0,
    "frequency_sweep": { "end_frequency": 40.0, "duration": 0.15, "curve": "exponential" }
  },
  "envelope": { "attack": 0.001, "decay": 0.2, "sustain": 0.0, "release": 0.1 }
}]
```

### Hi-Hat
```json
"layers": [{
  "synthesis": { "type": "noise", "noise_type": "white" },
  "envelope": { "attack": 0.001, "decay": 0.05, "sustain": 0.0, "release": 0.05 },
  "filter": { "type": "highpass", "cutoff": 8000.0 }
}]
```

### Pad with Reverb
```json
"layers": [{
  "synthesis": { "type": "additive", "harmonics": [1.0, 0.5, 0.25, 0.125] },
  "envelope": { "attack": 0.5, "decay": 0.3, "sustain": 0.7, "release": 1.0 }
}],
"effects": [{ "type": "reverb", "room_size": 0.8, "wet": 0.4 }]
```

## Validation Errors

Common errors and fixes:

| Code | Meaning | Fix |
|------|---------|-----|
| E001 | Invalid asset_id format | Use lowercase, 3-64 chars |
| E002 | Missing required field | Add the field to spec |
| E003 | Recipe/asset type mismatch | Match recipe.kind to asset_type |
| E010 | Invalid synthesis params | Check synthesis type requirements |

## References

For detailed documentation:
- `references/audio-synthesis.md` - All 16+ synthesis types with params
- `references/audio-effects.md` - Effects chain reference
- `references/texture-nodes.md` - Procedural texture nodes
- `references/music-tracker.md` - XM/IT tracker format basics
- `references/music-compose-ir.md` - Pattern IR operators and authoring
- `references/mesh-blender.md` - Blender backend reference
- `references/spec-format.md` - Full JSON schema

## Character / Humanoid

For humanoid characters, use `skeletal_mesh_spec()` with `skeleton_preset = "humanoid_basic_v1"`. A blank-material template (single white material, all parts `material_index = 0`) is the standard starting point:

```starlark
skeletal_mesh_spec(
    asset_id = "my-humanoid-01",
    seed = 42,
    output_path = "characters/my_humanoid.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    body_parts = [
        body_part(bone = "spine", primitive = "cylinder", dimensions = [0.25,0.4,0.25], segments = 8, offset = [0,0,0.3], material_index = 0),
        # ... (see references/character-humanoid.md for full template)
    ],
    material_slots = [material_slot(name = "blank_white", base_color = [1.0,1.0,1.0,1.0])],
    skinning = skinning_config(max_bone_influences = 4, auto_weights = True),
    export = skeletal_export_settings(triangulate = True, include_skin_weights = True),
    constraints = skeletal_constraints(max_triangles = 5000, max_bones = 64, max_materials = 4),
    texturing = skeletal_texturing(uv_mode = "cylinder_project")
)
```

Full reference: [`references/character-humanoid.md`](references/character-humanoid.md) | [`docs/starlark-authoring.md`](../../../docs/starlark-authoring.md)

## Example Specs

Find working examples in `speccade/packs/preset_library_v1/`:
- `audio/drums/` - Kick, snare, hats
- `audio/bass/` - Bass sounds
- `audio/leads/` - Lead synths
- `audio/pads/` - Pad sounds
- `audio/fx/` - Risers, impacts, transitions
