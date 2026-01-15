# [EFFECT P2] Transient shaper

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 2)”.

## Goal

Add `transient_shaper` for attack/sustain control.

## Suggested spec surface

- Add `Effect::TransientShaper { attack: f64, sustain: f64, output_gain_db: f64 }`
  - `attack` and `sustain` in -100..=100 (as doc suggests), or normalize to -1..=1.

## Implementation notes

- Implement via envelope detection + two time-constant paths (attack vs sustain) with deterministic math.

## Acceptance criteria

- Audible change on percussive input.
- Deterministic; tests include a simple impulse/decay fixture.

