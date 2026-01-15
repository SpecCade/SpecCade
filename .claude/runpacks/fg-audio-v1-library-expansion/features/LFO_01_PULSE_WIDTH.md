# [LFO] Target: `pulse_width`

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing LFO Targets”.

## Goal

Add an LFO target to modulate PWM / duty cycle for pulse/square-style sounds.

## Suggested spec surface

- Add `ModulationTarget::PulseWidth { amount: f64 }`
  - Serde: `{ "target": "pulse_width", "amount": 0.2 }`
  - `amount` = maximum delta applied around the base duty; clamp result to `(0.01..0.99)`.

## Implementation notes

- Spec types: `speccade/crates/speccade-spec/src/recipe/audio/synthesis/modulation.rs`
- Backend touch points:
  - LFO dispatch: `speccade/crates/speccade-backend-audio/src/generate/layer.rs`
  - Likely requires re-synth for oscillator-based types (similar to pitch modulation path) so duty can vary per-sample.

## Acceptance criteria

- Target parses/serializes.
- Audible PWM modulation on `oscillator` / `multi_oscillator` where duty is meaningful.
- Deterministic output; tests updated.

