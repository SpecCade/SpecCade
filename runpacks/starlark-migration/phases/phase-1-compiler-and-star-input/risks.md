# Phase 1 Risks

## Summary

| ID | Risk | Severity | Likelihood | Mitigation |
|----|------|----------|------------|------------|
| R1 | Starlark infinite loops / CPU exhaustion | High | Medium | External timeout wrapper |
| R2 | Memory exhaustion from large specs | Medium | Low | System ulimits + cap crate |
| R3 | Non-deterministic Starlark evaluation | High | Low | Avoid non-deterministic builtins |
| R4 | Breaking changes to JSON spec users | High | Low | Strict input dispatch by extension |
| R5 | Starlark crate dependency stability | Medium | Low | Pin version, monitor releases |
| R6 | Spec validation bypass via Starlark | High | Low | Validate canonical IR only |
| R7 | Complex error attribution | Medium | Medium | Source location tracking |
| R8 | Increased build times | Low | High | Feature-gate Starlark support |

---

## R1: Starlark Infinite Loops / CPU Exhaustion

**Severity:** High
**Likelihood:** Medium (malicious or buggy specs)

**Description:**
The `starlark` crate does not provide built-in CPU time limits. A Starlark spec with an infinite loop (if recursion is enabled) or very long-running computation could hang the CLI indefinitely.

**Mitigation:**
1. **Do not enable recursion extension** in Starlark dialect
2. Wrap Starlark evaluation in a timeout:
   ```rust
   tokio::time::timeout(Duration::from_secs(30), async {
       eval_starlark(source)
   }).await?
   ```
3. Document maximum expected evaluation time for specs
4. Consider instruction counting if finer control needed (requires custom evaluator)

**Residual risk:** Low after mitigation

---

## R2: Memory Exhaustion from Large Specs

**Severity:** Medium
**Likelihood:** Low

**Description:**
A Starlark spec could allocate very large data structures (e.g., huge lists, deeply nested objects) causing memory exhaustion.

**Mitigation:**
1. Rely on system-level limits (ulimit) for memory caps
2. Optionally integrate `cap` crate as allocator wrapper with limits
3. Starlark's garbage collector helps, but doesn't prevent peak allocations

**Residual risk:** Low (system limits sufficient for CLI use)

---

## R3: Non-deterministic Starlark Evaluation

**Severity:** High
**Likelihood:** Low (if careful)

**Description:**
Starlark is designed to be deterministic, but certain operations could introduce non-determinism:
- Dictionary iteration order (pre-Python 3.7 semantics)
- Random number generation
- Time/date functions
- File system access

**Mitigation:**
1. Use standard Starlark dialect (iteration order is deterministic)
2. **Do not expose** `random()`, `time()`, or similar builtins
3. Provide a seeded RNG if needed (derive from spec seed)
4. No file/network access (Starlark sandbox enforces this)
5. Test determinism in CI (same input -> same output)

**Residual risk:** Very low with proper stdlib design

---

## R4: Breaking Changes to JSON Spec Users

**Severity:** High
**Likelihood:** Low (if careful)

**Description:**
Existing users with `.json` specs must not experience any behavior change. Risk includes:
- Validation logic changes
- Hash computation changes
- Output path changes

**Mitigation:**
1. Input dispatch is **strictly by file extension**:
   - `.json` -> existing JSON path (unchanged)
   - `.star` -> new Starlark path
2. Hashing computed on **canonical IR** (post-resolve), identical for both paths
3. Validation runs on **canonical IR**, not on source format
4. Golden tests verify JSON spec behavior unchanged

**Residual risk:** Very low with proper testing

---

## R5: Starlark Crate Dependency Stability

**Severity:** Medium
**Likelihood:** Low

**Description:**
The `starlark` crate is maintained by Meta for Buck2. Breaking changes or deprecation could impact SpecCade.

**Mitigation:**
1. Pin to specific version (e.g., `starlark = "=0.12.0"`)
2. Monitor releases and changelog
3. Abstract Starlark evaluation behind internal trait/interface
4. `starlark` is Apache 2.0 licensed; can fork if abandoned

**Residual risk:** Low (actively maintained, widely used)

---

## R6: Spec Validation Bypass via Starlark

**Severity:** High
**Likelihood:** Low

**Description:**
Starlark could theoretically produce a `Spec` that bypasses validation if validation is done at the wrong stage.

**Mitigation:**
1. Starlark produces a JSON-like value
2. Convert to `Spec` via `Spec::from_value()`
3. **All validation runs on canonical `Spec`** (same as JSON path)
4. Never trust Starlark output without validation

**Current architecture already enforces this:**
```
Starlark eval -> JSON value -> Spec::from_value() -> validate_spec() -> ...
```

**Residual risk:** Very low (architecture enforces validation)

---

## R7: Complex Error Attribution

**Severity:** Medium
**Likelihood:** Medium

**Description:**
When validation fails on a Starlark-generated spec, users need to understand where in their `.star` file the problem originated. Starlark evaluation errors are separate from spec validation errors.

**Mitigation:**
1. Preserve Starlark source locations in evaluation errors
2. For validation errors, indicate they apply to the "generated spec"
3. Consider a `--emit-ir` flag to output intermediate JSON for debugging
4. Add `source_kind` and `source_hash` to reports for provenance

**Residual risk:** Medium (acceptable UX tradeoff)

---

## R8: Increased Build Times

**Severity:** Low
**Likelihood:** High

**Description:**
The `starlark` crate is substantial. Adding it as a dependency will increase compile times.

**Mitigation:**
1. Feature-gate Starlark support: `speccade-cli --features starlark`
2. Default builds can omit Starlark for faster iteration
3. CI builds always include Starlark for full testing
4. Consider separate `speccade-compiler` crate if needed

**Residual risk:** Low (feature gates solve this)

---

## Risk Matrix

```
Likelihood
    ^
    |
 H  |                R8
 M  |  R7            R1
 L  |  R2 R5         R3 R4 R6
    +------------------------>
       Low  Medium  High     Severity
```

---

## Acceptance Criteria for Risk Mitigation

Before Phase 1 is considered complete:

- [ ] Starlark evaluation wrapped with 30s timeout
- [ ] No recursion extension enabled in dialect
- [ ] No non-deterministic builtins exposed
- [ ] Golden tests verify JSON spec hash stability
- [ ] `source_kind` field added to reports
- [ ] Feature gate `starlark` documented in README
- [ ] Error messages include source attribution
