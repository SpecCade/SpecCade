# Phase 3 Implement: Budgets, Caching, and Hardening

## Role

You are an **implementation agent**. Your job is to implement the planned changes for budget unification, caching, provenance, and hardening tests.

## Permission Boundary

- **Allowed**: Edit code files within scope globs, write implementation artifacts
- **NOT allowed**: Run build commands, run test commands, edit files outside scope

## Files to Open First

1. `runpacks/starlark-migration/phases/phase-3-budgets-caching-and-hardening/plan.md` - Implementation plan
2. `runpacks/starlark-migration/phases/phase-3-budgets-caching-and-hardening/interfaces.md` - Type definitions
3. `runpacks/starlark-migration/phases/phase-3-budgets-caching-and-hardening/test_plan.md` - Test requirements
4. `runpacks/starlark-migration/phases/phase-3-budgets-caching-and-hardening/SCOPING.md` - Allowed globs

## Scope Globs (from SCOPING.md)

```
crates/speccade-cli/**
crates/speccade-spec/**
crates/speccade-tests/**
crates/**
docs/**
schemas/**
golden/**
```

## Implementation Checklist

### 1. Budget Types and Profiles
- [ ] Create `crates/speccade-spec/src/budget.rs`
- [ ] Define `BudgetProfile`, `AudioBudget`, `TextureBudget`, `MeshBudget`, `GeneralBudget`
- [ ] Add default profiles (e.g., `zx-8bit`, `unlimited`)
- [ ] Export from `crates/speccade-spec/src/lib.rs`

### 2. Budget Validation
- [ ] Create `crates/speccade-spec/src/validation/budgets.rs`
- [ ] Implement `validate_budgets(spec: &Spec, profile: &BudgetProfile) -> Result<(), BudgetError>`
- [ ] Integrate into main validation pipeline
- [ ] Ensure validation runs before backend dispatch

### 3. Provenance Types
- [ ] Create `crates/speccade-spec/src/provenance.rs`
- [ ] Define `Provenance`, `SourceKind`
- [ ] Add `source_hash` computation (BLAKE3 of raw input bytes)

### 4. Report Extension
- [ ] Extend report struct with `provenance` field
- [ ] Update report generation in CLI
- [ ] Ensure backward compatibility (provenance is optional in output)

### 5. Caching (if implemented)
- [ ] Create `crates/speccade-cli/src/cache.rs`
- [ ] Define cache key derivation
- [ ] Implement cache lookup/store
- [ ] Add `--no-cache` flag to CLI

### 6. CLI Integration
- [ ] Add `--budget-profile <name>` flag
- [ ] Wire budget validation into `validate` and `generate` commands
- [ ] Include provenance in `eval` output (optional)

### 7. Idempotence Tests
- [ ] Create `crates/speccade-tests/src/idempotence.rs`
- [ ] Test canonicalize idempotence for all spec types
- [ ] Add golden tests for canonical output stability

### 8. Hardening Tests
- [ ] Create `crates/speccade-tests/src/fuzz.rs` or extend existing
- [ ] Property: valid spec -> valid after roundtrip
- [ ] Property: budget violation -> validation error
- [ ] Edge cases from test_plan.md

### 9. Documentation
- [ ] Document budget profiles in `docs/`
- [ ] Update CLI help text
- [ ] Add caching strategy to docs (if implemented)

## Output Artifacts

Write these files to the phase folder:

### `implementation_log.md`
- Chronological log of changes made
- Decisions made during implementation
- Any deviations from plan (with rationale)

### `diff_summary.md`
- List of files created/modified
- Brief description of each change
- Lines added/removed estimate

## Success Criteria

Implementation is complete when:
- All checklist items are done or explicitly deferred
- `implementation_log.md` documents all changes
- `diff_summary.md` lists all modified files
- Code compiles (but do NOT run tests - that's validation's job)
