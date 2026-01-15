# [EFFECT P2] Cabinet simulation

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 2)”.

## Goal

Add `cabinet_sim` (speaker/amp coloration).

## Suggested spec surface

- Add `Effect::CabinetSim { cabinet_type: CabinetType, mic_position: f64 }`
- Add `CabinetType`: `guitar_1x12`, `guitar_4x12`, `bass_1x15`, `radio`, `telephone`

## Implementation notes

- Deterministic implementation options:
  - Small fixed IRs embedded in code (per cabinet_type), convolved deterministically.
  - Or a stable filter stack approximation (bandpass/peaks) per type.

## Acceptance criteria

- Distinct tonal coloration per cabinet type.
- Deterministic; schema/docs/tests updated.

