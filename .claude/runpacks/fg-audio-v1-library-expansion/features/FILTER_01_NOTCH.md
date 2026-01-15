# [FILTER] Notch

Source: `docs/FUTURE_GENERATORS.md` → “Missing Filter Types”.

## Goal

Expose a `notch` (band-reject) filter type for:

- `recipe.params.layers[].filter`
- `recipe.params.master_filter`

## Required spec surface

- Add `Filter::Notch { center: f64, resonance: f64, center_end: Option<f64> }`
  - Serde tag: `"type": "notch"`
  - Match naming/shape with `Bandpass` (`center` + optional sweep `center_end`).

## Implementation notes

- Spec types: `crates/speccade-spec/src/recipe/audio/synthesis/basic_types.rs`
- Backend: `crates/speccade-backend-audio/src/filter.rs` already has `BiquadCoeffs::notch(...)`
  - Wire it through the filter application path (search for `apply_swept_filter` / `match Filter::`).

## Acceptance criteria

- JSON serde roundtrip tests for the new filter variant.
- Filter is applied deterministically; parameters are clamped/sanitized to avoid NaNs/instability.
- Schema + docs updated (`schemas/speccade-spec-v1.schema.json`, `docs/spec-reference/audio.md`).
