# Phase 2 Research Prompt

## Role

You are a **research agent**. Your job is to understand the Phase 1 compiler implementation, existing asset types, and LLM-friendly API patterns to inform stdlib design.

---

## Permission boundary

- **Allowed**: Read files, search code, analyze structure
- **NOT allowed**: Apply patches, edit code, run commands

---

## Files to open first

Read these in order:

1. `runpacks/starlark-migration/ARCHITECTURE_PROPOSAL.md` - Migration architecture
2. `runpacks/starlark-migration/phases/phase-2-starlark-stdlib-and-examples/SCOPING.md` - Scope boundaries
3. `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/followups.md` - Deferred items from Phase 1
4. `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/implementation_log.md` - Phase 1 implementation details
5. `crates/speccade-cli/src/compiler.rs` - Phase 1 compiler module
6. `crates/speccade-spec/src/lib.rs` - Spec types
7. `crates/speccade-spec/src/audio.rs` - Audio spec types (if exists)
8. `crates/speccade-spec/src/texture.rs` - Texture spec types (if exists)
9. `crates/speccade-spec/src/mesh.rs` - Mesh spec types (if exists)
10. `schemas/` - JSON schemas for IR types

---

## Research questions to answer

### Phase 1 compiler state
- How does the Phase 1 compiler evaluate Starlark?
- How is stdlib currently registered (if at all)?
- What safety limits are in place?
- What is the entry point for adding new builtins?

### Asset type coverage
- What audio IR types exist? (synth, tracker, samples, etc.)
- What texture IR types exist? (noise, gradient, bitmap, etc.)
- What mesh IR types exist? (primitives, transforms, etc.)
- Which types are most commonly used in existing JSON specs?

### Existing examples
- Are there existing `.json` spec examples in `packs/` or `golden/`?
- What patterns do they follow?
- Which could serve as templates for `.star` examples?

### LLM-friendly API patterns
- What makes an API "LLM-friendly"? (simple signatures, explicit params, good errors)
- How do other Starlark projects structure their stdlib? (Bazel, Buck2)
- What error message format aids LLM debugging?

### Golden test strategy
- How do existing golden tests work in `speccade-tests`?
- What is the expected golden file format?
- How is byte-identity verified?

---

## Output artifacts

Write these files to this phase folder:

### `research.md`
Document your findings for each research question. Include:
- Phase 1 compiler entry points for stdlib
- Complete list of audio/texture/mesh IR types
- Existing example patterns
- Recommended stdlib function signatures
- Error code format recommendation
- Golden test implementation strategy

### `risks.md`
List identified risks with severity and mitigation:
- **Determinism**: stdlib functions must be pure
- **API bloat**: keep stdlib minimal, avoid feature creep
- **Naming conflicts**: stdlib names vs Starlark builtins
- **Error verbosity**: balance LLM-friendliness with brevity
- **Golden test brittleness**: formatting changes break tests

### `questions.md` (only if blocking)
If you cannot proceed without answers, list blocking questions here.
Do NOT create this file if there are no blocking questions.

---

## Completion criteria

- [ ] Phase 1 compiler structure understood
- [ ] All asset IR types catalogued
- [ ] LLM-friendly patterns documented
- [ ] Golden test strategy understood
- [ ] All findings in `research.md`
- [ ] Risks documented in `risks.md`
- [ ] No code edits made
- [ ] No commands run
