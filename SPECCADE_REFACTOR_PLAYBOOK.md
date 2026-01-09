# SpecCade — Executable Build Playbook (New Repo)

> **"Specs in. Assets out."**

## Project Identity

**Name:** SpecCade
**CLI:** `speccade`
**Repo:** `SpecCade/SpecCade` (`https://github.com/SpecCade/SpecCade`)

**Tagline:** A JSON-to-game-assets compiler.

**One-liner:** SpecCade turns declarative specs into game-ready WAV, XM/IT, PNG, and GLB assets—deterministically and safely.

**Brand themes:**
- **Declarative** — describe what you want, not how to make it
- **Deterministic** — same spec + seed = identical output
- **Portable** — specs are version-control friendly
- **Safe** — pure data specs, no code execution

**Tone:** Clean, confident, slightly playful—arcade energy without being childish.

**Do NOT:** Lead with "AI" messaging. Emphasize reproducibility and build-pipeline usefulness.

---

## Repository Boundary (Non-Negotiable)

- **SpecCade is a completely separate repo** (`SpecCade/SpecCade`) with its own CLI (`speccade`).
- **`ai-studio-core` is legacy reference material only**:
  - used to inventory features and guide migration
  - **NOT** a runtime dependency and **NOT** required to run SpecCade
  - do not couple SpecCade’s schema/backends to ai-studio-core internals

Assumption for this playbook: you have `ai-studio-core` checked out locally (as a sibling repo or in a scratch workspace) to audit legacy behavior and to run migration/reference comparisons.

---

## Naming & Scope Decisions (Locked for v1)

### Asset types (what the spec is “about”)

These names are chosen for long-term clarity (engine-friendly, not “character”-specific):

- `audio_sfx` — one-shot sound effects (WAV; optional derived formats later)
- `audio_instrument` — instrument/sample “patches” (WAV)
- `music` — tracker modules (XM/IT)
- `texture_2d` — 2D texture outputs (PNG maps)
- `static_mesh` — non-skinned meshes (GLB)
- `skeletal_mesh` — skinned meshes with a skeleton (GLB). This covers “characters” AND any other rigged mesh.
- `skeletal_animation` — animation clips targeting a skeleton (GLB)

Planned extension (not required for “full legacy suite”):
- `sprite_2d` — 2D sprite sheets (PNG)

Notes:
- Yes, meshes are inherently 3D. We **do not** encode “3d” in the asset type name; `static_mesh`/`skeletal_mesh` implies 3D by definition.
- Avoid `character_3d`: it’s too semantically narrow and becomes wrong the moment you rig a non-character prop.

### Recipe kinds (how it’s produced)

Recipe kinds are versioned, implementation-addressable identifiers:

- `audio_sfx.layered_synth_v1`
- `audio_instrument.synth_patch_v1`
- `music.tracker_song_v1`
- `texture_2d.material_maps_v1`
- `texture_2d.normal_map_v1`
- `static_mesh.blender_primitives_v1`
- `skeletal_mesh.blender_rigged_mesh_v1`
- `skeletal_animation.blender_clip_v1`

---

## Canonical Spec v1 (Contract + Recipe)

### Top-level contract (required)

- `spec_version`: integer (`1`)
- `asset_id`: string (stable identifier)
- `asset_type`: string enum (see “Asset types” above)
- `license`: string (SPDX-style where possible)
- `seed`: uint32 (`0..2^32-1`)
- `outputs[]`: expected artifacts with `{ kind, format, path }`

Optional contract fields:
- `style_tags[]`, `description`, `engine_targets[]`, `variants[]`

### Recipe (required for `generate`)

- `recipe.kind`: string (one of the recipe kinds above)
- `recipe.params`: typed object validated by `recipe.kind`

Rules:
- `speccade validate` may allow contract-only specs (no `recipe`) if desired, but `speccade generate` requires `recipe`.
- `recipe.kind` **must** match `asset_type` (no cross-type recipes).

---

## Determinism & Safety (Make This True)

### Safety guarantees

- Specs are JSON-only data. **No code execution** in validate/generate/preview.
- Migration may execute trusted legacy `.spec.py` only with an explicit opt-in flag; it is not part of the main pipeline.
- Output paths are treated as **untrusted**:
  - must be relative
  - must not contain `..`
  - must not escape the output root after normalization

### Determinism guarantees (what we can realistically promise)

Seeds alone are not enough. Determinism requires:
- a fixed RNG algorithm (PCG32) and explicit seed-derivation rules
- deterministic iteration order (sort keys, stable ordering for outputs/jobs)
- no dependence on wall-clock time, OS randomness, or hash-map iteration order
- deterministic encoders (or “derived outputs” excluded from hash gates)

Policy (v1):
- **Tier 1 (Rust backends):** deterministic output hashes are required **per platform target triple** and `backend_version`.
  - Do not promise cross-OS/CPU bit-identity unless you explicitly engineer for it.
- **Tier 2 (Blender outputs):** validate **metrics** (not file hashes) in CI by default.
  - Blender/GLB byte hashes are allowed for caching, but not required for pass/fail.

Artifact comparison rules (v1):
- WAV: hash PCM frames only (ignore RIFF header fields that may vary)
- XM/IT: hash full file bytes
- PNG: hash full file bytes (use a deterministic encoder configuration)
- GLB: metric validation (triangle count, bounds, UV presence, material slots, etc.)

Spec hashing (recommended for caching + reporting):
- `spec_hash = BLAKE3(JCS(spec_json_bytes))` where JCS is JSON Canonicalization Scheme (RFC 8785).

---

## Model Assignment Reference

In Claude Code, when spawning sub-agents via the `Task` tool, you can optionally choose a model:

- `model: "haiku"` — quick exploration/search (recommended for low-latency, low-cost tasks)
- `model: "sonnet"` — default choice for most coding/docs/test work
- `model: "opus"` — deepest reasoning/architecture/tricky algorithm work

If you don’t specify `model`, the sub-agent inherits the current session model.

| Task model | Best For | Use When |
|-----------|----------|----------|
| `haiku` | Fast searches, file inventory | Grep, “find all X”, quick summaries |
| `sonnet` | Implementation + docs | Most coding, tests, migrations |
| `opus` | Architecture + hard problems | RFC/spec design, determinism policy, complex refactors |

Example Task tool call:

```yaml
subagent_type: "Explore"
model: "haiku"
prompt: "Find all files that handle authentication"
```

---

## Orchestration (Branch + PR Workflow)

This playbook is designed to be executed as a sequence of small PRs (one task per branch) so `main` stays stable.

Copy/paste-ready orchestrator prompts live in `SPECCADE_PROMPT_PACKAGE.md`.

---

## Phase 0: Parity Inventory + Golden Corpus

### Task 0.1: Legacy Spec Inventory
**Model:** Claude Opus (`model: "opus"`)
**Output:** `PARITY_MATRIX.md`

```
You are auditing the legacy .studio spec system in ai-studio-core.

Read these files:
- ai-studio-core/ai_studio_core/templates/project/studio/generate.py
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/sound.py
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/music.py
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/texture.py
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/normal.py
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/mesh.py
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/character.py
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/animation.py

For each parser, extract:
1. All dict keys expected in specs (e.g., SOUND["layers"], SONG["patterns"])
2. Which keys are required vs optional
3. Any validation logic or constraints
4. Default values
5. Any implicit randomness (random/np.random) and whether it is seeded

Output a markdown table: PARITY_MATRIX.md with columns:
| Asset Type | Key | Required | Type | Constraints | Default | Notes |

Do NOT write any code. Research only.
```

### Task 0.2: Golden Corpus Selection
**Model:** Claude Opus (`model: "opus"`)
**Output:** `golden/` directory with reference corpus + SpecCade corpus

```
Based on PARITY_MATRIX.md, select or create golden corpus specs.

Requirements:
- 10 audio_sfx (mix of: simple beep, complex layered, FM synth, noise-based, percussion)
- 5 audio_instrument (contrasting synthesis types, envelopes)
- 5 music (different tempos, channel counts, pattern complexities)
- 5 texture_2d (albedo/roughness/normal coherence, tileable vs non-tileable)
- 3 normal_map (pattern-driven normals; match legacy normal.py concepts)
- 5 static_mesh (primitives/modifiers/UV variations)
- 2 skeletal_mesh (simple biped + more complex materials)
- 5 skeletal_animation (walk, idle, attack, etc. using documented bone names)

For each spec:
1. (Legacy reference) Create a minimal `.spec.py` that exercises the feature (where legacy supports it)
2. Run it through the legacy system to produce a **human reference** output (not a CI hash gate)
3. Define the SpecCade **canonical JSON** spec that corresponds to the same intent
4. Once SpecCade backends exist, generate outputs and record:
   - Tier 1: artifact hashes (WAV/XM/IT/PNG)
   - Tier 2: metric JSON baselines (GLB)

Output structure:
golden/
  legacy/                       # optional human reference inputs/outputs
    sounds/*.spec.py
    instruments/*.spec.py
    music/*.spec.py
    textures/*.spec.py
    normals/*.spec.py
    meshes/*.spec.py
    characters/*.spec.py
    animations/*.spec.py
    outputs/**                  # captured legacy outputs for visual/audio reference
  speccade/
    specs/**.json               # canonical JSON specs (the CI source of truth)
    expected/
      hashes/
        audio_sfx/*.hash
        audio_instrument/*.hash
        music/*.hash
        texture_2d/*.hash
        normal_map/*.hash
      metrics/
        static_mesh/*.json
        skeletal_mesh/*.json
        skeletal_animation/*.json

Important: do **not** require legacy outputs to be deterministic; treat them as reference only. SpecCade defines the v1 deterministic baseline.
```

---

## Phase 1: RFC + Schema Design

### Task 1.1: RFC-0001 Draft
**Model:** Claude Opus (`model: "opus"`)
**Output:** `docs/rfcs/RFC-0001-canonical-spec.md`

```
Write RFC-0001: Canonical Spec Architecture for SpecCade.

Read first:
- PARITY_MATRIX.md (legacy feature inventory)
- ai-studio-core/ai_studio_core/specs/models.py (existing Pydantic models)
- SPECCADE_REFACTOR_PLAYBOOK.md (this playbook; naming + determinism sections)

RFC must define:

1. Spec Structure
   - spec_version: 1
   - Contract fields: asset_id, asset_type, license, seed, outputs[]
   - Optional fields: style_tags[], description, engine_targets[], variants[]
   - recipe.kind + recipe.params structure

2. Recipe Kinds (v1)
   - audio_sfx.layered_synth_v1
   - audio_instrument.synth_patch_v1
   - music.tracker_song_v1
   - texture_2d.material_maps_v1
   - texture_2d.normal_map_v1
   - static_mesh.blender_primitives_v1
   - skeletal_mesh.blender_rigged_mesh_v1
   - skeletal_animation.blender_clip_v1

3. Determinism Policy
   - Tier 1 (Rust backends): deterministic hashes required per target triple + backend_version
   - Tier 2 (Blender): metric validation only (triangle count ±0.1%, bounds ±0.001)
   - Seed derivation: variant_seed = hash(spec_seed, variant_name)
   - RNG: PCG32 for all Rust backends

4. Report Schema
   - report_version, spec_hash, ok, errors[], warnings[], outputs[], duration_ms

5. Deprecation Timeline
   - SpecCade v0.1: validate/generate for at least 1 asset type
   - SpecCade v0.2: full suite MVP (all legacy categories covered)
   - SpecCade v1.0: “specs in / assets out” stable contract, migration tooling hardened

Include example specs for: audio_sfx, music, texture_2d, static_mesh, skeletal_mesh, skeletal_animation.

Do NOT reference "AI" in the RFC — this is a general-purpose tool.
```

### Task 1.2: JSON Schema Generation
**Model:** Claude Sonnet (`model: "sonnet"`)
**Output:** `schemas/speccade-spec-v1.schema.json`

```
Based on RFC-0001, generate a JSON Schema for SpecCade specs.

The schema must:
1. Use JSON Schema draft-07
2. Define all contract fields with correct types
3. Use discriminated union for asset_type → recipe.kind validation
4. Include field descriptions for LLM consumption
5. Define enums for asset_type, output.kind, output.format

Output: schemas/speccade-spec-v1.schema.json

Test the schema validates the example specs from RFC-0001.
```

### Task 1.3: Determinism Policy (Math-Heavy)
**Model:** Claude Opus (`model: "opus"`)
**Output:** `docs/DETERMINISM.md`

```
Define the determinism policy for SpecCade asset generation.

Requirements:
1. Choose RNG algorithm: PCG32 recommended (fast, deterministic, good distribution)
2. Define seed derivation:
   - Base seed from spec
   - Per-layer seed: hash(base_seed, layer_index)
   - Per-variant seed: hash(base_seed, variant_name)
3. Define canonical hashing:
   - Spec canonicalization: RFC 8785 (JCS)
   - Spec hash algorithm: BLAKE3 (fast, deterministic)
4. Define artifact comparison:
   - WAV: hash PCM data only (skip RIFF header)
   - XM/IT: hash full file
   - PNG: hash full file
   - GLB: metric validation (triangle count, bounds, UV islands, material slots)

Provide pseudocode for:
- canonical_spec_hash(spec) -> bytes
- derive_seed(base_seed, context_string) -> u32
- compare_wav(expected, actual) -> bool
- compare_png(expected, actual) -> bool
- validate_glb_metrics(expected_metrics, actual_glb) -> bool
```

---

## Phase 2: Rust Core Library

### Task 2.1: Spec Crate Scaffold
**Model:** Claude Opus (`model: "opus"`)
**Output:** `crates/speccade-spec/` Rust crate

```
Create the speccade-spec Rust crate.

Read first:
- RFC-0001 (spec structure)
- schemas/speccade-spec-v1.schema.json
- ai-studio-core/ai_studio_core/specs/models.py (reference for field names)

Implement:
1. Serde types for all spec structures
   - SpecCadeSpec (top-level)
   - OutputSpec, VariantSpec
   - Recipe enum with typed params per kind
2. Validation
   - asset_id format: [a-z][a-z0-9_-]{2,63}
   - seed range: 0 to 2^32-1
   - outputs must have unique paths
   - outputs.path must be safe (relative, no traversal, no drive/URI absolute forms)
   - recipe.kind must match asset_type
3. Canonical hashing
   - JSON canonicalization via RFC 8785 (JCS)
   - BLAKE3 hash
4. JSON Schema generation (optional, use schemars)

Crate structure:
crates/speccade-spec/
  Cargo.toml
  src/
    lib.rs
    spec.rs      # Main types
    recipe/
      mod.rs
      audio_sfx.rs
      music.rs
      mesh.rs
      character.rs
    validation.rs
    hash.rs

Include unit tests for:
- Parsing example specs from RFC-0001
- Hash stability (same spec = same hash)
- Validation error messages
```

### Task 2.2: Report Writer
**Model:** Claude Sonnet (`model: "sonnet"`)
**Output:** `crates/speccade-spec/src/report.rs`

```
Add report types and writer to speccade-spec crate.

Report schema (from RFC-0001):
- report_version: u32 (1)
- spec_hash: String (hex BLAKE3)
- ok: bool
- errors: Vec<ReportError>
- warnings: Vec<ReportWarning>
- outputs: Vec<OutputResult>
- duration_ms: u64
- backend_version: String

ReportError/Warning:
- code: String (e.g., "E001", "W001")
- message: String
- path: Option<String> (JSON path to problematic field)

OutputResult:
- kind: OutputKind
- format: OutputFormat
- path: PathBuf
- hash: Option<String> (for Tier 1 outputs)
- metrics: Option<OutputMetrics> (for Tier 2 outputs)

Implement:
- ReportBuilder for ergonomic construction
- JSON serialization
- Report file naming: {asset_id}.report.json
```

---

## Phase 3: CLI Scaffold

### Task 3.1: CLI Binary
**Model:** Claude Opus (`model: "opus"`)
**Output:** `crates/speccade-cli/` Rust crate

```
Create the speccade-cli binary crate.

Commands:
1. speccade validate --spec <path> [--artifacts]
   - Parse spec
   - Run validation
   - Write report
   - Exit 0 if valid, 1 if invalid

2. speccade generate --spec <path> [--out-root <path>]
   - Parse and validate spec
   - Dispatch to backend based on recipe.kind
   - Write outputs
   - Write report
   - Exit 0 if success, 1 if spec error, 2 if generation error

3. speccade preview --spec <path> [--out-root <path>]
   - Supported for Blender-backed asset types in v1
   - Produce preview renders and a report

4. speccade doctor
   - Check Blender installation
   - Check output directory permissions
   - Report versions

5. speccade migrate --project <path>
   - (Stub for Phase 7)

6. speccade fmt --spec <path>
   - Reformat JSON to canonical style (stable ordering, indentation)
   - Optional but strongly recommended to reduce diffs and make spec hashing stable

Use clap for argument parsing.
Backend dispatch is stubbed — return "not implemented" error for now.

Crate structure:
crates/speccade-cli/
  Cargo.toml
  src/
    main.rs
    commands/
      mod.rs
      validate.rs
      generate.rs
      preview.rs
      doctor.rs
      migrate.rs
    dispatch.rs  # Backend dispatch (stubs)
```

---

## Phase 4: Backends (Parallel Execution)

### Task 4A: Audio Backend
**Model:** Claude Opus (`model: "opus"`)
**Output:** `crates/speccade-backend-audio/`

```
Implement the audio_sfx.layered_synth_v1 backend.

Read first:
- PARITY_MATRIX.md (audio features)
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/sound.py (legacy implementation)
- docs/DETERMINISM.md (RNG policy)

Port these synthesis types:
- fm_synth (FM synthesis with modulation)
- karplus_strong (plucked string)
- noise_burst (filtered noise)
- pitched_body (frequency sweep)
- additive (harmonic partials)

Implement:
1. DSP primitives (oscillators, envelopes, filters)
2. Layer mixing
3. WAV writer (deterministic, no timestamp in header)
4. Golden test harness

Recipe params (audio_sfx.layered_synth_v1):
- duration_seconds: f32
- sample_rate: u32 (default 44100)
- layers: Vec<LayerSpec>

LayerSpec:
- synthesis: SynthesisType
- envelope: ADSR
- volume: f32
- pan: f32

Use PCG32 for all randomness, seeded from spec.seed.

Golden tests:
- Compare output WAV PCM data hash against golden/speccade/expected/hashes/audio_sfx/*.hash
- Must pass for all 10 golden audio specs
```

### Task 4B: Instrument Backend
**Model:** Claude Opus (`model: "opus"`)
**Output:** `crates/speccade-backend-instrument/`

```
Implement the audio_instrument.synth_patch_v1 backend.

Goal:
- Generate deterministic single-note instrument samples (WAV) suitable for tracker instruments.

Read first:
- PARITY_MATRIX.md (instrument-related keys in sound.py/music.py)
- docs/DETERMINISM.md (RNG + hashing policy)

Implement:
1. Patch synthesis (reuse audio DSP primitives)
2. Multi-sample options (optional): different pitches/velocities
3. WAV writer
4. Golden test harness

Golden tests:
- Compare output WAV PCM data hash against golden/speccade/expected/hashes/audio_instrument/*.hash
```

### Task 4C: Music Backend
**Model:** Claude Opus (`model: "opus"`)
**Output:** `crates/speccade-backend-music/`

```
Implement the music.tracker_song_v1 backend.

Read first:
- PARITY_MATRIX.md (music features)
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/music.py (legacy implementation)
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/xm_writer.py (format writer)
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/it_writer.py (format writer)

Implement:
1. XM format writer (FastTracker 2 module)
2. IT format writer (Impulse Tracker module)
3. Instrument sample generation (reuse audio DSP)
4. Pattern encoding
5. Golden test harness

Recipe params (music.tracker_song_v1):
- format: "xm" | "it"
- bpm: u16
- speed: u8
- channels: u8
- instruments: Vec<InstrumentSpec>
- patterns: HashMap<String, PatternSpec>
- arrangement: Vec<ArrangementEntry>

Note: Instruments contain inline synthesis specs (same as audio layers).
Patterns contain explicit note data (not generative).

Golden tests:
- Compare output XM/IT file hash against golden/speccade/expected/hashes/music/*.hash
- Must pass for all 5 golden music specs
```

### Task 4D: Texture Backend (Material Maps)
**Model:** Claude Opus (`model: "opus"`)
**Output:** `crates/speccade-backend-texture/`

```
Implement the texture_2d.material_maps_v1 backend.

Goal:
- Deterministically generate coherent material maps (PNG) from a single spec:
  albedo / roughness / metallic / ao / normal / emissive (subset allowed by outputs[])

Read first:
- PARITY_MATRIX.md (texture.py)
- docs/DETERMINISM.md (RNG + hashing policy)

Implement:
1. Noise + pattern primitives in Rust (do NOT depend on Python `pyfastnoiselite` or other Python-only deps)
   - Legacy uses `pyfastnoiselite`; SpecCade should implement or vendor the needed noise algorithms directly.
2. Layering/compositing rules (port core ideas, not necessarily byte-for-byte parity)
3. Deterministic PNG encoding (fixed compression/filter settings)
4. Golden test harness (hash final PNG bytes)

Golden tests:
- Compare PNG file hashes against golden/speccade/expected/hashes/texture_2d/*.hash
```

### Task 4E: Normal Map Backend (Pattern Normals)
**Model:** Claude Opus (`model: "opus"`)
**Output:** `crates/speccade-backend-normal/`

```
Implement the texture_2d.normal_map_v1 backend.

Goal:
- Deterministically generate normal maps (PNG) from parameterized patterns.

Read first:
- PARITY_MATRIX.md (normal.py)

Golden tests:
- Compare PNG file hashes against golden/speccade/expected/hashes/normal_map/*.hash
```

---

## Phase 5: Blender Backends (3D + Animation)

### Task 5A: Static Mesh (Blender)
**Model:** Claude Opus (`model: "opus"`)
**Output:** `crates/speccade-backend-blender/` + `blender/entrypoint.py`

```
Implement the Blender integration backend.

Two components:

1. Rust orchestrator (crates/speccade-backend-blender/)
   - Spawn Blender: blender --background --factory-startup --python entrypoint.py -- --mode <generate|preview|validate> --spec <path> --out-root <path> --report <path>
   - Parse Blender's report JSON
   - Handle Blender errors/crashes
   - Timeout handling (default 5 minutes)

2. Blender Python script (blender/entrypoint.py)
   - stdlib only (json, sys, argparse)
   - Read canonical JSON spec
   - Implement static_mesh.blender_primitives_v1:
     - Create primitives (cube, sphere, cylinder, etc.)
     - Apply modifiers (subdivision, bevel, etc.)
     - UV unwrap
     - Export GLB
   - Write report JSON with metrics:
     - triangle_count
     - bounding_box (min/max xyz)
     - uv_island_count
     - material_slot_count

Golden tests (Tier 2):
- Parse golden/speccade/expected/metrics/static_mesh/*.json
- Compare against generated GLB metrics
- Triangle count within ±0.1%
- Bounds within ±0.001
```

---

### Task 5B: Skeletal Mesh (Blender)
**Model:** Claude Opus (`model: "opus"`)
**Output:** Extend `blender/entrypoint.py` + add metrics/validation

```
Implement skeletal_mesh.blender_rigged_mesh_v1.

Goal:
- Deterministically generate a rigged/skinned mesh with a skeleton and weights.
- Export GLB with skinning data.

Validation/metrics (Tier 2):
- triangle_count
- bounding_box
- bone_count
- max_bone_influences
- has_uvs / has_normals / has_tangents (as required by outputs/params)

Golden tests (Tier 2):
- Parse golden/speccade/expected/metrics/skeletal_mesh/*.json
- Compare against generated GLB metrics
```

### Task 5C: Skeletal Animation (Blender)
**Model:** Claude Opus (`model: "opus"`)
**Output:** Extend `blender/entrypoint.py` + add metrics/validation

```
Implement skeletal_animation.blender_clip_v1.

Goal:
- Apply a declarative animation spec to an armature and export a GLB containing the clip.

Validation/metrics (Tier 2):
- clip_duration_seconds
- frame_count
- referenced_bones[] validation vs skeleton preset

Golden tests (Tier 2):
- Parse golden/speccade/expected/metrics/skeletal_animation/*.json
- Compare against generated GLB metrics
```

---

## Phase 6: Golden Corpus + CI Gates (SpecCade Baseline)

### Task 6.1: Lock v1 Golden Outputs
**Model:** Claude Sonnet (`model: "sonnet"`)
**Output:** Golden hashes/metrics committed for SpecCade outputs

```
Using golden/speccade/specs/**.json, generate all outputs and record:
- Tier 1 hashes (wav/xm/it/png)
- Tier 2 metrics JSON (glb)

CI must fail if:
- a Tier 1 hash changes without an explicit golden update
- a Tier 2 metric falls outside tolerance

Do NOT gate on legacy outputs; they are reference-only.
```

---

## Phase 7: Migration

### Task 7.1: Migration Tool
**Model:** Claude Sonnet (`model: "sonnet"`)
**Output:** `crates/speccade-cli/src/commands/migrate.rs`

```
Implement speccade migrate --project <path>.

Keep the implementation simple and predictable:

1. Find all .spec.py files under .studio/specs/
2. For each spec:
   a. Execute with Python ONLY if user passes --allow-exec-specs (trusted local input)
   b. Extract the dict (SOUND, INSTRUMENT, SONG, TEXTURE, NORMAL, MESH, CHARACTER, ANIMATION)
   c. Map to canonical JSON structure
   d. Write to specs/{asset_type}/{asset_id}.json
3. Generate migration report

Mapping rules:
- Legacy category `sounds` → asset_type: "audio_sfx", recipe.kind: "audio_sfx.layered_synth_v1"
- Legacy category `instruments` → asset_type: "audio_instrument", recipe.kind: "audio_instrument.synth_patch_v1"
- Legacy category `music` → asset_type: "music", recipe.kind: "music.tracker_song_v1"
- Legacy category `textures` → asset_type: "texture_2d", recipe.kind: "texture_2d.material_maps_v1"
- Legacy category `normals` → asset_type: "texture_2d", recipe.kind: "texture_2d.normal_map_v1"
- Legacy category `meshes` → asset_type: "static_mesh", recipe.kind: "static_mesh.blender_primitives_v1"
- Legacy category `characters` → asset_type: "skeletal_mesh", recipe.kind: "skeletal_mesh.blender_rigged_mesh_v1"
- Legacy category `animations` → asset_type: "skeletal_animation", recipe.kind: "skeletal_animation.blender_clip_v1"

Handle edge cases:
- Warn on Python expressions that can't be statically converted
- Warn on unsupported features (flag for manual review)
- Add `migration_notes[]` to the output JSON for manual fixes (JSON cannot contain comments)
```

### Task 7.2: Documentation
**Model:** Claude Sonnet (`model: "sonnet"`)
**Output:** `docs/` updates

```
Write user-facing documentation for SpecCade.

1. README.md
   - What is SpecCade (declarative asset generation)
   - Quick start (install, validate, generate)
   - Example spec
   - NOT AI-specific — works with hand-authored specs too

2. docs/MIGRATION.md
   - How to migrate from legacy .studio system
   - speccade migrate command
   - Manual fixes for complex specs
   - Deprecation timeline

3. docs/SPEC_REFERENCE.md
   - Full spec schema documentation
   - All recipe kinds and their params
   - Examples for each asset type

4. docs/DETERMINISM.md
   - Already written, review and polish
```

---

## Execution Summary

| Phase | Task | Model | Parallelizable |
|-------|------|-------|----------------|
| 0 | 0.1 Inventory | `opus` | No |
| 0 | 0.2 Corpus (legacy + speccade) | `opus` | No (needs 0.1) |
| 1 | 1.1 RFC | `opus` | No |
| 1 | 1.2 JSON Schema | `sonnet` | After 1.1 |
| 1 | 1.3 Determinism | `opus` | After 1.1 |
| 2 | 2.1 Spec Crate | `opus` | After 1.x |
| 2 | 2.2 Report Writer | `sonnet` | After 2.1 |
| 3 | 3.1 CLI | `opus` | After 2.x |
| 4 | 4A Audio SFX | `opus` | After 3.1 ✓ |
| 4 | 4B Instrument | `opus` | After 3.1 ✓ |
| 4 | 4C Music | `opus` | After 3.1 ✓ |
| 4 | 4D Texture | `opus` | After 3.1 ✓ |
| 4 | 4E Normal Map | `opus` | After 3.1 ✓ |
| 5 | 5A Static Mesh | `opus` | After 3.1 ✓ |
| 5 | 5B Skeletal Mesh | `opus` | After 3.1 ✓ |
| 5 | 5C Skeletal Animation | `opus` | After 3.1 ✓ |
| 6 | 6.1 CI Golden Gates | `sonnet` | After 4–5 |
| 7 | 7.1 Migration | `sonnet` | After 4–6 |
| 7 | 7.2 Docs | `sonnet` | After 4–6 ✓ |

**Parallel opportunities:**
- 1.2 + 1.3 can run in parallel after 1.1
- 4A–4E can run in parallel after 3.1
- 5A–5C can run in parallel after 3.1
- 7.1 + 7.2 can run in parallel

---

## Human Review Gates

| After | Gate |
|-------|------|
| Phase 0 | Review PARITY_MATRIX.md, approve golden corpus |
| Phase 1 | **Approve RFC-0001** (all downstream depends on this) |
| Phase 2 | Run `cargo test`, verify spec parsing works |
| Phase 3 | Run `speccade validate` on golden specs |
| Phase 4 | Run `speccade generate` on golden specs, verify hashes |
| Phase 6 | Verify CI gates + golden update workflow |
| Phase 7 | Test migration on real project |

---

## Quick Reference: Starting Phase 0

Copy this prompt to begin:

```
I'm starting the SpecCade build (new repo), using ai-studio-core as legacy reference only.

Context:
- This is a declarative asset generation system
- Specs in JSON → game-ready assets out (audio, music, textures, meshes, characters, animations)
- Replacing legacy .spec.py (exec-based) with safe JSON specs
- Rust backends for non-Blender assets; Blender boundary for meshes/rigs/animations

First task: Create PARITY_MATRIX.md by auditing the legacy parsers.

Read these files and extract all spec dict keys:
- ai-studio-core/ai_studio_core/templates/project/studio/generate.py
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/sound.py
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/music.py
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/texture.py
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/normal.py
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/mesh.py
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/character.py
- ai-studio-core/ai_studio_core/templates/project/studio/parsers/animation.py

Output: PARITY_MATRIX.md with columns:
| Asset Type | Key | Required | Type | Constraints | Default | Notes |
```
