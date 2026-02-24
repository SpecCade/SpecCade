# SpecCade Architecture

Deterministic asset pipeline for procedural game asset generation.

> This file is a high-level map of crates + the determinism model.
> For usage and examples, start with `README.md`, `docs/README.md`, and `PARITY_MATRIX.md`.

## TL;DR (mental model)

SpecCade takes a `Spec` (authored in JSON or Starlark) and produces one or more artifacts (WAV/PNG/XM/IT/…) plus a `Report`.

- Starlark files (.star) are compiled to canonical JSON IR
- Validation and hashing live in `speccade-spec`
- The CLI (`speccade-cli`) validates + dispatches to a backend
- Tier 1 backends are Rust-only and aim for byte-identical output
- Tier 2 backends use external tools (Blender) and are validated differently

## Repo map

```
speccade/
├── crates/
│   ├── speccade-spec             # Core types, validation, hashing
│   ├── speccade-cli              # CLI entry point + dispatch
│   ├── speccade-backend-audio    # Audio/SFX generation (Tier 1)
│   ├── speccade-backend-music    # XM/IT tracker generation (Tier 1)
│   ├── speccade-backend-texture  # Procedural textures (Tier 1)
│   ├── speccade-backend-blender  # Mesh/animation via Blender (Tier 2)
│   ├── speccade-lint             # Semantic quality lints for generated assets
│   ├── speccade-editor           # Tauri editor backend (IPC + file watching)
│   ├── speccade-mcp              # MCP server for IDE/editor integration
│   └── speccade-tests            # Integration + determinism validation
├── blender/
│   ├── entrypoint.py             # Thin shim (imports speccade.main)
│   ├── structural_metrics.py     # LLM-friendly 3D feedback
│   └── speccade/                 # Modular Python package (23 modules)
├── editor/                       # Tauri editor frontend (TypeScript)
├── schemas/                      # JSON schemas
├── packs/                        # Example packs/inputs
├── specs/                        # Starlark spec files (organized by type)
├── stdlib/                       # Stdlib snapshot for drift detection
├── scripts/                      # Helper scripts (generate_all.sh/ps1)
└── docs/                         # Additional docs
```

## Crate Purposes

### speccade-spec
Core specification library. Defines JSON spec format, validation rules, and canonical hashing.

**Key modules:**
- `spec` - Main `Spec` type and builder
- `recipe/` - Recipe types for each backend (audio, music, texture, mesh, animation)
- `validation/` - Spec validation with error reporting
- `validation/budgets` - Budget enforcement system (profiles: default, strict, zx-8bit, nethercore)
- `hash` - BLAKE3-based canonical hashing and seed derivation
- `report` - Generation result reporting

### speccade-cli
Command-line tool for spec operations.

**Key modules:**
- `compiler/` - Starlark-to-JSON compiler pipeline
- `compiler/stdlib/` - Starlark stdlib functions (core, audio, music, texture, mesh, character, animation)
- `input` - Unified spec loading (JSON/Starlark dispatch by extension)
- `dispatch/` - Routes specs to appropriate backends
- `commands/` - CLI command implementations (eval, validate, generate, fmt, lint, etc.)

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

### blender/ (Python)
Python scripts executed by Blender subprocess. Entry point is `blender/entrypoint.py` which delegates to the `blender/speccade/` package.

**Package structure (`blender/speccade/`):**
- `main.py` - CLI parsing and handler dispatch
- `report.py` - Report generation
- `scene.py` - Scene setup and primitives
- `metrics.py` - Mesh/skeletal/animation metrics
- `modifiers.py` - Blender modifier application
- `uv_mapping.py` - UV projection and texel density
- `normals.py` - Normal/shading settings
- `materials.py` - Material creation
- `skeleton_presets.py` - Humanoid bone preset constants
- `skeleton.py` - Armature creation
- `body_parts.py` - Mesh part creation and skinning
- `ik_fk.py` - IK chains and IK/FK switching
- `constraints.py` - Bone constraints
- `controls.py` - Space switch, fingers, foot system
- `animation.py` - Animation creation and baking
- `rig_config.py` - Widgets, bone collections, colors
- `export.py` - GLB export, LOD, collision
- `rendering.py` - Camera, preview, atlas
- `handlers_mesh.py` - Static mesh handlers
- `handlers_skeletal.py` - Skeletal mesh handlers
- `handlers_render.py` - Render/validation handlers

### speccade-tests
Integration tests and determinism validation framework.

**Key modules:**
- `determinism/` - Determinism verification framework
- `format_validators/` - Binary format validators (WAV, PNG, XM, IT, glTF)
- `audio_analysis/` - Audio signal analysis utilities
- `harness` - Test harness utilities

### speccade-lint
Semantic quality lint rules and analyzers for generated assets.

Used by the CLI `speccade lint` subcommand.

### speccade-editor
Editor backend crate used by the Tauri app under `editor/`.

Wraps `speccade-cli` for eval/validate/generate workflows and wires filesystem watching + previews.

### speccade-mcp
Model Context Protocol (MCP) server for IDE and editor integration. Exposes SpecCade operations as MCP tools over stdio transport using `rmcp`.

**Tools exposed:**
- **Discovery:** `stdlib_reference`, `list_templates`, `get_template`, `list_specs`, `read_spec`
- **Authoring:** `write_spec`, `eval_spec`
- **Generation:** `validate_spec`, `generate_preview`, `generate_full`, `generate_png_outputs`
- **Analysis:** `analyze_asset`, `compare_assets`

**Architecture:** Delegates to `speccade-cli` commands via subprocess (`cli_runner`). This keeps the MCP server thin and ensures parity with CLI behavior.

**Usage:** `speccade-mcp --project-dir <path>` — runs as a stdio MCP server for tools like Claude Code, VS Code extensions, or any MCP-compatible client.

## Starlark Compilation Pipeline

SpecCade supports authoring specs in Starlark (.star files) which compile to canonical JSON IR:

```
.star file
    -> crates/speccade-cli/src/input.rs (load_spec) - dispatches by extension
    -> crates/speccade-cli/src/compiler/eval.rs - Starlark evaluation with timeout
       ├── stdlib registered (audio, texture, mesh, music, core)
       ├── timeout enforced (default: 30s)
       └── load() disabled by default (enable_load=false)
    -> crates/speccade-cli/src/compiler/convert.rs - starlark::Value -> serde_json::Value
    -> speccade-spec::Spec::from_value() - parse canonical IR
    -> crates/speccade-spec/src/validation/budgets.rs - enforce resource limits (optional)
    -> Backend generation (same as JSON path)
```

**Stdlib modules** (`compiler/stdlib/`):
- `core.rs` - spec(), output() scaffolding functions
- `audio/` - envelope(), oscillator(), fm_synth(), filter(), effect(), layer()
- `music/` - instrument(), pattern(), song() tracker composition
- `texture/` - noise_node(), gradient_node(), graph() procedural textures
- `mesh/` - mesh_primitive(), mesh_recipe() mesh generation
- `character/` - skeletal mesh authoring helpers
- `animation/` - skeletal animation authoring helpers

**Budget system** (`validation/budgets.rs`):
- Profiles: `default`, `strict`, `zx-8bit`, `nethercore`
- Per-asset limits: AudioBudget, TextureBudget, MusicBudget, MeshBudget, GeneralBudget
- Enforced at validation stage before generation
- CLI flag: `--budget <profile>`

## Module Dependencies

```
speccade-cli
    ├── speccade-spec
    ├── speccade-backend-audio
    ├── speccade-backend-music
    ├── speccade-backend-texture
    └── speccade-backend-blender

speccade-lint
    └── speccade-spec

speccade-editor
    ├── speccade-cli
    ├── speccade-spec
    └── speccade-lint

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
| CLI main | `crates/speccade-cli/src/main.rs` |
| Load spec (JSON/Starlark) | `speccade-cli::input::load_spec()` |
| Starlark compilation | `speccade-cli::compiler::compile()` |
| Audio generation | `speccade-backend-audio::generate()` |
| Music generation | `speccade-backend-music::generate_music()` |
| Texture generation | `speccade-backend-texture::generate_graph()` |
| Spec validation | `speccade-spec::validate_spec()` |
| Budget validation | `speccade-spec::validation::budgets::BudgetProfile` |

## Determinism Tiers

- **Tier 1** (Rust-only): Audio, music, texture - byte-identical output guaranteed
- **Tier 2** (Blender subprocess): Mesh, animation - validated by metrics; output can vary by OS/Blender version

## Key invariants (read before changing behavior)

- **Hashing drives determinism:** canonical hashing and seed derivation live in `speccade-spec` (`hash` module). Backends should only use derived seeds, not OS RNG/time.
- **Stable iteration matters:** avoid nondeterministic ordering (e.g., unordered map iteration) in any path that affects output bytes.
- **Tier 2 is special:** Blender output can vary with platform/Blender version; treat Tier 2 as “validated, not byte-identical”.

## Glossary

- **Spec**: the canonical JSON document describing *what* to generate.
- **Recipe**: backend-specific parameters inside the spec.
- **Backend**: a generator crate (audio/music/texture/blender).
- **Tier 1 / Tier 2**: determinism guarantees (Rust-only vs external tool).
- **Report**: structured summary of what was generated (paths, hashes, metrics).
