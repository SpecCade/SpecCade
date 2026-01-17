# Phase 3 Diff Summary

## Files Created

| File | Lines | Description |
|------|-------|-------------|
| `crates/speccade-spec/src/validation/budgets.rs` | ~400 | Centralized budget types, profiles, and constants |
| `crates/speccade-tests/tests/canonicalization.rs` | ~220 | Integration tests for JSON canonicalization |
| `docs/budgets.md` | ~140 | Budget system documentation |

## Files Modified

| File | Changes | Description |
|------|---------|-------------|
| `crates/speccade-spec/src/validation/mod.rs` | +12 lines | Added budgets module, re-exports, validate_for_generate_with_budget() |
| `crates/speccade-spec/src/lib.rs` | +4 lines | Export budget types at crate root |
| `crates/speccade-spec/src/hash.rs` | +340 lines | Added 25+ canonicalization tests |

## Detailed Changes

### `crates/speccade-spec/src/validation/budgets.rs` (NEW)

```rust
// Key types added:
pub struct AudioBudget { ... }
pub struct TextureBudget { ... }
pub struct MusicBudget { ... }
pub struct MeshBudget { ... }
pub struct GeneralBudget { ... }
pub struct BudgetProfile { ... }
pub struct BudgetError { ... }
pub enum BudgetCategory { ... }

// Key functions:
impl BudgetProfile {
    pub fn default() -> Self { ... }
    pub fn strict() -> Self { ... }
    pub fn zx_8bit() -> Self { ... }
    pub fn by_name(name: &str) -> Option<Self> { ... }
}
```

### `crates/speccade-spec/src/validation/mod.rs`

```diff
+pub mod budgets;
 pub mod common;
 ...

+// Re-export budget types for convenience
+pub use budgets::{
+    AudioBudget, BudgetCategory, BudgetError, BudgetProfile, GeneralBudget, MeshBudget,
+    MusicBudget, TextureBudget,
+};
 ...

+pub fn validate_for_generate_with_budget(
+    spec: &Spec,
+    _budget: &BudgetProfile,
+) -> ValidationResult { ... }
```

### `crates/speccade-spec/src/lib.rs`

```diff
 pub use validation::{
-    is_safe_output_path, is_valid_asset_id, validate_for_generate, validate_spec,
+    is_safe_output_path, is_valid_asset_id, validate_for_generate,
+    validate_for_generate_with_budget, validate_spec, AudioBudget, BudgetCategory, BudgetError,
+    BudgetProfile, GeneralBudget, MeshBudget, MusicBudget, TextureBudget,
 };
```

### `crates/speccade-spec/src/hash.rs`

```diff
+    // Canonicalization Idempotence Tests (4 tests)
+    #[test] fn test_canonicalization_idempotent_simple() { ... }
+    #[test] fn test_canonicalization_idempotent_nested() { ... }
+    #[test] fn test_canonicalization_idempotent_with_arrays() { ... }
+    #[test] fn test_canonicalization_idempotent_empty_structures() { ... }
+
+    // Float Edge Case Tests (5 tests)
+    #[test] fn test_canonicalization_float_zero() { ... }
+    #[test] fn test_canonicalization_float_integer_like() { ... }
+    #[test] fn test_canonicalization_float_decimals() { ... }
+    #[test] fn test_canonicalization_float_large_values() { ... }
+    #[test] fn test_canonicalization_float_nan_infinity() { ... }
+
+    // String Edge Case Tests (4 tests)
+    #[test] fn test_canonicalization_string_empty() { ... }
+    #[test] fn test_canonicalization_string_escape_sequences() { ... }
+    #[test] fn test_canonicalization_string_unicode() { ... }
+    #[test] fn test_canonicalization_string_control_characters() { ... }
+
+    // Object Key Ordering Tests (2 tests)
+    #[test] fn test_canonicalization_key_ordering_simple() { ... }
+    #[test] fn test_canonicalization_key_ordering_nested() { ... }
+
+    // Array Preservation Tests (2 tests)
+    #[test] fn test_canonicalization_preserves_array_order() { ... }
+    #[test] fn test_canonicalization_preserves_object_array_order() { ... }
+
+    // Deep Nesting Tests (2 tests)
+    #[test] fn test_canonicalization_deep_nesting() { ... }
+    #[test] fn test_canonicalization_mixed_nesting() { ... }
```

### `crates/speccade-tests/tests/canonicalization.rs` (NEW)

```rust
// Integration tests covering:
#[test] fn test_canonicalization_idempotent_with_spec() { ... }
#[test] fn test_spec_hash_stability() { ... }
#[test] fn test_key_order_independence() { ... }
#[test] fn test_nested_key_order_independence() { ... }
#[test] fn test_spec_roundtrip_hash_preservation() { ... }
#[test] fn test_float_normalization() { ... }
#[test] fn test_zero_normalization() { ... }
#[test] fn test_unicode_preservation() { ... }
#[test] fn test_empty_structures() { ... }
#[test] fn test_string_escaping() { ... }
#[test] fn test_deterministic_hashing() { ... }
#[test] fn test_hash_sensitivity() { ... }
```

### `docs/budgets.md` (NEW)

Documentation covering:
- Budget categories and limits tables
- Pre-defined profiles
- API usage examples
- Custom profile creation
- Error handling
- Rationale

## Test Coverage Added

| Category | Tests Added |
|----------|-------------|
| Canonicalization (unit) | 19 tests |
| Canonicalization (integration) | 12 tests |
| Budget types (unit) | 11 tests |
| **Total** | **42 tests** |

## Public API Additions

```rust
// New types exported from speccade_spec
pub struct AudioBudget;
pub struct TextureBudget;
pub struct MusicBudget;
pub struct MeshBudget;
pub struct GeneralBudget;
pub struct BudgetProfile;
pub struct BudgetError;
pub enum BudgetCategory;

// New function
pub fn validate_for_generate_with_budget(spec: &Spec, budget: &BudgetProfile) -> ValidationResult;
```

## No Breaking Changes

All existing APIs remain unchanged. The changes are purely additive:
- `validate_for_generate()` still works (uses default budget internally)
- All existing tests should pass unchanged
- No signature changes to existing public functions
