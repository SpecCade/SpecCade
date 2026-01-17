# Starlark -> Canonical JSON IR Migration (Architecture Proposal)

This is the **SSOT** for the migration runpacks in this folder. Phase runpacks should not "re-decide" these points; they can refine details, but must record changes explicitly.

## Core decision

- **Primary authoring format:** Starlark (human/LLM friendly)
- **Canonical backend contract:** existing SpecCade JSON Spec v1 ("IR/assembly")
- **SpecCade becomes compiler + VM:**
  - Starlark eval -> resolve/expand -> validate/budget -> execute generators
- **Custom tools** can emit JSON IR directly if it validates to the same canonical IR and budgets.

## Pipeline stages (A-E)

### A) Parse authoring input
- Accept either:
  - Starlark spec (`*.star`, `*.bzl`, etc.)
  - Raw JSON IR (`*.json`) matching Spec v1
- Produce `SourceDoc { kind, bytes, origin_path, source_hash }`.

### B) Normalize / Resolve
- Convert input into canonical `speccade_spec::Spec`:
  - Starlark returns a JSON-like structure -> parse into `Spec`
  - JSON IR parses directly into `Spec`
- Resolve authoring-level constructs:
  - Expand `music.tracker_song_compose_v1` -> `music.tracker_song_v1`
  - Future: presets/macros/default outputs
- Canonicalize only where semantics permit (set-like arrays, id-sorted nodes).

### C) Validation (schema + invariants + budgets)
- Validate on **canonical IR**, independent of source format:
  - Schema/shape + invariants (existing `speccade_spec::validation`)
  - Hard budgets (duration, resolution caps, node counts, etc.)

### D) Execution
- Dispatch based on canonical IR only (existing CLI dispatch stays).
- No backend should need Starlark knowledge.

### E) Output + determinism report
- Artifacts under `--out-root`
- `${asset_id}.report.json` includes:
  - `ir_hash` (post-resolve canonical IR)
  - provenance: `source_kind`, `source_hash`, and `stdlib_version` (for Starlark)

## IR contract and versioning

- Canonical IR remains **Spec v1** (fields/types defined in `crates/speccade-spec` and `schemas/`).
- `spec_version` remains the IR version gate (no breaking change in this migration).
- Introduce `stdlib_version` as a toolchain/provenance concept (report + cache key), not necessarily embedded into IR.

## Storage policy (recommended)

- Commit Starlark (`*.star`) as SSOT for humans.
- Generate canonical IR on demand and cache it.
- Optionally commit IR snapshots for releases/distribution.

## CLI contract (target)

- `speccade eval <spec.star|ir.json>` -> prints canonical IR JSON
- `speccade validate <spec.star|ir.json>`
- `speccade generate <spec.star|ir.json> --out-root <dir>`
- Optional: `speccade explain`, `--json` diagnostics, `--budget-profile`

## Determinism + hashing

- Continue to use RFC 8785 JCS + BLAKE3 for canonical hashing (already implemented in `speccade_spec::hash`).
- Compute hashes on canonical IR (post-resolve).
- Tier 1: byte-identical outputs; Tier 2: metric validation (existing policy).
