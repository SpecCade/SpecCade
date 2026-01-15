# SpecCade Development Plugin

Claude Code plugin for SpecCade asset development. Provides contextual knowledge and autonomous agents for authoring deterministic asset specs.

## Features

- **Skill**: `speccade-authoring` - Comprehensive knowledge of spec format, synthesis types, effects, and workflows
- **Agent**: `spec-architect` - Proactively designs and creates specs from high-level descriptions
- **Agent**: `spec-debugger` - Diagnoses validation errors and generation issues

## Installation

### Global Installation
```bash
claude --plugin-dir /path/to/speccade-dev-plugin
```

### Per-Project
Copy to your project's `.claude-plugin/` directory.

### Add to settings.json
```json
{
  "plugins": [
    "/path/to/speccade-dev-plugin"
  ]
}
```

## Usage

### Creating Assets

Just describe what you want:

> "Create a punchy kick drum with sub bass"

> "Design a rough stone texture for a dungeon wall"

> "Make a synth pad with chorus and reverb"

The `spec-architect` agent will automatically:
1. Research existing patterns
2. Design appropriate spec
3. Validate and optionally generate

### Debugging Issues

When you encounter errors:

> "Why is my spec failing validation?"

> "Fix this E010 error"

> "The sound is clipping, help me fix it"

The `spec-debugger` agent will:
1. Analyze the error
2. Identify root cause
3. Suggest minimal fix
4. Verify the solution

### Direct Questions

For specific knowledge:

> "What synthesis types are available?"

> "How do I add reverb to my sound?"

> "What's the texture node format?"

The `speccade-authoring` skill provides immediate answers.

## Asset Types

| Type | Recipe | Output | Backend |
|------|--------|--------|---------|
| audio | `audio_v1` | WAV | Rust (Tier 1) |
| texture | `texture.procedural_v1` | PNG | Rust (Tier 1) |
| music | `music.tracker_song_v1` | XM/IT | Rust (Tier 1) |
| static_mesh | `static_mesh.blender_primitives_v1` | GLB | Blender (Tier 2) |
| skeletal_mesh | `skeletal_mesh.blender_rigged_mesh_v1` | GLB | Blender (Tier 2) |
| skeletal_animation | `skeletal_animation.blender_clip_v1` | GLB | Blender (Tier 2) |

## Quick Reference

### CLI Commands
```bash
speccade validate --spec FILE      # Validate spec
speccade generate --spec FILE      # Generate asset
speccade doctor                    # Check dependencies
```

### Spec Structure
```json
{
  "spec_version": 1,
  "asset_id": "my_asset",
  "asset_type": "audio",
  "license": "CC0-1.0",
  "seed": 12345,
  "outputs": [{"kind": "primary", "format": "wav", "path": "out.wav"}],
  "recipe": {"kind": "audio_v1", "params": {...}}
}
```

## References

Detailed documentation in `skills/speccade-authoring/references/`:
- `audio-synthesis.md` - 16+ synthesis types
- `audio-effects.md` - Effect chain reference
- `texture-nodes.md` - Procedural node types
- `music-tracker.md` - XM/IT tracker format
- `mesh-blender.md` - Blender backend
- `spec-format.md` - Full JSON schema

## Requirements

- Claude Code CLI
- SpecCade repository (for generation)
- Blender 3.x/4.x (for mesh generation, optional)

## License

MIT
