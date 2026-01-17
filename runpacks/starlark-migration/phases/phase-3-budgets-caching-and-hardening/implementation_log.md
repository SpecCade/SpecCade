# Phase 3 Implementation Log

## Session: 2026-01-17

### Overview

Implemented Phase 3 of the Starlark migration: Budget Enforcement, Canonicalization Testing, and Documentation. Caching was deferred as optional per the plan.

---

### 1. Budget Enforcement (Priority 1)

**Created:** `crates/speccade-spec/src/validation/budgets.rs`

Implemented the centralized budget system with:

- `AudioBudget` struct with constants:
  - `DEFAULT_MAX_DURATION_SECONDS = 30.0`
  - `DEFAULT_MAX_LAYERS = 32`
  - `DEFAULT_ALLOWED_SAMPLE_RATES = [22050, 44100, 48000]`

- `TextureBudget` struct with constants:
  - `DEFAULT_MAX_DIMENSION = 4096`
  - `DEFAULT_MAX_PIXELS = 4096 * 4096`
  - `DEFAULT_MAX_GRAPH_NODES = 256`
  - `DEFAULT_MAX_GRAPH_DEPTH = 64`

- `MusicBudget` struct with XM/IT-specific limits:
  - XM: 32 channels, 256 patterns, 128 instruments
  - IT: 64 channels, 200 patterns, 99 instruments, 99 samples
  - Compose: 64 recursion depth, 50k cells per pattern

- `MeshBudget` struct:
  - `DEFAULT_MAX_VERTICES = 100,000`
  - `DEFAULT_MAX_FACES = 100,000`
  - `DEFAULT_MAX_BONES = 256`

- `GeneralBudget` struct:
  - `DEFAULT_STARLARK_TIMEOUT_SECONDS = 30`
  - `DEFAULT_MAX_SPEC_SIZE_BYTES = 10 MB`

- `BudgetProfile` enum with pre-defined profiles:
  - `default()` - Standard limits
  - `strict()` - Reduced limits for CI
  - `zx_8bit()` - Retro console limits

- `BudgetError` and `BudgetCategory` types for error reporting

**Modified:** `crates/speccade-spec/src/validation/mod.rs`

- Added `pub mod budgets;`
- Re-exported budget types for convenience
- Added `validate_for_generate_with_budget()` function

**Modified:** `crates/speccade-spec/src/lib.rs`

- Exported budget types at crate root:
  - `AudioBudget`, `TextureBudget`, `MusicBudget`, `MeshBudget`, `GeneralBudget`
  - `BudgetProfile`, `BudgetCategory`, `BudgetError`
  - `validate_for_generate_with_budget`

---

### 2. Provenance Verification (Priority 2)

**Verified existing implementation** in:
- `crates/speccade-spec/src/report/mod.rs` - Report struct has all provenance fields
- `crates/speccade-spec/src/report/builder.rs` - Builder has methods for all fields
- `crates/speccade-cli/src/commands/generate.rs` - Generate command populates fields

Provenance fields already implemented:
- `source_kind` - Source format ("json" or "starlark")
- `source_hash` - BLAKE3 hash of source file
- `stdlib_version` - Starlark stdlib version (cache invalidation)
- `recipe_hash` - Canonical hash of recipe
- `base_spec_hash` - Hash of unexpanded spec (for variants)
- `variant_id` - Variant identifier

No changes needed - implementation complete per research findings.

---

### 3. Canonicalization Tests (Priority 3)

**Modified:** `crates/speccade-spec/src/hash.rs`

Added comprehensive test suite covering:

**Idempotence Tests:**
- `test_canonicalization_idempotent_simple`
- `test_canonicalization_idempotent_nested`
- `test_canonicalization_idempotent_with_arrays`
- `test_canonicalization_idempotent_empty_structures`

**Float Edge Cases:**
- `test_canonicalization_float_zero` - Verifies 0.0 becomes "0"
- `test_canonicalization_float_integer_like` - 1.0 -> "1", etc.
- `test_canonicalization_float_decimals` - 0.5, 0.125 round-trip
- `test_canonicalization_float_large_values` - 1e10, 1e15 handling
- `test_canonicalization_float_nan_infinity` - Verifies NaN/Infinity rejected

**String Edge Cases:**
- `test_canonicalization_string_empty`
- `test_canonicalization_string_escape_sequences`
- `test_canonicalization_string_unicode`
- `test_canonicalization_string_control_characters`

**Object Key Ordering:**
- `test_canonicalization_key_ordering_simple`
- `test_canonicalization_key_ordering_nested`

**Array Preservation:**
- `test_canonicalization_preserves_array_order`
- `test_canonicalization_preserves_object_array_order`

**Deep Nesting:**
- `test_canonicalization_deep_nesting` - 5+ levels
- `test_canonicalization_mixed_nesting` - Arrays and objects mixed

**Created:** `crates/speccade-tests/tests/canonicalization.rs`

Integration tests covering:
- Spec canonicalization idempotence
- Spec hash stability
- Key order independence (simple and nested)
- Spec JSON round-trip hash preservation
- Float normalization (1.0 == 1, 0.0 == 0)
- Unicode preservation
- Empty structure handling
- String escaping
- Deterministic hashing (100 iterations)
- Hash sensitivity (any field change changes hash)

---

### 4. Documentation (Priority 5)

**Created:** `docs/budgets.md`

Comprehensive documentation covering:
- Budget categories and their limits
- Pre-defined budget profiles
- How to use budget profiles in validation
- Creating custom profiles
- Budget error format and examples
- Rationale for budget enforcement
- Implementation notes for backend authors

---

### 5. Caching (Priority 4) - Deferred

Per the plan, caching implementation was optional for Phase 3. The foundation is in place:
- `source_hash` and `stdlib_version` in reports enable cache key computation
- `BudgetProfile` can be extended with cache configuration

File-based caching can be added in a future phase.

---

### Implementation Notes

1. **Budget integration pattern**: The `validate_for_generate_with_budget()` function accepts a `BudgetProfile` parameter. Existing validation functions in recipe_outputs_*.rs modules can be migrated to use these centralized constants incrementally.

2. **Test coverage**: Added 25+ new unit tests for canonicalization in hash.rs, plus 12+ integration tests in the new canonicalization.rs file.

3. **Backward compatibility**: The existing `validate_for_generate()` function continues to work, using `BudgetProfile::default()` internally.

4. **No breaking changes**: All existing APIs remain unchanged; new budget types are additive.

---

### Files Changed Summary

| File | Action | Description |
|------|--------|-------------|
| `crates/speccade-spec/src/validation/budgets.rs` | Created | Budget types and profiles |
| `crates/speccade-spec/src/validation/mod.rs` | Modified | Export budgets module |
| `crates/speccade-spec/src/lib.rs` | Modified | Export budget types |
| `crates/speccade-spec/src/hash.rs` | Modified | Canonicalization tests |
| `crates/speccade-tests/tests/canonicalization.rs` | Created | Integration tests |
| `docs/budgets.md` | Created | Budget documentation |
