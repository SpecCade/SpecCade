# Phase 1 Implementation Plan

## Overview

This plan implements the Starlark input layer for SpecCade, enabling the CLI to accept both `.json` and `.star` files while maintaining full backward compatibility for JSON users.

---

## Step 1: Dependency Setup

**Complexity:** S (Small)

**Objective:** Add the Starlark crate to the workspace and configure feature gates.

### Files to Modify

| File | Change |
|------|--------|
| `Cargo.toml` (workspace root) | Add `starlark = "0.12"` to `[workspace.dependencies]` |
| `crates/speccade-cli/Cargo.toml` | Add `starlark` as optional dependency with feature gate |

### Details

1. Add to workspace dependencies:
   ```toml
   starlark = "0.12"
   tokio = { version = "1", features = ["rt", "time"] }
   ```

2. Add feature gate to CLI:
   ```toml
   [features]
   default = ["starlark"]
   starlark = ["dep:starlark", "dep:tokio"]

   [dependencies]
   starlark = { workspace = true, optional = true }
   tokio = { workspace = true, optional = true }
   ```

3. Pin exact version for stability: `starlark = "=0.12.0"` (evaluate latest stable at implementation time)

### Dependencies
- None (first step)

### Risks Addressed
- R5 (Starlark crate stability): Version pinning
- R8 (Build times): Feature gate allows opting out

---

## Step 2: Input Abstraction Layer

**Complexity:** M (Medium)

**Objective:** Create a unified input loading abstraction that dispatches by file extension.

### Files to Create

| File | Purpose |
|------|---------|
| `crates/speccade-cli/src/input.rs` | Input abstraction module |

### Files to Modify

| File | Change |
|------|--------|
| `crates/speccade-cli/src/main.rs` | Add `mod input;` |

### Interface

```rust
// crates/speccade-cli/src/input.rs

/// Source type for provenance tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Json,
    Starlark,
}

/// Result of loading and compiling a spec
pub struct LoadResult {
    /// The canonical spec (IR)
    pub spec: Spec,
    /// Source type for provenance
    pub source_kind: SourceKind,
    /// Hash of the source file (before compilation)
    pub source_hash: String,
    /// Warnings from compilation (Starlark only)
    pub warnings: Vec<CompileWarning>,
}

/// Unified spec loading function
pub fn load_spec(path: &Path) -> Result<LoadResult, InputError>;
```

### Details

1. Detect extension: `.json` vs `.star`/`.bzl`
2. For JSON: Load directly via `Spec::from_json()`
3. For Starlark: Delegate to compiler module (Step 3)
4. Compute source hash (BLAKE3) for provenance
5. Return `LoadResult` with all metadata

### Dependencies
- Step 1 (dependency setup)

### Risks Addressed
- R4 (Breaking JSON changes): Strict extension dispatch
- R7 (Error attribution): Source kind tracking

---

## Step 3: Starlark Compiler Module

**Complexity:** L (Large)

**Objective:** Implement Starlark evaluation that produces canonical `Spec` IR.

### Files to Create

| File | Purpose |
|------|---------|
| `crates/speccade-cli/src/compiler/mod.rs` | Compiler module root |
| `crates/speccade-cli/src/compiler/eval.rs` | Starlark evaluation logic |
| `crates/speccade-cli/src/compiler/convert.rs` | Starlark Value -> JSON Value conversion |
| `crates/speccade-cli/src/compiler/error.rs` | Compiler error types |
| `crates/speccade-cli/src/compiler/safety.rs` | Safety limits (timeout wrapper) |

### Files to Modify

| File | Change |
|------|--------|
| `crates/speccade-cli/src/main.rs` | Add `mod compiler;` |
| `crates/speccade-cli/src/input.rs` | Import and use compiler |

### Details

#### Evaluation Flow

```
.star file
    |
    v
AstModule::parse() -- syntax errors here
    |
    v
Evaluator::eval_module() -- runtime errors here
    |                     -- wrapped with timeout (30s)
    v
starlark::Value (dict)
    |
    v
convert_to_json() -- convert to serde_json::Value
    |
    v
Spec::from_value() -- existing JSON parsing
    |
    v
Canonical Spec IR
```

#### Starlark Dialect Configuration

```rust
let dialect = Dialect {
    enable_def: true,
    enable_lambda: true,
    enable_load: false,        // No external file loading in Phase 1
    enable_recursion: false,   // Prevents infinite loops
    enable_top_level_stmt: true,
    ..Dialect::Standard
};
```

#### Safety Limits

1. **Timeout (R1):**
   ```rust
   tokio::time::timeout(Duration::from_secs(30), async {
       eval_starlark(source)
   }).await?
   ```

2. **No recursion** (R1): Disabled in dialect
3. **No non-deterministic builtins** (R3): Use standard globals only

#### Value Conversion

Starlark `Value` -> `serde_json::Value`:
- `dict` -> JSON object
- `list` -> JSON array
- `string` -> JSON string
- `int` -> JSON number
- `float` -> JSON number
- `bool` -> JSON boolean
- `None` -> JSON null

### Dependencies
- Step 1 (Starlark crate)
- Step 2 (input abstraction)

### Risks Addressed
- R1 (Infinite loops): Timeout + no recursion
- R3 (Non-determinism): Standard dialect only
- R6 (Validation bypass): Convert to Spec via existing from_value()

---

## Step 4: CLI Command Updates

**Complexity:** M (Medium)

**Objective:** Update existing commands to use input abstraction and add new `eval` command.

### Files to Create

| File | Purpose |
|------|---------|
| `crates/speccade-cli/src/commands/eval.rs` | New `eval` command |

### Files to Modify

| File | Change |
|------|--------|
| `crates/speccade-cli/src/main.rs` | Add `Eval` variant to `Commands` enum |
| `crates/speccade-cli/src/commands/mod.rs` | Add `pub mod eval;` |
| `crates/speccade-cli/src/commands/validate.rs` | Use `load_spec()` instead of direct JSON loading |
| `crates/speccade-cli/src/commands/generate.rs` | Use `load_spec()` instead of direct JSON loading |
| `crates/speccade-cli/src/commands/expand.rs` | Use `load_spec()` instead of direct JSON loading |
| `crates/speccade-cli/src/commands/fmt.rs` | Use `load_spec()` (for format-on-save of Starlark -> JSON) |

### New Command: `eval`

```
speccade eval <spec.star|ir.json>
```

Prints the canonical IR JSON to stdout. This enables:
- Debugging Starlark specs
- Piping IR to other tools
- Verifying Starlark -> IR conversion

### Command Argument Updates

Update help text in `Commands` enum:
```rust
/// Path to the spec file (JSON or Starlark)
#[arg(short, long)]
spec: String,
```

### Dependencies
- Step 2 (input abstraction)
- Step 3 (compiler module)

---

## Step 5: Report Provenance Fields

**Complexity:** S (Small)

**Objective:** Add source provenance fields to reports for debugging and cache invalidation.

### Files to Modify

| File | Change |
|------|--------|
| `crates/speccade-spec/src/report/mod.rs` | Add `source_kind`, `source_hash`, `stdlib_version` fields |
| `crates/speccade-spec/src/report/builder.rs` | Add builder methods for new fields |
| `crates/speccade-cli/src/commands/validate.rs` | Set provenance in report |
| `crates/speccade-cli/src/commands/generate.rs` | Set provenance in report |

### New Report Fields

```rust
/// Source file format ("json" or "starlark")
#[serde(skip_serializing_if = "Option::is_none")]
pub source_kind: Option<String>,

/// BLAKE3 hash of the source file (before compilation)
#[serde(skip_serializing_if = "Option::is_none")]
pub source_hash: Option<String>,

/// Starlark stdlib version (for cache invalidation)
#[serde(skip_serializing_if = "Option::is_none")]
pub stdlib_version: Option<String>,
```

### Dependencies
- Step 4 (commands updated to use LoadResult)

### Risks Addressed
- R7 (Error attribution): Source provenance in reports

---

## Step 6: Compose Expansion Integration (Optional)

**Complexity:** M (Medium)

**Objective:** Move `music.tracker_song_compose_v1` expansion into the compiler stage.

### Feasibility Assessment

Currently, compose expansion happens in:
1. `commands/expand.rs` - For manual expansion
2. `dispatch/music.rs` - During generation

Moving to compiler stage would:
- Unify expansion for both JSON and Starlark inputs
- Allow `eval` command to output fully expanded IR
- Simplify backend dispatch (always receives canonical `tracker_song_v1`)

### Implementation Approach

1. Add expansion pass after Spec loading in `load_spec()`:
   ```rust
   pub fn load_spec(path: &Path, expand: bool) -> Result<LoadResult, InputError> {
       let mut spec = /* load and parse */;

       if expand {
           spec = expansion::expand_compose(spec)?;
       }

       Ok(LoadResult { spec, ... })
   }
   ```

2. Create expansion module:
   ```
   crates/speccade-cli/src/expansion/mod.rs
   crates/speccade-cli/src/expansion/compose.rs
   ```

3. Move expansion logic from `dispatch/music.rs` to `expansion/compose.rs`

4. Update dispatch to expect only canonical `tracker_song_v1`

### Decision

**Defer to Phase 1b or Phase 2.** The current architecture works, and this optimization can be done after the basic Starlark flow is stable.

### Dependencies
- Steps 1-5 complete

---

## Step 7: Tests

**Complexity:** M (Medium)

**Objective:** Comprehensive test coverage for the new Starlark input path.

### Files to Create/Modify

| File | Purpose |
|------|---------|
| `crates/speccade-cli/src/compiler/tests.rs` | Unit tests for compiler |
| `crates/speccade-tests/tests/starlark_input.rs` | Integration tests |
| `golden/starlark/` | Golden test fixtures (`.star` + expected `.json`) |

### Test Categories

1. **Unit tests** (in compiler module):
   - Starlark parsing success
   - Starlark parsing failures (syntax errors)
   - Value conversion accuracy
   - Timeout enforcement
   - Invalid spec detection

2. **Integration tests** (in speccade-tests):
   - `eval` command produces valid JSON
   - `validate` accepts `.star` files
   - `generate` works with `.star` input
   - Errors include source attribution

3. **Golden tests**:
   - `minimal.star` -> `minimal.expected.json`
   - `audio_sfx.star` -> `audio_sfx.expected.json`
   - Byte-identical IR output verification

4. **Regression tests**:
   - All existing JSON golden tests still pass
   - JSON spec hashes unchanged
   - No behavior changes for JSON users

### Dependencies
- Steps 1-5 complete

---

## Summary

| Step | Complexity | Dependencies | Key Deliverable |
|------|------------|--------------|-----------------|
| 1. Dependency Setup | S | None | `starlark` crate in workspace |
| 2. Input Abstraction | M | Step 1 | `load_spec()` function |
| 3. Compiler Module | L | Steps 1-2 | Starlark eval -> Spec |
| 4. CLI Commands | M | Steps 2-3 | `eval` command, updated validate/generate |
| 5. Report Provenance | S | Step 4 | `source_kind`, `source_hash` fields |
| 6. Compose Expansion | M | Steps 1-5 | (Deferred to Phase 1b) |
| 7. Tests | M | Steps 1-5 | Full coverage |

### Acceptance Criteria Mapping

| Criterion | Steps |
|-----------|-------|
| CLI accepts .json AND .star | Steps 2, 3, 4 |
| New `eval` command | Step 4 |
| Backends consume canonical Spec only | Step 3 (no changes to backends) |
| Hashes computed on canonical IR | Step 3 (existing hash.rs unchanged) |
| No breaking changes for JSON | Steps 2, 4, 7 (regression tests) |

### Estimated Total Effort

- **Small (S):** ~1-2 hours
- **Medium (M):** ~3-6 hours
- **Large (L):** ~8-16 hours

**Total:** ~20-35 hours of implementation time
