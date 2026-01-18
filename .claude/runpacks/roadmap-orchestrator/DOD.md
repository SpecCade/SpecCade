# Definition of Done (DoD) - Roadmap Tasks

This DoD is referenced by `.claude/runpacks/roadmap-orchestrator/ORCHESTRATOR.md`.

## Always

- The task's deliverable is implemented end-to-end (not partially wired).
- No new placeholders: no `TODO`, `todo!()`, `unimplemented!()`, "stub", or commented-out dead code.
- Determinism guardrails preserved (no wall-clock time, OS RNG, unstable iteration ordering, etc.).
- No file should end up > 600 LoC (split into modules if needed).
- `docs/ROADMAP.md` is updated only after verification passes:
  - checkbox flipped to `[x]`
  - add a short "Done: YYYY-MM-DD (commit <sha>)" note on the same item

## If Rust code changed

- `cargo fmt` passes.
- `cargo clippy` passes for the touched crates with `-D warnings`.
- `cargo test` passes for the touched crates.
- If golden fixtures / integration corpus changed: `cargo test -p speccade-tests` passes.

## If the spec/IR changed

- Update `crates/speccade-spec` types + validation.
- Update `schemas/speccade-spec-v1.schema.json` (and any other affected schema outputs).
- Update the canonical docs under `docs/spec-reference/`.
- Add/adjust tests:
  - serde roundtrip / validation unit tests
  - integration tests / golden specs when they meaningfully improve confidence

## If the CLI UX changed

- Help/flags documented where needed (`docs/README.md`, relevant spec docs).
- Machine-readable outputs are stable if the task adds them (schema/versioning, stable codes).

## Decision tasks (e.g. "*-Q###" or "Decide ...")

- The decision is written down in `docs/ROADMAP.md` under the task with:
  - the chosen option
  - 1-3 key reasons
  - any follow-up tasks created/updated (IDs)
- If user confirmation is required, the task is not checked off until confirmed.

