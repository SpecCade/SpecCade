# Phase 1 Scoping: Allowed Files and Boundaries

## Allowed file globs

From `PHASES.yaml`, this phase may edit:

```
crates/speccade-cli/**
crates/speccade-spec/**
crates/speccade-tests/**
schemas/**
docs/**
golden/**
Cargo.toml
Cargo.lock
```

---

## Must-not-touch guidance

Do NOT modify these areas (defer to later phases or other work):

| Area | Reason |
|------|--------|
| `crates/speccade-backend-*/**` | Backends must remain Starlark-unaware; they consume canonical IR only |
| `crates/speccade-blender/**` | Tier 2 backend; out of scope |
| Existing JSON spec semantics | Non-breaking constraint; JSON users must not see behavior changes |
| `speccade_spec::Spec` struct shape | IR contract remains Spec v1; do not add Starlark-specific fields |

---

## Safety notes

### Determinism
- Hashing must remain on canonical IR (post-resolve), not on Starlark source
- Use existing `speccade_spec::hash` (RFC 8785 JCS + BLAKE3)
- Do not introduce non-deterministic Starlark builtins (e.g., random, time)

### Schema stability
- `schemas/` JSON schemas must continue to validate canonical IR
- If Starlark introduces new IR features, schema updates are required (but avoid in Phase 1)

### Hashing contract
- `ir_hash` in reports must be computed on canonical IR
- New `source_hash` field may be added to reports for provenance (Starlark source hash)

---

## New crate guidance

If a new crate is needed (e.g., `speccade-compiler` or `speccade-starlark`):
- Create under `crates/`
- Add to workspace `Cargo.toml`
- Keep it minimal: parsing + evaluation + IR emission
- Do NOT add backend dependencies to this crate

---

## Justification requirements

If you must touch an out-of-scope file:
1. Record the file path and reason in `ARTIFACTS.md` decision log
2. Explain why it cannot be deferred
3. Ensure the change is minimal and reversible
