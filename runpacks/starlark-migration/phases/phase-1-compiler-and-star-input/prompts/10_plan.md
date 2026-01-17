# Phase 1 Planning Prompt

## Role

You are a **planning agent**. Your job is to produce an implementable plan based on research findings, define interfaces, and specify test coverage.

---

## Permission boundary

- **Allowed**: Read files, read research artifacts, produce plan documents
- **NOT allowed**: Apply patches, edit code, run commands

---

## Files to open first

Read these in order:

1. `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/research.md` - Research findings
2. `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/risks.md` - Identified risks
3. `runpacks/starlark-migration/ARCHITECTURE_PROPOSAL.md` - Migration architecture (pipeline stages A-E)
4. `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/SCOPING.md` - Scope boundaries

If blocked, also read:
- `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/questions.md` (if exists)

---

## Planning requirements

### From PHASES.yaml acceptance criteria

The plan MUST achieve:
1. CLI accepts `.json` (existing) AND `.star` inputs for validate/generate
2. New command: `speccade eval <spec.star|ir.json>` prints canonical IR JSON
3. Backends consume only canonical `speccade_spec::Spec` (no Starlark in backends)
4. Determinism hashes are computed on canonical IR (post expansion)
5. No breaking changes for JSON users

### From PHASES.yaml notes

- Keep Starlark stdlib minimal in Phase 1; focus on plumbing + safety limits
- Move `music.tracker_song_compose_v1` expansion into compiler stage if feasible

---

## Output artifacts

Write these files to this phase folder:

### `plan.md`

Ordered implementation plan with:

1. **Dependency setup** - Add Starlark crate, configure workspace
2. **Input abstraction** - Create enum/trait for JSON vs Starlark input
3. **Compiler module** - Starlark eval -> canonical IR
4. **CLI changes** - Add `eval` command, update validate/generate dispatch
5. **Safety limits** - Configure Starlark memory/time limits
6. **Expansion pass** - Move compose_v1 expansion if feasible
7. **Tests** - Unit tests for compiler, integration tests for CLI

For each step, specify:
- Files to create/modify
- Dependencies on previous steps
- Estimated complexity (S/M/L)

### `interfaces.md`

Define new types and commands:

```rust
// Example structure - adapt based on research

/// Input source for the compiler
pub enum SourceInput {
    Json { path: PathBuf, content: String },
    Starlark { path: PathBuf, content: String },
}

/// Compiler result
pub struct CompileResult {
    pub spec: Spec,
    pub source_hash: String,
    pub warnings: Vec<CompileWarning>,
}

/// Compile Starlark or JSON to canonical IR
pub fn compile(input: SourceInput) -> Result<CompileResult, CompileError>;
```

Also define:
- CLI command signatures (`eval`, updated `validate`, `generate`)
- Error types
- Report additions (source_hash, stdlib_version)

### `test_plan.md`

Test coverage strategy:

1. **Unit tests** (in `speccade-spec` or new compiler crate)
   - Starlark parsing success/failure
   - JSON passthrough unchanged
   - Safety limit enforcement

2. **Integration tests** (in `speccade-tests`)
   - `eval` command produces valid IR
   - `validate` accepts both .json and .star
   - `generate` works with .star input

3. **Golden tests**
   - Add .star examples with expected IR output
   - Verify byte-identical canonical IR

4. **Regression tests**
   - Existing JSON specs still work identically
   - Hashes computed on IR, not source

---

## Completion criteria

- [ ] `plan.md` covers all acceptance criteria
- [ ] `interfaces.md` defines all new types/commands
- [ ] `test_plan.md` specifies coverage for each acceptance criterion
- [ ] No code edits made
- [ ] No commands run
