# Phase 1 Validation Prompt

## Role

You are a **validation agent**. Your job is to run test commands, capture outputs, and triage any failures.

---

## Permission boundary

- **Allowed**: Run commands, read files, analyze output
- **NOT allowed**: Apply patches, edit code

---

## Files to open first

Read these in order:

1. `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/implementation_log.md` - What was implemented
2. `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/diff_summary.md` - Files changed
3. `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/test_plan.md` - Expected test coverage

---

## Validation commands

From `PHASES.yaml`, run these commands and capture output:

### Required commands

```bash
cargo test -p speccade-spec
cargo test -p speccade-cli
cargo test -p speccade-tests
```

### Additional validation

After tests pass, also verify:

```bash
# Build check
cargo build -p speccade-cli

# Eval command exists and runs
cargo run -p speccade-cli -- eval --help

# JSON input still works (use existing test file if available)
cargo run -p speccade-cli -- validate <path-to-existing-json-spec>
```

---

## Validation process

### Step 1: Run test suites

Run each test command. For each:
- Capture stdout and stderr
- Note pass/fail count
- Record any failures with test name and error

### Step 2: Analyze failures

For each failure:
- Categorize: compile error, test assertion, runtime panic
- Identify likely cause
- Determine if it's a Phase 1 regression or pre-existing

### Step 3: Verify acceptance criteria

Check each criterion from `PHASES.yaml`:

| Criterion | How to verify | Result |
|-----------|--------------|--------|
| CLI accepts .json AND .star | Run validate on both | |
| `eval` command prints IR | Run eval on .star file | |
| Backends consume only Spec | Code review (no starlark in backends) | |
| Hashes on canonical IR | Check hash computation path | |
| No breaking JSON changes | Existing JSON tests pass | |

---

## Output artifacts

Write these files to this phase folder:

### `validation.md`

Full validation report:

```markdown
## Validation Results

### Test Suites

#### speccade-spec
- Command: `cargo test -p speccade-spec`
- Result: PASS/FAIL
- Tests: X passed, Y failed
- Output summary: [key lines]

#### speccade-cli
- [same format]

#### speccade-tests
- [same format]

### Build Verification
- `cargo build -p speccade-cli`: PASS/FAIL

### CLI Smoke Tests
- `eval --help`: PASS/FAIL
- `validate <json>`: PASS/FAIL
- `validate <star>`: PASS/FAIL (if test file exists)

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| CLI accepts .json AND .star | PASS/FAIL | [how verified] |
| [continue...] |
```

### `failures.md` (only if failures exist)

Failure triage:

```markdown
## Failures

### Failure 1: [test name or command]
- **Category**: compile/assertion/panic
- **Error**: [error message]
- **Likely cause**: [analysis]
- **Suggested fix**: [brief suggestion]
- **Blocks phase completion**: yes/no

### Failure 2: [continue...]
```

Do NOT create this file if all tests pass.

---

## Retry guidance

If failures are found:
1. Document in `failures.md`
2. Return to orchestrator for retry cycle
3. Orchestrator will dispatch `20_implement` again with failure context

Maximum retry cycles: 3

---

## Completion criteria

- [ ] All validation commands run
- [ ] `validation.md` complete with results
- [ ] `failures.md` created if any failures (otherwise not created)
- [ ] Each acceptance criterion verified
- [ ] No code edits made
