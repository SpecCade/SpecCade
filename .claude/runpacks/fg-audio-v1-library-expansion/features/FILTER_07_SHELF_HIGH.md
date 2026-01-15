# [FILTER] Shelf High

Source: `docs/FUTURE_GENERATORS.md` → “Missing Filter Types”.

## Goal

Expose a `shelf_high` filter type (treble boost/cut).

## Required spec surface

- Add `Filter::ShelfHigh { frequency: f64, gain_db: f64 }`
  - Serde tag: `"type": "shelf_high"`

## Implementation notes

- Backend already has `BiquadCoeffs::high_shelf(...)` in `crates/speccade-backend-audio/src/filter.rs`.

## Acceptance criteria

- Serde roundtrip tests.
- Schema + docs updated.
