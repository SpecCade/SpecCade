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

## How to Use This Doc

- If it's a **task** or **decision**, it belongs here with an ID.
- Other documents should **link here** instead of duplicating task lists.
- Keep items **actionable**: verb + concrete deliverable + (when possible) file/command touch points.

---

## Editor / Real-Time Preview (RFC-0009)

Reference: `docs/rfcs/RFC-0009-editor-architecture.md`

- [x] `EDITOR-001` ~~Decide "editor" delivery shape~~ **RESOLVED: Tauri standalone app only**
  - No VSCode extension (no user demand)
  - Tauri 2.x with Monaco editor, three.js for 3D, Web Audio for sound
  - Single codebase, no abstraction layer needed for v1
  - Layout: `crates/speccade-editor/` (Rust backend) + `editor/` (Tauri frontend)

Resolved questions:
- [x] `EDITOR-Q001` ~~GPU acceleration for preview~~ **RESOLVED: WebGL2-only for v1**
  - three.js with WebGL2 is sufficient for preview quality
  - WebGPU deferred to v2+ when platform support matures
- [x] `EDITOR-Q002` ~~Large mesh handling in preview~~ **RESOLVED: LOD proxies with progressive refinement**
  - Generate low-poly proxy immediately (sub-100ms)
  - Refine to full quality on user request or when idle
  - Show visual indicator for "preview" vs "full quality" state
- [x] `EDITOR-Q003` ~~Collaboration~~ **RESOLVED: Explicitly deferred to v2+**
  - v1 is single-user only
  - No collaboration infrastructure needed

---

## Mesh/Character Verification Loop (RFC-0010)

Reference: `docs/rfcs/RFC-0010-mesh-llm-verification.md`

- [x] `MESHVER-003` ~~VLM integration policy~~ **RESOLVED: Experimental/opt-in with minimal v1 scope**
  - Off by default
  - User provides API key at runtime via `--vlm-key` (not persisted)
  - Only rendered images uploaded (never mesh data)
  - Marked as "experimental" in docs

Resolved questions:
- [x] `MESHVER-Q001` ~~VLM latency targets~~ **RESOLVED: Batch mode only (10-30s acceptable)**
  - VLM verification is manual trigger, not hot-reload
  - Show progress indicator during VLM call
  - 30s default timeout (configurable)
- [x] `MESHVER-Q002` ~~Verification caching~~ **RESOLVED: Cache by spec_hash + render_settings_hash**
  - Geometric metrics always cached (deterministic)
  - VLM results cached with hash key
  - Invalidate on: spec change, render settings change, VLM model change
  - Cache location: `~/.cache/speccade/verification/`
  - `--no-cache` flag to force re-verification
- [x] `MESHVER-Q003` ~~Hallucination guardrails~~ **RESOLVED: Start simple, iterate based on experience**
  - Single prompt, transparent output in v1
  - Show raw VLM response alongside structured report
  - Geometric metrics are ground truth (VLM cannot override)
  - Add ensemble prompts if hallucination becomes a problem

---

## Textures

- [x] `TEX-003` ~~Channel swizzle ops~~ **RESOLVED: Not needed now**
  - Current approach (combine maps to RGB channels on export) is sufficient
  - Users can create separate maps and combine them in the output stage
  - Revisit if users request more granular in-pipeline channel manipulation
- [x] `TEX-008` ~~Define and implement matcap generation (`texture.matcap_v1`) for stylized shading presets.~~ Done: 2026-01-24
  - Tier 1 implementation with 8 presets (ToonBasic, ToonRim, Metallic, Ceramic, Clay, Skin, Plastic, Velvet), toon steps, outline, curvature/cavity masks.
- [x] `TEX-009` ~~Add a material preset system for stable art direction ("preset + parameterization" at CLI-time).~~ Done: 2026-01-24
  - Tier 1 implementation with 8 PBR presets (ToonMetal, StylizedWood, NeonGlow, CeramicGlaze, SciFiPanel, CleanPlastic, RoughStone, BrushedMetal). Generates 4 outputs: albedo, roughness, metallic, normal.

---

## New Asset Types (2D VFX / UI / Fonts)

- [x] `GEN-005` ~~Add VFX particle "material/profile" presets (additive/soft/distort/etc.).~~ Done: 2026-01-24
  - Tier 1 metadata-only recipe with 6 profiles (Additive, Soft, Distort, Multiply, Screen, Normal). Outputs rendering hints for particle systems.
- [x] `GEN-006` ~~Add UI kit presets and item card templates with slots (icon/rarity/background).~~ Done: 2026-01-24
  - Tier 1 implementation generating PNG atlas + metadata JSON. 5 rarity tiers (Common, Uncommon, Rare, Epic, Legendary) with customizable slots (icon, rarity indicator, background).
- [ ] `GEN-007` Add deterministic damage-number sprites (font + outline + crit styles).

---

## Mesh/Animation Feature Expansion (Blender Tier)

- [ ] `MESH-008` Add a render-to-sprite bridge (render `static_mesh` with lighting preset -> `sprite.sheet_v1`).
- [ ] `MESH-009` Add modular kit generators (walls/doors/pipes) built from primitives + modifiers.
- [ ] `MESH-010` Add organic modeling gap-fill (metaballs -> remesh -> smooth -> displacement noise) with strict budgets.
- [ ] `MESH-011` Add shrinkwrap workflows (armor/clothes wrapping onto body parts) with strict stability validation.
- [ ] `MESH-012` Add boolean kitbashing (union/difference + cleanup) with determinism/validation constraints.
- [ ] `MESH-013` Add animation helper presets (IK targets + constraint presets) for procedural walk/run cycles.

### Skeletal Animation / Rigging / IK (Blender Tier)

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
