# [FILTER] Shelf High

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Filter Types”.

## Goal

Expose a `shelf_high` filter type (treble boost/cut).

## Suggested spec surface

- Add `Filter::ShelfHigh { frequency: f64, gain_db: f64 }`
  - Serde tag: `"type": "shelf_high"`

## Implementation notes

- Backend already has `BiquadCoeffs::high_shelf(...)` in `speccade/crates/speccade-backend-audio/src/filter.rs`.

## Acceptance criteria

- Serde roundtrip tests.
- Schema + docs updated.

