# Phase 2 Scoping: Allowed Files and Boundaries

## Allowed file globs

From `PHASES.yaml`, this phase may edit:

```
crates/speccade-cli/**
crates/speccade-spec/**
crates/speccade-tests/**
crates/**              # new crate(s) for starlark stdlib
docs/**
golden/**
packs/**
schemas/**
```

Note: `Cargo.toml` and `Cargo.lock` are implicitly allowed when adding new crates.

---

## Must-not-touch guidance

Do NOT modify these areas (defer to later phases or other work):

| Area | Reason |
|------|--------|
| `crates/speccade-backend-*/**` | Backends must remain Starlark-unaware; they consume canonical IR only |
| `crates/speccade-blender/**` | Tier 2 backend; out of scope |
| Phase 1 compiler core logic | Compiler is stable; only extend stdlib, not rewrite |
| Existing JSON spec semantics | Non-breaking constraint; JSON users must not see behavior changes |
| `speccade_spec::Spec` struct shape | IR contract remains Spec v1; stdlib emits existing types |

---

## Safety notes

### Determinism
- All stdlib functions MUST be deterministic (no random, no time, no network)
- Stdlib outputs must hash identically for same inputs + seed
- Golden tests enforce byte-identical canonical IR

### Schema stability
- stdlib emits existing IR types; no new schema fields in Phase 2
- If new fields are needed, document in decision log and update `schemas/`

### LLM-friendliness
- stdlib API should use simple, flat function signatures
- Avoid deep nesting; prefer explicit parameters over config objects
- Error messages must include stable error codes (E0001, E0002, etc.)
- Support `--json` output for machine-parseable diagnostics

### Hashing contract
- `ir_hash` remains computed on canonical IR (unchanged from Phase 1)
- `source_hash` covers Starlark source for provenance
- Consider `stdlib_version` in cache keys (Phase 3 may formalize)

---

## New crate guidance

If a new crate is needed (e.g., `speccade-starlark-stdlib`):
- Create under `crates/`
- Add to workspace `Cargo.toml`
- Keep it focused: stdlib function definitions only
- Do NOT add backend dependencies to this crate
- Re-export from `speccade-cli` or compiler crate

---

## Examples and packs guidance

Example `.star` files should be placed in:
- `packs/examples/` for user-facing examples
- `golden/` for determinism test fixtures

Example naming convention:
- `audio_synth_basic.star` - simple audio example
- `texture_noise_perlin.star` - texture generation example
- `mesh_primitive_cube.star` - static mesh example

Each example must have a corresponding `.expected.json` golden file.

---

## Justification requirements

If you must touch an out-of-scope file:
1. Record the file path and reason in `ARTIFACTS.md` decision log
2. Explain why it cannot be deferred
3. Ensure the change is minimal and reversible
