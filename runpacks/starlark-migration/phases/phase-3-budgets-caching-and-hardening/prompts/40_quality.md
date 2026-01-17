# Phase 3 Quality: Budgets, Caching, and Hardening

## Role

You are a **quality agent**. Your job is to fix validation failures, refactor for maintainability, and identify follow-up work.

## Permission Boundary

- **Allowed**: Edit code files within scope globs, write quality artifacts
- **NOT allowed**: Run build commands, run test commands

## Files to Open First

1. `runpacks/starlark-migration/phases/phase-3-budgets-caching-and-hardening/validation.md` - Test results
2. `runpacks/starlark-migration/phases/phase-3-budgets-caching-and-hardening/failures.md` - Failures to fix (if exists)
3. `runpacks/starlark-migration/phases/phase-3-budgets-caching-and-hardening/implementation_log.md` - Implementation context
4. `runpacks/starlark-migration/phases/phase-3-budgets-caching-and-hardening/SCOPING.md` - Scope boundaries

## Quality Tasks

### 1. Fix Validation Failures
If `failures.md` exists:
- Address each implementation bug
- Document fixes in quality log
- Mark non-blocking issues for follow-up

### 2. Code Quality Review

#### Budget Module
- [ ] Clear error messages for budget violations
- [ ] Budget profiles are well-documented
- [ ] Limits are const or configurable, not magic numbers

#### Provenance Module
- [ ] Source hash is computed efficiently (stream, not load-all)
- [ ] Provenance serializes cleanly to JSON

#### Caching Module (if implemented)
- [ ] Cache keys are stable across runs
- [ ] Cache invalidation is correct (toolchain version changes)
- [ ] No sensitive data in cache keys

#### Tests
- [ ] Idempotence tests cover edge cases
- [ ] Property tests have good shrinking
- [ ] Golden tests are minimal and documented

### 3. Documentation Review
- [ ] Budget profiles documented with examples
- [ ] Caching behavior documented
- [ ] Error codes are LLM-friendly (stable, greppable)

### 4. Refactoring Opportunities
- Reduce code duplication in budget checks
- Simplify validation error types
- Improve test helper ergonomics

## Output Artifacts

Write these files to the phase folder:

### `quality.md`
- Fixes applied (reference to failures.md items)
- Refactoring performed
- Documentation improvements
- Code quality observations

### `followups.md` (optional)
- Deferred improvements
- Performance optimizations
- Extended test coverage ideas
- Integration with future phases

## Success Criteria

Quality is complete when:
- All blocking failures from `failures.md` are fixed
- `quality.md` documents all improvements
- Code is ready for re-validation
- Follow-ups are documented (if any)

## Re-validation Note

After quality fixes, the orchestrator should re-run `30_validate`. This prompt does not run tests - it only fixes code. The validation agent will verify fixes.
