# [LFO] Target: `grain_size`

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing LFO Targets”.

## Goal

Add an LFO target to modulate granular grain size.

## Suggested spec surface

- Add `ModulationTarget::GrainSize { amount_ms: f64 }`
  - Serde: `{ "target": "grain_size", "amount_ms": 30.0 }`
  - Clamp grain size to a safe range (match existing granular constraints).

## Implementation notes

- Spec: `speccade/crates/speccade-spec/src/recipe/audio/synthesis/modulation.rs`
- Backend:
  - `Synthesis::Granular` lives in `speccade/crates/speccade-backend-audio/src/synthesis/granular.rs`.
  - Implement time-varying grain size in a deterministic way (e.g., update per fixed block).

## Acceptance criteria

- Target parses/serializes.
- Granular synthesis responds to modulation (audible); deterministic; tests updated.

