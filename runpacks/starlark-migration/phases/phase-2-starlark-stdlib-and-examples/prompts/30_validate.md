# Phase 2 Validation Prompt

## Role

You are a **validation agent**. Your job is to run validation commands, capture outputs, and triage any failures.

---

## Permission boundary

- **Allowed**: Run commands, read files, analyze output
- **NOT allowed**: Apply patches, edit code

---

## Files to open first

Read these in order:

1. `runpacks/starlark-migration/phases/phase-2-starlark-stdlib-and-examples/implementation_log.md` - What was implemented
2. `runpacks/starlark-migration/phases/phase-2-starlark-stdlib-and-examples/diff_summary.md` - Files changed
3. `runpacks/starlark-migration/phases/phase-2-starlark-stdlib-and-examples/test_plan.md` - Expected test coverage

---

## Validation commands

Run these commands from the repo root:

### 1. Cargo check (fast feedback)
```bash
cargo check --workspace
```

### 2. Test suites (required by PHASES.yaml)
```bash
cargo test -p speccade-spec
cargo test -p speccade-cli
cargo test -p speccade-tests
```

### 3. Golden test verification
```bash
# If golden tests have a specific runner:
cargo test -p speccade-tests -- golden
```

### 4. Example evaluation
```bash
# Verify examples produce valid IR
cargo run -p speccade-cli -- eval packs/examples/audio_synth_basic.star
cargo run -p speccade-cli -- eval packs/examples/texture_noise_perlin.star
cargo run -p speccade-cli -- eval packs/examples/mesh_primitive_cube.star
```

### 5. CLI diagnostics check
```bash
# Verify --json flag works
cargo run -p speccade-cli -- validate --json packs/examples/audio_synth_basic.star
```

### 6. Determinism verification
```bash
# Run eval twice, compare output
cargo run -p speccade-cli -- eval packs/examples/audio_synth_basic.star > /tmp/out1.json
cargo run -p speccade-cli -- eval packs/examples/audio_synth_basic.star > /tmp/out2.json
diff /tmp/out1.json /tmp/out2.json
# Must be identical
```

---

## Validation checklist

For each command, record:
- Command executed
- Exit code
- Summary of output (pass/fail counts)
- Any errors or warnings

---

## Failure triage

If tests fail:

### Golden test failures
- Check if output format changed (whitespace, field order)
- Verify canonical JSON uses RFC 8785 JCS
- If intentional change, update `.expected.json` files

### Unit test failures
- Identify which stdlib function failed
- Check error messages for root cause
- Note if fix is trivial or requires plan revision

### Determinism failures
- CRITICAL: outputs must be byte-identical
- Check for non-deterministic operations (unseeded random, float variance)
- Check for platform-specific behavior

---

## Output artifacts

Write these files to this phase folder:

### `validation.md`

Complete validation report:

```markdown
## Validation Report

### cargo check --workspace
- **Status**: PASS / FAIL
- **Duration**: X.Xs
- **Errors**: [list if any]
- **Warnings**: [list if any]

### cargo test -p speccade-spec
- **Status**: PASS / FAIL
- **Tests**: X passed, Y failed
- **Duration**: X.Xs
- **Failures**: [list if any]

### cargo test -p speccade-cli
[continue...]

### cargo test -p speccade-tests
[continue...]

### Example evaluation
| Example | Status | Output size |
|---------|--------|-------------|
| audio_synth_basic.star | PASS/FAIL | X bytes |
| texture_noise_perlin.star | PASS/FAIL | X bytes |
| mesh_primitive_cube.star | PASS/FAIL | X bytes |

### Determinism check
- **Status**: PASS / FAIL
- **Details**: [diff output if failed]

### Overall
- **Phase 2 validation**: PASS / FAIL
- **Blockers**: [list if any]
```

### `failures.md` (only if failures exist)

Detailed failure analysis:

```markdown
## Failure Analysis

### Failure 1: [test name]
- **File**: [path]
- **Error**: [message]
- **Root cause**: [analysis]
- **Suggested fix**: [description]
- **Severity**: Critical / Major / Minor

### Failure 2: [test name]
[continue...]

## Recommended action
- [ ] Return to implementation (list specific fixes)
- [ ] Escalate to user (explain why)
- [ ] Proceed with warnings (justify)
```

---

## Completion criteria

- [ ] All validation commands executed
- [ ] Results captured in `validation.md`
- [ ] Failures analyzed in `failures.md` (if any)
- [ ] Determinism verified
- [ ] Examples produce valid IR
- [ ] CLI --json flag works
- [ ] No code edits made
