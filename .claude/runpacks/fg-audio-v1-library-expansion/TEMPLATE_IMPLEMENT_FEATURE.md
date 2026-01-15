# Template — Implement One Audio Feature (Worker Prompt)

You are a subagent implementing **exactly one** feature from this runpack.

## Inputs you will receive

- The feature prompt file from `speccade/.claude/runpacks/fg-audio-v1-library-expansion/features/`
- A short “touch-points” checklist from `fg-spec-scout`

## Output you must produce

- A small, reviewable change set implementing the feature end-to-end:
  - Spec types/serde + validation (as needed)
  - Backend implementation
  - Schema + docs updated
  - Tests/fixtures updated or added
- A short report: changed files + commands run + any follow-ups

## Process (do not skip)

1. Verify the feature is actually missing (search for existing variants/targets).
2. Implement spec surface first:
   - Update `speccade/crates/speccade-spec/src/recipe/audio/**`
   - Add/update validation in `speccade/crates/speccade-spec/src/validation/recipe_outputs_audio.rs` if new params need range checks
3. Implement backend support:
   - Update `speccade/crates/speccade-backend-audio/src/**`
   - Preserve determinism (stable iteration, fixed RNG seeding)
4. Update schema + docs:
   - `speccade/schemas/speccade-spec-v1.schema.json`
   - `speccade/docs/spec-reference/audio.md`
   - `speccade/docs/audio_synthesis_methods.md` (synthesis list/status)
5. Add/adjust tests:
   - Unit tests close to the code
   - Add example/golden specs if they improve confidence
6. Quality gates:
   - No file > 600 LoC; refactor into modules if needed
   - No copy/paste: extract helpers
   - No stubs/TODOs in new code

## Minimal command loop (run and fix)

- `cargo fmt`
- `cargo clippy -p speccade-spec -p speccade-backend-audio -p speccade-cli -p speccade-tests --all-targets -- -D warnings`
- `cargo test -p speccade-spec -p speccade-backend-audio`
- If fixtures/golden/preset library changed:
  - `cargo test -p speccade-tests`
  - `python validate_all.py`
