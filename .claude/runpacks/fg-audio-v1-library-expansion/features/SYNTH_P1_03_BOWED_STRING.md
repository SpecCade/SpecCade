# [SYNTH P1] Bowed string synthesis

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Synthesis Types (Priority 1)”.

## Goal

Add `bowed_string` synthesis for sustained bowed instruments (violin/cello-ish).

## Suggested spec surface (MVP)

- Add `Synthesis::BowedString { frequency: f64, bow_pressure: f64, bow_position: f64, damping: f64 }`

## Implementation notes

- MVP is allowed to be an approximation as long as it is:
  - deterministic
  - stable (no NaNs / runaway feedback)
  - audibly distinct from `karplus_strong`
- Consider reusing waveguide building blocks with a deterministic excitation model.

## Acceptance criteria

- Sustained bowed character (continuous excitation), not a pluck.
- Deterministic; schema/docs/tests updated.

