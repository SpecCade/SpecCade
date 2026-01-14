# SpecCade Architecture

Deterministic asset pipeline for procedural game asset generation.

## Crate Overview

```
speccade/
├── speccade-spec          # Core types, validation, hashing
├── speccade-cli           # Command-line interface
├── speccade-backend-audio # Audio/SFX generation (Tier 1)
├── speccade-backend-music # XM/IT tracker generation (Tier 1)
├── speccade-backend-texture # Procedural textures (Tier 1)
├── speccade-backend-blender # Mesh/animation via Blender (Tier 2)
└── speccade-tests         # Integration tests, determinism validation
```

## Crate Purposes

### speccade-spec
Core specification library. Defines JSON spec format, validation rules, and canonical hashing.

**Key modules:**
- `spec` - Main `Spec` type and builder
- `recipe/` - Recipe types for each backend (audio, music, texture, mesh, animation)
- `validation/` - Spec validation with error reporting
- `hash` - BLAKE3-based canonical hashing and seed derivation
- `report` - Generation result reporting

### speccade-cli
Command-line tool for spec operations.

**Key modules:**
- `dispatch/` - Routes specs to appropriate backends
- `commands/` - CLI command implementations (generate, validate, fmt, migrate)

### speccade-backend-audio
Generates WAV files from synthesis specifications.

**Synthesis types:** FM, Karplus-Strong, oscillators, noise, additive, modal, vocoder, formant, vector

**Key modules:**
- `generate/` - Main generation pipeline
- `synthesis/` - Synthesis algorithm implementations
- `mixer/` - Layer mixing with volume/pan
- `wav/` - Deterministic WAV writer
- `filter` - Biquad filters
- `envelope` - ADSR envelopes

### speccade-backend-music
Generates XM (FastTracker II) and IT (Impulse Tracker) module files.

**Key modules:**
- `generate/` - Main generation entry point
- `compose/` - Pattern expansion and composition DSL
- `xm/` - XM format writer and validator
- `it/` - IT format writer and validator
- `xm_gen/`, `it_gen/` - Format-specific generation
- `synthesis/` - Instrument sample synthesis
- `note/` - Note/frequency conversion

### speccade-backend-texture
Generates procedural textures as PNG files.

**Key modules:**
- `generate/` - Main generation and node graph evaluation
- `noise/` - Noise primitives (Perlin, Simplex, Worley, FBM)
- `pattern/` - Pattern generators (brick, checker, wood, scratches)
- `normal_map/` - Normal map generation from height
- `packing/` - Channel packing (ORM maps)
- `maps/` - Texture buffer types

### speccade-backend-blender
Generates meshes and animations via Blender subprocess (Tier 2).

**Key modules:**
- `static_mesh` - Static mesh generation
- `skeletal_mesh` - Rigged character meshes
- `animation` - Skeletal animation clips

### speccade-tests
Integration tests and determinism validation framework.

**Key modules:**
- `determinism/` - Determinism verification framework
- `format_validators/` - Binary format validators (WAV, PNG, XM, IT, glTF)
- `audio_analysis/` - Audio signal analysis utilities
- `harness` - Test harness utilities

## Module Dependencies

```
speccade-cli
    ├── speccade-spec
    ├── speccade-backend-audio
    ├── speccade-backend-music
    ├── speccade-backend-texture
    └── speccade-backend-blender

speccade-backend-* (all)
    └── speccade-spec

speccade-tests
    ├── speccade-spec
    ├── speccade-backend-audio
    ├── speccade-backend-music
    └── speccade-backend-texture
```

## Key Types

| Type | Crate | Purpose |
|------|-------|---------|
| `Spec` | speccade-spec | Main specification container |
| `Recipe` | speccade-spec | Backend-specific parameters |
| `Report` | speccade-spec | Generation result with metrics |
| `GenerateResult` | backend-audio | Audio generation output |
| `GenerateResult` | backend-music | Music generation output |
| `TextureResult` | backend-texture | Texture generation output |

## Entry Points

| Operation | Entry Point |
|-----------|-------------|
| CLI main | `speccade-cli/src/main.rs` |
| Audio generation | `speccade-backend-audio::generate()` |
| Music generation | `speccade-backend-music::generate_music()` |
| Texture generation | `speccade-backend-texture::generate_graph()` |
| Spec validation | `speccade-spec::validate_spec()` |

## Determinism Tiers

- **Tier 1** (Rust-only): Audio, music, texture - byte-identical output guaranteed
- **Tier 2** (Blender subprocess): Mesh, animation - hash-validated but platform-dependent

## File Statistics

- **360 source files** across all crates
- **Largest implementation file:** ~575 lines (harmony.rs)
- **All implementation files:** ≤600 lines (test files may be larger)
