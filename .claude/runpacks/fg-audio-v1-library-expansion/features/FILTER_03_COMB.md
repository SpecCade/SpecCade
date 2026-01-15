# [FILTER] Comb

Source: `docs/FUTURE_GENERATORS.md` → “Missing Filter Types”.

## Goal

Expose a `comb` filter type for resonant/metallic coloration.

## Required spec surface

- Add `Filter::Comb { delay_ms: f64, feedback: f64, wet: f64 }`
  - Serde tag: `"type": "comb"`
  - Keep it simple: feedback comb with wet/dry mix (no extra parameters in this runpack).

## Implementation notes

- Spec types: `crates/speccade-spec/src/recipe/audio/synthesis/basic_types.rs`
- Backend:
  - Implement the comb filter in `crates/speccade-backend-audio/src/filter.rs` as a small deterministic delay-line + feedback filter.
  - Watch file length: `crates/speccade-backend-audio/src/filter.rs` is already close to 600 LoC — refactor into modules if needed.

## Acceptance criteria

- Serde roundtrip tests.
- Comb filter is stable (feedback clamped < 1.0), deterministic, no NaNs.
- Schema + docs updated.
