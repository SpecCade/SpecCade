# Phase 3 Plan: Budgets, Caching, and Hardening

## Role

You are a **planning agent**. Your job is to produce an implementable plan for budget unification, caching, provenance, and hardening tests.

## Permission Boundary

- **Allowed**: Read files, write planning artifacts
- **NOT allowed**: Edit code files, run commands, apply patches

## Files to Open First

1. `runpacks/starlark-migration/phases/phase-3-budgets-caching-and-hardening/research.md` - Research findings
2. `runpacks/starlark-migration/phases/phase-3-budgets-caching-and-hardening/risks.md` - Identified risks
3. `runpacks/starlark-migration/ARCHITECTURE_PROPOSAL.md` - Pipeline stages C and E
4. `crates/speccade-spec/src/validation/mod.rs` - Current validation
5. `crates/speccade-spec/src/hash.rs` - Hashing for cache keys
6. `crates/speccade-cli/src/main.rs` - CLI structure

## Planning Requirements

### 1. Budget Unification
- Define `BudgetProfile` type with limits for each asset category
- Move budget checks from backends to validation stage
- Ensure same budgets apply to JSON and Starlark inputs
- Design `--budget-profile <name>` CLI flag

### 2. Provenance in Reports
- Add `Provenance` struct: `source_kind`, `source_hash`, `stdlib_version`
- Extend report JSON with provenance field
- Compute source_hash before parsing (raw input hash)

### 3. Caching Strategy
- Define cache key: `(ir_hash, toolchain_version, budget_profile)`
- Document cache location and invalidation rules
- Implementation is optional but preferred

### 4. Idempotence Tests
- Test: `canonicalize(x) == canonicalize(canonicalize(x))`
- Cover: arrays with set semantics, ID sorting, nested structures

### 5. Hardening Tests
- Property-based tests for validation (valid inputs stay valid)
- Edge cases: empty specs, max-budget specs, malformed inputs

## Output Artifacts

Write these files to the phase folder:

### `plan.md`
- Ordered implementation steps
- File-by-file changes
- Dependencies between steps

### `interfaces.md`
```rust
// Example structures to define:

pub struct BudgetProfile {
    pub name: String,
    pub audio: AudioBudget,
    pub texture: TextureBudget,
    pub mesh: MeshBudget,
    pub general: GeneralBudget,
}

pub struct Provenance {
    pub source_kind: SourceKind,
    pub source_hash: String,
    pub stdlib_version: Option<String>,
}

pub enum SourceKind {
    Json,
    Starlark,
}
```

### `test_plan.md`
- Idempotence test cases
- Budget enforcement test cases
- Provenance test cases
- Property-based test strategy

## Success Criteria

Plan is complete when:
- All acceptance criteria have mapped implementation steps
- `interfaces.md` defines all new public types
- `test_plan.md` covers all testable requirements
- No unresolved dependencies or blockers
