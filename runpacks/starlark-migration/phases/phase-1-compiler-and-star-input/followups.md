# Phase 1 Follow-ups

Items identified during quality review that should be addressed in later phases.

---

## Phase 2 (stdlib) Work

### 1. Deduplicate CompileWarning Types

**Issue:** `CompileWarning` is defined in both `input.rs` and `compiler/mod.rs` with identical structure.

**Location:**
- `crates/speccade-cli/src/input.rs:47-70`
- `crates/speccade-cli/src/compiler/mod.rs:75-99`

**Recommendation:** In Phase 2, when adding stdlib builtins:
1. Move `CompileWarning` to `compiler/mod.rs` as the canonical definition
2. Re-export from `input.rs` or use `compiler::CompileWarning` directly
3. Remove the duplicate type

**Priority:** Low (code smell, not a bug)

### 2. Add Starlark Linting/Warnings Infrastructure

**Issue:** The warning system is in place but currently unused. Starlark evaluation doesn't generate warnings.

**Recommendation:** In Phase 2, consider adding:
- Deprecation warnings for future stdlib changes
- Style warnings (e.g., unused variables)
- Performance hints (e.g., large lists)

**Priority:** Medium

---

## Phase 3 (hardening) Work

### 3. Memory Limits for Starlark Evaluation

**Issue:** While timeout is enforced, there's no memory limit. A malicious spec could allocate unbounded memory.

**Recommendation:** Investigate Starlark crate memory limiting options:
- Check if starlark 0.12.0 supports heap limits
- If not, consider using `spawn_blocking` with memory monitoring
- Document the limitation in the meantime

**Priority:** High for production use

### 4. Recursion Depth Limits

**Issue:** Starlark Standard dialect disables recursion by default, but deep call stacks via mutual `def` calls could still occur.

**Recommendation:** Verify recursion behavior and document:
- Test deep call stack behavior
- Add explicit recursion limit if needed
- Document in safety section

**Priority:** Medium

### 5. Enhanced Error Location Tracking

**Issue:** Some error messages show "unknown" for location when task panics in `spawn_blocking`.

**Location:** `crates/speccade-cli/src/compiler/eval.rs:128-131`

**Recommendation:**
- Capture panic location information
- Use `catch_unwind` for better panic handling
- Preserve original error location through async boundary

**Priority:** Low (edge case)

---

## Technical Debt

### 6. Float Conversion Workaround

**Issue:** Float conversion uses string parsing as a workaround for private `unpack_num()` API.

**Location:** `crates/speccade-cli/src/compiler/convert.rs:52-62`

```rust
if value.get_type() == "float" {
    let s = value.to_str();
    if let Ok(f) = s.parse::<f64>() { ... }
}
```

**Recommendation:** When upgrading starlark crate version:
- Check if public float extraction API is available
- Replace string parsing with direct extraction
- Add comment explaining the workaround if it must remain

**Priority:** Low (works correctly, just inelegant)

### 7. Tokio Runtime Per-Evaluation

**Issue:** A new tokio runtime is created for each Starlark evaluation.

**Location:** `crates/speccade-cli/src/compiler/eval.rs:99-105`

**Recommendation:** If performance becomes a concern:
- Consider runtime pooling
- Or use a single global runtime with `#[tokio::main]`
- Current approach is fine for CLI use (low evaluation frequency)

**Priority:** Low (optimization, not correctness)

---

## Documentation Improvements

### 8. Add Starlark Spec Authoring Guide

**Recommendation:** Create documentation for spec authors:
- Starlark dialect limitations (no `load()`)
- Available functions and features
- Examples of common patterns
- Error message interpretation

**Location:** `docs/book/src/starlark-specs.md` (new file)

**Priority:** Medium (user experience)

### 9. Document Timeout Configuration

**Recommendation:** Add documentation for:
- Default timeout value
- How to configure custom timeouts
- When to increase timeouts (large specs)

**Priority:** Low

---

## Future Enhancements (Not Required for Migration)

### 10. Starlark Language Server Protocol

**Potential:** Add LSP support for IDE integration:
- Syntax highlighting
- Autocompletion for spec fields
- Hover documentation

**Priority:** Not planned (nice-to-have)

### 11. Starlark Spec Templates

**Potential:** Add template command support for Starlark:
- `speccade template copy --format star`
- Convert JSON templates to Starlark

**Priority:** Not planned (nice-to-have)
