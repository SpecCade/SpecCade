# [SYNTH P3] VOSIM

Source: `docs/FUTURE_GENERATORS.md` → “Missing Synthesis Types (Priority 3)”.

## Goal

Add `vosim` synthesis (efficient formant pulse trains for robotic voices).

## Required spec surface

- Add `Synthesis::Vosim { frequency: f64, formant_freq: f64, pulses: u8, breathiness: f64 }`

## Implementation notes

- Deterministic pulse-train + exponential decay shaping; optional noise for breathiness must be seeded deterministically.

## Acceptance criteria

- Robotic vowel-ish tones; deterministic; docs/schema/tests updated.
