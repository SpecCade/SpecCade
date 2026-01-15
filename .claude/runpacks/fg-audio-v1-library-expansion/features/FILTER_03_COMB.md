# [FILTER] Comb

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Filter Types”.

## Goal

Expose a `comb` filter type for resonant/metallic coloration.

## Suggested spec surface (MVP)

- Add `Filter::Comb { delay_ms: f64, feedback: f64, wet: f64 }`
  - Serde tag: `"type": "comb"`
  - Keep it simple: feedback comb with wet/dry mix.

If you need a stable/safer surface, consider adding `damping` and/or a `mode` (feedforward vs feedback), but keep MVP minimal.

## Implementation notes

- Spec types: `speccade/crates/speccade-spec/src/recipe/audio/synthesis/basic_types.rs`
- Backend:
  - Add a small deterministic comb filter implementation (delay line) in `speccade/crates/speccade-backend-audio/src/filter.rs` or a new `filter/comb.rs`.
  - Watch file length: `speccade-backend-audio/src/filter.rs` is already close to 600 LoC — refactor into modules if needed.

## Acceptance criteria

- Serde roundtrip tests.
- Comb filter is stable (feedback clamped < 1.0), deterministic, no NaNs.
- Schema + docs updated.

