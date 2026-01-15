# [EFFECT P1] Gate / Expander

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 1)”.

## Goal

Add a dynamics `gate_expander` effect for tightening drums / noise reduction.

## Suggested spec surface

- Add `Effect::GateExpander { threshold_db: f64, ratio: f64, attack_ms: f64, hold_ms: f64, release_ms: f64, range_db: f64 }`
  - Serde tag: `"type": "gate_expander"`

## Implementation notes

- Backend likely belongs in `effects/dynamics.rs`.
- Deterministic envelope follower + gain computer; clamp ranges for stability.

## Acceptance criteria

- Audible gating/expansion behavior with stable parameter ranges.
- Tests for threshold/hold behavior on a synthetic signal.

