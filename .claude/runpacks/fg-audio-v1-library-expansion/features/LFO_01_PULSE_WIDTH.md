# [LFO] Target: `pulse_width`

Source: `docs/FUTURE_GENERATORS.md` → “Missing LFO Targets”.

## Goal

Add an LFO target to modulate PWM / duty cycle for pulse/square-style sounds.

## Required spec surface

- Add `ModulationTarget::PulseWidth { amount: f64 }`
  - Serde: `{ "target": "pulse_width", "amount": 0.2 }`
  - `amount` = maximum delta applied around the base duty; clamp result to `(0.01..0.99)`.
  - Validate `amount` in `[0.0, 0.49]` (larger ranges just slam into the clamp and are not useful).

## Required behavior (no silent no-op)

- Validation:
  - This target is valid only for:
    - `Synthesis::Oscillator` with `waveform: square|pulse`
    - `Synthesis::MultiOscillator` with at least one oscillator using `waveform: square|pulse`
  - Any other synthesis type + `pulse_width` must fail spec validation (do not accept a target that cannot take effect).

## Implementation notes

- Spec types: `crates/speccade-spec/src/recipe/audio/synthesis/modulation.rs`
- Backend touch points:
  - LFO dispatch: `crates/speccade-backend-audio/src/generate/layer.rs`
  - Implement via re-synthesis for oscillator-based types (same shape as the existing pitch-LFO path):
    - Generate the LFO curve once for the layer duration.
    - For each sample, compute `duty = clamp(base_duty + bipolar * amount, 0.01, 0.99)` and re-generate the oscillator sample using that duty.

## Acceptance criteria

- Target parses/serializes.
- Audible PWM modulation on `oscillator` / `multi_oscillator` where duty is meaningful.
- Deterministic output; tests updated.
