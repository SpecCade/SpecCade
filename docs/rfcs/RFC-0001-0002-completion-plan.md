# RFC-0001 & RFC-0002 Completion Plan (SpecCade)

- **Status:** Draft (plan)
- **Last updated:** 2026-01-12

## Goal

Make RFC-0001 and RFC-0002 accurate, fully implemented, and ready to mark **Implemented**.  
Do **not** touch RFC-0003.

## Scope Decisions (locked)

- **OGG is out** of RFC-0001 (remove from the contract and docs).
- RFC-0001/0002 should match **current behavior** once work is done.

## Definition of Done (apply to both RFCs)

- RFC text matches actual behavior (schema + validation + generator).
- Golden specs/tests cover the RFC behavior.
- RFC status lines updated to **Implemented** with a `Last reviewed: YYYY-MM-DD`.

---

## RFC-0001: Canonical Spec Architecture — Completion Checklist

### 1) Contract Surface Alignment (docs + schema + code)

- **Remove OGG from v1 contract**
  - Update `speccade/docs/rfcs/RFC-0001-canonical-spec.md` output format list.
  - Update `speccade/docs/spec-reference/README.md` format list.
  - Update `speccade/schemas/speccade-spec-v1.schema.json` to drop `"ogg"`.
  - Remove `OutputFormat::Ogg` and related uses/tests:
    - `speccade/crates/speccade-spec/src/output.rs`
    - `speccade/crates/speccade-tests/src/harness.rs` (OGG validation stub)
    - `speccade/crates/speccade-spec/src/validation/mod.rs` tests referencing OGG

- **Resolve `.blend` output format mismatch**
  - Decide and execute one path (pick one to unblock completion):
    - **Preferred (simpler, matches actual generation):** remove `"blend"` from
      `speccade/schemas/speccade-spec-v1.schema.json` + docs + `OutputFormat` enum.
    - **Alternative:** keep `"blend"` but mark it **reserved** (not allowed in outputs) and
      update validation + RFC-0001 accordingly.

- **Output kinds: `metadata` / `preview`**
  - Update RFC-0001 to explicitly mark them as **reserved + invalid** for v1 (validation rejects them).
  - Align RFC-0001’s output-kind table with `speccade/crates/speccade-spec/src/validation/mod.rs`.

### 2) Validation Rules Alignment

- Update RFC-0001 Section 7 rules to match actual validation:
  - E006 message → “No primary **or packed** output declared.”
  - Add a note that `metadata`/`preview` output kinds are rejected (use report JSON instead).

### 3) Report Metrics Alignment

- Update RFC-0001 Report Metrics object to include:
  - `animation_frame_count`
  - `animation_duration_seconds`
  - (these exist in `speccade/crates/speccade-spec/src/report.rs`)

### 4) Acceptance Tests

- Ensure `speccade validate` rejects:
  - `metadata` / `preview` outputs.
  - Unsupported output formats (OGG, BLEND if removed).
- Ensure docs + schema + code agree on allowed output kinds/formats.

### 5) Status Update

- After all items above pass, update:
  - `speccade/docs/rfcs/RFC-0001-canonical-spec.md`  
    `Status: Draft` → `Status: Implemented`  
    `Last reviewed: 2026-01-12` (or actual completion date)

---

## RFC-0002: Unopinionated Channel Packing — Completion Checklist

### 1) Define Missing Semantics (spec decisions)

RFC-0002 currently leaves **height-source** and **pattern semantics** ambiguous.  
Lock these down in the RFC before implementation:

- **Height source for `from_height`:**
  - Require a map key **`height`** if any map uses `from_height: true`.
  - If missing, validation fails (use `E012 InvalidRecipeParams` or add a new E024+ code).

- **`from_height` behavior (grayscale maps):**
  - If `from_height: true` and **`ao_strength` is set**, generate AO using
    `AoGenerator` (strength default = `1.0` if omitted; clamp to `[0,1]`).
  - Otherwise, use the raw height map as the grayscale output (0..1).

- **Pattern maps (MapDefinition::Pattern):**
  - Restrict `pattern` to `"noise"` for v1.
  - Allow `noise_type`: `"perlin" | "simplex" | "fbm" | "worley"`.
  - Use `octaves` only for `"fbm"` (ignore/validate otherwise).

Update RFC-0002 examples to include a `height` map where `from_height` is used
(including the ORM/MRE/smoothness examples).

### 2) Validation Updates (spec crate)

Add packed-map validation in `speccade/crates/speccade-spec/src/validation/mod.rs`:

- `value` in `[0.0, 1.0]`
- `ao_strength` in `[0.0, 1.0]`
- `from_height` requires a `height` map (if we choose the rule above)
- `pattern` + `noise_type` must be supported (or invalid params)

### 3) Backend Implementation (generation)

Implement packed map generation in `speccade-backend-texture` and wire it into CLI:

- Add a **shared height map** builder:
  - Use `maps["height"]` if present; error if required and missing.
  - For `pattern: "noise"`, generate grayscale using the noise modules in
    `speccade/crates/speccade-backend-texture/src/noise/`.
  - Seed using `speccade_spec::hash::derive_variant_seed(spec.seed, "height")`.
  - Respect `resolution` + `tileable`.

- Implement `MapDefinition::Pattern` generation:
  - Produce a grayscale buffer (0..1) for the requested noise.
  - Use deterministic seed derived from `spec.seed` + map key.

- Implement `from_height`:
  - Use the shared height map.
  - AO path uses `AoGenerator` (`speccade-backend-texture/src/maps/ao.rs`).
  - Non-AO path returns raw height map.

- Replace the stub in `speccade/crates/speccade-cli/src/dispatch.rs`
  (`generate_map_buffer`) with real generation or move logic into backend crate.

### 4) Golden Tests + Determinism

- Add golden PNG outputs for:
  - `golden/speccade/specs/texture/packed_orm.json`
  - `golden/speccade/specs/texture/packed_mre.json`
  - `golden/speccade/specs/texture/packed_smoothness.json`
- Update or add tests to validate:
  - Deterministic hashes across runs.
  - Channel correctness (AO/roughness/metallic placement).
  - Inversion for smoothness example.

### 5) Docs Cleanup

Update docs to remove “not implemented” notes:

- `speccade/docs/rfcs/RFC-0002-channel-packing.md`
- `speccade/docs/spec-reference/texture.md`
- `speccade/docs/TODOS.md` (remove the packed-texture TODOs once done)

### 6) Status Update

- After all items above pass, update:
  - `speccade/docs/rfcs/RFC-0002-channel-packing.md`  
    `Status: Draft` → `Status: Implemented`  
    `Last reviewed: 2026-01-12` (or actual completion date)

---

## Final Close-Out Checklist

- RFC-0001 & RFC-0002 status set to **Implemented**.
- `speccade validate` passes for all golden specs.
- `speccade generate` produces deterministic outputs for packed textures.
- Docs, schema, and validation are aligned (no unsupported fields in “core” sections).
