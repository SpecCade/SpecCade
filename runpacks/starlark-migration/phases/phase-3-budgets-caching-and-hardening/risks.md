# Phase 3 Risks: Budgets, Caching, and Hardening

**Date**: 2026-01-17
**Phase**: Phase 3 - Budgets, Caching, and Hardening

---

## Risk 1: Budget Constant Duplication

### Description

Budget limits (e.g., `MAX_AUDIO_DURATION_SECONDS`, `MAX_AUDIO_LAYERS`) are defined in both `speccade-spec` validation and individual backends. This creates maintenance risk and potential inconsistency.

### Current State

```rust
// speccade-spec/src/validation/recipe_outputs_audio.rs:110-111
const MAX_AUDIO_DURATION_SECONDS: f64 = 30.0;
const MAX_AUDIO_LAYERS: usize = 32;

// speccade-backend-audio/src/generate/mod.rs:88-89
const MAX_AUDIO_DURATION_SECONDS: f64 = 30.0;
const MAX_AUDIO_LAYERS: usize = 32;
```

### Impact

- **Medium**: If values diverge, validation could pass but generation could fail (or vice versa)
- **Maintenance burden**: Changes require updating multiple locations

### Mitigation

1. Define canonical budget constants in `speccade-spec/src/validation/budgets.rs`
2. Have backends import from spec crate
3. Add CI test that verifies no duplicate budget definitions

### Probability: Medium | Impact: Medium

---

## Risk 2: Memory Exhaustion in Starlark Evaluation

### Description

Phase 1 followup identified that while timeout is enforced for Starlark evaluation, there is **no memory limit**. A malicious or buggy spec could allocate unbounded memory.

### Current State

```rust
// speccade-cli/src/compiler/eval.rs uses tokio::time::timeout but no memory limit
```

### Impact

- **High**: OOM could crash the process or affect system stability
- **Security**: Potential DoS vector

### Mitigation

1. Investigate starlark crate memory limit options
2. If not available, consider process-level limits (ulimit wrapper)
3. Add large-allocation detection heuristics
4. Document the limitation with security notice

### Probability: Low (benign users) / Medium (hostile input) | Impact: High

---

## Risk 3: Canonicalization Edge Cases

### Description

The JCS canonicalization implementation has potential edge cases that are not thoroughly tested:

1. **Float precision**: `format_jcs_number()` has special handling for integer-like floats
2. **Special values**: NaN, Infinity are converted to null (per JCS spec)
3. **Idempotence**: No explicit tests that `canonicalize(canonicalize(x)) == canonicalize(x)`

### Current State

```rust
// speccade-spec/src/hash.rs:130-135
if f.fract() == 0.0 && f.abs() < 1e15 {
    // Integer-like float
    return format!("{}", f as i64);
}
```

### Impact

- **Medium**: Non-idempotent canonicalization would break caching
- **Low**: Hash collisions from edge cases

### Mitigation

1. Add explicit idempotence test suite
2. Add property-based tests covering float edge cases
3. Test with real-world specs from golden fixtures
4. Add validation that canonicalized output re-parses identically

### Probability: Low | Impact: Medium

---

## Risk 4: Cache Invalidation Correctness

### Description

When implementing caching, incorrect invalidation could cause stale outputs to be used.

### Potential Issues

1. **stdlib_version not bumped**: Stdlib changes without version bump would serve stale cache
2. **git_dirty handling**: Building with uncommitted changes could pollute cache
3. **Backend version parsing**: Non-semver versions could confuse cache logic
4. **Clock skew**: File timestamps unreliable for invalidation

### Impact

- **High**: Stale cached outputs could produce incorrect assets
- **Hard to debug**: Symptoms appear intermittently

### Mitigation

1. Use content-based keys (hash), not timestamps
2. Always invalidate if `git_dirty=true`
3. Include stdlib_version in cache key for Starlark sources
4. Add `--no-cache` flag for debugging
5. Log cache hit/miss decisions at debug level

### Probability: Medium | Impact: High

---

## Risk 5: Breaking Change from Unified Budgets

### Description

Centralizing budget enforcement at validation stage could reject specs that currently work (if backends had looser limits or validation was incomplete).

### Current State

Some limits exist only in backends:
- XM channel limits (32)
- IT sample limits (99)
- Compose expander limits

### Impact

- **Medium**: Existing specs could fail validation after changes
- **User friction**: Requires spec updates

### Mitigation

1. Audit existing golden specs against proposed unified limits
2. Implement as warnings first, errors later
3. Document migration path for affected specs
4. Provide `--budget-profile legacy` escape hatch

### Probability: Low | Impact: Medium

---

## Risk 6: No Fuzz Testing Infrastructure

### Description

No property-based or fuzz testing exists for:
- JSON parsing
- Starlark evaluation
- Validation edge cases

### Impact

- **Medium**: Unknown edge cases in parser/validator
- **Security**: Potential for parser exploits

### Mitigation

1. Add proptest dependency for property-based testing
2. Create fuzz targets for `Spec::from_json()`
3. Create fuzz targets for Starlark compiler
4. Integrate with cargo-fuzz CI

### Probability: Low | Impact: Medium-High

---

## Risk 7: Performance Impact of Validation-Stage Budgets

### Description

Moving budget checks from backend to validation could add overhead, especially for complex checks (e.g., graph cycle detection already in validation).

### Current State

Graph cycle detection in `recipe_outputs_texture.rs` already runs at validation.

### Impact

- **Low**: Validation is not on hot path
- **Minimal**: Budget checks are O(n) in recipe size

### Mitigation

1. Benchmark validation before/after changes
2. Ensure budget checks are O(1) or O(n) where n is recipe size
3. Avoid repeated traversals of recipe structure

### Probability: Low | Impact: Low

---

## Risk Summary Matrix

| Risk | Probability | Impact | Priority |
|------|-------------|--------|----------|
| Budget constant duplication | Medium | Medium | P2 |
| Memory exhaustion (Starlark) | Medium | High | P1 |
| Canonicalization edge cases | Low | Medium | P3 |
| Cache invalidation correctness | Medium | High | P1 |
| Breaking change from unified budgets | Low | Medium | P3 |
| No fuzz testing | Low | Medium-High | P2 |
| Validation performance | Low | Low | P4 |

---

## Action Items (by Priority)

### P1 - Must Address

1. Document Starlark memory limit gap (or implement if feasible)
2. Design cache invalidation strategy with content-based keys
3. Add `--no-cache` flag early in implementation

### P2 - Should Address

1. Centralize budget constants before adding new ones
2. Add proptest dependency and basic property tests
3. Ensure stdlib_version is bumped on stdlib changes (add CI check)

### P3 - Nice to Have

1. Add idempotence test suite for canonicalization
2. Audit golden specs against proposed unified limits
3. Add explicit documentation for budget limits

### P4 - Future

1. Benchmark validation performance (if issues arise)
