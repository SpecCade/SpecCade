# Restore “Voided” Golden Specs Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Restore the 12 deleted “golden” SpecCade JSON fixtures (and required tier-1 hash expectations) so we regain deterministic coverage for texture/music and maintain Blender-gated fixture coverage.

**Architecture:**
- Keep `golden/speccade/specs/<asset_type>/*.json` as the canonical fixtures consumed by `speccade-tests`.
- Author the missing fixtures from maintainable Starlark sources in `specs/**` (preferred) and compile them to JSON via `speccade eval`.
- For Tier-1 backends (music/texture), generate `golden/speccade/expected/hashes/<asset_type>/*.hash` via `SPECCADE_UPDATE_GOLDEN_HASHES=1`.
- Add guardrails so empty fixture sets don’t silently pass, and ensure deterministic fixture ordering.

**Tech Stack:** Rust (`cargo test`), SpecCade CLI (`cargo run -p speccade-cli -- eval/validate`), Blender backend tests (optional via `SPECCADE_RUN_BLENDER_TESTS=1`).

---

## Background / Why This Matters

- `crates/speccade-tests/tests/golden_hash_verification.rs` reads JSON fixtures from `golden/speccade/specs/{audio,texture,music}/`.
- Right now, missing directories result in **0 specs** and tests pass with **no coverage**.
- Blender-gated tests (`crates/speccade-tests/tests/e2e_generation.rs`) also consume fixtures from `golden/speccade/specs/{skeletal_mesh,static_mesh,skeletal_animation}/`.

## Target Fixtures To Restore (12)

- `golden/speccade/specs/music/music_comprehensive.spec.json`
- `golden/speccade/specs/texture/texture_comprehensive.spec.json`
- `golden/speccade/specs/texture/normal_comprehensive.spec.json`
- `golden/speccade/specs/skeletal_animation/animation_comprehensive.spec.json`
- `golden/speccade/specs/skeletal_mesh/character_comprehensive.spec.json`
- `golden/speccade/specs/skeletal_mesh/humanoid_male.spec.json`
- `golden/speccade/specs/skeletal_mesh/humanoid_female.spec.json`
- `golden/speccade/specs/skeletal_mesh/quadruped_dog.spec.json`
- `golden/speccade/specs/skeletal_mesh/creature_spider.spec.json`
- `golden/speccade/specs/skeletal_mesh/skinned_mesh_basic.spec.json`
- `golden/speccade/specs/static_mesh/environment_house.spec.json`
- `golden/speccade/specs/static_mesh/mesh_comprehensive.spec.json`

---

## Task 0: Worktree Setup (Recommended)

**Files:** none

**Step 1: Create a worktree**

Run:

```bash
git worktree add .worktrees/restore-goldens -b restore/voided-goldens
```

Expected: new directory `.worktrees/restore-goldens` exists and is on branch `restore/voided-goldens`.

---

## Task 1: Make Fixture Listing Deterministic + Fail On Empty Tier-1 Fixtures

**Files:**
- Modify: `crates/speccade-tests/src/fixtures.rs`
- Modify: `crates/speccade-tests/tests/golden_hash_verification.rs`

**Step 1: Update fixture listing to be deterministic**

In `crates/speccade-tests/src/fixtures.rs`, update `GoldenFixtures::list_speccade_specs()` to:

- Collect entries, filter, then `sort_by_key(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))`.

**Step 2: Add a failing guardrail when Tier-1 fixture dirs are empty**

In `crates/speccade-tests/tests/golden_hash_verification.rs`, after `let results = verify_specs("texture", ...)` etc:

- Assert that `GoldenFixtures::list_speccade_specs("audio").len() > 0`.
- Assert that `GoldenFixtures::list_speccade_specs("texture").len() > 0`.
- Assert that `GoldenFixtures::list_speccade_specs("music").len() > 0`.

Rationale: avoids silent “0 specs, 0 hashes” passing.

**Step 3: Run the test (should fail until fixtures are restored)**

Run:

```bash
cargo test -p speccade-tests --test golden_hash_verification
```

Expected: FAIL complaining that audio/texture/music fixtures are empty.

**Step 4: Commit**

```bash
git add crates/speccade-tests/src/fixtures.rs crates/speccade-tests/tests/golden_hash_verification.rs
git commit -m "test(tests): require non-empty tier-1 golden fixtures"
```

---

## Task 2: Recreate Tier-1 Fixture Directories

**Files:**
- Create: `golden/speccade/specs/texture/`
- Create: `golden/speccade/specs/music/`
- Create: `golden/speccade/expected/hashes/`

**Step 1: Create directories**

Run:

```bash
mkdir -p golden/speccade/specs/texture golden/speccade/specs/music
```

Expected: both directories exist.

---

## Task 3: Restore `texture_comprehensive` + `normal_comprehensive` (Starlark Source -> JSON Fixture)

**Files:**
- Create: `specs/texture/texture_comprehensive.star`
- Create: `specs/texture/normal_comprehensive.star`
- Create: `golden/speccade/specs/texture/texture_comprehensive.spec.json`
- Create: `golden/speccade/specs/texture/normal_comprehensive.spec.json`

**Step 1: Create `specs/texture/texture_comprehensive.star`**

Implementation notes (keep it deterministic, self-contained):

- Use `asset_type = "texture"`.
- Use recipe `texture.procedural_v1`.
- Add multiple primary outputs that reference different graph nodes so the hash combines multiple PNGs.
- Cover: noise -> color ramp -> blend layers -> packed ORM/MRE patterns if supported by current contract.

**Step 2: Create `specs/texture/normal_comprehensive.star`**

Implementation notes:

- Use `asset_type = "texture"`.
- Use `texture.procedural_v1` and produce a normal-like output (either via dedicated normal-from-height node or by building the normal map node chain used by existing examples).

**Step 3: Verify both specs validate**

Run:

```bash
cargo run -p speccade-cli -- validate --spec specs/texture/texture_comprehensive.star --budget strict
cargo run -p speccade-cli -- validate --spec specs/texture/normal_comprehensive.star --budget strict
```

Expected: both commands succeed.

**Step 4: Generate JSON fixtures**

Run (PowerShell):

```powershell
cargo run -p speccade-cli -- eval --spec specs/texture/texture_comprehensive.star --pretty |
  Out-File -Encoding utf8NoBOM golden/speccade/specs/texture/texture_comprehensive.spec.json

cargo run -p speccade-cli -- eval --spec specs/texture/normal_comprehensive.star --pretty |
  Out-File -Encoding utf8NoBOM golden/speccade/specs/texture/normal_comprehensive.spec.json
```

Expected: both `.spec.json` files exist.

**Step 5: Commit**

```bash
git add specs/texture/texture_comprehensive.star specs/texture/normal_comprehensive.star \
  golden/speccade/specs/texture/texture_comprehensive.spec.json \
  golden/speccade/specs/texture/normal_comprehensive.spec.json
git commit -m "test(golden): restore texture comprehensive fixtures"
```

---

## Task 4: Restore `music_comprehensive` (Starlark Source -> JSON Fixture)

**Files:**
- Create: `specs/music/music_comprehensive.star`
- Create: `golden/speccade/specs/music/music_comprehensive.spec.json`

**Step 1: Create `specs/music/music_comprehensive.star`**

Implementation notes:

- Use `asset_type = "music"`.
- Use recipe `music.tracker_song_v1`.
- Include at least: 1 drum channel, 1 bass channel, 1 lead/pad channel.
- Use patterns/rows that exercise: volume changes, instrument changes, note-off, and at least one effect/automation mechanism supported by the current contract.

**Step 2: Validate**

Run:

```bash
cargo run -p speccade-cli -- validate --spec specs/music/music_comprehensive.star --budget strict
```

Expected: success.

**Step 3: Generate JSON fixture**

Run (PowerShell):

```powershell
cargo run -p speccade-cli -- eval --spec specs/music/music_comprehensive.star --pretty |
  Out-File -Encoding utf8NoBOM golden/speccade/specs/music/music_comprehensive.spec.json
```

**Step 4: Commit**

```bash
git add specs/music/music_comprehensive.star golden/speccade/specs/music/music_comprehensive.spec.json
git commit -m "test(golden): restore music_comprehensive fixture"
```

---

## Task 5: Generate Tier-1 Expected Hashes

**Files:**
- Create: `golden/speccade/expected/hashes/texture/*.hash`
- Create: `golden/speccade/expected/hashes/music/*.hash`

**Step 1: Generate hashes in update mode**

Run (PowerShell):

```powershell
$env:SPECCADE_UPDATE_GOLDEN_HASHES=1
cargo test -p speccade-tests --test golden_hash_verification
Remove-Item Env:SPECCADE_UPDATE_GOLDEN_HASHES
```

Expected:

- Test passes.
- New files exist under `golden/speccade/expected/hashes/texture/` and `golden/speccade/expected/hashes/music/`.

**Step 2: Re-run in compare mode**

Run:

```bash
cargo test -p speccade-tests --test golden_hash_verification
```

Expected: PASS.

**Step 3: Commit**

```bash
git add golden/speccade/expected/hashes
git commit -m "test(golden): add expected hashes for restored fixtures"
```

---

## Task 6: Recreate Blender-Gated Fixture Directories

**Files:**
- Create: `golden/speccade/specs/skeletal_mesh/`
- Create: `golden/speccade/specs/skeletal_animation/`
- Create: `golden/speccade/specs/static_mesh/`

**Step 1: Create directories**

Run:

```bash
mkdir -p golden/speccade/specs/skeletal_mesh golden/speccade/specs/skeletal_animation golden/speccade/specs/static_mesh
```

Expected: all exist.

---

## Task 7: Restore Skeletal Mesh Fixtures (Armature-Driven)

**Files:**
- Create: `specs/character/humanoid_male.star`
- Create: `specs/character/humanoid_female.star`
- Create: `specs/character/quadruped_dog.star`
- Create: `specs/character/creature_spider.star`
- Create: `specs/character/character_comprehensive.star`
- Create: `golden/speccade/specs/skeletal_mesh/humanoid_male.spec.json`
- Create: `golden/speccade/specs/skeletal_mesh/humanoid_female.spec.json`
- Create: `golden/speccade/specs/skeletal_mesh/quadruped_dog.spec.json`
- Create: `golden/speccade/specs/skeletal_mesh/creature_spider.spec.json`
- Create: `golden/speccade/specs/skeletal_mesh/character_comprehensive.spec.json`

**Step 1: Implement humanoid male/female via `skeletal_mesh.armature_driven_v1`**

Notes:

- Base on `specs/character/character_humanoid.star` and/or `specs/character/preset_humanoid.star`.
- Use `skeleton_preset = "humanoid_basic_v1"` (or `humanoid_game_v1` if twist bones needed).
- Differentiate male/female via:
  - `bone_meshes` radii/bulges (torso/hips/chest)
  - attachments (e.g., shoulders, head proportions)
  - material slots (optional)

**Step 2: Implement quadruped dog and spider using custom `skeleton`**

Notes:

- Use `skeleton` (custom bone list) in params of `armature_driven_v1`.
- Keep bone counts modest; ensure parent chain is valid.
- Use `bone_meshes` with mirrored limbs where applicable (`mirror` refs).

**Step 3: Implement `character_comprehensive`**

Notes:

- Purposefully cover: bool shapes, modifiers, attachments, multiple material slots, export settings.
- Keep under strict constraints to avoid bloat.

**Step 4: Validate each spec**

Run:

```bash
cargo run -p speccade-cli -- validate --spec specs/character/humanoid_male.star --budget strict
cargo run -p speccade-cli -- validate --spec specs/character/humanoid_female.star --budget strict
cargo run -p speccade-cli -- validate --spec specs/character/quadruped_dog.star --budget strict
cargo run -p speccade-cli -- validate --spec specs/character/creature_spider.star --budget strict
cargo run -p speccade-cli -- validate --spec specs/character/character_comprehensive.star --budget strict
```

Expected: success.

**Step 5: Generate JSON fixtures (PowerShell)**

```powershell
cargo run -p speccade-cli -- eval --spec specs/character/humanoid_male.star --pretty |
  Out-File -Encoding utf8NoBOM golden/speccade/specs/skeletal_mesh/humanoid_male.spec.json
cargo run -p speccade-cli -- eval --spec specs/character/humanoid_female.star --pretty |
  Out-File -Encoding utf8NoBOM golden/speccade/specs/skeletal_mesh/humanoid_female.spec.json
cargo run -p speccade-cli -- eval --spec specs/character/quadruped_dog.star --pretty |
  Out-File -Encoding utf8NoBOM golden/speccade/specs/skeletal_mesh/quadruped_dog.spec.json
cargo run -p speccade-cli -- eval --spec specs/character/creature_spider.star --pretty |
  Out-File -Encoding utf8NoBOM golden/speccade/specs/skeletal_mesh/creature_spider.spec.json
cargo run -p speccade-cli -- eval --spec specs/character/character_comprehensive.star --pretty |
  Out-File -Encoding utf8NoBOM golden/speccade/specs/skeletal_mesh/character_comprehensive.spec.json
```

**Step 6: Commit**

```bash
git add specs/character/humanoid_male.star specs/character/humanoid_female.star \
  specs/character/quadruped_dog.star specs/character/creature_spider.star \
  specs/character/character_comprehensive.star \
  golden/speccade/specs/skeletal_mesh/humanoid_male.spec.json \
  golden/speccade/specs/skeletal_mesh/humanoid_female.spec.json \
  golden/speccade/specs/skeletal_mesh/quadruped_dog.spec.json \
  golden/speccade/specs/skeletal_mesh/creature_spider.spec.json \
  golden/speccade/specs/skeletal_mesh/character_comprehensive.spec.json
git commit -m "test(golden): restore skeletal_mesh fixtures"
```

---

## Task 8: Restore `skinned_mesh_basic` Without External Files

**Files:**
- Create: `specs/character/skinned_mesh_basic.star`
- Create: `golden/speccade/specs/skeletal_mesh/skinned_mesh_basic.spec.json`
- Modify (if needed): `crates/speccade-spec/src/validation/*` (only if contract changes are required)

**Step 1: Decide how to make it self-contained**

Preferred approach (recommended): use `mesh_asset` referencing a mesh generated by the pipeline (no filesystem read).

Fallback approach: embed a tiny GLB in-repo and reference it via `mesh_file`.

**Step 2: Implement `specs/character/skinned_mesh_basic.star`**

- Use recipe `skeletal_mesh.skinned_mesh_v1`.
- Bind to `skeleton_preset = "humanoid_basic_v1"`.
- Use `binding.mode = "auto_weights"` (or `rigid` if using vertex groups).

**Step 3: Validate + generate JSON**

Run:

```bash
cargo run -p speccade-cli -- validate --spec specs/character/skinned_mesh_basic.star --budget strict
```

Then generate fixture:

```powershell
cargo run -p speccade-cli -- eval --spec specs/character/skinned_mesh_basic.star --pretty |
  Out-File -Encoding utf8NoBOM golden/speccade/specs/skeletal_mesh/skinned_mesh_basic.spec.json
```

**Step 4: Blender-gated verification**

Run (requires Blender):

```powershell
$env:SPECCADE_RUN_BLENDER_TESTS=1
cargo test -p speccade-tests --test e2e_generation -- --ignored
Remove-Item Env:SPECCADE_RUN_BLENDER_TESTS
```

Expected: PASS.

**Step 5: Commit**

```bash
git add specs/character/skinned_mesh_basic.star golden/speccade/specs/skeletal_mesh/skinned_mesh_basic.spec.json
git commit -m "test(golden): restore skinned_mesh_basic fixture"
```

---

## Task 9: Restore Static Mesh Fixtures

**Files:**
- Create: `specs/mesh/mesh_comprehensive.star`
- Create: `specs/mesh/environment_house.star`
- Create: `golden/speccade/specs/static_mesh/mesh_comprehensive.spec.json`
- Create: `golden/speccade/specs/static_mesh/environment_house.spec.json`

**Step 1: Implement `mesh_comprehensive.star`**

Notes:

- Use `asset_type = "static_mesh"`.
- Use a recipe that exercises modifiers and multiple primitives (e.g. `static_mesh.blender_primitives_v1` or `static_mesh.boolean_kit_v1`).

**Step 2: Implement `environment_house.star`**

Notes:

- Use `static_mesh.modular_kit_v1` if the contract supports it (preferred).
- Otherwise use `static_mesh.boolean_kit_v1` to approximate a simple house form.

**Step 3: Validate + generate JSON fixtures**

Run:

```bash
cargo run -p speccade-cli -- validate --spec specs/mesh/mesh_comprehensive.star --budget strict
cargo run -p speccade-cli -- validate --spec specs/mesh/environment_house.star --budget strict
```

Generate fixtures:

```powershell
cargo run -p speccade-cli -- eval --spec specs/mesh/mesh_comprehensive.star --pretty |
  Out-File -Encoding utf8NoBOM golden/speccade/specs/static_mesh/mesh_comprehensive.spec.json

cargo run -p speccade-cli -- eval --spec specs/mesh/environment_house.star --pretty |
  Out-File -Encoding utf8NoBOM golden/speccade/specs/static_mesh/environment_house.spec.json
```

**Step 4: Blender-gated verification**

Run (requires Blender):

```powershell
$env:SPECCADE_RUN_BLENDER_TESTS=1
cargo test -p speccade-tests --test e2e_generation -- --ignored
Remove-Item Env:SPECCADE_RUN_BLENDER_TESTS
```

Expected: PASS.

**Step 5: Commit**

```bash
git add specs/mesh/mesh_comprehensive.star specs/mesh/environment_house.star \
  golden/speccade/specs/static_mesh/mesh_comprehensive.spec.json \
  golden/speccade/specs/static_mesh/environment_house.spec.json
git commit -m "test(golden): restore static_mesh fixtures"
```

---

## Task 10: Restore Skeletal Animation Fixture

**Files:**
- Create: `specs/animation/animation_comprehensive.star`
- Create: `golden/speccade/specs/skeletal_animation/animation_comprehensive.spec.json`

**Step 1: Implement `animation_comprehensive.star`**

Notes:

- Use `asset_type = "skeletal_animation"`.
- Target `skeleton_preset = "humanoid_basic_v1"`.
- Include multiple keyframes and a variety of bone transforms (root motion optional).

**Step 2: Validate + generate fixture**

Run:

```bash
cargo run -p speccade-cli -- validate --spec specs/animation/animation_comprehensive.star --budget strict
```

Generate:

```powershell
cargo run -p speccade-cli -- eval --spec specs/animation/animation_comprehensive.star --pretty |
  Out-File -Encoding utf8NoBOM golden/speccade/specs/skeletal_animation/animation_comprehensive.spec.json
```

**Step 3: Blender-gated verification**

Run (requires Blender):

```powershell
$env:SPECCADE_RUN_BLENDER_TESTS=1
cargo test -p speccade-tests --test e2e_generation -- --ignored
Remove-Item Env:SPECCADE_RUN_BLENDER_TESTS
```

Expected: PASS.

**Step 4: Commit**

```bash
git add specs/animation/animation_comprehensive.star \
  golden/speccade/specs/skeletal_animation/animation_comprehensive.spec.json
git commit -m "test(golden): restore skeletal_animation fixture"
```

---

## Task 11: Final Verification (CI-ish)

**Files:** none

**Step 1: Run Tier-1 tests**

Run:

```bash
cargo test -p speccade-tests --test golden_hash_verification
cargo test --workspace
```

Expected: PASS.

**Step 2: Optional Blender-gated run**

Run (requires Blender):

```powershell
$env:SPECCADE_RUN_BLENDER_TESTS=1
cargo test -p speccade-tests --test e2e_generation -- --ignored
Remove-Item Env:SPECCADE_RUN_BLENDER_TESTS
```

Expected: PASS.

---

## Open Decision (Answer Before Implementation)

Do we need these fixtures to be byte-for-byte compatible with the historical versions, or is “valid, comprehensive, and deterministic under the current contract” sufficient?

- If historical compatibility is required: start by extracting the last versions from `git` and port them minimally.
- If current-contract fidelity is sufficient (recommended): implement new Starlark sources and regenerate fixtures + hashes.
