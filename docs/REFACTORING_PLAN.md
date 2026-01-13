# SpecCade Refactoring Plan (2026-01)

This plan is the working agreement for refactoring the `speccade/` workspace with the goals of:

- **SSOT**: one canonical definition per concept (spec schema, recipe kind strings, seed derivation, packing types).
- **DRY/KISS**: remove duplicated logic and “stringly-typed” glue where practical.
- **Reliability**: make tests and validation match how the repo is actually used (golden specs, current recipe kinds).
- **Doc accuracy**: update docs/schema/scripts so they match the codebase as it exists *now*.

## 0. Non-goals (for this pass)

- Rewriting the whole spec type system to be fully typed (e.g., `Recipe { kind: RecipeKind, params: RecipeParams }`).
- Implementing full legacy `.studio` → canonical param mapping (migration currently passes through legacy dicts).
- Redesigning determinism policy or changing hashing/seed derivation algorithms (would invalidate goldens).

## 1. Canonical Naming (SSOT)

The repo already uses (and the golden corpus demonstrates) the **new canonical** naming:

- Asset types: `audio`, `music`, `texture`, `static_mesh`, `skeletal_mesh`, `skeletal_animation`
- Recipe kinds:
  - `audio_v1`
  - `music.tracker_song_v1`
  - `texture.procedural_v1`
  - Blender: `static_mesh.blender_primitives_v1`, `skeletal_mesh.blender_rigged_mesh_v1`, `skeletal_animation.blender_clip_v1`

Legacy names (`audio_sfx.*`, `audio_instrument.*`, `texture_2d.*`) are now treated as **removed API**: they are rejected by parsing/validation and are not supported by any backend dispatch.

Deliverables:

- Update CLI dispatch/doctor/migrate/scripts/tests/docs/schema to prefer the canonical names above.
- Remove legacy alias support to keep the public API clean and unambiguous.

## 2. Determinism Utilities (DRY)

Problems observed:

- Multiple crates re-implement identical BLAKE3 seed derivation (`derive_layer_seed`, `derive_variant_seed`).

Plan:

- Make `speccade-spec` the SSOT for seed derivation helpers.
- Update backends to call `speccade_spec::hash::*` for seed derivation instead of duplicating logic.
- Keep backend-local RNG wrappers (`Pcg32` init, `DeterministicRng`) but remove duplicated seed-hash code.

## 3. Procedural Packing (RFC alignment)

Problems observed:

- `texture.packed_v1` and `outputs[].channels` were a pre-graph workaround.
- Channel packing should be expressed as graph composition (e.g., `compose_rgba`).

Plan:

- Treat `texture.procedural_v1` as the only canonical texture recipe kind.
- Remove `outputs[].kind = "packed"` and `outputs[].channels` from schema/docs/validation.
- Document packing as a graph pattern (build grayscale maps -> `compose_rgba` -> output PNG).

## 4. Workspace Hygiene (Cargo / deps)

Problems observed:

- Workspace defines shared dependencies, but several crates pin their own versions.

Plan:

- Normalize all crate `Cargo.toml` files to use `[workspace.dependencies]` where possible.
- Add missing shared deps (e.g., `hound`) to the workspace list if used in multiple crates.

## 5. Tests (Correctness + practicality)

Problems observed:

- `speccade-tests` harness runs `cargo run` from a temp directory without `--manifest-path`.
- E2E tests and fixtures must reference the canonical golden directories (`audio/`, `music/`, `texture/`, etc.).

Plan:

- Fix the harness to run with an explicit `--manifest-path` pointing at `speccade/Cargo.toml` while still writing outputs to the temp work dir.
- Update E2E tests to reference the actual golden directories and canonical recipe kinds.

## 6. Documentation + Schema Accuracy

Problems observed:

- `schemas/speccade-spec-v1.schema.json` and multiple docs still describe pre-refactor names and omit `texture.procedural_v1`.

Plan:

- Update `schemas/speccade-spec-v1.schema.json` to match the canonical API used by code and golden specs.
- Update docs (`README.md`, `docs/SPEC_REFERENCE.md`, `docs/spec-reference/*`, `docs/MIGRATION.md`, `docs/CONTRIBUTING.md`, RFCs) to:
  - Use canonical asset/recipe names.
  - Clearly mark deprecated/legacy names where relevant.
  - Match current CLI behavior (`validate`, `generate`, `generate-all`, `doctor`, `migrate`).
