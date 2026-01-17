# Phase 1 Implementation Log

## Overview

This document chronicles the implementation of Phase 1: Starlark Compiler and .star Input support for SpecCade.

---

## Step 1: Dependency Setup

**Status:** Completed

### What was done

1. Added `starlark = "=0.12.0"` and `tokio = { version = "1", features = ["rt", "time"] }` to workspace dependencies in `Cargo.toml`
2. Added feature gate `starlark = ["dep:starlark", "dep:tokio", "dep:thiserror"]` to `crates/speccade-cli/Cargo.toml`
3. Set `default = ["starlark"]` so Starlark support is enabled by default
4. Added optional dependencies: `starlark`, `tokio`, and `thiserror` to CLI crate

### Deviations from plan

- Added `thiserror` as an optional dependency for the CLI crate (was already a workspace dependency for speccade-spec)

### Blockers

None

---

## Step 2: Input Abstraction Layer

**Status:** Completed

### What was done

1. Created `crates/speccade-cli/src/input.rs` with:
   - `SourceKind` enum (Json, Starlark) with serde support and `as_str()` method
   - `CompileWarning` struct for Starlark compilation warnings
   - `LoadResult` struct containing spec, source_kind, source_hash, and warnings
   - `InputError` enum for all loading errors with cfg-gated Starlark-specific variants
   - `load_spec()` function that dispatches by file extension
   - `load_json_spec()` and `load_starlark_spec()` internal functions
2. Added `pub mod input;` to `main.rs`
3. Added constants `JSON_EXTENSIONS` and `STARLARK_EXTENSIONS`
4. Implemented source hash computation using BLAKE3

### Deviations from plan

- Added `Display` impl for `SourceKind`
- Made `CompileWarning` methods (`new`, `with_location`) for ergonomic construction
- Added `StarlarkNotEnabled` variant to `InputError` for builds without starlark feature

### Blockers

None

---

## Step 3: Starlark Compiler Module

**Status:** Completed

### What was done

1. Created `crates/speccade-cli/src/compiler/` directory with:
   - `mod.rs` - Module root with `STDLIB_VERSION`, `DEFAULT_TIMEOUT_SECONDS`, `CompilerConfig`, `CompileWarning`, `CompileResult`, and `compile()` function
   - `error.rs` - `CompileError` enum with variants for syntax, runtime, timeout, type, conversion, and validation errors
   - `convert.rs` - `starlark_to_json()` function for Starlark Value to JSON conversion
   - `eval.rs` - Starlark evaluation with timeout using tokio
2. Added `#[cfg(feature = "starlark")] mod compiler;` to `main.rs`
3. Configured Starlark dialect for safety:
   - `enable_def: true` - Functions allowed
   - `enable_lambda: true` - Lambdas allowed
   - `enable_load: false` - No external file loading
   - `enable_top_level_stmt: true` - Top-level statements allowed

### Deviations from plan

- Did not create separate `safety.rs` file; timeout handling is in `eval.rs`
- Used `spawn_blocking` for evaluation within tokio timeout wrapper

### Blockers

None

---

## Step 4: CLI Commands

**Status:** Completed

### What was done

1. Created `crates/speccade-cli/src/commands/eval.rs`:
   - New `eval` command that loads a spec and prints canonical IR JSON to stdout
   - Supports `--pretty` flag for formatted output
   - Prints warnings to stderr
   - Comprehensive error handling with colored output
2. Updated `Commands` enum in `main.rs`:
   - Added `Eval` variant with `spec` and `pretty` arguments
   - Updated help text for all spec arguments to say "(JSON or Starlark)"
3. Updated `commands/validate.rs`:
   - Now uses `load_spec()` instead of direct JSON loading
   - Prints source kind and hash prefix
   - Prints load warnings
4. Updated `commands/generate.rs`:
   - Now uses `load_spec()` instead of direct JSON loading
   - Prints source kind and hash prefix
   - Prints load warnings
5. Updated `commands/mod.rs` to include `pub mod eval;`
6. Added CLI parsing tests for `eval` command

### Deviations from plan

- Did not update `commands/fmt.rs` - deferring Starlark-to-JSON format conversion to later phase
- Did not update `commands/expand.rs` - deferring to later phase per plan.md

### Blockers

None

---

## Step 5: Report Provenance Fields

**Status:** Completed

### What was done

1. Updated `crates/speccade-spec/src/report/mod.rs`:
   - Added `source_kind: Option<String>` field
   - Added `source_hash: Option<String>` field
   - Added `stdlib_version: Option<String>` field
   - All fields have `#[serde(skip_serializing_if = "Option::is_none")]`
2. Updated `crates/speccade-spec/src/report/builder.rs`:
   - Added fields to `ReportBuilder` struct
   - Added `source_provenance(kind, hash)` builder method
   - Added `stdlib_version(version)` builder method
   - Updated `build()` to include new fields
3. Updated `commands/validate.rs`:
   - Calls `source_provenance()` on report builder
   - Calls `stdlib_version()` for Starlark sources
4. Updated `commands/generate.rs`:
   - Created `with_provenance` helper that sets source_provenance and stdlib_version
   - Updated all report builder usage to use `with_provenance`

### Deviations from plan

None

### Blockers

None

---

## Step 6: Compose Expansion Integration

**Status:** Deferred (as planned)

Per plan.md, compose expansion integration is deferred to Phase 1b.

---

## Step 7: Tests

**Status:** Completed

### What was done

1. Added unit tests in compiler module files (`mod.rs`, `eval.rs`, `convert.rs`)
2. Created golden test fixtures in `golden/starlark/`:
   - `minimal.star` - Simplest valid Starlark spec
   - `with_functions.star` - Demonstrates functions and variables
   - `with_comprehensions.star` - Demonstrates list comprehensions
3. Created integration test `crates/speccade-tests/tests/starlark_input.rs`:
   - Tests for loading minimal, functions, and comprehensions specs
   - Tests for JSON-Starlark equivalence
   - Tests for error cases (syntax, runtime, invalid spec)
   - Tests for source hash computation
4. Added `speccade-cli` as dependency to speccade-tests
5. Added test entry in `speccade-tests/Cargo.toml`

### Deviations from plan

- Did not create `.expected.json` files for golden tests; instead using in-test assertions

### Blockers

None

---

## Summary

All planned implementation steps have been completed (with Step 6 deferred as planned).

### Key Deliverables

1. **Starlark crate** added to workspace with feature gate
2. **Input abstraction** layer with `load_spec()` function
3. **Compiler module** for Starlark evaluation with safety limits
4. **Eval command** for debugging Starlark specs
5. **Updated validate/generate** commands to accept .star files
6. **Report provenance** fields for source tracking
7. **Golden test fixtures** and integration tests

### Acceptance Criteria Status

| Criterion | Status |
|-----------|--------|
| CLI accepts .json AND .star | Implemented |
| New `eval` command | Implemented |
| Backends consume canonical Spec only | Preserved (no backend changes) |
| Hashes computed on canonical IR | Preserved (existing hash.rs unchanged) |
| No breaking changes for JSON | Preserved (all existing behavior maintained) |

### Risks Mitigated

- R1 (Infinite loops): Timeout mechanism + standard dialect (no recursion by default in Starlark)
- R3 (Non-determinism): Standard Starlark globals only, no custom builtins
- R4 (Breaking JSON changes): Extension-based dispatch, JSON path unchanged
- R5 (Starlark crate stability): Version pinned to `=0.12.0`
- R6 (Validation bypass): Starlark output parsed through existing `Spec::from_value()`
- R7 (Error attribution): Source provenance tracked in reports
- R8 (Build times): Starlark feature is optional (default on, can be disabled)

---

## Retry 1: Compilation Fixes

### Fix 1: convert.rs - Public API for numerics

The `value.unpack_num()` method is private in starlark 0.12.0. Replaced with a type-check and string parsing approach:

```rust
// Before (private API):
if let Some(f) = value.unpack_num().and_then(|n| n.as_float()) { ... }

// After (public API):
if value.get_type() == "float" {
    let s = value.to_str();
    if let Ok(f) = s.parse::<f64>() { ... }
}
```

This uses the public `get_type()` method to check for floats, then parses from the string representation.

### Fix 2: eval.rs - Tokio Builder API

The tokio `rt` feature does not include `Runtime::new()`. Replaced with the Builder API:

```rust
// Before:
use tokio::runtime::Runtime;
let rt = Runtime::new().map_err(...)?;

// After:
use tokio::runtime::Builder;
let rt = Builder::new_current_thread()
    .enable_time()
    .build()
    .map_err(...)?;
```

The Builder API is available with the `rt` feature and explicitly enables the time feature needed for timeout.

### Fix 3: lib.rs - Add library target

The `speccade-tests` crate tried to import `speccade_cli` as a library, but it was only configured as a binary.

Changes:
1. Added `[lib]` section to `crates/speccade-cli/Cargo.toml` with `name = "speccade_cli"` and `path = "src/lib.rs"`
2. Created `crates/speccade-cli/src/lib.rs` with public module exports:
   - `pub mod input;`
   - `#[cfg(feature = "starlark")] pub mod compiler;`
   - `pub mod commands;`
   - `mod dispatch;` (private, used internally by commands)
   - `pub mod parity_data;`
   - `pub mod parity_matrix;`
3. Updated `main.rs` to import from the library crate instead of declaring modules locally

---

## Retry 2: Use of Moved Value Fix

### Fix 4: eval.rs - E0382 "use of moved value: config"

The `config` value was cloned and moved into the `spawn_blocking` closure, but then `config.timeout_seconds` was accessed in the `Err` arm of the timeout match after the move.

```rust
// Before:
let config = config.clone();
// ... config moved into spawn_blocking ...
Err(_) => Err(CompileError::Timeout {
    seconds: config.timeout_seconds,  // ERROR: config already moved
}),

// After:
let timeout_seconds = config.timeout_seconds;  // Extract before clone
let config = config.clone();
// ... config moved into spawn_blocking ...
Err(_) => Err(CompileError::Timeout {
    seconds: timeout_seconds,  // Use pre-extracted value
}),
```

The fix extracts the `timeout_seconds` value before cloning `config`, so the value is available in the outer scope for the error message after `config` has been moved into the async block.
