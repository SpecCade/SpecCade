# [LFO] Target: `grain_density`

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing LFO Targets”.

## Goal

Add an LFO target to modulate granular grain density.

## Suggested spec surface

- Add `ModulationTarget::GrainDensity { amount: f64 }`
  - Serde: `{ "target": "grain_density", "amount": 20.0 }`
  - Clamp density to `>= 0.0` and a safe max.

## Implementation notes

- Backend implementation should be deterministic (fixed block updates; stable RNG usage).

## Acceptance criteria

- Target parses/serializes.
- Granular density changes over time; deterministic; tests updated.

