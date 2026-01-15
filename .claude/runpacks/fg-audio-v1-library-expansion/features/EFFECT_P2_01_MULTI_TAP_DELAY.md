# [EFFECT P2] Multi-tap delay

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 2)”.

## Goal

Add `multi_tap_delay` with a list of taps.

## Suggested spec surface

- Add `Effect::MultiTapDelay { taps: Vec<DelayTap> }`
- Add `DelayTap { time_ms: f64, feedback: f64, pan: f64, level: f64, filter_cutoff: f64 }`

## Implementation notes

- Backend:
  - Extend `effects/delay.rs` or add `effects/multi_tap_delay.rs`.
  - Deterministic processing; stable ordering of taps.
  - If `filter_cutoff` is supported, reuse filter helpers.

## Acceptance criteria

- Matches schema/docs.
- Tests cover multi-tap ordering and basic pan behavior.

