# Phase 3 Scoping: Budgets, Caching, and Hardening

## Allowed File Globs

These paths are in-scope for this phase:

```
crates/speccade-cli/**
crates/speccade-spec/**
crates/speccade-tests/**
crates/**
docs/**
schemas/**
golden/**
```

## Must-Not-Touch Guidance

- **Backend internals** (`crates/speccade-*/src/backend*.rs`): Do not refactor backend execution logic unless directly related to budget enforcement at validation stage.
- **Existing hash implementation** (`speccade_spec::hash`): Extend, do not replace. JCS + BLAKE3 strategy is SSOT per `ARCHITECTURE_PROPOSAL.md`.
- **Tier 2 backend validation**: Do not change external tool integrations (Blender, etc.) unless adding budget checks.
- **Phase 1/2 artifacts**: Do not modify files created by earlier phases unless fixing bugs.

## Safety Notes

### Determinism
- Budget checks must be deterministic (same input -> same validation result).
- Caching keys must be derived from canonical IR hash + toolchain version.

### Hashing
- Continue using RFC 8785 JCS + BLAKE3 per existing `speccade_spec::hash`.
- Provenance fields (source_hash, stdlib_version) are metadata, not part of IR hash.

### Schema Stability
- Do not break Spec v1 schema compatibility.
- New report fields (provenance) are additive.

### Idempotence
- Canonicalization must be idempotent: `canonicalize(canonicalize(x)) == canonicalize(x)`.
- Tests must verify this property.

## Budget Scope

Budgets to enforce at validation stage (examples from existing backends):
- Audio: max duration, sample rate limits
- Texture: max resolution, format constraints
- Mesh: max vertex/face counts
- General: node count limits, nesting depth

Budgets should be configurable via profile (e.g., `--budget-profile zx-8bit`).
