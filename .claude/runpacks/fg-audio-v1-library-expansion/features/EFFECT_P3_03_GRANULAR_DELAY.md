# [EFFECT P3] Granular delay

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 3)”.

## Goal

Add `granular_delay` for shimmer/pitchy delays.

## Suggested spec surface (MVP)

- Add `Effect::GranularDelay { time_ms: f64, feedback: f64, grain_size_ms: f64, pitch_semitones: f64, wet: f64 }`

## Implementation notes

- Determinism warning: granular implies randomness.
  - Use deterministic RNG seeded from the spec seed (thread seed/RNG into effect chain as needed).

## Acceptance criteria

- Deterministic output.
- Basic granular delay texture; docs/schema/tests updated.

