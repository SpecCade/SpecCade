# Phase 2 Planning Prompt

## Role

You are a **planning agent**. Your job is to produce an implementable plan for the stdlib, examples, and golden tests based on research findings.

---

## Permission boundary

- **Allowed**: Read files, analyze research, design APIs
- **NOT allowed**: Apply patches, edit code, run commands

---

## Files to open first

Read these in order:

1. `runpacks/starlark-migration/phases/phase-2-starlark-stdlib-and-examples/research.md` - Research findings
2. `runpacks/starlark-migration/phases/phase-2-starlark-stdlib-and-examples/risks.md` - Identified risks
3. `runpacks/starlark-migration/phases/phase-2-starlark-stdlib-and-examples/SCOPING.md` - Scope boundaries
4. `runpacks/starlark-migration/ARCHITECTURE_PROPOSAL.md` - Migration architecture
5. `crates/speccade-cli/src/compiler.rs` - Where stdlib hooks in
6. `crates/speccade-spec/src/lib.rs` - Spec types to emit

---

## Planning requirements

### stdlib API design

Design stdlib functions that:
- Use simple, flat function signatures (no nested config objects)
- Have explicit parameters (avoid magic defaults)
- Return IR-compatible structures
- Include helpful error messages with stable codes

Organize by asset domain:
- `audio.*` - Audio constructors
- `texture.*` - Texture constructors
- `mesh.*` - Mesh constructors

### Examples to create

Plan at least one example for each domain:
- Audio: synth, tracker song, or sample-based
- Texture: noise (Perlin, Worley), gradient, or pattern
- Mesh: primitive (cube, sphere) or composed shape

Each example should be:
- Self-contained (no imports)
- Well-commented for LLM understanding
- Representative of common use cases

### Golden test strategy

Plan golden tests that:
- Cover each stdlib function
- Assert byte-identical canonical IR
- Use RFC 8785 JCS for JSON canonicalization
- Fail clearly when output differs

### CLI diagnostics

Plan error codes and JSON output:
- `E0001` through `E0099`: Parsing errors
- `E0100` through `E0199`: Validation errors
- `E0200` through `E0299`: Generation errors
- `--json` flag for machine-parseable output

---

## Output artifacts

Write these files to this phase folder:

### `plan.md`

Ordered implementation plan:

```markdown
## Implementation Plan

### Step 1: stdlib module structure
- Create `crates/speccade-cli/src/stdlib/mod.rs`
- Add submodules: audio.rs, texture.rs, mesh.rs
- Register stdlib with Starlark evaluator

### Step 2: Audio stdlib functions
- `audio.synth(...)` -> SynthSpec
- `audio.tracker_song(...)` -> TrackerSongSpec
- [list all functions with signatures]

### Step 3: Texture stdlib functions
[continue...]

### Step 4: Mesh stdlib functions
[continue...]

### Step 5: Example .star files
- Create packs/examples/ directory structure
- Add audio example
- Add texture example
- Add mesh example

### Step 6: Golden tests
- Create golden/starlark/ directory
- Add test fixtures and expected outputs
- Add test runner in speccade-tests

### Step 7: CLI diagnostics
- Add error code enum
- Add --json output option
- Update help text

### Step 8: Documentation
- docs/starlark-authoring.md
- docs/stdlib-reference.md
- docs/error-codes.md
```

### `interfaces.md`

Define all new types and function signatures:

```markdown
## stdlib Functions

### audio module

#### audio.synth
```starlark
audio.synth(
    waveform: str,      # "sine" | "square" | "saw" | "triangle"
    frequency: float,   # Hz
    duration: float,    # seconds
    envelope: dict,     # {attack, decay, sustain, release}
) -> AudioSpec
```

[continue for all functions...]

## Error Codes

| Code | Category | Message Template |
|------|----------|------------------|
| E0001 | Parse | Invalid Starlark syntax at {line}:{col} |
| E0101 | Validate | Field '{field}' is required |
[continue...]

## CLI Flags

| Flag | Description |
|------|-------------|
| `--json` | Output diagnostics as JSON |
| `--error-format` | Error format: human (default), json, compact |
```

### `test_plan.md`

Test coverage requirements:

```markdown
## Test Plan

### Unit tests (in stdlib modules)
- Each stdlib function has at least one unit test
- Test valid inputs produce correct IR
- Test invalid inputs produce correct error codes

### Integration tests
- .star file round-trip through compiler
- Verify canonical IR matches expected

### Golden tests
- `golden/starlark/audio_synth_basic.star` -> .expected.json
- `golden/starlark/texture_noise_perlin.star` -> .expected.json
- `golden/starlark/mesh_primitive_cube.star` -> .expected.json
- Tests assert byte-identity using BLAKE3 hash

### Error message tests
- Verify error codes are stable
- Verify --json output is valid JSON
- Verify human-readable format is helpful
```

---

## Completion criteria

- [ ] All stdlib functions designed with signatures
- [ ] Examples planned for each domain
- [ ] Golden test strategy defined
- [ ] Error codes and CLI flags specified
- [ ] Implementation order is logical and dependency-aware
- [ ] `plan.md` complete
- [ ] `interfaces.md` complete
- [ ] `test_plan.md` complete
- [ ] No code edits made
- [ ] No commands run
