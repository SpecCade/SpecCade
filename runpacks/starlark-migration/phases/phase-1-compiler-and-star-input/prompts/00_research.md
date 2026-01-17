# Phase 1 Research Prompt

## Role

You are a **research agent**. Your job is to understand the existing codebase structure relevant to this phase, identify integration points, and surface risks.

---

## Permission boundary

- **Allowed**: Read files, search code, analyze structure
- **NOT allowed**: Apply patches, edit code, run commands

---

## Files to open first

Read these in order:

1. `runpacks/starlark-migration/ARCHITECTURE_PROPOSAL.md` - Migration architecture
2. `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/SCOPING.md` - Scope boundaries
3. `crates/speccade-cli/src/main.rs` - CLI entry point
4. `crates/speccade-cli/src/lib.rs` - CLI library (if exists)
5. `crates/speccade-cli/Cargo.toml` - CLI dependencies
6. `crates/speccade-spec/src/lib.rs` - Spec crate entry
7. `crates/speccade-spec/src/validation.rs` - Existing validation
8. `crates/speccade-spec/src/hash.rs` - Hashing implementation
9. `Cargo.toml` - Workspace dependencies

---

## Research questions to answer

### CLI structure
- How does the CLI currently dispatch commands?
- Where does JSON spec loading happen?
- How are validate/generate commands structured?

### Spec crate
- What is the `Spec` struct shape?
- How is validation invoked?
- How is hashing (ir_hash) computed?

### Integration points
- Where should Starlark parsing hook in?
- Can input dispatch be abstracted (JSON vs Starlark)?
- Are there existing "expansion" passes (e.g., tracker_song_compose_v1)?

### Dependencies
- What Starlark crate should be used? (e.g., `starlark` crate on crates.io)
- What safety limits does it support (memory, time)?

---

## Output artifacts

Write these files to this phase folder:

### `research.md`
Document your findings for each research question. Include:
- File paths and relevant code snippets
- Entry points for CLI commands
- Spec struct location and key fields
- Validation flow
- Hashing flow
- Recommended Starlark crate and version

### `risks.md`
List identified risks with severity and mitigation:
- Starlark safety (infinite loops, memory)
- Determinism concerns
- Breaking change risks
- Dependency concerns

### `questions.md` (only if blocking)
If you cannot proceed without answers, list blocking questions here.
Do NOT create this file if there are no blocking questions.

---

## Completion criteria

- [ ] All research questions answered in `research.md`
- [ ] Risks documented in `risks.md`
- [ ] No code edits made
- [ ] No commands run
