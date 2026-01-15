# [SYNTH P1] Membrane / drum synthesis

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Synthesis Types (Priority 1)”.

## Goal

Add `membrane_drum` synthesis for drumhead physics / realistic toms and hand drums.

## Suggested spec surface (MVP)

- Add `Synthesis::MembraneDrum { frequency: f64, decay: f64, tone: f64, strike: f64 }`

## Implementation notes

- A physically-inspired MVP is fine:
  - e.g., a membrane-mode ratio set (modal synthesis macro) with noise/impulse excitation.
  - Reuse existing modal synthesis infrastructure if practical, but expose this as a distinct type.

## Acceptance criteria

- Sounds like a drum membrane (clear modes) and is distinct from layered noise.
- Deterministic; schema/docs/tests updated.

