# [EFFECT P3] Granular delay

Source: `docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 3)”.

## Goal

Add `granular_delay` for shimmer/pitchy delays.

## Required spec surface

- Add `Effect::GranularDelay { time_ms: f64, feedback: f64, grain_size_ms: f64, pitch_semitones: f64, wet: f64 }`

## Implementation notes

- Determinism warning: granular implies randomness.
  - Thread the audio `seed: u32` into effect-chain processing and derive a dedicated RNG stream for `granular_delay` (stable across runs).

## Acceptance criteria

- Deterministic output.
- Basic granular delay texture; docs/schema/tests updated.
