# [EFFECT P2] Auto-filter / Envelope follower

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 2)”.

## Goal

Add `auto_filter` effect (auto-wah / dynamic filter sweeps driven by signal level).

## Suggested spec surface

- Add `Effect::AutoFilter { sensitivity: f64, attack_ms: f64, release_ms: f64, depth: f64, base_frequency: f64 }`

## Implementation notes

- Backend:
  - Deterministic envelope follower, map envelope → cutoff modulation.
  - Prefer reusing existing filter coefficients; avoid unstable resonance.

## Acceptance criteria

- Audible auto-wah behavior on a dynamic signal.
- Deterministic; tests updated.

