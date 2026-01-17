# Phase 3 Follow-up Items

**Date**: 2026-01-17
**Phase**: Phase 3 - Budgets, Caching, and Hardening

---

## Future Improvements

### 1. Budget Enforcement Integration (Medium Priority)

**Location**: `crates/speccade-spec/src/validation/recipe_outputs_*.rs`

**Current State**: Recipe validation modules use inline constants for budget limits:
- `recipe_outputs_audio.rs` has its own duration/layer limits
- `recipe_outputs_music.rs` has its own channel/pattern limits
- `recipe_outputs_texture.rs` has its own dimension/graph limits

**Recommended Change**: Migrate these modules to import and use constants from `BudgetProfile`:

```rust
// Before
const MAX_DURATION: f64 = 30.0;

// After
use crate::validation::budgets::AudioBudget;
let max_duration = AudioBudget::DEFAULT_MAX_DURATION_SECONDS;
```

**Benefit**: Single source of truth for budget limits, easier to adjust globally.

---

### 2. File-Based Caching Implementation (Low Priority)

**Location**: New module in `speccade-cli` or `speccade-cache` crate

**Current State**: Caching infrastructure is documented and cache keys are computable from report fields, but no actual caching is implemented.

**Recommended Implementation**:
```rust
struct CacheKey {
    recipe_hash: String,      // From Report.recipe_hash
    backend_version: String,  // From Report.backend_version
    stdlib_version: String,   // From Report.stdlib_version (Starlark only)
}

struct CacheEntry {
    key: CacheKey,
    outputs: Vec<PathBuf>,
    created_at: SystemTime,
}
```

**Storage Options**:
- Local: `~/.cache/speccade/` directory
- CI: Environment variable for cache directory
- Cloud: Optional R2/S3 backend for shared caching

**Benefit**: Avoid redundant generation for unchanged specs.

---

### 3. Budget Profile CLI Flag (Low Priority)

**Location**: `crates/speccade-cli/src/commands/generate.rs`

**Current State**: Generate command uses default budget profile.

**Recommended Change**: Add `--budget-profile` flag:
```bash
speccade generate spec.json --budget-profile=strict
speccade generate spec.json --budget-profile=zx-8bit
```

**Implementation**:
```rust
#[derive(Parser)]
struct GenerateArgs {
    #[arg(long, default_value = "default")]
    budget_profile: String,
}
```

**Benefit**: CI/CD can enforce stricter limits without modifying specs.

---

### 4. Custom Budget Profile Loading (Low Priority)

**Location**: `crates/speccade-spec/src/validation/budgets.rs`

**Current State**: Only three hardcoded profiles (default, strict, zx-8bit).

**Recommended Change**: Support loading custom profiles from TOML/JSON:
```toml
# .speccade/budgets/mobile.toml
name = "mobile"

[audio]
max_duration_seconds = 15.0
max_layers = 16

[texture]
max_dimension = 1024
```

**Benefit**: Project-specific budget constraints without code changes.

---

## Technical Debt

### 1. Unused Budget Parameter

**Location**: `crates/speccade-spec/src/validation/mod.rs:271`

**Issue**: The `_budget` parameter in `validate_for_generate_with_budget()` is currently unused:
```rust
pub fn validate_for_generate_with_budget(
    spec: &Spec,
    _budget: &BudgetProfile,  // Prefixed with _ to suppress warning
) -> ValidationResult {
```

**Resolution**: Complete the migration of inline constants in recipe_outputs_*.rs modules to use the budget parameter.

---

### 2. Canonicalization Float Precision

**Location**: `crates/speccade-spec/src/hash.rs:133-134`

**Current Behavior**: Integer-like floats (e.g., 1.0) are formatted as integers when `|f| < 1e15`.

**Potential Issue**: Very large floats near the boundary could have precision loss.

**Current Mitigation**: Spec values are unlikely to approach 1e15. If this becomes an issue, consider using a decimal library for exact representation.

---

### 3. Report Source Hash Consistency

**Location**: Report provenance fields

**Current State**: `source_hash` is computed from file content before any processing.

**Edge Case**: If a file is read with different line endings (CRLF vs LF), the hash will differ.

**Recommendation**: Consider normalizing line endings before hashing, or document that source_hash is sensitive to exact file content.

---

## Notes

- All follow-up items are non-blocking for Phase 3 completion
- Budget integration (item 1) is the most valuable near-term improvement
- Caching (item 2) would provide the most user-visible improvement but requires careful design for correctness
