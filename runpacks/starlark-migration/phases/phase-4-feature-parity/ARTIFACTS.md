# Phase 4 Artifacts

## Input artifacts (from previous phases)

- `FEATURE_PARITY_GAPS.md` - Complete gap analysis

## Output artifacts (this phase will create)

### Research stage
- `research.md` - Complete IR type catalog
- `risks.md` - Implementation risks

### Plan stage
- `plan.md` - Implementation plan
- `interfaces.md` - New function signatures
- `test_plan.md` - Test coverage plan

### Implement stage
- `implementation_log.md` - What was added
- `diff_summary.md` - Files changed

### Validate stage
- `validation.md` - Test results
- `failures.md` - Any failures and resolutions

### Quality stage
- `quality.md` - Code quality assessment
- `followups.md` - Remaining work

## Key files to modify

### Stdlib
- `crates/speccade-cli/src/compiler/stdlib/audio.rs` - ~50 new functions
- `crates/speccade-cli/src/compiler/stdlib/texture.rs` - ~10 new functions
- `crates/speccade-cli/src/compiler/stdlib/mesh.rs` - ~8 new functions
- `crates/speccade-cli/src/compiler/stdlib/music.rs` - NEW file, ~5 functions
- `crates/speccade-cli/src/compiler/stdlib/mod.rs` - Register new module

### Budget enforcement
- `crates/speccade-spec/src/validation/mod.rs` - Wire budget profile
- `crates/speccade-cli/src/commands/validate.rs` - Add --budget flag
- `crates/speccade-cli/src/commands/generate.rs` - Add --budget flag

### Tests
- `crates/speccade-tests/tests/starlark_input.rs` - New test cases
- `golden/starlark/*.star` - ~30+ new golden files

### Docs
- `docs/stdlib-reference.md` - Update with new functions
- `docs/budgets.md` - Update with CLI flag info
