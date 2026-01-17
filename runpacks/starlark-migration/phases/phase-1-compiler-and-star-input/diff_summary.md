# Phase 1 Diff Summary

## Files Changed

| File | Change | Reason |
|------|--------|--------|
| `Cargo.toml` | Modified | Added `starlark = "=0.12.0"` and `tokio` to workspace dependencies |
| `crates/speccade-cli/Cargo.toml` | Modified | Added `[features]` section with starlark feature gate; added optional deps |
| `crates/speccade-cli/src/main.rs` | Modified | Removed local module declarations, imports from lib crate, `Eval` command, updated help text |
| `crates/speccade-cli/src/lib.rs` | Created | Library crate root with public module exports |
| `crates/speccade-cli/src/input.rs` | Created | Input abstraction layer with `load_spec()`, `SourceKind`, `InputError` |
| `crates/speccade-cli/src/compiler/mod.rs` | Created | Compiler module root, `CompilerConfig`, `compile()` function |
| `crates/speccade-cli/src/compiler/error.rs` | Created | `CompileError` enum for Starlark errors |
| `crates/speccade-cli/src/compiler/convert.rs` | Created | Starlark Value to JSON conversion |
| `crates/speccade-cli/src/compiler/eval.rs` | Created | Starlark evaluation with timeout |
| `crates/speccade-cli/src/commands/mod.rs` | Modified | Added `pub mod eval;`, updated test |
| `crates/speccade-cli/src/commands/eval.rs` | Created | New `eval` command implementation |
| `crates/speccade-cli/src/commands/validate.rs` | Modified | Use `load_spec()`, add provenance to report |
| `crates/speccade-cli/src/commands/generate.rs` | Modified | Use `load_spec()`, add provenance to report |
| `crates/speccade-spec/src/report/mod.rs` | Modified | Added `source_kind`, `source_hash`, `stdlib_version` fields |
| `crates/speccade-spec/src/report/builder.rs` | Modified | Added `source_provenance()`, `stdlib_version()` builder methods |
| `crates/speccade-tests/Cargo.toml` | Modified | Added `speccade-cli` dependency, `starlark_input` test entry |
| `crates/speccade-tests/tests/starlark_input.rs` | Created | Integration tests for Starlark input |
| `golden/starlark/minimal.star` | Created | Golden test fixture - minimal spec |
| `golden/starlark/with_functions.star` | Created | Golden test fixture - functions demo |
| `golden/starlark/with_comprehensions.star` | Created | Golden test fixture - comprehensions demo |

## Files NOT Changed (per scope)

| File/Directory | Reason |
|----------------|--------|
| `crates/speccade-backend-*/**` | Backends remain Starlark-unaware |
| `crates/speccade-spec/src/spec.rs` | Spec struct unchanged (IR contract v1) |
| `crates/speccade-spec/src/hash.rs` | Hashing unchanged (operates on canonical IR) |
| `schemas/**` | No schema changes needed (Starlark produces same IR) |
| `crates/speccade-cli/src/commands/fmt.rs` | Starlark format support deferred |
| `crates/speccade-cli/src/commands/expand.rs` | Starlark support deferred |

## Summary Statistics

| Category | Count |
|----------|-------|
| Files created | 11 |
| Files modified | 10 |
| Lines added (approx.) | ~1520 |
| Test files created | 1 |
| Golden fixtures created | 3 |

## Key Changes by Category

### New Modules

- `speccade-cli/src/lib.rs` - Library crate root with public module exports
- `speccade-cli/src/input.rs` - Input abstraction (JSON/Starlark dispatch)
- `speccade-cli/src/compiler/` - Starlark compilation (4 files)
- `speccade-cli/src/commands/eval.rs` - New CLI command

### Updated Commands

- `validate` - Now accepts .star files, includes source provenance in reports
- `generate` - Now accepts .star files, includes source provenance in reports

### Report Schema Additions

```rust
// New optional fields in Report struct
pub source_kind: Option<String>,    // "json" or "starlark"
pub source_hash: Option<String>,    // BLAKE3 hash of source file
pub stdlib_version: Option<String>, // Starlark stdlib version (for cache invalidation)
```

### Feature Gate

```toml
[features]
default = ["starlark"]
starlark = ["dep:starlark", "dep:tokio", "dep:thiserror"]
```

Building without Starlark: `cargo build --no-default-features`
