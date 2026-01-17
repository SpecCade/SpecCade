# Phase 2 Failure Triage

**Date**: 2026-01-17
**Validation Round**: 4
**Total Issues**: 5 (2 test failures + 3 compilation errors)
**Blocking**: Partial - CLI works, test suite incomplete

---

## Issue Category 1: Test Expectation Bugs (speccade-cli)

### Summary
| Attribute | Value |
|-----------|-------|
| Category | test |
| Count | 2 |
| Severity | Low |
| Impact | Test failures only, implementation is correct |

### Issue 1.1: test_levenshtein_distance Incorrect Expectation

**Location**: `crates/speccade-cli/src/compiler/stdlib/validation.rs:229`

**Test Code**:
```rust
#[test]
fn test_levenshtein_distance() {
    assert_eq!(levenshtein_distance("sine", "sine"), 0);
    assert_eq!(levenshtein_distance("sine", "sin"), 1);
    assert_eq!(levenshtein_distance("sinwave", "sine"), 4);  // <-- INCORRECT
    assert_eq!(levenshtein_distance("sqare", "square"), 1);
}
```

**Error**:
```
assertion `left == right` failed
  left: 3
 right: 4
```

**Analysis**:
The test expectation is wrong. Levenshtein distance from "sinwave" to "sine":
- Delete 'w' (sinave)
- Delete 'a' (sinve)
- Delete 'v' (sine)
- Result: 3 operations, not 4

**Fix**:
```rust
assert_eq!(levenshtein_distance("sinwave", "sine"), 3);  // Correct value
```

---

### Issue 1.2: test_find_similar Threshold Mismatch

**Location**: `crates/speccade-cli/src/compiler/stdlib/validation.rs:219`

**Test Code**:
```rust
#[test]
fn test_find_similar() {
    let waveforms = &["sine", "square", "sawtooth", "triangle"];
    assert_eq!(find_similar("sinwave", waveforms), Some("sine"));  // <-- FAILS
    assert_eq!(find_similar("sin", waveforms), Some("sine"));
    assert_eq!(find_similar("sqare", waveforms), Some("square"));
    assert_eq!(find_similar("xyz", waveforms), None);
}
```

**Error**:
```
assertion `left == right` failed
  left: None
 right: Some("sine")
```

**Analysis**:
The `find_similar()` function has a threshold of `<= 2` for Levenshtein matches:
```rust
fn find_similar<'a>(value: &str, allowed: &[&'a str]) -> Option<&'a str> {
    // ...
    for &candidate in allowed {
        if levenshtein_distance(&value_lower, candidate) <= 2 {  // Threshold is 2
            return Some(candidate);
        }
    }
    None
}
```

Distance from "sinwave" to "sine" is 3, which exceeds the threshold of 2.

**Fix Options**:

**Option A**: Update test expectation to match implementation behavior:
```rust
assert_eq!(find_similar("sinwave", waveforms), None);  // Distance 3 > threshold 2
```

**Option B**: Increase threshold to 3 in implementation:
```rust
if levenshtein_distance(&value_lower, candidate) <= 3 {
```

**Recommendation**: Option B is more user-friendly for typo suggestions.

---

## Issue Category 2: Type Mismatch in Test Assertions (speccade-tests)

### Summary
| Attribute | Value |
|-----------|-------|
| Error Code | E0308 |
| Category | compile |
| Count | 3 |
| Severity | Medium |
| Impact | Prevents speccade-tests from compiling |

### Issue 2.1-2.3: AssetType Enum vs String Comparison

**Location**: `crates/speccade-tests/tests/starlark_input.rs`

**Error Messages**:
```
error[E0308]: mismatched types
   --> crates\speccade-tests\tests\starlark_input.rs:285:40
    |
285 |     assert_eq!(result.spec.asset_type, "audio");
    |                                        ^^^^^^^ expected `AssetType`, found `&str`

error[E0308]: mismatched types
   --> crates\speccade-tests\tests\starlark_input.rs:313:40
    |
313 |     assert_eq!(result.spec.asset_type, "texture");
    |                                        ^^^^^^^^^ expected `AssetType`, found `&str`

error[E0308]: mismatched types
   --> crates\speccade-tests\tests\starlark_input.rs:332:40
    |
332 |     assert_eq!(result.spec.asset_type, "static_mesh");
    |                                        ^^^^^^^^^^^^^ expected `AssetType`, found `&str`
```

**Root Cause**:
The `asset_type` field is an `AssetType` enum (defined in `speccade-spec/src/spec.rs`):
```rust
pub enum AssetType {
    Audio,
    Music,
    Texture,
    StaticMesh,
    SkeletalMesh,
    // ...
}
```

The tests compare against string literals instead of enum variants.

**Affected Tests**:
| Test Function | Line | Incorrect | Correct |
|---------------|------|-----------|---------|
| `load_stdlib_audio_oscillator` | 285 | `"audio"` | `AssetType::Audio` |
| `load_stdlib_texture_noise` | 313 | `"texture"` | `AssetType::Texture` |
| `load_stdlib_mesh_cube` | 332 | `"static_mesh"` | `AssetType::StaticMesh` |

**Fix**:
```rust
// Line 285
assert_eq!(result.spec.asset_type, AssetType::Audio);

// Line 313
assert_eq!(result.spec.asset_type, AssetType::Texture);

// Line 332
assert_eq!(result.spec.asset_type, AssetType::StaticMesh);
```

Add import if not present:
```rust
use speccade_spec::AssetType;
```

---

## Summary Table

| Issue | Type | File | Line | Fix Complexity | Status |
|-------|------|------|------|----------------|--------|
| Levenshtein test expectation | test | validation.rs | 229 | Trivial | Needs fix |
| find_similar test/threshold | test | validation.rs | 219 | Simple | Needs fix |
| AssetType vs "audio" | compile | starlark_input.rs | 285 | Trivial | Needs fix |
| AssetType vs "texture" | compile | starlark_input.rs | 313 | Trivial | Needs fix |
| AssetType vs "static_mesh" | compile | starlark_input.rs | 332 | Trivial | Needs fix |

---

## Files Requiring Changes

| File | Changes Needed |
|------|----------------|
| `crates/speccade-cli/src/compiler/stdlib/validation.rs` | Fix line 229 test expectation (3 not 4); consider threshold adjustment for find_similar |
| `crates/speccade-tests/tests/starlark_input.rs` | Change 3 assertions to use `AssetType::*` instead of string literals |

---

## Warnings (Non-Blocking, Cleanup)

### Unused Imports
| File | Line | Import |
|------|------|--------|
| audio.rs | 9 | `starlark::values::list::AllocList` |
| core.rs | 11 | `validate_positive_int` |
| texture.rs | 9 | `none::NoneType` |
| starlark_input.rs | 11 | `std::path::Path` |
| input.rs | 271 | `std::io::Write` |
| eval.rs | 111 | `std::io::Write` |

### Unused Functions
| File | Line | Function |
|------|------|----------|
| validation.rs | 136 | `extract_optional_float` |
| validation.rs | 152 | `extract_float` |
| validation.rs | 168 | `extract_optional_string` |

---

## Resolution Priority

### Immediate (Required for Test Suite)
1. Fix `test_levenshtein_distance` expectation: `4` -> `3`
2. Fix `test_find_similar` expectation or increase threshold
3. Fix 3 `assert_eq!` statements in `starlark_input.rs` to use `AssetType` enum

### Optional (Cleanup)
4. Remove unused imports
5. Remove or use unused functions

---

## Previous Issues (Resolved in Round 4)

The following issues from previous rounds have been resolved:

| Issue | Round Fixed |
|-------|-------------|
| Dict::new() SmallMap type mismatch (4 errors) | Round 4 |
| eval_with_stdlib lifetime error (1 error) | Round 4 |
| Hashed type import location (8 errors) | Round 3 |
| Raw string literal parsing (7 errors) | Round 3 |

**Build now succeeds.** All remaining issues are test-level problems.
