# [EFFECT P1] Stereo Widener

Source: `docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 1)”.

## Goal

Add `stereo_widener` effect for stereo enhancement.

## Required spec surface

- Add `Effect::StereoWidener { width: f64, mode: StereoWidenerMode, delay_ms: f64 }`
- Add `StereoWidenerMode`: `simple`, `haas`, `mid_side`

## Implementation notes

- Backend:
  - Implement as mid/side scaling (mid_side), simple L/R crossmix (simple), and small delay on one channel (haas).
  - Ensure mono input is handled (duplicate to stereo before widening, or no-op).

## Acceptance criteria

- Deterministic.
- Width=0 yields mono-ish; width=1 default; >1 increases side.
- Tests cover basic invariants (e.g., energy not exploding).
