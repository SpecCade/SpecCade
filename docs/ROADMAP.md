# SpecCade Roadmap (Single Source of Truth)

This file is the **single source of truth** for:

- Planned work (features, refactors, tooling)
- Open questions that require a decision
- Cross-document follow-ups (RFCs, docs, validation/schema alignment)

Design rationale and proposals live in `docs/rfcs/`. The canonical spec contract lives in `docs/spec-reference/` and `crates/speccade-spec/`.

## How to Use This Doc

- If it's a **task** or **decision**, it belongs here with an ID.
- Other documents should **link here** instead of duplicating task lists.
- Keep items **actionable**: verb + concrete deliverable + (when possible) file/command touch points.

## Suggested Execution Order

This file is grouped by domain, but a typical dependency order is:

1) `LLM-001`, `LLM-002`, `LLM-008`, `LLM-003` (make the API/diagnostics/tool loop reliable)
2) `LLM-004`, `LLM-005`, `LLM-006` (reduce first-draft failure rate via reuse and constructors)
3) `LLM-007` plus `QA-003` and `EDITOR-002/003` (iteration speed + previews)
4) `LLM-009/010/011`, `MESHVER-*`, and `GEN-*`/`MESH-*` (advanced/longer-term)

---

## LLM-Native Authoring (RFC-0008)

Reference: `docs/rfcs/RFC-0008-llm-native-asset-authoring.md`

- [x] `LLM-001` Add machine-readable stdlib dump: `speccade stdlib dump --format json`. **Done: 2026-01-18**
  - Deliverable: stable JSON describing stdlib functions, params (required/default), enums, ranges, examples, and `STDLIB_VERSION`.
  - Touch points: `crates/speccade-cli/src/compiler/stdlib/`, CLI command plumbing under `crates/speccade-cli/src/commands/`.
- [x] `LLM-002` Add `--json` structured output for `eval`, `validate`, `generate`. **Done: 2026-01-18**
  - Deliverable: machine-readable diagnostics (stable codes, path, file/line/col where available, suggestions).
  - Touch points: `crates/speccade-cli/src/commands/{eval,validate,generate}.rs`, `crates/speccade-spec` validation error structures.
- [x] `LLM-008` Add a Nethercore-oriented budget profile (22050 Hz audio, modern constraints). **Done: 2026-01-18**
  - Deliverable: `BudgetProfile::nethercore()` (or `by_name("nethercore")`) with documented limits; avoids "zx-8bit" naming confusion.
  - Touch points: `crates/speccade-spec/src/validation/budgets.rs`, `docs/budgets.md`.
- [x] `LLM-003` Add `speccade analyze` baseline metrics (start with audio + textures). **Done: 2026-01-18**
  - Deliverable: deterministic metrics JSON for iteration loops (audio peak/clipping/DC/loudness proxy; texture histogram/contrast/tileability checks).
  - Reference spec: `docs/rfcs/RFC-0008-appendix-audio-analysis-spec.md`.
- [x] `LLM-004` Add preset/template retrieval primitives (CLI-time, no runtime IO in Starlark). **Done: 2026-01-18**
  - Deliverable: a way to pick "known good" starting points by tags/keywords (e.g. `style_tags`, curated kit IDs) without copying 200-line presets into every spec.
- [x] `LLM-005` Expand `speccade template` beyond textures (audio/music first). **Done: 2026-01-18** (via LLM-004)
  - Deliverable: `template list/show/copy` works for `audio` and `music` templates in `packs/`.
  - Touch points: `crates/speccade-cli/src/commands/template.rs`, `packs/`.
- [x] `LLM-006` Add higher-level Starlark constructors to reduce raw recipe dict authoring. **Done: 2026-01-18**
  - Deliverable: `audio_spec(...)`, `texture_spec(...)`, `mesh_spec(...)` helpers analogous to `music_spec(...)`.
  - Touch points: `crates/speccade-cli/src/compiler/stdlib/{core,audio,texture,mesh}.rs`.
- [x] `LLM-007` Iteration-speed features for agentic loops (preview + partial generation + caching). **Done: 2026-01-18**
  - Deliverable: at least one of: `generate --preview`, per-layer generation for audio, or content-addressed caching keyed by canonical hashes + backend versions.
  - Implemented: `generate --preview <duration_seconds>` for fast audio preview generation.
- [x] `LLM-009` Extend `speccade analyze` with embedding export for similarity search (explicit opt-in). **Done: 2026-01-18**
  - Implemented: `--embeddings` flag outputs 48-dimension deterministic feature vectors for audio and texture files.
- [x] `LLM-010` Add batch analysis modes (`--input-dir`, CSV/JSONL outputs) for clustering/auditing. **Done: 2026-01-18**
  - Implemented: `--input-dir` recursively scans for .wav/.png; `--output-format json|jsonl|csv` with summary statistics.
- [x] `LLM-011` Add a real-time analysis mode (e.g. WebSocket server) for editor/iterative workflows. **Done: 2026-01-19**
  - Implemented: `speccade analyze --serve [port]` WebSocket server (feature-gated); JSON protocol with analyze_path/analyze_data requests.

---

## Editor / Real-Time Preview (RFC-0009)

Reference: `docs/rfcs/RFC-0009-editor-architecture.md`

- [ ] `EDITOR-001` Decide "editor" delivery shape (Tauri app vs VSCode extension vs both).
  - Deliverable: a committed decision + minimal repo layout plan (new crate? new top-level directory?).
- [x] `EDITOR-002` Implement CLI-side preview artifacts (useful even without a full editor). **Done: 2026-01-18**
  - Deliverable: standardized preview outputs (audio waveform/spectrogram PNG; texture thumbnails) emitted by `generate` and recorded in reports.
  - Touch points: `crates/speccade-cli`, `crates/speccade-spec` report structures.
  - Implemented: audio waveform PNG (1024x256) generated alongside WAV output and recorded in reports with `kind: "preview"`.
- [x] `EDITOR-003` Add fast "preview mode" generation knobs. **Done: 2026-01-18** (via LLM-007)
  - Deliverable: deterministic downscaled/shortened preview generation flags (e.g. audio first N ms; texture 256x256), clearly labeled as preview.
  - Implemented: `generate --preview <duration_seconds>` for audio preview; texture/mesh knobs deferred.

Open questions (track decisions here, not in the RFC):
- [ ] `EDITOR-Q001` GPU acceleration for preview (WebGPU) vs WebGL2-only.
- [ ] `EDITOR-Q002` Large mesh handling in preview (LOD/proxies) strategy.
- [ ] `EDITOR-Q003` Collaboration: explicitly defer to v2 or define minimal v1 stance.

---

## Mesh/Character Verification Loop (RFC-0010)

Reference: `docs/rfcs/RFC-0010-mesh-llm-verification.md`

- [x] `MESHVER-001` Implement geometric metrics for Tier-2 assets (bounds, verts/faces, UV sanity, bone counts). **Done: 2026-01-19**
  - Deliverable: deterministic metrics emitted into `${asset_id}.report.json` for mesh/character/animation outputs.
  - Implemented: OutputMetrics extended with vertex/face/edge counts, quad_percentage, manifold checks, UV coverage/overlap, bounds; GLB analysis via `speccade analyze`.
- [x] `MESHVER-002` Add a constraint/verification surface (validation-time vs post-generate). **Done: 2026-01-19**
  - Deliverable: a minimal constraint schema and a place to run it (e.g. `speccade verify --spec ...`).
  - Implemented: `speccade verify` command with 10 constraint types; JSON constraint files; pass/fail results with actual values.
- [ ] `MESHVER-003` Decide if/when VLM integration is supported (and how it is configured).
  - Deliverable: explicit policy: off by default; user-provided credentials; what gets uploaded (renders only).
- [ ] `MESHVER-004` Add skeletal mesh deformation/weight verification probes (result-based, not spec-based).
  - Deliverable: additional `speccade verify` constraints and/or Blender-side probes for max influences, unweighted verts, and deformation sanity poses.
  - Touch points: `blender/entrypoint.py`, `crates/speccade-backend-blender/src/metrics.rs`, `crates/speccade-spec/src/validation/constraints/`.
- [ ] `MESHVER-005` Add animation motion verification (post-IK/post-bake) for joint direction + constraint enforcement.
  - Deliverable: machine-readable motion validation section in reports (hinge axis/sign calibration, range violations, knee/elbow pops, root motion sanity).
  - Touch points: `blender/entrypoint.py`, `crates/speccade-backend-blender/src/metrics.rs`, `crates/speccade-spec/src/validation/constraints/`.

Open questions:
- [ ] `MESHVER-Q001` Acceptable VLM latency targets for interactive use.
- [ ] `MESHVER-Q002` Verification caching keyed by spec hash (and what invalidates it).
- [ ] `MESHVER-Q003` Guardrails for VLM hallucinations (ensemble prompts, thresholds, human override).

---

## Music (Compose / Pattern IR)

- [x] `MUSIC-001` Add snapshot tests for `docs/examples/music/*.expanded.params.json` via `speccade expand`. **Done: 2026-01-18**
  - Deliverable: tests that compare expansion output to checked-in snapshots (stable formatting).
  - Implemented: Generated eurobeat_4bars.expanded.params.json; all compose specs now have snapshot coverage.
- [ ] `MUSIC-002` Extend Pattern IR operators and hard limits (keep determinism + reviewability).
  - Deliverable: RFC + schema/types changes + tests for any new ops; avoid silent behavior changes.
- [x] `MUSIC-003` Improve `speccade expand` UX for review workflows. **Done: 2026-01-18**
  - Deliverable: stable formatting, optional file output, and diff-friendly mode.
  - Implemented: `--output`, `--pretty`, `--compact`, `--json` flags; Starlark input support.
- [x] `MUSIC-004` Tighten XM/IT parity checks and document known differences. **Done: 2026-01-18**
  - Deliverable: automated structural checks + a short listening checklist doc.
  - Implemented: `check_parity()` function in backend-music; `docs/xm-it-differences.md` with QA checklist.
- [x] `MUSIC-008` Expand tracker effect coverage + validation (arp/porta/vibrato/retrig/vol slide, etc.). **Done: 2026-01-18**
  - Implemented: 30+ typed TrackerEffect variants with XM/IT validation; `docs/tracker-effects.md` reference.
- [x] `MUSIC-009` Add deterministic swing/humanize macros to Pattern IR (timing + velocity ranges with explicit constraints). **Done: 2026-01-19**
  - Implemented: HumanizeVol (per-cell volume variation) and Swing (offbeat note-delay) TransformOp variants; Starlark helpers humanize_vol() and swing().

## Music (Content / Workflow)

- [x] `MUSIC-005` Prefer `TrackerInstrument.ref` (external `audio_v1` specs) in examples/goldens where possible. **Done: 2026-01-19**
  - Implemented: Added JSON drum specs (kick/snare/hihat.spec.json); updated compose_eurobeat_4bars.json to use ref pattern; regenerated expanded snapshots.
- [x] `MUSIC-006` Add a tuned, high-quality drum example (kick/snare/hat) and gate it with basic metrics (no clipping, sane levels). **Done: 2026-01-19**
  - Implemented: docs/examples/music/drums/ with kick.star, snare.star, hihat.star; quality test validates peak_db < 0, dc_offset < 0.01, rms_db in range.
- [ ] `MUSIC-007` Grow "genre kits" as data packages: curated compose defs + instrument refs + timebase/harmony defaults.
- [ ] `MUSIC-010` Add cue templates (`loop_low/loop_main/loop_hi`, stingers, transitions) as compile-time helpers or templates.

---

## Audio

- [x] `AUDIO-001` Keep expanding and tuning the `audio_v1` preset library with QA gates. **Done: 2026-01-18**
  - Deliverable: baseline checks (no clipping, sane RMS, low DC offset) applied in CI or via `speccade analyze`.
  - Implemented: Rust test validates all 255 presets; 36 presets fixed for DC offset/compressor issues.
- [x] `AUDIO-002` Add an "audio audit" report command (or `analyze` sub-mode) to catch regressions. **Done: 2026-01-18**
  - Deliverable: peak/RMS/DC metrics for golden audio fixtures + budgeted tolerances.
  - Implemented: `speccade audit` command with tolerances, baseline management, and `--update-baselines` flag.
- [x] `AUDIO-003` Improve loop-point generation + click-free defaults for tracker instrument baking. **Done: 2026-01-18**
  - Implemented: LoopConfig with zero-crossing detection, crossfade at boundaries, click-free defaults.
- [x] `AUDIO-004` Add missing effects needed for production mixing (start with parametric EQ + limiter). **Done: 2026-01-19**
  - Notes: also consider gate/expander and stereo widener.
  - Verified: ParametricEq and Limiter effects exist in spec (effects/mod.rs), backend DSP (effects/chain.rs), and stdlib helpers (dynamics.rs).
- [x] `AUDIO-005` Add missing filter types (notch/allpass first; then comb/formant/shelves as needed). **Done: 2026-01-19**
  - Verified: Notch and Allpass filters exist in spec, backend DSP, and Starlark stdlib; tests added for coverage.
- [ ] `AUDIO-006` Add missing synthesis types with highest leverage (unison/supersaw, waveguide, bowed string, membrane/drum).
- [x] `AUDIO-007` Add loudness targets (LUFS) and true-peak limiting workflows for production-ready output levels. **Done: 2026-01-19**
  - Implemented: ITU-R BS.1770 integrated LUFS + true_peak_db in analyze; TruePeakLimiter effect with oversampling; true_peak_limiter() Starlark helper.
- [ ] `AUDIO-008` Add one-shot + loop pairing helpers (transient + loopable sustain from the same recipe).
- [ ] `AUDIO-009` Add foley layering helpers (impact builder: transient/body/tail; whoosh builder: noise + sweep).
- [ ] `AUDIO-010` Add batch SFX variation sets (seed sweeps + constraints + sample-set export).
- [ ] `AUDIO-011` Expand LFO targets where it materially improves sound design (pulse_width, fm_index, delay_time, etc.).

---

## Textures

- [x] `TEX-001` Add additional `texture.procedural_v1` ops (blur/warp/morphology/blend modes/UV transforms). **Done: 2026-01-18**
  - References: RFC-0005/0006 future-work notes.
  - Implemented: Blur, Erode, Dilate, Warp, BlendScreen/Overlay/SoftLight/Difference, UvScale/Rotate/Translate.
- [ ] `TEX-002` Add graph libraries/templates without introducing new recipe kinds.
  - Deliverable: reusable templates in `packs/` + CLI support to copy/instantiate.
- [ ] `TEX-003` Decide if richer channel "swizzle/component extract" ops are needed for packing workflows.
- [ ] `TEX-004` Add stochastic tiling (Wang tiles / texture bombing) to reduce visible repetition.
- [ ] `TEX-005` Add trimsheets/atlases with deterministic packing + mip-safe gutters and metadata.
  - Candidate: `texture.trimsheet_v1`.
- [ ] `TEX-006` Add decal workflows (RGBA + optional normal/roughness + placement metadata).
  - Candidate: `texture.decal_v1`.
- [ ] `TEX-007` Add terrain "splat set" workflows (albedo/normal/roughness + splat masks + macro variation).
  - Candidate: `texture.splat_set_v1`.
- [ ] `TEX-008` Define and implement matcap generation (`texture.matcap_v1`) for stylized shading presets.
  - Notes: toon steps/ramps, curvature/cavity masks, outline, "preset + overrides" art direction.
- [ ] `TEX-009` Add a material preset system for stable art direction ("preset + parameterization" at CLI-time).

---

## New Asset Types (2D VFX / UI / Fonts)

Migrated from `docs/FUTURE_GENERATORS.md` (now deprecated).

- [ ] `GEN-001` Define scope and write an RFC for deterministic spritesheets (`sprite.sheet_v1`) and animation clips (`sprite.animation_v1`).
  - Deliverable: spec types + schema + minimal generator + golden tests.
- [ ] `GEN-002` Implement a baseline flipbook/VFX generator (`vfx.flipbook_v1` or `vfx.smoke_puff_v1`) with deterministic packing.
- [ ] `GEN-003` Implement UI generators: nine-slice panels (`ui.nine_slice_v1`) and an icon set generator (`ui.icon_set_v1`).
- [ ] `GEN-004` Implement font generators (bitmap pixel fonts and/or MSDF with JSON metrics).
  - Candidates: `font.bitmap_v1`, `font.msdf_v1`.
- [ ] `GEN-005` Add VFX particle "material/profile" presets (additive/soft/distort/etc.).
  - Candidate: `vfx.particle_profile_v1`.
- [ ] `GEN-006` Add UI kit presets and item card templates with slots (icon/rarity/background).
  - Candidate: `ui.item_card_v1`.
- [ ] `GEN-007` Add deterministic damage-number sprites (font + outline + crit styles).

---

## Mesh/Animation Feature Expansion (Blender Tier)

Migrated from `docs/FUTURE_GENERATORS.md` (now deprecated).

- [ ] `MESH-001` Add a curated `modifier_stack[]` for `static_mesh.blender_primitives_v1` (mirror/solidify/bevel/subdivide/array/triangulate).
- [ ] `MESH-002` Add UV unwrap/pack automation with texel-density targets (optional lightmap UVs).
- [ ] `MESH-003` Add normals automation presets (auto_smooth, weighted normals, hard-edge-by-angle).
- [ ] `MESH-004` Add deterministic LOD generation (decimate to target tri counts) + validate bounds/tri metrics.
- [ ] `MESH-005` Add collision mesh generation outputs (convex hull / simplified mesh).
- [ ] `MESH-006` Add navmesh hints/metadata outputs (walkable surfaces, slope/stair tagging).
- [ ] `MESH-007` Add a baking suite (high->low normal/AO/curvature, vertex colors, dilation).
- [ ] `MESH-008` Add a render-to-sprite bridge (render `static_mesh` with lighting preset -> `sprite.sheet_v1`).
- [ ] `MESH-009` Add modular kit generators (walls/doors/pipes) built from primitives + modifiers.
- [ ] `MESH-010` Add organic modeling gap-fill (metaballs -> remesh -> smooth -> displacement noise) with strict budgets.
- [ ] `MESH-011` Add shrinkwrap workflows (armor/clothes wrapping onto body parts) with strict stability validation.
- [ ] `MESH-012` Add boolean kitbashing (union/difference + cleanup) with determinism/validation constraints.
- [ ] `MESH-013` Add animation helper presets (IK targets + constraint presets) for procedural walk/run cycles.

### Skeletal Mesh / Character (Blender Tier)

- [ ] `CHAR-001` Add spec reference docs for Blender-backed mesh/character/animation assets.
  - Deliverable: `docs/spec-reference/mesh.md` (static mesh), `docs/spec-reference/character.md` (skeletal mesh), `docs/spec-reference/animation.md` (skeletal animation), plus updates to `docs/SPEC_REFERENCE.md`.
  - Touch points: `docs/spec-reference/`, `crates/speccade-spec/src/recipe/{mesh,character,animation}/`, `schemas/speccade-spec-v1.schema.json`.
- [ ] `CHAR-002` Add Starlark constructors for `skeletal_mesh` authoring (and docs).
  - Deliverable: ergonomic helpers for skeleton presets/custom skeletons, body parts, skinning/export settings; documented under `docs/stdlib-reference.md`.
  - Touch points: `crates/speccade-cli/src/compiler/stdlib/` (new module), `docs/stdlib-*.md`.
- [ ] `CHAR-003` Expand skeletal mesh validation/verification beyond counts (topology, UV presence, skin weights).
  - Deliverable: additional constraints in `speccade verify` and report fields for non-manifold edges, degenerate faces, UV sanity, max influences, and unweighted vertices.
  - Touch points: `blender/entrypoint.py`, `crates/speccade-spec/src/validation/constraints/`, `crates/speccade-backend-blender/src/metrics.rs`.

### Skeletal Animation / Rigging / IK (Blender Tier)

- [ ] `ANIM-001` Expose `skeletal_animation.blender_rigged_v1` end-to-end (IK/rig-aware animation generation).
  - Deliverable: `speccade generate` supports `skeletal_animation.blender_rigged_v1` and dispatches Blender with `--mode rigged_animation`.
  - Touch points: `crates/speccade-spec/src/validation/mod.rs`, `schemas/speccade-spec-v1.schema.json`, `crates/speccade-cli`, `crates/speccade-backend-blender/src/orchestrator.rs`, `blender/entrypoint.py`.
- [ ] `ANIM-002` Decide and enforce the clip-vs-rigged schema split, then migrate fixtures/examples.
  - Deliverable: `skeletal_animation.blender_clip_v1` stays "simple keyframes"; all IK/rigging keys live under `skeletal_animation.blender_rigged_v1`; migrate `golden/speccade/specs/skeletal_animation/*.json` accordingly.
  - Touch points: `golden/speccade/specs/skeletal_animation/`, `crates/speccade-backend-blender/src/animation.rs`, `crates/speccade-backend-blender/src/orchestrator.rs`.
- [ ] `ANIM-003` Add Starlark constructors for `skeletal_animation` authoring (phases/poses/procedural layers/IK keyframes).
  - Deliverable: ergonomic helpers with safe defaults, plus docs/examples.
  - Touch points: `crates/speccade-cli/src/compiler/stdlib/` (new module), `docs/stdlib-reference.md`, `docs/examples/`.
- [ ] `ANIM-004` Fill remaining rigging parity gaps and document them (IK stretch, foot roll, missing presets).
  - Deliverable: reference docs + probe specs for IK stretch settings, foot roll systems, and any missing presets (e.g. basic spine).
  - Touch points: `blender/entrypoint.py`, `crates/speccade-spec/src/recipe/animation/`, `golden/speccade/specs/skeletal_animation/`.
- [ ] `ANIM-005` Validate "hard" constraint types in real assets (ball/planar + stiffness/influence semantics).
  - Deliverable: probe specs + verification checks for ball socket constraints, planar constraints, and influence/stiffness behavior.
  - Touch points: `blender/entrypoint.py`, `crates/speccade-spec/src/recipe/animation/constraints.rs`, `crates/speccade-spec/src/validation/constraints/`.
- [ ] `ANIM-006` Add root motion controls + validation (export + verify).
  - Deliverable: explicit root motion settings (extract/lock/validate) with a report section and a `speccade verify` constraint.
  - Touch points: `crates/speccade-spec/src/recipe/animation/`, `blender/entrypoint.py`, `crates/speccade-backend-blender/src/metrics.rs`.

---

## Tooling / QA

- [x] `QA-001` Grow Tier-1 golden corpus (`golden/`) and run it in CI. **Done: 2026-01-18**
  - Implemented: 81 hash files (43 audio, 29 texture, 9 music); hash verification test; CI integration.
- [x] `QA-002` Add "inspect" style commands/flags where helpful for review (compose->expanded JSON, intermediate texture maps). **Done: 2026-01-18**
  - Implemented: `speccade inspect` command emits per-node texture PNGs and expanded compose params.
- [x] `QA-003` Add content-addressed caching keyed by canonical spec/recipe hash + backend versions (iteration speed). **Done: 2026-01-18**
- [x] `QA-004` Add perceptual diffing / quality controls (image SSIM/DeltaE; audio loudness/spectral) where feasible. **Done: 2026-01-18**
  - Implemented: `speccade compare` command with SSIM, DeltaE (CIE76), histogram diff for textures; spectral correlation for audio.
- [x] `QA-005` Add profiling/observability: per-stage timings, memory stats, and reproducible perf runs. **Done: 2026-01-18**
  - Implemented: `--profile` flag adds `stages[]` timing breakdown to report; documented in `docs/profiling.md`.
- [ ] `QA-006` Define a plugin/backends extension story (subprocess or WASM) with strict I/O contracts + determinism reporting.

---

## Migration (Legacy `.studio` / spec.py)

Reference: `docs/MIGRATION.md`

- [ ] `MIGRATE-001` Implement a real params mapping layer in the migrator (legacy keys -> canonical recipe schemas).
- [ ] `MIGRATE-002` Add migration fixtures + tests that validate migrated specs against `speccade validate`.
- [ ] `MIGRATE-003` Map legacy `ANIMATION` dict keys to canonical `skeletal_animation` params (incl. rig_setup/poses/phases/IK).
  - Deliverable: a tested conversion that produces canonical `skeletal_animation.blender_rigged_v1` params (and rejects/flags unknown keys with actionable diagnostics).
  - Touch points: `crates/speccade-cli/src/commands/migrate/`, `docs/legacy/PARITY_MATRIX_LEGACY_SPEC_PY.md`, `crates/speccade-spec/src/recipe/animation/`.
- [ ] `MIGRATE-004` Map legacy `CHARACTER` dict keys to canonical `skeletal_mesh` params (skeleton + parts/body_parts + skinning/export).
  - Deliverable: a tested conversion that emits canonical `skeletal_mesh.blender_rigged_mesh_v1` params and preserves triangle budgets/material intent where possible.
  - Touch points: `crates/speccade-cli/src/commands/migrate/`, `docs/legacy/PARITY_MATRIX_LEGACY_SPEC_PY.md`, `crates/speccade-spec/src/recipe/character/`.
