# Phase 2 Implementation Prompt

## Role

You are an **implementation agent**. Your job is to implement the stdlib, examples, and golden tests, following the plan exactly and staying within scope.

---

## Permission boundary

- **Allowed**: Read files, edit code within scope globs
- **NOT allowed**: Run build/test commands (validation happens in `30_validate`)

---

## Files to open first

Read these in order:

1. `runpacks/starlark-migration/phases/phase-2-starlark-stdlib-and-examples/plan.md` - Implementation plan
2. `runpacks/starlark-migration/phases/phase-2-starlark-stdlib-and-examples/interfaces.md` - Type definitions
3. `runpacks/starlark-migration/phases/phase-2-starlark-stdlib-and-examples/test_plan.md` - Test requirements
4. `runpacks/starlark-migration/phases/phase-2-starlark-stdlib-and-examples/SCOPING.md` - Scope boundaries
5. `crates/speccade-cli/src/compiler.rs` - Where stdlib registers

---

## Implementation order

Follow the order in `plan.md`. Typical sequence:

### 1. stdlib module structure
- Create `crates/speccade-cli/src/stdlib/mod.rs`
- Create submodules: `audio.rs`, `texture.rs`, `mesh.rs`
- Wire stdlib registration into compiler

### 2. Audio stdlib functions
- Implement functions from `interfaces.md`
- Each function returns IR-compatible struct
- Add doc comments with examples
- Add unit tests

### 3. Texture stdlib functions
- Implement functions from `interfaces.md`
- Ensure deterministic output (seeded RNG if needed)
- Add doc comments with examples
- Add unit tests

### 4. Mesh stdlib functions
- Implement functions from `interfaces.md`
- Add doc comments with examples
- Add unit tests

### 5. Example .star files
- Create `packs/examples/` directory
- Add audio example with comments
- Add texture example with comments
- Add mesh example with comments

### 6. Golden tests
- Create `golden/starlark/` directory
- Add test fixtures (`.star` files)
- Add expected outputs (`.expected.json` files)
- Add test runner in `crates/speccade-tests/`

### 7. CLI diagnostics
- Add error code enum with stable identifiers
- Add `--json` flag for diagnostics
- Update CLI help text

### 8. Documentation
- Create `docs/starlark-authoring.md`
- Create `docs/stdlib-reference.md`
- Create `docs/error-codes.md`

---

## Scope enforcement

Check `SCOPING.md` before editing any file. Allowed globs:
- `crates/speccade-cli/**`
- `crates/speccade-spec/**`
- `crates/speccade-tests/**`
- `crates/**` (new crates)
- `docs/**`
- `golden/**`
- `packs/**`
- `schemas/**`

If you must edit an out-of-scope file:
1. Document justification in `implementation_log.md`
2. Keep the change minimal
3. Record in `ARTIFACTS.md` decision log

---

## stdlib implementation guidelines

### Function signatures
```rust
// Good: explicit parameters, simple types
fn audio_synth(
    waveform: &str,
    frequency: f64,
    duration: f64,
    envelope: AudioEnvelope,
) -> Result<AudioSpec, StdlibError>

// Bad: nested config object
fn audio_synth(config: AudioConfig) -> Result<AudioSpec, StdlibError>
```

### Error handling
```rust
// Use stable error codes
#[derive(Debug, Clone)]
pub enum StdlibError {
    #[error("E0101: field '{field}' is required")]
    MissingField { field: String },

    #[error("E0102: value {value} is out of range [{min}, {max}]")]
    OutOfRange { value: f64, min: f64, max: f64 },
}
```

### Determinism
- All stdlib functions must be pure (no side effects)
- Use seeded RNG from spec seed if randomness needed
- Avoid floating-point operations that vary by platform

---

## Output artifacts

Write these files to this phase folder:

### `implementation_log.md`

Chronological log of changes:

```markdown
## Implementation Log

### Step 1: stdlib module structure
- Created `crates/speccade-cli/src/stdlib/mod.rs`
- Created submodules: audio.rs, texture.rs, mesh.rs
- Registered stdlib with Starlark GlobalsBuilder

### Step 2: Audio stdlib functions
- Implemented audio.synth() with ADSR envelope support
- Implemented audio.tracker_pattern() for pattern sequencing
- [continue for each step...]
```

Include:
- What was done
- Any deviations from plan (with justification)
- Blockers encountered

### `diff_summary.md`

Summary of all files changed:

```markdown
## Files Changed

| File | Change | Reason |
|------|--------|--------|
| `crates/speccade-cli/src/stdlib/mod.rs` | Created | stdlib entry point |
| `crates/speccade-cli/src/stdlib/audio.rs` | Created | Audio constructors |
| `packs/examples/audio_synth_basic.star` | Created | Audio example |
| `golden/starlark/audio_synth_basic.star` | Created | Golden test fixture |
| [continue...] |
```

---

## Quality guidelines

- Follow existing code style in each crate
- Add doc comments for all public APIs
- Use `thiserror` for error types
- Keep stdlib minimal; avoid feature creep
- Examples should be self-documenting
- Error messages should be actionable

---

## Completion criteria

- [ ] All steps in `plan.md` implemented
- [ ] All interfaces from `interfaces.md` exist
- [ ] Tests from `test_plan.md` added (not run yet)
- [ ] Examples are well-commented and self-contained
- [ ] Golden test fixtures have expected outputs
- [ ] Documentation is complete
- [ ] `implementation_log.md` complete
- [ ] `diff_summary.md` complete
- [ ] No out-of-scope files edited without justification
- [ ] No build/test commands run
