# RUNPACK Orchestrator — FG Audio V1 Library Expansion

You are the **orchestrator**. Keep your own context small and stable: delegate almost all work to subagents and only track state/progress.

## Objective

Implement **all** audio feature gaps enumerated in `speccade/docs/FUTURE_GENERATORS.md`:

1. Priority 1–3 missing **synthesis types**
2. Priority 1–3 missing **effect types**
3. Missing **LFO targets**
4. Missing **filter types**

Use the checklist in `speccade/.claude/runpacks/fg-audio-v1-library-expansion/FEATURE_INDEX.md` as the source of truth.

## Subagents (required)

Use these agents heavily; do not “solo” large implementations:

- `fg-spec-scout` — find touch-points, propose minimal spec surface + naming
- `fg-audio-implementer` — implement the Rust changes (spec + backend) for one feature
- `fg-schema-docs` — update JSON schema + docs for one feature
- `fg-tests` — add/update unit + integration coverage (golden/spec fixtures if appropriate)
- `fg-qa` — run build/test loops and enforce quality rules

## Global constraints (non-negotiable)

- Determinism: no wall-clock, OS RNG, unstable iteration ordering, or thread timing dependency.
- Code quality:
  - No file should end up **> 600 LoC**. If a file would exceed this, refactor into modules.
  - Avoid DRY violations: extract helpers instead of copy/paste.
  - Keep changes scoped to the audio surface area (`speccade-spec`, `speccade-backend-audio`, docs/schema/tests).
- No placeholder/stub implementations (`todo!()`, `unimplemented!()`, “TODO: implement” in new code).

## Recommended execution order

Follow `FEATURE_INDEX.md` order. If you need a dependency tweak:

- Filters → LFO targets → Effects → Synthesis types

## Per-feature loop (repeat for each unchecked item)

For each feature prompt in `features/`:

1. **Read the feature file** (and only the minimum adjacent docs/code needed).
2. Ask `fg-spec-scout` for:
   - Where to change spec types/serde tags
   - Validation expectations (ranges, invariants)
   - Backend touch points
   - Docs/schema/test files to update
   Keep the response as a short checklist with file paths.
3. Ask `fg-audio-implementer` to implement the feature (use `TEMPLATE_IMPLEMENT_FEATURE.md` + the feature file + scout checklist).
4. Ask `fg-schema-docs` to update:
   - `speccade/schemas/speccade-spec-v1.schema.json`
   - `speccade/docs/spec-reference/audio.md`
   - `speccade/docs/audio_synthesis_methods.md` (if synthesis types changed)
5. Ask `fg-tests` to add/adjust coverage:
   - Unit tests in `speccade-spec` / `speccade-backend-audio`
   - New/updated example specs (prefer `speccade/golden/speccade/specs/audio/` when it helps)
6. Ask `fg-qa` to run and fix (minimum viable loop per feature):
   - `cargo fmt`
   - `cargo clippy -p speccade-spec -p speccade-backend-audio -p speccade-cli -p speccade-tests --all-targets -- -D warnings`
   - `cargo test -p speccade-spec -p speccade-backend-audio`
   - `cargo test -p speccade-tests` (when fixtures/golden are touched)
   - `python validate_all.py` (when preset library compatibility is relevant)
7. Mark the feature done by checking it off in `FEATURE_INDEX.md` and add a 1–2 line note if anything subtle happened.

If a feature is too large to finish safely in one go, **split it** into smaller “MVP then polish” steps inside that feature prompt, but still keep forward progress and keep the checklist honest.
