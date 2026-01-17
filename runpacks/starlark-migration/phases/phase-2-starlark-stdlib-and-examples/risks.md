# Phase 2 Risks: Starlark Stdlib and Examples

## Risk Assessment Matrix

| Risk | Severity | Likelihood | Impact | Mitigation Status |
|------|----------|------------|--------|-------------------|
| R1: Non-deterministic stdlib | Critical | Low | High | Mitigated |
| R2: API bloat | Medium | Medium | Medium | Mitigated |
| R3: Naming conflicts | Low | Low | Low | Mitigated |
| R4: Error verbosity | Medium | Medium | Medium | Mitigated |
| R5: Golden test brittleness | Medium | High | Medium | Partially Mitigated |
| R6: Stdlib version coupling | Medium | Medium | Medium | Mitigated |
| R7: Type safety gaps | Medium | Medium | Medium | Mitigated |

---

## R1: Non-Deterministic Stdlib (CRITICAL)

### Description

Stdlib functions must be pure and deterministic. Any non-determinism would break SpecCade's core guarantee: same spec + seed = identical output.

### Sources of Non-Determinism to Avoid

- `random()`, `time()`, `uuid()` or similar functions
- HashMap iteration order (use sorted or ordered collections)
- Floating-point operations that differ by platform
- System locale or timezone access
- Network or filesystem access

### Severity: Critical

If stdlib is non-deterministic, the entire migration fails. Spec hashes would be unstable, caching would break, and reproducibility would be lost.

### Mitigation

1. **No non-deterministic builtins**: Do not expose random, time, or IO functions
2. **Explicit seed threading**: If any "random" behavior is needed, it must come from spec seed
3. **Use deterministic data structures**: BTreeMap over HashMap where order matters
4. **Platform-independent math**: Use Rust's consistent f64 operations
5. **Review checklist**: All stdlib PRs must confirm determinism

### Verification

- Add determinism tests that run stdlib functions multiple times with same inputs
- Compare canonical JSON output across runs

---

## R2: API Bloat

### Description

Adding too many helper functions creates:
- Larger learning curve for spec authors
- More surface area for bugs
- Harder maintenance and versioning

### Severity: Medium

Bloated API degrades usability but doesn't break functionality.

### Mitigation

1. **Minimal viable stdlib**: Start with 10-15 core functions covering 80% of use cases
2. **Composable primitives**: Prefer functions that combine well over specialized ones
3. **Justify each addition**: Document the use case for every new function
4. **Phase additions**: Add more functions in later phases based on actual usage

### Recommended Initial Scope

**Core (must have)**:
- `spec()`, `output()` - Scaffolding
- `envelope()` - Universal ADSR
- `oscillator()`, `fm_synth()`, `noise_burst()` - Common synthesis
- `lowpass()`, `highpass()` - Filters
- `audio_layer()` - Layer composition

**Nice to have (add if time permits)**:
- `reverb()`, `delay()`, `compressor()` - Effects
- `noise_node()`, `gradient_node()`, `threshold_node()` - Texture
- `mesh_primitive()` - Mesh

**Defer to later phases**:
- Music/tracker helpers (complex, lower priority)
- Animation/skeletal helpers (Tier 2, lower priority)

---

## R3: Naming Conflicts

### Description

Stdlib function names might conflict with:
- Starlark built-in names (e.g., `len`, `str`, `list`)
- User-defined variables in specs
- Future Starlark language additions

### Severity: Low

Conflicts are catchable at evaluation time and can be documented.

### Mitigation

1. **Check against Starlark builtins**: Avoid shadowing `len`, `str`, `int`, `list`, `dict`, `range`, `enumerate`, `zip`, `any`, `all`, `sorted`, `reversed`, `hasattr`, `getattr`, `type`, `repr`, `print`, `fail`, `True`, `False`, `None`

2. **Use descriptive prefixes for domain-specific functions**:
   - `audio_layer()` not `layer()`
   - `noise_node()` not `noise()`
   - `mesh_primitive()` not `primitive()`

3. **Document reserved names**: Publish list of stdlib names in docs

### Names to Avoid

- `filter` (Python/Starlark builtin for filtering lists)
- `type` (Starlark builtin)
- `input` / `output` (common variable names - but `output()` for spec outputs is acceptable with docs)

---

## R4: Error Verbosity

### Description

Error messages that are too verbose or too terse both hurt usability:
- Too verbose: Hard to find the actual problem
- Too terse: Not enough context to fix

LLMs benefit from structured, consistent error formats.

### Severity: Medium

Poor errors slow down development but don't break functionality.

### Mitigation

1. **Error code prefix**: All errors get stable codes (S001, S101, etc.)

2. **Structured format**:
   ```
   S101: oscillator(): 'frequency' must be positive, got -440
     at spec.star:15:5
   ```

3. **Field paths**: Include JSON path when relevant:
   ```
   S006: invalid spec: missing required field 'seed' (at root)
   ```

4. **Machine-readable option**: Support `--json` for structured error output:
   ```json
   {"code": "S101", "function": "oscillator", "param": "frequency", "message": "must be positive", "got": -440}
   ```

5. **Suggestions for common mistakes**:
   ```
   S102: waveform must be one of: sine, sawtooth, square, triangle
     Got: "sinwave"
     Did you mean: "sine"?
   ```

---

## R5: Golden Test Brittleness

### Description

Golden tests (`.expected.json` files) can become brittle if:
- Serialization order changes
- Default values change
- Optional fields are added/removed
- Floating-point representation varies

### Severity: Medium

Brittle tests cause CI flakiness and maintenance burden.

### Likelihood: High

Schema evolution is inevitable; some brittleness is expected.

### Mitigation

1. **Canonical JSON only**: Always compare JCS-canonicalized JSON, not raw strings

2. **Semantic comparison for optional fields**:
   ```rust
   // Don't compare raw JSON strings
   // DO compare parsed Spec structs
   assert_eq!(actual_spec.asset_id, expected_spec.asset_id);
   ```

3. **Tolerance for floats**: Use approximate comparison for floating-point:
   ```rust
   assert!((actual.frequency - expected.frequency).abs() < 1e-10);
   ```

4. **Golden update script**: Provide tooling to regenerate golden files:
   ```bash
   cargo test -- --update-golden
   ```

5. **Minimal golden files**: Only test what stdlib functions produce, not full generation pipelines

### Recommended Test Strategy

| Test Type | Brittleness | Coverage |
|-----------|-------------|----------|
| Stdlib unit tests | Low | Function signatures, defaults |
| IR equality tests | Medium | Full spec structure |
| Determinism tests | Low | Hash stability |
| Generation tests | High | End-to-end (use sparingly) |

---

## R6: Stdlib Version Coupling

### Description

The `STDLIB_VERSION` constant ties compiled specs to a specific stdlib release. Changes to stdlib could:
- Invalidate cached specs
- Require re-generation of assets
- Break backward compatibility

### Severity: Medium

Coupling creates migration work but is manageable with versioning.

### Mitigation

1. **Semantic versioning**: Use semver for `STDLIB_VERSION`
   - Patch: Bug fixes, no output change
   - Minor: New functions, backward compatible
   - Major: Breaking changes to existing functions

2. **Include in cache key**: Already planned - `stdlib_version` in report metadata

3. **Deprecation warnings**: Add warnings before removing/changing functions

4. **Compatibility mode**: Consider supporting multiple stdlib versions in future phases

### Current State

- `STDLIB_VERSION = "0.1.0"` defined in `compiler/mod.rs`
- Tracked in report provenance (`stdlib_version` field)
- Not yet part of cache key (Phase 3 scope)

---

## R7: Type Safety Gaps

### Description

Starlark is dynamically typed. Stdlib functions receive `Value` arguments and must validate types at runtime. Gaps in validation could:
- Allow invalid specs to compile
- Produce confusing runtime errors
- Cause backend failures later

### Severity: Medium

Type errors are catchable but hurt developer experience.

### Mitigation

1. **Validate at function entry**: Check types immediately, not deep in logic
   ```rust
   fn oscillator(freq: Value) -> anyhow::Result<Dict> {
       let freq: f64 = freq.unpack_f64()
           .ok_or_else(|| anyhow!("frequency must be a number"))?;
       if freq <= 0.0 {
           return Err(anyhow!("frequency must be positive"));
       }
       // ...
   }
   ```

2. **Clear type annotations in docs**:
   ```python
   def oscillator(freq: float, waveform: str = "sine") -> dict:
       """
       Args:
           freq: Frequency in Hz (must be positive)
           waveform: One of "sine", "sawtooth", "square", "triangle"
       Returns:
           Synthesis block dict
       """
   ```

3. **Enum validation**: For string enums, validate against known values:
   ```rust
   let valid_waveforms = ["sine", "sawtooth", "square", "triangle", "pulse", "noise"];
   if !valid_waveforms.contains(&waveform) {
       return Err(anyhow!("waveform must be one of: {}", valid_waveforms.join(", ")));
   }
   ```

4. **Range validation**: Validate numeric ranges:
   ```rust
   if !(0.0..=1.0).contains(&volume) {
       return Err(anyhow!("volume must be between 0.0 and 1.0"));
   }
   ```

---

## Risk Dependencies

```
R1 (Determinism) blocks everything - must be addressed first
       |
       v
R7 (Type Safety) blocks R4 (Errors) - good validation enables good errors
       |
       v
R2 (API Bloat) informs R3 (Naming) - smaller API = fewer conflicts
       |
       v
R5 (Golden Tests) depends on stable API from R2
       |
       v
R6 (Versioning) follows once API stabilizes
```

## Summary

The most critical risk is **R1: Non-deterministic stdlib**. All other risks are manageable with careful design and testing. The mitigation strategies outlined here should be incorporated into the implementation plan.
