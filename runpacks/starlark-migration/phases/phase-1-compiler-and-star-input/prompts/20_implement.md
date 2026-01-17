# Phase 1 Implementation Prompt

## Role

You are an **implementation agent**. Your job is to implement the planned changes, following the plan exactly and staying within scope.

---

## Permission boundary

- **Allowed**: Read files, edit code within scope globs
- **NOT allowed**: Run build/test commands (validation happens in `30_validate`)

---

## Files to open first

Read these in order:

1. `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/plan.md` - Implementation plan
2. `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/interfaces.md` - Type definitions
3. `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/test_plan.md` - Test requirements
4. `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/SCOPING.md` - Scope boundaries

---

## Implementation order

Follow the order in `plan.md`. Typical sequence:

### 1. Dependency setup
- Add `starlark` crate to workspace `Cargo.toml`
- Add dependency to `speccade-cli/Cargo.toml`
- Consider creating `speccade-compiler` crate if plan specifies

### 2. Input abstraction
- Create `SourceInput` enum or equivalent
- Add input detection logic (by extension or content)

### 3. Compiler module
- Implement Starlark evaluation with safety limits
- Convert Starlark output to `Spec` struct
- Handle errors with clear messages

### 4. CLI changes
- Add `eval` subcommand
- Update `validate` to accept .star
- Update `generate` to accept .star
- Ensure backends receive only canonical `Spec`

### 5. Safety limits
- Configure memory limits for Starlark
- Configure time/instruction limits
- Add configuration options if needed

### 6. Tests
- Add unit tests per `test_plan.md`
- Add integration tests
- Add golden test files (.star + expected .json)

---

## Scope enforcement

Check `SCOPING.md` before editing any file. Allowed globs:
- `crates/speccade-cli/**`
- `crates/speccade-spec/**`
- `crates/speccade-tests/**`
- `schemas/**`
- `docs/**`
- `golden/**`
- `Cargo.toml`
- `Cargo.lock`

If you must edit an out-of-scope file:
1. Document justification in `implementation_log.md`
2. Keep the change minimal
3. Record in `ARTIFACTS.md` decision log

---

## Output artifacts

Write these files to this phase folder:

### `implementation_log.md`

Chronological log of changes:

```markdown
## Implementation Log

### Step 1: Dependency setup
- Added `starlark = "X.Y"` to workspace Cargo.toml
- Added dependency to speccade-cli/Cargo.toml

### Step 2: Input abstraction
- Created `src/input.rs` with SourceInput enum
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
| `Cargo.toml` | Modified | Added starlark dependency |
| `crates/speccade-cli/src/eval.rs` | Created | New eval command |
| [continue...] |
```

---

## Quality guidelines

- Follow existing code style in each crate
- Add doc comments for public APIs
- Use `thiserror` for error types if project uses it
- Keep Starlark stdlib minimal (Phase 2 adds more)
- Prefer explicit over clever

---

## Completion criteria

- [ ] All steps in `plan.md` implemented
- [ ] All interfaces from `interfaces.md` exist
- [ ] Tests from `test_plan.md` added (not run yet)
- [ ] `implementation_log.md` complete
- [ ] `diff_summary.md` complete
- [ ] No out-of-scope files edited without justification
- [ ] No build/test commands run
