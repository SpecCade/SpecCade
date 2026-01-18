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
- [ ] `LLM-006` Add higher-level Starlark constructors to reduce raw recipe dict authoring.
  - Deliverable: `audio_spec(...)`, `texture_spec(...)`, `mesh_spec(...)` helpers analogous to `music_spec(...)`.
  - Touch points: `crates/speccade-cli/src/compiler/stdlib/{core,audio,texture,mesh}.rs`.
- [ ] `LLM-007` Iteration-speed features for agentic loops (preview + partial generation + caching).
  - Deliverable: at least one of: `generate --preview`, per-layer generation for audio, or content-addressed caching keyed by canonical hashes + backend versions.
- [ ] `LLM-009` Extend `speccade analyze` with embedding export for similarity search (explicit opt-in).
- [ ] `LLM-010` Add batch analysis modes (`--input-dir`, CSV/JSONL outputs) for clustering/auditing.
- [ ] `LLM-011` Add a real-time analysis mode (e.g. WebSocket server) for editor/iterative workflows.

---

## Editor / Real-Time Preview (RFC-0009)

Reference: `docs/rfcs/RFC-0009-editor-architecture.md`

- [ ] `EDITOR-001` Decide "editor" delivery shape (Tauri app vs VSCode extension vs both).
  - Deliverable: a committed decision + minimal repo layout plan (new crate? new top-level directory?).
- [ ] `EDITOR-002` Implement CLI-side preview artifacts (useful even without a full editor).
  - Deliverable: standardized preview outputs (audio waveform/spectrogram PNG; texture thumbnails) emitted by `generate` and recorded in reports.
  - Touch points: `crates/speccade-cli`, `crates/speccade-spec` report structures.
- [ ] `EDITOR-003` Add fast "preview mode" generation knobs.
  - Deliverable: deterministic downscaled/shortened preview generation flags (e.g. audio first N ms; texture 256x256), clearly labeled as preview.

Open questions (track decisions here, not in the RFC):
- [ ] `EDITOR-Q001` GPU acceleration for preview (WebGPU) vs WebGL2-only.
- [ ] `EDITOR-Q002` Large mesh handling in preview (LOD/proxies) strategy.
- [ ] `EDITOR-Q003` Collaboration: explicitly defer to v2 or define minimal v1 stance.

---

## Mesh/Character Verification Loop (RFC-0010)

Reference: `docs/rfcs/RFC-0010-mesh-llm-verification.md`

- [ ] `MESHVER-001` Implement geometric metrics for Tier-2 assets (bounds, verts/faces, UV sanity, bone counts).
  - Deliverable: deterministic metrics emitted into `${asset_id}.report.json` for mesh/character/animation outputs.
- [ ] `MESHVER-002` Add a constraint/verification surface (validation-time vs post-generate).
  - Deliverable: a minimal constraint schema and a place to run it (e.g. `speccade verify --spec ...`).
- [ ] `MESHVER-003` Decide if/when VLM integration is supported (and how it is configured).
  - Deliverable: explicit policy: off by default; user-provided credentials; what gets uploaded (renders only).

Open questions:
- [ ] `MESHVER-Q001` Acceptable VLM latency targets for interactive use.
- [ ] `MESHVER-Q002` Verification caching keyed by spec hash (and what invalidates it).
- [ ] `MESHVER-Q003` Guardrails for VLM hallucinations (ensemble prompts, thresholds, human override).

---

## Music (Compose / Pattern IR)

- [ ] `MUSIC-001` Add snapshot tests for `docs/examples/music/*.expanded.params.json` via `speccade expand`.
  - Deliverable: tests that compare expansion output to checked-in snapshots (stable formatting).
- [ ] `MUSIC-002` Extend Pattern IR operators and hard limits (keep determinism + reviewability).
  - Deliverable: RFC + schema/types changes + tests for any new ops; avoid silent behavior changes.
- [ ] `MUSIC-003` Improve `speccade expand` UX for review workflows.
  - Deliverable: stable formatting, optional file output, and diff-friendly mode.
- [ ] `MUSIC-004` Tighten XM/IT parity checks and document known differences.
  - Deliverable: automated structural checks + a short listening checklist doc.
- [ ] `MUSIC-008` Expand tracker effect coverage + validation (arp/porta/vibrato/retrig/vol slide, etc.).
- [ ] `MUSIC-009` Add deterministic swing/humanize macros to Pattern IR (timing + velocity ranges with explicit constraints).

## Music (Content / Workflow)

- [ ] `MUSIC-005` Prefer `TrackerInstrument.ref` (external `audio_v1` specs) in examples/goldens where possible.
- [ ] `MUSIC-006` Add a tuned, high-quality drum example (kick/snare/hat) and gate it with basic metrics (no clipping, sane levels).
- [ ] `MUSIC-007` Grow "genre kits" as data packages: curated compose defs + instrument refs + timebase/harmony defaults.
- [ ] `MUSIC-010` Add cue templates (`loop_low/loop_main/loop_hi`, stingers, transitions) as compile-time helpers or templates.

---

## Audio

- [ ] `AUDIO-001` Keep expanding and tuning the `audio_v1` preset library with QA gates.
  - Deliverable: baseline checks (no clipping, sane RMS, low DC offset) applied in CI or via `speccade analyze`.
- [ ] `AUDIO-002` Add an "audio audit" report command (or `analyze` sub-mode) to catch regressions.
  - Deliverable: peak/RMS/DC metrics for golden audio fixtures + budgeted tolerances.
- [ ] `AUDIO-003` Improve loop-point generation + click-free defaults for tracker instrument baking.
- [ ] `AUDIO-004` Add missing effects needed for production mixing (start with parametric EQ + limiter).
  - Notes: also consider gate/expander and stereo widener.
- [ ] `AUDIO-005` Add missing filter types (notch/allpass first; then comb/formant/shelves as needed).
- [ ] `AUDIO-006` Add missing synthesis types with highest leverage (unison/supersaw, waveguide, bowed string, membrane/drum).
- [ ] `AUDIO-007` Add loudness targets (LUFS) and true-peak limiting workflows for production-ready output levels.
- [ ] `AUDIO-008` Add one-shot + loop pairing helpers (transient + loopable sustain from the same recipe).
- [ ] `AUDIO-009` Add foley layering helpers (impact builder: transient/body/tail; whoosh builder: noise + sweep).
- [ ] `AUDIO-010` Add batch SFX variation sets (seed sweeps + constraints + sample-set export).
- [ ] `AUDIO-011` Expand LFO targets where it materially improves sound design (pulse_width, fm_index, delay_time, etc.).

---

## Textures

- [ ] `TEX-001` Add additional `texture.procedural_v1` ops (blur/warp/morphology/blend modes/UV transforms).
  - References: RFC-0005/0006 future-work notes.
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

---

## Tooling / QA

- [ ] `QA-001` Grow Tier-1 golden corpus (`golden/`) and run it in CI.
- [ ] `QA-002` Add "inspect" style commands/flags where helpful for review (compose->expanded JSON, intermediate texture maps).
- [ ] `QA-003` Add content-addressed caching keyed by canonical spec/recipe hash + backend versions (iteration speed).
- [ ] `QA-004` Add perceptual diffing / quality controls (image SSIM/DeltaE; audio loudness/spectral) where feasible.
- [ ] `QA-005` Add profiling/observability: per-stage timings, memory stats, and reproducible perf runs.
- [ ] `QA-006` Define a plugin/backends extension story (subprocess or WASM) with strict I/O contracts + determinism reporting.

---

## Migration (Legacy `.studio` / spec.py)

Reference: `docs/MIGRATION.md`

- [ ] `MIGRATE-001` Implement a real params mapping layer in the migrator (legacy keys -> canonical recipe schemas).
- [ ] `MIGRATE-002` Add migration fixtures + tests that validate migrated specs against `speccade validate`.
