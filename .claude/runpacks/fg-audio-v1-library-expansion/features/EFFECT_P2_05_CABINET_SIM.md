# [EFFECT P2] Cabinet simulation

Source: `docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 2)”.

## Goal

Add `cabinet_sim` (speaker/amp coloration).

## Required spec surface

- Add `Effect::CabinetSim { cabinet_type: CabinetType, mic_position: f64 }`
- Add `CabinetType`: `guitar_1x12`, `guitar_4x12`, `bass_1x15`, `radio`, `telephone`

## Implementation notes

- Procedural-only requirement: implement as a stable filter-stack approximation (no IR/convolution, no embedded impulse tables).
- Per `cabinet_type`, define a deterministic EQ/filter curve (biquad cascade).

## Acceptance criteria

- Distinct tonal coloration per cabinet type.
- Deterministic; schema/docs/tests updated.
