# RFC-0007: Procedural Texture Templates (“Texture Kits”)

- **Status:** COMPLETED
- **Author:** SpecCade Team
- **Created:** 2026-01-13
- **Target Version:** SpecCade v1 (pre-release)
- **Depends on:** RFC-0006 (Unified Procedural Textures)
- **Last reviewed:** 2026-01-13

## Summary

RFC-0006 intentionally hard-breaks procedural texturing down to one recipe kind (`texture.procedural_v1`) to eliminate “material vs normal vs packed vs graph” confusion.

This RFC brings back the *ergonomics* people liked from the old “modes” without reintroducing multiple recipe kinds:

- Ship a curated set of **texture templates** (“texture kits”) implemented purely as `texture.procedural_v1` specs.
- Optionally add CLI support to **list, copy, and instantiate** those templates as starting points.

Templates are **content**, not new generator types. The core contract remains:

> “Build a deterministic DAG. Name your nodes. Bind outputs to named nodes.”

## 1. Motivation

After unification, a new user will ask:

- “Okay, but how do I quickly make a believable brick material set?”
- “What’s a sane default for an ORM packed texture?”
- “How do I structure graphs so they stay readable?”

Previously, those answers were hidden behind multiple recipe kinds (`material_v1`, `normal_v1`, `packed_v1`), which reduced initial effort but increased conceptual complexity.

We want:

- **One recipe kind** (no confusion)
- **Fast starts** (templates)
- **Reusability** (copy and modify, like audio preset packs)
- **Unopinionated core** (templates are optional and non-binding)

This mirrors the role of SpecCade’s existing `packs/preset_library_v1` audio presets: a sane, curated starting library that doesn’t change the underlying primitives.

## 2. Goals

- Provide “basic templates” for common procedural texturing workflows.
- Keep the canonical texture surface area to a single recipe kind (`texture.procedural_v1`).
- Keep templates deterministic and engine-agnostic.
- Make templates easy to discover, copy, and modify.
- Ensure templates demonstrate good graph hygiene (naming, staging, output binding).

## 3. Non-goals (v1)

- Adding new node ops to support templates (templates must use whatever `texture.procedural_v1` provides).
- A full templating language or parameter system embedded into the spec format.
- A node editor UI.
- Cross-spec graph dependencies (e.g., “use another asset’s generated PNG as an input”).

## 4. Definitions

### 4.1 “Template”

A template is a **canonical SpecCade spec file** whose `recipe.kind` is `texture.procedural_v1` and whose purpose is to serve as a reusable starting point.

Templates are valid specs. They can be generated directly, or copied and edited into new assets.

### 4.2 “Texture Kit”

A texture kit is a **named collection** of related templates (e.g., “PBR basics”, “stylized ramps”, “packed conventions”). Kits are organizational; they do not change semantics.

## 5. Proposed Design

### 5.1 Storage: Pack Layout

Extend the existing preset pack idea by adding a texture section (either as a new pack or by extending `preset_library_v1`):

Option A (extend existing pack):

- `packs/preset_library_v1/texture/*.json`

Option B (new pack dedicated to textures):

- `packs/texture_kits_v1/texture/*.json`

Each template is a normal spec file with:

- `asset_type: "texture"`
- `recipe.kind: "texture.procedural_v1"`
- one or more `primary` outputs with `format: "png"` and `source` bindings

### 5.2 Naming conventions

To keep “template” asset ids from colliding with production assets:

- Prefer a prefix such as `preset_texture_*` (if kept inside `preset_library_v1`), or `kit_texture_*` (if in a dedicated pack).

### 5.3 Template categories (recommended minimum set)

These are *recommended* templates; the exact set can evolve without changing the spec contract.

**Single-output building blocks**

- `noise_height_basic` (tileable height field)
- `mask_threshold_basic` (mask generation pattern)
- `color_ramp_stylized` (stylized albedo from height/noise)

**Normal workflows**

- `normal_from_height_basic` (height → normal, strength knob as constant node)
- `normal_from_height_with_mask` (masked detail normals)

**Packing workflows**

- `packed_orm_example` (occlusion/roughness/metallic packed into RGB via `compose_rgba`)
- `packed_mre_example` (metallic/roughness/emissive packed into RGB)

**Multi-output “material-like” sets**

These templates demonstrate that a “material” is just multiple named outputs:

- `material_set_basic` outputs `albedo`, `normal`, `roughness`, `metallic`, `ao` as separate PNGs
- `material_set_with_packed_orm` outputs both separate maps and a packed ORM

### 5.4 Output conventions

Templates must remain unopinionated at the contract level, but can demonstrate conventions as examples:

- Separate outputs: `albedo.png`, `normal.png`, `roughness.png`, `metallic.png`, `ao.png`
- Packed outputs: `orm.png` (or `packed_orm.png`)

These names are not reserved; they are examples.

### 5.5 Optional CLI UX (recommended)

Add simple ergonomics so templates feel like “kits” rather than hidden files:

- `speccade template list --asset-type texture`
- `speccade template show <id>` (prints path + description)
- `speccade template copy <id> --to <path>` (copies JSON verbatim)

No parameterization is required for v1; users edit the resulting spec (just like audio presets).

## 6. Determinism + Production Readiness

Templates are only as production-ready as the underlying procedural texture backend:

- They must be Tier 1 deterministic (same spec + seed → same bytes).
- They should include at least one “golden” hash fixture per category to prevent regressions.
- They should avoid fragile tricks (e.g. relying on resolution-divisibility edge cases) unless documented.

## 7. Validation

Templates are validated the same way as any other `texture.procedural_v1` spec.

No extra template-only validation is required.

## 8. Migration / Compatibility

This RFC assumes RFC-0006’s hard break is accepted:

- All texture templates must use `texture.procedural_v1`.
- Old recipe kinds are not used in templates.

## 9. Tracking

All implementation follow-ups are tracked in `docs/ROADMAP.md` under **Textures** and **Tooling / QA**.

## 10. Alternatives Considered

### 10.1 Reintroduce “modes” as recipe kinds

Rejected: it reintroduces user confusion and duplicates validation/dispatch logic.

### 10.2 Add spec-level templating/parameterization

Deferred: a parameter system can be valuable, but it adds complexity and deserves its own RFC once we have real needs.
