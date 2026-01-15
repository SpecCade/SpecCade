# [FILTER] Allpass

Source: `docs/FUTURE_GENERATORS.md` → “Missing Filter Types”.

## Goal

Expose an `allpass` filter type for layer/master filters (phase shaping; phaser building block).

## Required spec surface

- Add `Filter::Allpass { frequency: f64, resonance: f64, frequency_end: Option<f64> }`
  - Serde tag: `"type": "allpass"`
  - Use `frequency` naming (not `cutoff`) to avoid implying magnitude shaping.
  - Optional `frequency_end` allows swept phasing.

## Implementation notes

- Spec types: `crates/speccade-spec/src/recipe/audio/synthesis/basic_types.rs`
- Backend: `crates/speccade-backend-audio/src/filter.rs` already has `BiquadCoeffs::allpass(...)`

## Acceptance criteria

- Serde roundtrip tests.
- `apply_swept_filter` supports allpass with optional sweep.
- Schema + docs updated.
