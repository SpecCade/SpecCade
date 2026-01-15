# [EFFECT P2] Tape saturation

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 2)”.

## Goal

Add `tape_saturation` (warmth that differs from basic waveshaping).

## Suggested spec surface

- Add `Effect::TapeSaturation { drive: f64, bias: f64, wow_rate: f64, flutter_rate: f64, hiss_level: f64 }`

## Implementation notes

- Determinism warning: hiss implies noise.
  - Either thread a deterministic RNG/seed into effect processing, or implement hiss as a deterministic pseudo-noise function derived from `seed` and sample index.
- Wow/flutter can be implemented as deterministic low-frequency pitch/time modulation in the delay-line domain.

## Acceptance criteria

- Deterministic (bit-identical) output for same spec/seed.
- No new stubs; tests updated.

