# [EFFECT P1] Parametric EQ

Source: `docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 1)”.

## Goal

Add `parametric_eq` for tonal shaping.

## Required spec surface

- Add `Effect::ParametricEq { bands: Vec<EqBand> }`
- Add `EqBand { frequency: f64, gain_db: f64, q: f64, band_type: EqBandType }`
- Add `EqBandType`: `lowshelf`, `highshelf`, `peak`, `notch`

## Implementation notes

- Spec: `crates/speccade-spec/src/recipe/audio/effects.rs`
- Backend:
  - Implement as cascaded deterministic biquads.
  - Reuse coefficient helpers in `crates/speccade-backend-audio/src/filter.rs` where possible (peaking, shelves, notch).
  - Keep band order stable (apply in listed order).

## Acceptance criteria

- Bands serialize/deserialize; schema/docs updated.
- EQ is stable and doesn’t produce NaNs for sane parameter ranges.
- Tests cover at least one band of each type.
