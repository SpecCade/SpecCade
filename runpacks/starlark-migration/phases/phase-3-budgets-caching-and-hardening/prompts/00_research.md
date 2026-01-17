# Phase 3 Research: Budgets, Caching, and Hardening

## Role

You are a **research agent**. Your job is to understand the current state of budget enforcement, caching, and validation in SpecCade, then document findings and risks.

## Permission Boundary

- **Allowed**: Read files, search codebase, write research artifacts
- **NOT allowed**: Edit code files, run commands, apply patches

## Files to Open First

1. `runpacks/starlark-migration/ARCHITECTURE_PROPOSAL.md` - Pipeline stages, especially C (validation) and E (reports)
2. `runpacks/starlark-migration/phases/phase-3-budgets-caching-and-hardening/SCOPING.md` - Allowed globs and safety notes
3. `crates/speccade-spec/src/lib.rs` - Spec types overview
4. `crates/speccade-spec/src/validation/mod.rs` - Current validation implementation
5. `crates/speccade-spec/src/hash.rs` - Hashing implementation (JCS + BLAKE3)
6. `crates/speccade-cli/src/main.rs` - CLI entry point
7. `PARITY_MATRIX.md` - Determinism expectations

## Research Questions

### Budgets
1. Where are budgets currently enforced (validation vs backend)?
2. What budget types exist (duration, resolution, counts)?
3. How are budget limits configured?
4. Are budgets input-format agnostic (same for JSON and Starlark)?

### Caching
1. Does `generate` currently support caching?
2. What keys would be needed (IR hash, toolchain version, budget profile)?
3. Where should cache artifacts be stored?

### Reports
1. What does the current report format include?
2. How to add provenance (source_hash, stdlib_version)?
3. Is report schema documented?

### Idempotence
1. Is canonicalization currently idempotent?
2. What operations could break idempotence?
3. Are there existing tests for this?

### Hardening
1. What fuzz/property-based testing exists?
2. What edge cases are untested?

## Output Artifacts

Write these files to the phase folder:

### `research.md`
- Current budget enforcement locations
- Caching status and gaps
- Report format analysis
- Canonicalization analysis
- Existing test coverage

### `risks.md`
- Budget bypass risks (where enforcement is missing)
- Cache invalidation risks
- Idempotence edge cases
- Schema compatibility concerns

### `questions.md` (only if blocking)
- Questions that block planning
- Proposed answers or experiments

## Success Criteria

Research is complete when:
- All research questions have documented answers
- `risks.md` identifies at least 3 concrete risks
- No blocking questions remain (or they are documented for escalation)
