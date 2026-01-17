# Phase 3 Research: Budgets, Caching, and Hardening

**Date**: 2026-01-17
**Phase**: Phase 3 - Budgets, Caching, and Hardening

---

## 1. Budget Enforcement

### Current State

Budget enforcement is **split** between validation and backend stages, with some overlap:

#### Validation Stage (`speccade-spec/src/validation/`)

Budget checks exist in `recipe_outputs_audio.rs`:
- `MAX_AUDIO_DURATION_SECONDS = 30.0` (line 110)
- `MAX_AUDIO_LAYERS = 32` (line 111)
- Sample rate validation (specific values: 22050, 44100, 48000)
- Expanded layer count validation

Resolution validation in `common.rs`:
- `MAX_DIMENSION = 4096` (line 52)
- `MAX_PIXELS = MAX_DIMENSION * MAX_DIMENSION` (line 53)

#### Backend Stage (duplicated checks)

`speccade-backend-audio/src/generate/mod.rs` re-defines:
- `MAX_AUDIO_DURATION_SECONDS = 30.0` (line 88)
- `MAX_AUDIO_LAYERS = 32` (line 89)
- `MAX_NUM_SAMPLES` (line 90)

`speccade-backend-music/src/compose/expander_context.rs`:
- `MAX_RECURSION_DEPTH = 64` (line 16)
- `MAX_CELLS_PER_PATTERN = 50_000` (line 17)
- `MAX_TIME_LIST_LEN = 50_000` (line 18)
- `MAX_PATTERN_STRING_LEN = 100_000` (line 19)

`speccade-backend-music/src/xm/header.rs`:
- `XM_MAX_CHANNELS = 32` (line 16)
- `XM_MAX_PATTERNS = 256` (line 19)
- `XM_MAX_INSTRUMENTS = 128` (line 22)
- `XM_MAX_PATTERN_ROWS = 256` (line 25)

### Gaps Identified

1. **Duplication**: Audio budgets defined in both validation and backend
2. **No Texture Budget Validation**: Resolution limits exist in `common.rs` but are not consistently applied during validation
3. **No Mesh Budget Validation**: No vertex/face limits at validation stage
4. **No Budget Profiles**: No `--budget-profile` mechanism exists yet
5. **Format-Specific Limits**: XM/IT limits are in backend only, not validated early

### Recommendations

1. Centralize budget constants in `speccade-spec/src/validation/budgets.rs`
2. Add validation-stage budget checks for all asset types
3. Implement budget profile system for different targets (e.g., `zx-8bit`, `modern`)
4. Remove duplicate constants from backends (import from spec crate)

---

## 2. Report Provenance

### Current State

The `Report` struct (`speccade-spec/src/report/mod.rs`) already has provenance fields:

```rust
pub struct Report {
    // Core identification
    pub spec_hash: String,              // BLAKE3 hash of canonical IR
    pub base_spec_hash: Option<String>, // For variant runs
    pub variant_id: Option<String>,

    // Provenance fields (already implemented)
    pub source_kind: Option<String>,     // "json" or "starlark"
    pub source_hash: Option<String>,     // BLAKE3 of source file
    pub stdlib_version: Option<String>,  // For Starlark sources
    pub recipe_hash: Option<String>,     // Hash of recipe kind+params

    // Toolchain provenance
    pub backend_version: String,
    pub target_triple: String,
    pub git_commit: Option<String>,
    pub git_dirty: Option<bool>,
    // ...
}
```

### Builder Support

`ReportBuilder` (`report/builder.rs`) provides:
- `source_provenance(kind, hash)` - Sets source_kind and source_hash
- `stdlib_version(version)` - Sets stdlib version for cache invalidation
- `recipe_hash(hash)` - Sets recipe fingerprint

### Usage in CLI

`generate.rs` correctly populates provenance:
- `source_kind` and `source_hash` from `LoadResult`
- `stdlib_version` from `crate::compiler::STDLIB_VERSION` (currently "0.1.0")
- `recipe_hash` computed via `canonical_recipe_hash()`

### Status

**Complete** - All provenance fields are implemented and populated. The `stdlib_version` is tracked in `compiler/mod.rs:53`.

---

## 3. Caching Strategy

### Current State

**No caching is currently implemented** for generation outputs.

#### Existing Infrastructure

1. **Hashing**: Full JCS+BLAKE3 implementation in `speccade-spec/src/hash.rs`
   - `canonical_spec_hash()` - Full spec hash
   - `canonical_recipe_hash()` - Recipe-only hash (for generation caching)
   - `canonical_value_hash()` - General JSON value hash

2. **Report has cache-key fields**:
   - `spec_hash` - Post-resolve IR hash
   - `recipe_hash` - Recipe fingerprint
   - `stdlib_version` - For Starlark stdlib changes
   - `source_hash` - For source file change detection

3. **Backend identifiers**: Each backend can produce `backend_version` strings

#### In-memory Caching

- `speccade-backend-texture/src/generate/graph/operations.rs` uses a `HashMap` cache for graph node evaluation
- This is runtime cache, not persistent

### Cache Key Requirements

Per ARCHITECTURE_PROPOSAL.md, cache keys need:
1. **IR hash** - `spec_hash` (canonical post-resolve)
2. **Recipe hash** - Already computed
3. **Toolchain version** - `backend_version` + `git_commit`
4. **stdlib_version** - For Starlark sources

### Recommendations

1. Implement file-based cache under `.speccade/cache/`
2. Cache key format: `{recipe_hash}_{backend_version_hash}`
3. Store: generated artifacts + report.json
4. Invalidate on: stdlib_version change, git_dirty=true, backend version change
5. Add `--no-cache` flag to CLI

---

## 4. Canonicalization

### Current Implementation

`speccade-spec/src/hash.rs` implements RFC 8785 (JCS):

```rust
fn canonicalize_value(value: &serde_json::Value) -> String {
    match value {
        // ...
        serde_json::Value::Object(obj) => {
            // Sort keys lexicographically
            let mut sorted_keys: Vec<&String> = obj.keys().collect();
            sorted_keys.sort();
            // ...
        }
    }
}
```

Key features:
- Keys sorted lexicographically
- No whitespace between tokens
- JCS number formatting (no trailing zeros)
- Minimal string escaping

### Idempotence Testing

**No explicit idempotence tests exist.**

The hash tests verify:
- Same spec produces same hash (stability)
- Object key ordering is normalized
- Different specs produce different hashes

But there are **no tests** for:
- `canonicalize(canonicalize(x)) == canonicalize(x)`
- Edge cases: empty objects, nested objects, special characters

### Potential Issues

1. Float handling: `format_jcs_number()` has edge cases for integer-like floats
2. No validation that output is valid JSON (no re-parse check)
3. Arrays are not re-ordered (correct - arrays preserve order)

### Recommendations

1. Add explicit idempotence test:
   ```rust
   fn test_canonicalization_idempotent() {
       let spec = make_spec();
       let canon1 = canonicalize_json(&spec.to_value()).unwrap();
       let canon2 = canonicalize_json(&serde_json::from_str(&canon1).unwrap()).unwrap();
       assert_eq!(canon1, canon2);
   }
   ```

2. Add property-based tests for edge cases
3. Add round-trip validation (canonicalize -> parse -> canonicalize)

---

## 5. Validation Infrastructure

### Current Validation

`speccade-spec/src/validation/mod.rs` provides:
- `validate_spec()` - Contract + recipe validation
- `validate_for_generate()` - Adds recipe requirement + supported kinds check

Recipe-specific validation in submodules:
- `recipe_outputs_audio.rs` - Audio params, LFO targets, layer counts
- `recipe_outputs_music.rs` - Tracker format matching, instrument sources
- `recipe_outputs_texture.rs` - Graph cycle detection, type checking

### Test Coverage

Extensive unit tests in `validation/tests.rs`:
- Asset ID validation
- Output path safety
- Recipe type matching
- Format-specific checks (audio LFOs, music instruments)
- Graph cycle detection

### Gaps

1. **No fuzz testing** - No `proptest` or `quickcheck` usage
2. **No property-based tests** - Only example-based tests
3. **Memory limits** - Phase 1 followup identified missing Starlark memory limits
4. **Recursion depth** - Phase 1 followup identified deep call stack concerns

### Determinism Testing

`speccade-tests/src/determinism/` provides framework for:
- Verifying same seed produces same output
- Multi-run verification

Tests in `e2e_determinism.rs` cover:
- Audio determinism
- Texture determinism
- Different seeds produce different outputs

### Recommendations

1. Add property-based testing for spec validation
2. Add fuzz testing for parser inputs
3. Implement memory limits for Starlark (Phase 1 followup)
4. Add explicit idempotence tests for canonicalization
5. Increase edge case coverage for validation

---

## 6. Summary Tables

### Budget Enforcement Status

| Asset Type | Validation Stage | Backend Stage | Unified? |
|------------|-----------------|---------------|----------|
| Audio | Duration, layers, sample rate | Duration, layers, samples | Partially |
| Music | None | Channels, patterns, rows | No |
| Texture | Resolution (common.rs) | None specific | No |
| Mesh | None | tri_budget (optional) | No |

### Report Provenance Status

| Field | Implemented | Populated | Notes |
|-------|-------------|-----------|-------|
| spec_hash | Yes | Yes | Post-resolve IR |
| source_kind | Yes | Yes | "json" or "starlark" |
| source_hash | Yes | Yes | Raw source BLAKE3 |
| stdlib_version | Yes | Yes | For Starlark sources |
| recipe_hash | Yes | Yes | Recipe fingerprint |
| backend_version | Yes | Yes | Backend identifier |
| git_commit | Yes | Optional | Build-time env |

### Caching Status

| Component | Status |
|-----------|--------|
| Hash computation | Implemented |
| Cache keys | Not implemented |
| Disk cache | Not implemented |
| Cache invalidation | Not implemented |
| CLI flags | Not implemented |

### Test Coverage

| Area | Unit Tests | Integration | Property-based | Fuzz |
|------|------------|-------------|----------------|------|
| Validation | Extensive | Yes | No | No |
| Hashing | Good | Yes | No | No |
| Canonicalization | Basic | No | No | No |
| Determinism | Yes | Yes | No | No |
