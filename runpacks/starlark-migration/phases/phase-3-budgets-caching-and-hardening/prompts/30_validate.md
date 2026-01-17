# Phase 3 Validate: Budgets, Caching, and Hardening

## Role

You are a **validation agent**. Your job is to run the validation commands, capture outputs, and triage any failures.

## Permission Boundary

- **Allowed**: Run commands, write validation artifacts
- **NOT allowed**: Edit code files, apply patches

## Files to Open First

1. `runpacks/starlark-migration/phases/phase-3-budgets-caching-and-hardening/implementation_log.md` - What was implemented
2. `runpacks/starlark-migration/phases/phase-3-budgets-caching-and-hardening/diff_summary.md` - Files changed
3. `runpacks/starlark-migration/phases/phase-3-budgets-caching-and-hardening/test_plan.md` - Expected test coverage

## Validation Commands (from PHASES.yaml)

Run these commands in order and capture full output:

```bash
cargo test -p speccade-spec
cargo test -p speccade-cli
cargo test -p speccade-tests
```

## Additional Validation

### 1. Budget Enforcement Check
```bash
# Test that budget violations are caught at validation
cargo run -p speccade-cli -- validate <over-budget-spec> --budget-profile zx-8bit
# Expected: validation error, not backend error
```

### 2. Provenance Check
```bash
# Test that reports include provenance
cargo run -p speccade-cli -- generate <spec.star> --out-root /tmp/test
# Check: /tmp/test/*.report.json contains provenance field
```

### 3. Idempotence Check
```bash
# Already covered by tests, but manual spot-check:
cargo run -p speccade-cli -- eval <spec.star> > /tmp/ir1.json
cargo run -p speccade-cli -- eval /tmp/ir1.json > /tmp/ir2.json
# Check: ir1.json == ir2.json (diff should be empty)
```

### 4. Caching Check (if implemented)
```bash
# First run should cache, second should hit cache
cargo run -p speccade-cli -- generate <spec> --out-root /tmp/test1
cargo run -p speccade-cli -- generate <spec> --out-root /tmp/test2
# Check: second run is faster or reports cache hit
```

## Failure Triage Protocol

For each failure:
1. Identify: test name, error message, stack trace
2. Categorize:
   - **Implementation bug**: needs fix in `40_quality`
   - **Test bug**: test expectation is wrong
   - **Environment issue**: missing dependency, path issue
   - **Design issue**: requires plan revision
3. Document in `failures.md`

## Output Artifacts

Write these files to the phase folder:

### `validation.md`
- Command outputs (truncated if very long)
- Pass/fail summary for each test suite
- Overall status: PASS / FAIL / PARTIAL

### `failures.md` (only if there are failures)
- Failure details (test name, error, category)
- Proposed fix or owner
- Blocking vs non-blocking classification

## Success Criteria

Validation is complete when:
- All validation commands have been run
- `validation.md` documents all results
- If failures exist, `failures.md` documents them
- Overall status is determined
