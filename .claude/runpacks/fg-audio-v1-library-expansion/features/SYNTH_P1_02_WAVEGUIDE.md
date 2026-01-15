# [SYNTH P1] Waveguide synthesis (wind/brass)

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Synthesis Types (Priority 1)”.

## Goal

Add `waveguide` synthesis for wind/brass physical modeling (distinct from `karplus_strong`).

## Suggested spec surface (MVP)

- Add `Synthesis::Waveguide { frequency: f64, breath: f64, noise: f64, damping: f64, resonance: f64 }`
  - Keep MVP narrow; add embouchure/bore params later.

## Implementation notes

- Backend implementation should be deterministic and stable:
  - Delay-line waveguide with simple nonlinearity + filtered noise excitation is acceptable for MVP.
  - RNG usage must be seeded deterministically (derive from layer seed).

## Acceptance criteria

- Produces sustained “wind-ish” tones and responds to parameters.
- Deterministic; schema/docs/tests updated.

