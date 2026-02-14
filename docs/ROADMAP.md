# SpecCade Roadmap (Single Source of Truth)

This file is the **single source of truth** for:

- Planned work (features, refactors, tooling)
- Open questions that require a decision
- Cross-document follow-ups (RFCs, docs, validation/schema alignment)

Design rationale and proposals live in `docs/rfcs/`. The canonical spec contract lives in `docs/spec-reference/` and `crates/speccade-spec/`.

## Completed Milestones

The following major work areas are complete:

- **LLM-Native Authoring** (LLM-001–011): Machine-readable stdlib, structured CLI output, budget profiles, analysis/metrics, templates, preview generation, embeddings, batch analysis, and WebSocket server.
- **Audio** (AUDIO-001–011): Preset library with QA gates, audit/analyze tools, loop-point generation, effects (EQ/limiter), filters (notch/allpass), synthesis (supersaw/waveguide/membrane), LUFS/true-peak, oneshot/loop helpers, foley layering, batch variations, LFO expansion.
- **Music** (MUSIC-001–010): Pattern IR snapshot tests, extended operators, expand UX, XM/IT parity, tracker effects, swing/humanize, genre kits, cue templates, drum quality examples.
- **Mesh/Animation Core** (MESH-001–007, CHAR-001–003, ANIM-001–003): Modifier stacks, UV unwrap, normals automation, LOD generation, collision meshes, navmesh metadata, baking suite, Starlark constructors, validation/verification.
- **Textures Core** (TEX-001–002, 004–007): Procedural ops (blur/warp/morphology), templates, Wang tiles, trimsheets, decals, splat sets.
- **New Asset Types Core** (GEN-001–004): Spritesheets/animations, VFX flipbooks, UI nine-slice/icon sets, bitmap fonts.
- **Tooling/QA Core** (QA-001–005, 007–009): Golden corpus, inspect/compare commands, caching, perceptual diffing, profiling, stdlib drift/coverage guards, Tier-2 validation.
- **Lint System** (QA-010, EDITOR): Integrated lint into `speccade generate` (lint-before-generate gate), structural metrics for LLM-friendly 3D feedback, editor generate panel with lint integration.

## How to Use This Doc

- If it's a **task** or **decision**, it belongs here with an ID.
- Other documents should **link here** instead of duplicating task lists.
- Keep items **actionable**: verb + concrete deliverable + (when possible) file/command touch points.

---

## Editor / Real-Time Preview (RFC-0009)

Reference: `docs/rfcs/RFC-0009-editor-architecture.md`

### Resolved Decisions

  - No VSCode extension (no user demand)
  - Tauri 2.x with Monaco editor, three.js for 3D, Web Audio for sound
  - Single codebase, no abstraction layer needed for v1
  - Layout: `crates/speccade-editor/` (Rust backend) + `editor/` (Tauri frontend)
  - three.js with WebGL2 is sufficient for preview quality
  - WebGPU deferred to v2+ when platform support matures
  - Generate low-poly proxy immediately (sub-100ms)
  - Refine to full quality on user request or when idle
  - Show visual indicator for "preview" vs "full quality" state
  - v1 is single-user only
  - No collaboration infrastructure needed

### Implementation Tasks

  - Tauri 2.x app at `editor/` with full frontend (Monaco, three.js, Web Audio) + `crates/speccade-editor/` Rust backend with IPC commands.
  - Monaco wrapper component with Starlark language definition, token provider for keywords/builtins/stdlib, 500ms debounced validation with inline error display.
  - File watcher in `crates/speccade-editor/src/watcher.rs` with 100ms debounce, `file-changed` events to frontend, eval + validate on change.
  - `editor/src/components/MeshPreview.ts` (443 lines) with WebGLRenderer, OrbitControls, GLTFLoader, lighting, grid, quality badges, auto-refine.
  - `editor/src/components/AudioPreview.ts` (827 lines) + `MusicPreview.ts` with waveform visualization, spectrum analyzer, play/stop, loop regions, tracker module playback.
  - QEM edge-collapse decimation with silhouette preservation, auto-proxy for >10k triangle meshes, quality badges, refine button, auto-refine on 2s idle.

---

## Mesh/Character Verification Loop (RFC-0010)

Reference: `docs/rfcs/RFC-0010-mesh-llm-verification.md`

  - Off by default
  - User provides API key at runtime via `--vlm-key` (not persisted)
  - Only rendered images uploaded (never mesh data)
  - Marked as "experimental" in docs

Resolved questions:
  - VLM verification is manual trigger, not hot-reload
  - Show progress indicator during VLM call
  - 30s default timeout (configurable)
  - Geometric metrics always cached (deterministic)
  - VLM results cached with hash key
  - Invalidate on: spec change, render settings change, VLM model change
  - Cache location: `~/.cache/speccade/verification/`
  - `--no-cache` flag to force re-verification
  - Single prompt, transparent output in v1
  - Show raw VLM response alongside structured report
  - Geometric metrics are ground truth (VLM cannot override)
  - Add ensemble prompts if hallucination becomes a problem

---

## Textures

  - Current approach (combine maps to RGB channels on export) is sufficient
  - Users can create separate maps and combine them in the output stage
  - Revisit if users request more granular in-pipeline channel manipulation
  - Tier 1 implementation with 8 presets (ToonBasic, ToonRim, Metallic, Ceramic, Clay, Skin, Plastic, Velvet), toon steps, outline, curvature/cavity masks.
  - Tier 1 implementation with 8 PBR presets (ToonMetal, StylizedWood, NeonGlow, CeramicGlaze, SciFiPanel, CleanPlastic, RoughStone, BrushedMetal). Generates 4 outputs: albedo, roughness, metallic, normal.

---

## New Asset Types (2D VFX / UI / Fonts)

  - Tier 1 metadata-only recipe with 6 profiles (Additive, Soft, Distort, Multiply, Screen, Normal). Outputs rendering hints for particle systems.
  - Tier 1 implementation generating PNG atlas + metadata JSON. 5 rarity tiers (Common, Uncommon, Rare, Epic, Legendary) with customizable slots (icon, rarity indicator, background).
  - Tier 1 implementation with 3 style types (Normal, Critical, Healing). Generates PNG atlas + metadata JSON with outline and optional glow effects.

---

## Mesh/Animation Feature Expansion (Blender Tier)

  - Tier 2 (Blender) implementation with camera/lighting presets, multi-angle rendering, atlas packing.
  - Tier 2 (Blender) implementation with Wall (cutouts, baseboard, crown), Pipe (segments, bends, T-junctions), and Door (frame, panel, hinges) kit types.
  - Tier 2 (Blender) implementation with metaball primitives (sphere, capsule, cube, ellipsoid), remeshing modes (voxel, sharp, smooth), and displacement noise.
  - Tier 2 (Blender) implementation with nearest_surface/project/nearest_vertex modes, offset, smooth iterations, self-intersection validation. 3 golden specs.
  - Tier 2 (Blender) implementation with union/difference/intersect operations, exact/fast solver, cleanup (merge doubles, recalc normals). 3 golden specs.
  - Wired `skeletal_animation.helpers_v1` to backend dispatch: `animation_helpers.rs` backend module, Blender entrypoint handler, CLI dispatch. Walk cycle, run cycle, and idle sway presets generate end-to-end.
  - Added `save_blend: bool` to `MeshExportSettings`, save `.blend` in all Tier 2 mesh handlers (static mesh, modular kit, organic sculpt, mesh-to-sprite), parse `blend_path` in Rust backend. CLI `--save-blend` flag for forcing.
  - Plan: `docs/plans/2026-01-28-blender-export-for-meshes.md`

### Skeletal Animation / Rigging / IK (Blender Tier)

  - **IK stretch**: ✅ `StretchSettings` in `skeletal/settings.rs`, Blender `apply_stretch_settings()` at line 2931.
  - **Foot roll**: ✅ `FootSystem` in `skeletal/foot.rs`, Blender `setup_foot_system()` + `create_foot_roll_bones()`.
  - **Spine presets**: ❌ Not implemented (no SpineIK preset in `IkChainPreset` enum). Add if needed.
  - **Ball socket**: ✅ `BoneConstraint::Ball` in `constraints.rs`, Blender `_setup_ball_constraint()` at line 2542.
  - **Planar**: ✅ `BoneConstraint::Planar` in `constraints.rs`, Blender `_setup_planar_constraint()` at line 2576.
  - **Post-generation validation**: ❌ Not in validation constraints enum. Add `MaxBallViolations`/`MaxPlanarViolations` if runtime validation needed.
  - `RootMotionSettings` with 4 modes (Keep, Extract, BakeToHip, Lock) and per-axis control. Added to clip and rigged animation params. Blender implementation for all modes. `root_motion_mode` in report metrics.

---

## Tooling / QA

  - Extension types in speccade-spec (manifest, contract, determinism levels), subprocess runner with timeout/hash verification, reference implementation in examples/extensions/simple-subprocess/, architecture docs.
  - Non-opinionated geometry metrics in reports: aspect ratios, symmetry, component adjacency, bone coverage, scale reference. Blender computation in `structural_metrics.py`, Rust types in `report/structural.rs`, docs in `spec-reference/structural-metrics.md`.

---

## Music

- [ ] `MUSIC-011` Add instrument pitch validation and auto-correction for XM/IT export.
  - Pitch deviation measurement (`xm_pitch_deviation_cents`, `it_pitch_deviation_cents`) that simulates each tracker format's playback formula and reports error in cents.
  - Fix IT `c5_speed` truncation bug (round instead of truncate).
  - Surface `pitch_deviation_cents` in `MusicInstrumentLoopReport` for every exported instrument.
  - Plan: `docs/plans/2026-01-28-instrument-pitch-validation.md`

---

