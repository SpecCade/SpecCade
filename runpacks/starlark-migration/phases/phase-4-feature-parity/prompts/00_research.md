# Phase 4 Research Prompt

## Objective

Understand the full IR type surface area to plan stdlib coverage.

## Tasks

### 1. Audio IR Types

Read and document all types from:
- `crates/speccade-spec/src/audio/synthesis.rs` - All `Synthesis` enum variants
- `crates/speccade-spec/src/audio/filter.rs` - All `Filter` enum variants
- `crates/speccade-spec/src/audio/effect.rs` - All `Effect` enum variants
- `crates/speccade-spec/src/audio/lfo.rs` - LFO and modulation types
- `crates/speccade-spec/src/audio/envelope.rs` - Envelope types

For each type, extract:
- Variant name
- All fields with types
- Any validation constraints
- Default values

### 2. Texture IR Types

Read and document all types from:
- `crates/speccade-spec/src/texture/procedural.rs` - `TextureProceduralOp` enum

### 3. Mesh IR Types

Read and document all types from:
- `crates/speccade-spec/src/mesh/modifier.rs` - `MeshModifier` enum
- `crates/speccade-spec/src/mesh/material.rs` - Material types
- `crates/speccade-spec/src/mesh/uv.rs` - UV projection types

### 4. Music IR Types

Read and document all types from:
- `crates/speccade-spec/src/music/tracker.rs` - Tracker song types
- `crates/speccade-spec/src/music/compose.rs` - Compose DSL types

### 5. Existing Stdlib

Review current stdlib implementation:
- `crates/speccade-cli/src/compiler/stdlib/audio.rs`
- `crates/speccade-cli/src/compiler/stdlib/texture.rs`
- `crates/speccade-cli/src/compiler/stdlib/mesh.rs`

Document which functions exist and their signatures.

### 6. Budget System

Review:
- `crates/speccade-spec/src/validation/budgets.rs` - Budget types
- `crates/speccade-spec/src/validation/mod.rs` - How budgets are (not) enforced

## Deliverables

- `research.md` - Complete type catalog with all fields
- `risks.md` - Implementation risks and mitigations
