# [EFFECT P1] Limiter

Source: `docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 1)”.

## Goal

Add a `limiter` effect (brick-wall limiting, distinct from compressor).

## Required spec surface

- Add `Effect::Limiter { threshold_db: f64, release_ms: f64, lookahead_ms: f64, ceiling_db: f64 }`
  - Serde tag: `"type": "limiter"`

## Implementation notes

- Spec: `crates/speccade-spec/src/recipe/audio/effects.rs`
- Backend:
  - Implement in `crates/speccade-backend-audio/src/effects/dynamics.rs` and wire it into `crates/speccade-backend-audio/src/effects/mod.rs`.
  - Deterministic lookahead: fixed-size delay buffer derived from `lookahead_ms` and sample_rate.

## Acceptance criteria

- Prevents clipping above ceiling (within numeric tolerance).
- Stable, deterministic, no NaNs.
- Tests cover a clipped input case.
