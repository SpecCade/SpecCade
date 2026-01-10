# SpecCade Legacy Parity - Task Orchestration

This is a copy/paste-ready orchestration plan (Claude Code “Task tool” YAML blocks) for closing parity gaps between legacy `.spec.py` keys inventoried in `PARITY_MATRIX.md` and the current SpecCade implementation.

## Scope + Assumptions

- Repo: `speccade/` (this workspace).
- `PARITY_MATRIX.md` is the legacy key inventory SSOT.
- The golden corpus in `golden/` should not be broken without an explicit golden update step (even if golden gates are not wired up yet).
- Blender work is Tier 2 (metrics), so CI/dev machines must have Blender available (PATH or `BLENDER_PATH`).

## Definitions

### Implementation Status (`PARITY_MATRIX.md` → `Status` column)

- `✓` — Implemented end-to-end (spec supports it + backend behavior exists + validation/tests where applicable)
- `~` — Partial (spec field exists but backend ignores; backend exists but legacy behavior not fully matched; missing tests/validation)
- `✗` — Not implemented
- `-` — Deprecated / intentionally unsupported

### Key Identity (avoid collisions)

The same `Key` strings appear in multiple tables (e.g., `name`, `type`, `rotation`), so treat each parity row as a **fully-qualified key**:

`<SECTION>::<TABLE>::<KEY>`

Examples:
- `SOUND::Top-Level Keys::name`
- `SOUND::Layer Keys::type`
- `ANIMATION::IK Chain Keys::target.name`

Only markdown tables whose header row contains a `Key` column participate in parity status tracking. (Do not add `Status` columns to non-key tables like “Preset Names” or the “Randomness Audit” tables.)

## Task Structure

Each feature task follows this pattern:
1. **Implement** — Write the feature code
2. **Test** — Write tests (unit + integration where appropriate)
3. **Validate** — Run tests, fix failures
4. **Review** — Check correctness and refactor opportunities

## Execution Gates (non-negotiable)

- Rust tasks: `cargo fmt` + relevant `cargo test` must pass.
- Blender-dependent tests must be opt-in (`#[ignore]` by default) and only run when Blender is available (e.g., `SPECCADE_RUN_BLENDER_TESTS=1 cargo test ... -- --ignored`).
- Do not execute untrusted legacy `.spec.py` unless explicitly opted-in (the CLI already has `--allow-exec-specs`).

---

## Phase 1: PARITY_MATRIX Update

### Task 1.1: Update Status Column
```yaml
subagent_type: general-purpose
model: opus
description: Update PARITY_MATRIX status
prompt: |
  Update PARITY_MATRIX.md to add implementation status tracking.

  1. Add a "Status" column as the LAST column to every *key inventory* table (tables whose header contains a `Key` column).
     - Do NOT add Status to non-key tables (e.g. "Preset Names") or Randomness Audit tables.
  2. For each key row, determine status by reading actual implementation code (not just type definitions):
     - crates/speccade-spec/src/recipe/*.rs
     - crates/speccade-backend-*/src/*.rs
     - blender/entrypoint.py
  3. Mark status as: ✓ (implemented), ~ (partial), ✗ (not implemented), - (deprecated)
  4. After each asset SECTION (SOUND/INSTRUMENT/SONG/etc), add an "Implementation Status" summary block:
     - Count rows across all key tables in that section
     - Report counts for ✓/~ /✗/- and a % implemented (✓ only, excluding deprecated `-`)

  Be thorough - check actual code, not just type definitions.
```

### Task 1.2: Validate PARITY_MATRIX
```yaml
subagent_type: general-purpose
model: sonnet
description: Validate PARITY_MATRIX accuracy
prompt: |
  Validate PARITY_MATRIX.md status column accuracy.

  1. Read PARITY_MATRIX.md
  2. For each section, pick 3 keys marked ✓ and verify they're actually implemented (end-to-end)
  3. Pick 3 keys marked ✗ and verify they're actually missing
  4. Check that percentages in summaries are correct
  5. Report any discrepancies found
  6. Fix any incorrect status markings
```

---

## Phase 2: Parity Data + Migrator (Audit/Reporting)

### Task 2.1: Create Parity Parser
```yaml
subagent_type: general-purpose
model: opus
description: Create parity matrix parser
prompt: |
  Create a parity-data generator that parses PARITY_MATRIX.md at build-time.

  Create:
  - crates/speccade-cli/build.rs
  - crates/speccade-cli/src/parity_matrix.rs (parser logic, so it can be unit-tested)
  - crates/speccade-cli/src/parity_data.rs (module that `include!`s generated data)

  1. Read ../../PARITY_MATRIX.md
     - Use CARGO_MANIFEST_DIR to build an absolute path (do not assume current working directory).
     - Add `cargo:rerun-if-changed=../../PARITY_MATRIX.md`.
  2. Parse ONLY tables whose header contains a `Key` column.
     - Track SECTION from `## ...` headings (e.g. "SOUND (audio_sfx)").
     - Track TABLE from the last `### ...` heading before a table (e.g. "Layer Keys").
  3. Extract fields for each key row:
     - section (string)
     - table (string)
     - key (string, strip backticks)
     - required (bool): treat anything starting with "Yes" as required (e.g. "Yes (or mirror)")
     - status: ✓/~ /✗/- (from the new Status column)
  4. Generate OUT_DIR/parity_data.rs containing:
     - enum KeyStatus { Implemented, Partial, NotImplemented, Deprecated }
     - struct ParityKey { section: &'static str, table: &'static str, key: &'static str }
     - struct KeyInfo { key: ParityKey, required: bool, status: KeyStatus }
     - static ALL_KEYS: &[KeyInfo]
     - helper: fn find(section, table, key) -> Option<&'static KeyInfo> (linear search is fine to start)
  5. In crates/speccade-cli/src/parity_data.rs, `include!` the generated file and re-export helpers/types.
  6. Update crates/speccade-cli/Cargo.toml to enable the build script.
```

### Task 2.1-test: Test Parity Parser
```yaml
subagent_type: general-purpose
model: sonnet
description: Test parity parser
prompt: |
  Write unit tests for the parity matrix parser (do NOT test build.rs directly).

  Add tests to: crates/speccade-cli/src/parity_matrix.rs #[cfg(test)]

  Tests:
  1. test_parse_minimal_table: parses a small markdown snippet with a Key+Required+Status table
  2. test_required_parsing: "Yes" and "Yes (or mirror)" => required=true, "No" => false
  3. test_status_parsing: verify ✓/~/✗/- are parsed correctly
  4. test_qualified_identity: section+table+key are captured (no collisions)

  Run: cargo test -p speccade-cli parity_matrix
```

### Task 2.1-validate: Run and Fix
```yaml
subagent_type: Bash
model: sonnet
description: Run parity parser tests
prompt: |
  cd speccade
  cargo build -p speccade-cli
  cargo test -p speccade-cli parity_matrix

  If tests fail, read the error output and report what needs fixing.
```

### Task 2.2: Integrate into Migrator
```yaml
subagent_type: general-purpose
model: opus
description: Integrate parity data into migrator
prompt: |
  Update crates/speccade-cli/src/commands/migrate.rs to use parity data.

  1. Add a key-classification step for legacy keys discovered in each spec dict.
     - Start with TOP-LEVEL keys only (legacy.data keys).
     - (Optional follow-up) recurse into nested dict/list structures and emit dotted keys (e.g. `layers[].type`) once parity data supports it.
  2. Create MigrationKeyStatus enum with: Implemented, Partial, NotImplemented, Deprecated, Unknown
  3. For each migrated file, compute:
     - counts by MigrationKeyStatus
     - gap score = (implemented + 0.5*partial) / (total_used - deprecated)
  4. Update print_migration_report() to show:
     - per-file: implemented/partial/missing/unknown counts + gap score
     - overall: aggregated counts + gap score
  5. Keep existing behavior: migration still writes a canonical JSON spec even if warnings exist.
```

### Task 2.2-test: Test Migrator Integration
```yaml
subagent_type: general-purpose
model: sonnet
description: Test migrator integration
prompt: |
  Write tests for migrator parity integration.

  Add to: crates/speccade-cli/src/commands/migrate.rs #[cfg(test)]

  Tests:
  1. test_classify_implemented_key: key marked ✓ returns Implemented
  2. test_classify_missing_key: key marked ✗ returns NotImplemented
  3. test_classify_unknown_key: key not in matrix returns Unknown
  4. test_gap_score_calculation: verify percentage math

  Run: cargo test -p speccade-cli migrate
```

### Task 2.2-validate: Run and Fix
```yaml
subagent_type: Bash
model: sonnet
description: Run migrator tests
prompt: |
  cd speccade
  cargo test -p speccade-cli migrate

  If tests fail, report errors.
```

### Task 2.3: Add Audit Flag
```yaml
subagent_type: general-purpose
model: opus
description: Add migrate audit flag
prompt: |
  Add --audit flag to migrate command.

  1. Update crates/speccade-cli/src/main.rs CLI args:
     - add --audit (bool)
     - add --audit-threshold (float, default 0.90)
  2. When --audit:
     - scan .studio/specs/** for *.spec.py (reuse existing WalkDir logic)
     - parse specs using the existing allow_exec_specs flag (static by default, exec only with explicit opt-in)
     - collect legacy keys used across all specs (top-level keys at minimum)
     - report aggregate completeness using parity status
  3. Print "Top missing keys" sorted by frequency across specs (include qualified key identity if available).
  4. Exit codes:
     - 0 if completeness >= threshold
     - 1 otherwise
     - still return 1 on hard failures (I/O, parse errors)
```

### Task 2.3-test: Test Audit
```yaml
subagent_type: general-purpose
model: sonnet
description: Test audit flag
prompt: |
  Write tests for audit functionality.

  Tests:
  1. test_audit_empty_dir: no specs returns 100%
  2. test_audit_single_spec: verify key collection
  3. test_audit_multiple_specs: verify aggregation
  4. test_audit_report_format: verify output format

  Use temp dirs + minimal inline .spec.py fixtures (no repo-wide tests/fixtures directory exists today).
```

### Task 2.3-validate: Run and Fix
```yaml
subagent_type: Bash
model: sonnet
description: Run audit tests
prompt: |
  cd speccade
  cargo test -p speccade-cli audit

  If tests fail, report errors.
```

### Task 2-review: Review Phase 2
```yaml
subagent_type: general-purpose
model: opus
description: Review Phase 2 implementation
prompt: |
  Review all Phase 2 code for correctness and refactoring.

  Read:
  - crates/speccade-cli/build.rs
  - crates/speccade-cli/src/parity_matrix.rs
  - crates/speccade-cli/src/parity_data.rs
  - crates/speccade-cli/src/commands/migrate.rs

  Check:
  1. Error handling - are all errors properly propagated?
  2. Edge cases - empty files, malformed markdown?
  3. Performance - any unnecessary allocations?
  4. Code style - consistent with rest of codebase?
  5. Documentation - are functions documented?

  Report issues and suggest fixes. Apply any simple fixes directly.
```

---

## Phase 3: Feature Implementation (PARALLEL)

### Track 3A: Music WAV Samples

#### Task 3A-impl
```yaml
subagent_type: general-purpose
model: sonnet
description: Music WAV sample loading
prompt: |
  Implement WAV sample loading for `music.tracker_song_v1` instruments.

  Files (likely):
  - crates/speccade-backend-music/src/generate.rs
  - crates/speccade-backend-music/src/synthesis.rs (if you place helpers here)
  - crates/speccade-cli/src/dispatch.rs (to pass spec_dir/spec_path through)

  1. Add a WAV loader (use `hound`) that returns 16-bit PCM *bytes* (Vec<u8>, little-endian i16), mono.
     - If stereo/multi-channel: convert to mono deterministically (average channels).
     - Support 16-bit PCM at minimum; clearly error on unsupported formats.
  2. Add deterministic resampling to DEFAULT_SAMPLE_RATE (22050 Hz) when the WAV sample_rate differs.
     - Use a deterministic algorithm (e.g., linear interpolation).
  3. Wire it into `generate_xm_instrument()` and `generate_it_instrument()` for `InstrumentSynthesis::Sample { path, base_note }`.
  4. Fix the missing plumbing for spec-relative paths:
     - `speccade_backend_music::generate_music` currently cannot resolve paths because it receives only (params, seed).
     - Update the API to also accept a base directory (e.g., `spec_dir: &Path`) and pass it from the CLI dispatch layer (the CLI knows the spec file path).
  5. Use `base_note` (if present) to set appropriate pitch reference for XM/IT (at minimum: don’t ignore it; document any limitations).
```

#### Task 3A-test
```yaml
subagent_type: general-purpose
model: sonnet
description: Test WAV loading
prompt: |
  Write tests for WAV sample loading.

  Add to: crates/speccade-backend-music (wherever the loader/resampler lives) #[cfg(test)]

  Tests:
  1. test_load_wav_mono: load mono WAV file
  2. test_load_wav_stereo: load stereo, verify conversion to mono
  3. test_resample_wav: verify resampling works
  4. test_wav_path_resolution: verify spec-relative resolution (base dir + relative path)
  5. test_wav_not_found: verify error handling

  Prefer generating WAV fixtures at test-time using `hound` + `tempfile` (avoid repo-wide fixtures dirs).
```

#### Task 3A-validate
```yaml
subagent_type: Bash
model: sonnet
description: Run WAV tests
prompt: |
  cd speccade
  cargo test -p speccade-backend-music

  Report any failures.
```

### Track 3B: Procedural Character Modeling

#### Task 3B-impl
```yaml
subagent_type: general-purpose
model: sonnet
description: Procedural character modeling
prompt: |
  Implement legacy-style procedural character modeling (SPEC/CHARACTER "parts"/"steps") without breaking the existing v1 skeletal mesh recipe.

  IMPORTANT: The current recipe `skeletal_mesh.blender_rigged_mesh_v1` uses `BodyPartMesh` primitives and `SkeletonPreset`.
  Do NOT silently change that schema. Add a new recipe kind instead (e.g. `skeletal_mesh.blender_rigged_mesh_v2` or a new `skeletal_mesh.blender_procedural_*` kind).

  Rust (speccade-spec):
  - Add new types that map cleanly to PARITY_MATRIX.md "Part Keys" and "Step Keys" (BaseShape, ExtrusionStep, ProceduralBodyPart, etc.).
  - Update recipe parsing/validation + `RecipeKind` enum to include the new kind.

  Blender (blender/entrypoint.py + backend-blender):
  - Teach the skeletal mesh generation path to handle the new recipe params.
  - Implement: extrude, scale, bulge, tilt, translate, rotate.

  Reference PARITY_MATRIX.md "Part Keys" and "Step Keys".
```

#### Task 3B-test
```yaml
subagent_type: general-purpose
model: sonnet
description: Test procedural modeling
prompt: |
  Write tests for procedural character modeling.

  Rust unit tests (speccade-spec):
  1. test_base_shape_serialization
  2. test_extrusion_step_defaults
  3. test_procedural_body_part_validation (if validation exists)

  Blender integration tests:
  - Add an opt-in integration test (#[ignore]) that runs the Blender backend on a minimal procedural spec and verifies:
    - output GLB file exists
    - report metrics parse and are reasonable

  Put Blender-running tests behind an env gate (Blender is not guaranteed in CI/dev environments).
```

#### Task 3B-validate
```yaml
subagent_type: Bash
model: sonnet
description: Run procedural tests
prompt: |
  cd speccade
  cargo test -p speccade-spec
  # Opt-in Blender integration tests (only if Blender is installed):
  SPECCADE_RUN_BLENDER_TESTS=1 cargo test -p speccade-backend-blender -- --ignored

  Report any failures.
```

### Track 3C: Bone System

#### Task 3C-impl
```yaml
subagent_type: general-purpose
model: sonnet
description: Bone system with auto-connection
prompt: |
  Implement a canonical/custom bone system (legacy SPEC skeleton list) without breaking the existing v1 skeleton preset API.

  IMPORTANT: Current Blender pipeline uses `SkeletonPreset` + `create_armature(preset_name: str)` in blender/entrypoint.py.
  Add a new recipe kind (or v2) that supports custom bones.

  Rust (speccade-spec):
  - Add BoneDef/SkeletonDef types matching PARITY_MATRIX.md "Skeleton Bone Keys" (+ any needed extensions like `connected`).
  - Define the exact semantics for mirroring and connected bones.

  Blender:
  - Extend/replace armature creation to support custom skeleton definitions.
  - Keep preset skeletons working as-is for v1 specs.
```

#### Task 3C-test
```yaml
subagent_type: general-purpose
model: sonnet
description: Test bone system
prompt: |
  Write tests for bone system.

  Rust tests:
  1. test_bone_def_connected
  2. test_skeleton_def_preset_vs_custom
  3. test_bone_mirror

  Blender integration test (opt-in, #[ignore]):
  - Generate a minimal skeleton and verify connected bones have correct head/tail positions (via metrics/report or simple invariants).
```

#### Task 3C-validate
```yaml
subagent_type: Bash
model: sonnet
description: Run bone tests
prompt: |
  cd speccade
  cargo test -p speccade-spec
  SPECCADE_RUN_BLENDER_TESTS=1 cargo test -p speccade-backend-blender -- --ignored

  Report any failures.
```

### Track 3D: Blend Export

#### Task 3D-impl
```yaml
subagent_type: general-purpose
model: sonnet
description: Blend file export
prompt: |
  Implement save_blend option.

  Files:
  - crates/speccade-spec/src/recipe/animation.rs
  - crates/speccade-spec/src/recipe/character.rs
  - blender/entrypoint.py
  - crates/speccade-backend-blender/src/{skeletal_mesh,animation}.rs (as needed)
  - crates/speccade-spec/src/output.rs (if adding a Blend output format)

  1. Add `export.save_blend: bool` (default false) to the relevant recipe params.
  2. Decide + implement how the `.blend` output is represented:
     Option A (preferred): add `OutputFormat::Blend` and allow an additional output spec with `format: "blend"`.
     Option B: emit an undeclared sidecar `<primary>.blend` and surface it only in the report.
  3. In Blender: if save_blend, call `bpy.ops.wm.save_as_mainfile(filepath=...)` AFTER GLB export.
  4. Ensure the Rust Blender backend reports the blend output (either as an OutputResult or a dedicated report field).
```

#### Task 3D-test
```yaml
subagent_type: general-purpose
model: sonnet
description: Test blend export
prompt: |
  Write tests for blend export.

  Tests:
  1. test_save_blend_field_default_false
  2. test_save_blend_creates_file (opt-in Blender integration, #[ignore])
  3. test_blend_in_outputs_or_report (based on the chosen representation)

  Create minimal spec with save_blend: true, verify .blend file exists.
```

#### Task 3D-validate
```yaml
subagent_type: Bash
model: sonnet
description: Run blend tests
prompt: |
  cd speccade
  cargo test -p speccade-spec
  SPECCADE_RUN_BLENDER_TESTS=1 cargo test -p speccade-backend-blender -- --ignored

  Report any failures.
```

### Task 3-review: Review Phase 3
```yaml
subagent_type: general-purpose
model: opus
description: Review Phase 3 implementation
prompt: |
  Review all Phase 3 code for correctness and refactoring.

  Read all modified files from tracks 3A-3D.

  Check:
  1. Consistency - do all tracks follow same patterns?
  2. Error handling - are edge cases covered?
  3. Blender code - is Python idiomatic?
  4. Type safety - are Rust types properly constrained?
  5. Tests - adequate coverage?

  Report issues. Apply simple fixes directly.
```

---

## Phase 4: Advanced Rigging (Sequential)

### Task 4.1-impl: IK Chain System
```yaml
subagent_type: general-purpose
model: opus
description: IK chain system
prompt: |
  Implement IK chains for rigging.

  Files:
  - crates/speccade-spec/src/recipe/animation.rs
  - blender/entrypoint.py

  IMPORTANT: The current `skeletal_animation.blender_clip_v1` schema is simple keyframes.
  Do not break it. Add a new recipe kind for rigging (v2 or a new `skeletal_animation.blender_rigged_*` kind).

  Types: IkChain, IkPreset (6 types), IkTargetConfig, PoleConfig
  Blender: setup_ik_chain(), setup_ik_preset()

  Reference PARITY_MATRIX.md "IK Chain Keys" and "Preset Names".
```

### Task 4.1-test
```yaml
subagent_type: general-purpose
model: sonnet
description: Test IK chains
prompt: |
  Write tests for IK chains.

  Rust: test_ik_chain_serialization, test_ik_preset_enum
  Blender integration (opt-in, #[ignore]): create spec with humanoid_legs preset, verify IK bones created (via report/metrics or simple invariants)

  Run tests and report.
```

### Task 4.2-impl: Bone Constraints
```yaml
subagent_type: general-purpose
model: opus
description: Bone constraints
prompt: |
  Implement bone constraints.

  Types: BoneConstraint enum (Hinge, Ball, Planar, Soft)
  Blender: setup_constraint() creating appropriate Blender constraints

  Reference PARITY_MATRIX.md "Constraint Keys".
```

### Task 4.2-test
```yaml
subagent_type: general-purpose
model: sonnet
description: Test constraints
prompt: |
  Write tests for bone constraints.

  Test each constraint type: Hinge, Ball, Planar, Soft
  Verify Blender creates correct constraint types (opt-in integration, #[ignore]).
```

### Task 4.3-impl: Animator Rig
```yaml
subagent_type: general-purpose
model: opus
description: Animator rig features
prompt: |
  Implement animator rig features.

  Types: AnimatorRigConfig
  Blender:
  - create_widget_shapes() - 5 shapes
  - organize_bone_collections() - 4 groups
  - Bone colors: L=blue, R=red, center=yellow

  Reference PARITY_MATRIX.md "Animator Rig Config Keys".
```

### Task 4.3-test
```yaml
subagent_type: general-purpose
model: sonnet
description: Test animator rig
prompt: |
  Write tests for animator rig.

  Test widget creation, collection assignment, color coding.
  Create spec with animator_rig config, verify in generated .blend (opt-in integration, #[ignore]).
```

### Task 4-validate
```yaml
subagent_type: Bash
model: sonnet
description: Run Phase 4 tests
prompt: |
  cd speccade
  cargo test -p speccade-spec
  SPECCADE_RUN_BLENDER_TESTS=1 cargo test -p speccade-backend-blender -- --ignored

  Report any failures.
```

### Task 4-review: Review Phase 4
```yaml
subagent_type: general-purpose
model: opus
description: Review Phase 4 implementation
prompt: |
  Review Phase 4 rigging code.

  Check:
  1. IK math correctness
  2. Constraint limits in degrees vs radians
  3. Widget mesh generation efficiency
  4. Bone collection logic

  Report issues. Apply fixes.
```

---

## Phase 5: Final Verification

### Task 5.1: Integration Tests
```yaml
subagent_type: general-purpose
model: opus
description: Integration tests
prompt: |
  Create comprehensive end-to-end tests for parity-critical flows.

  IMPORTANT: The workspace root is currently NOT a Cargo package, so `speccade/tests/*.rs` will not run.
  Choose one:
  - Option A: add a small workspace package at the root to host `tests/*.rs`
  - Option B (preferred): create a new crate `crates/speccade-tests` and put integration tests there

  Tests:
  1. Migrate each asset type
  2. Generate each asset type
  3. Verify outputs exist and are valid
  4. Test migrator audit on test fixtures

  Use `golden/legacy/**` as fixtures where possible; for tiny synthetic cases use `tempfile` to build a minimal `.studio/specs` tree.
```

### Task 5.2: Run Full Test Suite
```yaml
subagent_type: Bash
model: sonnet
description: Run full test suite
prompt: |
  cd speccade
  cargo test --all

  Report pass/fail status for each crate.
```

### Task 5.3: Final Audit
```yaml
subagent_type: general-purpose
model: opus
description: Final audit and PARITY_MATRIX update
prompt: |
  Run final audit.

  1. Run speccade migrate --audit on test fixtures
  2. Update PARITY_MATRIX.md status for newly implemented features
  3. Verify percentages are accurate
  4. Report final completeness status

  Target: >90% implemented.
```

### Task 5-review: Final Review
```yaml
subagent_type: general-purpose
model: opus
description: Final code review
prompt: |
  Final review of all changes.

  1. Read all modified files
  2. Check for any remaining TODOs
  3. Check for dead code
  4. Check documentation completeness
  5. Verify PARITY_MATRIX.md is up to date

  Report final status and any remaining issues.
```

---

## Execution Summary

| Phase | Tasks | Parallel | Model |
|-------|-------|----------|-------|
| 1 | 2 | No | opus |
| 2 | 9 | No | opus/sonnet |
| 3 | 12 | Yes (4 tracks) | sonnet |
| 4 | 8 | No | opus/sonnet |
| 5 | 5 | No | opus/sonnet |
| **Total** | **36** | | |
