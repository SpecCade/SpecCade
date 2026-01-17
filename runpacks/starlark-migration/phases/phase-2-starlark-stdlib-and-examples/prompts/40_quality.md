# Phase 2 Quality Prompt

## Role

You are a **quality agent**. Your job is to refactor for maintainability, improve documentation, and enhance LLM-friendliness.

---

## Permission boundary

- **Allowed**: Read files, edit code within scope globs
- **NOT allowed**: Run build/test commands

---

## Files to open first

Read these in order:

1. `runpacks/starlark-migration/phases/phase-2-starlark-stdlib-and-examples/validation.md` - Validation results
2. `runpacks/starlark-migration/phases/phase-2-starlark-stdlib-and-examples/failures.md` - Failure analysis (if exists)
3. `runpacks/starlark-migration/phases/phase-2-starlark-stdlib-and-examples/implementation_log.md` - Implementation details
4. `runpacks/starlark-migration/phases/phase-2-starlark-stdlib-and-examples/SCOPING.md` - Scope boundaries

---

## Quality focus areas

### 1. LLM-friendliness

Ensure stdlib is easy for LLMs to use:

- [ ] Function names are descriptive and unambiguous
- [ ] Parameters have clear names (not `x`, `y`, `opts`)
- [ ] Doc comments include usage examples
- [ ] Error messages explain what went wrong AND how to fix it
- [ ] No hidden defaults that surprise users

Example of good error message:
```
E0102: frequency 22000.0 is out of range [20.0, 20000.0].
       Human hearing range is 20-20000 Hz.
       Did you mean to use 2200.0?
```

### 2. Code organization

- [ ] stdlib modules have clear, single responsibilities
- [ ] Shared utilities are extracted to common module
- [ ] No duplicate code between audio/texture/mesh modules
- [ ] Tests are organized parallel to implementation

### 3. Documentation quality

- [ ] `docs/starlark-authoring.md` explains the authoring workflow
- [ ] `docs/stdlib-reference.md` has complete function docs
- [ ] `docs/error-codes.md` lists all error codes with examples
- [ ] Examples in docs match examples in `packs/examples/`

### 4. Example quality

- [ ] Each example has a header comment explaining its purpose
- [ ] Examples demonstrate best practices
- [ ] Examples avoid unnecessary complexity
- [ ] Examples can be copy-pasted and modified

### 5. Error code stability

- [ ] Error codes are documented in a single location
- [ ] Codes follow consistent numbering scheme
- [ ] No gaps in numbering (or gaps are documented)
- [ ] Codes are stable (won't change in future versions)

### 6. Golden test robustness

- [ ] Golden tests don't rely on unstable formatting
- [ ] Expected outputs use canonical JSON (RFC 8785)
- [ ] Test assertions give clear failure messages
- [ ] Easy to update expected outputs when intentional changes occur

---

## Refactoring guidelines

### DRY without over-abstraction
```rust
// Good: shared helper for common validation
fn validate_range(value: f64, min: f64, max: f64, field: &str) -> Result<(), StdlibError> {
    if value < min || value > max {
        Err(StdlibError::OutOfRange { value, min, max, field: field.to_string() })
    } else {
        Ok(())
    }
}

// Bad: over-abstracted builder pattern
AudioBuilder::new().with_frequency(440.0).with_waveform("sine").build()
```

### Error messages for LLMs
```rust
// Good: actionable, specific
"E0101: missing required field 'frequency' in audio.synth(). Add frequency=440.0 for A4 note."

// Bad: vague
"E0101: validation error"
```

### Documentation style
```rust
/// Creates a synthesized audio waveform.
///
/// # Example
/// ```starlark
/// audio.synth(
///     waveform = "sine",
///     frequency = 440.0,  # A4 note
///     duration = 1.0,     # 1 second
/// )
/// ```
///
/// # Errors
/// - E0101: Missing required field
/// - E0102: Frequency out of range [20.0, 20000.0]
pub fn audio_synth(...) -> Result<AudioSpec, StdlibError>
```

---

## Output artifacts

Write these files to this phase folder:

### `quality.md`

Quality improvements made:

```markdown
## Quality Improvements

### LLM-friendliness
- Improved error messages in audio.rs to include suggestions
- Added usage examples to all doc comments
- Renamed `env` parameter to `envelope` for clarity

### Code organization
- Extracted validate_range() to stdlib/common.rs
- Removed duplicate frequency validation

### Documentation
- Added workflow diagram to starlark-authoring.md
- Completed all function docs in stdlib-reference.md
- Added examples to error-codes.md

### Examples
- Added header comments to all examples
- Simplified texture example (removed unused parameters)

### Tests
- Improved golden test failure messages
- Added update script for expected outputs
```

### `followups.md` (optional)

Items deferred to Phase 3+:

```markdown
## Deferred Items

### Phase 3: Budgets
- Add parameter range budgets to stdlib validation
- Integrate with unified budget system

### Phase 3: Caching
- Add stdlib_version to cache keys
- Document stdlib versioning strategy

### Future
- Consider adding audio.sample() for sample-based audio
- Consider adding mesh.import() for external formats
- Consider presets library (common instruments, materials)
```

---

## Scope enforcement

Check `SCOPING.md` before editing any file. Only edit within allowed globs.

If you made out-of-scope edits during quality improvements:
1. Document justification in `quality.md`
2. Record in `ARTIFACTS.md` decision log

---

## Completion criteria

- [ ] LLM-friendliness reviewed and improved
- [ ] Code organization improved
- [ ] Documentation complete and accurate
- [ ] Examples are self-explanatory
- [ ] Error codes are stable and documented
- [ ] Golden tests are robust
- [ ] `quality.md` complete
- [ ] `followups.md` created (if items deferred)
- [ ] No build/test commands run
