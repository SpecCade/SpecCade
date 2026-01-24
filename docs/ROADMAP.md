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

- [ ] `EDITOR-001` Decide "editor" delivery shape (Tauri app vs VSCode extension vs both).
  - Deliverable: a committed decision + minimal repo layout plan (new crate? new top-level directory?).

Open questions (track decisions here, not in the RFC):
- [ ] `EDITOR-Q001` GPU acceleration for preview (WebGPU) vs WebGL2-only.
- [ ] `EDITOR-Q002` Large mesh handling in preview (LOD/proxies) strategy.
- [ ] `EDITOR-Q003` Collaboration: explicitly defer to v2 or define minimal v1 stance.

---

## Mesh/Character Verification Loop (RFC-0010)

Reference: `docs/rfcs/RFC-0010-mesh-llm-verification.md`

- [ ] `MESHVER-003` Decide if/when VLM integration is supported (and how it is configured).
  - Deliverable: explicit policy: off by default; user-provided credentials; what gets uploaded (renders only).

Open questions:
- [ ] `MESHVER-Q001` Acceptable VLM latency targets for interactive use.
- [ ] `MESHVER-Q002` Verification caching keyed by spec hash (and what invalidates it).
- [ ] `MESHVER-Q003` Guardrails for VLM hallucinations (ensemble prompts, thresholds, human override).

---

## Textures

- [ ] `TEX-003` Decide if richer channel "swizzle/component extract" ops are needed for packing workflows.
- [ ] `TEX-008` Define and implement matcap generation (`texture.matcap_v1`) for stylized shading presets.
  - Notes: toon steps/ramps, curvature/cavity masks, outline, "preset + overrides" art direction.
- [ ] `TEX-009` Add a material preset system for stable art direction ("preset + parameterization" at CLI-time).

---

## New Asset Types (2D VFX / UI / Fonts)

- [ ] `GEN-005` Add VFX particle "material/profile" presets (additive/soft/distort/etc.).
  - Candidate: `vfx.particle_profile_v1`.
- [ ] `GEN-006` Add UI kit presets and item card templates with slots (icon/rarity/background).
  - Candidate: `ui.item_card_v1`.
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
