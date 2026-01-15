# [EFFECT P2] Multi-tap delay

Source: `docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 2)”.

## Goal

Add `multi_tap_delay` with a list of taps.

## Required spec surface

- Add `Effect::MultiTapDelay { taps: Vec<DelayTap> }`
- Add `DelayTap { time_ms: f64, feedback: f64, pan: f64, level: f64, filter_cutoff: f64 }`

## Implementation notes

- Backend:
  - Add `crates/speccade-backend-audio/src/effects/multi_tap_delay.rs` and wire it into `crates/speccade-backend-audio/src/effects/mod.rs`.
  - Extract the delay-line helper from `crates/speccade-backend-audio/src/effects/delay.rs` into a new shared module and reuse it (no copy/paste).
  - Deterministic processing; stable ordering of taps.
  - Implement `filter_cutoff` per-tap using existing filter helpers (deterministic biquad coefficients).

## Acceptance criteria

- Matches schema/docs.
- Tests cover multi-tap ordering and basic pan behavior.
