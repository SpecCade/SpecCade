# [FILTER] Shelf Low

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Filter Types”.

## Goal

Expose a `shelf_low` filter type (bass boost/cut).

## Suggested spec surface

- Add `Filter::ShelfLow { frequency: f64, gain_db: f64 }`
  - Serde tag: `"type": "shelf_low"`

## Implementation notes

- Backend already has `BiquadCoeffs::low_shelf(...)` in `speccade/crates/speccade-backend-audio/src/filter.rs`.
- Wire through the filter application path.

## Acceptance criteria

- Serde roundtrip tests.
- Stable behavior and sensible gain bounds in validation (if added).
- Schema + docs updated.

