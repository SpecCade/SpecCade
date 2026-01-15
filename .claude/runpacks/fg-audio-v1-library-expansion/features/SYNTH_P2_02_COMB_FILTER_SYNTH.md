# [SYNTH P2] Comb filter synthesis

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Synthesis Types (Priority 2)”.

## Goal

Add `comb_filter_synth` for resonant metallic tones / Karplus-Strong variants.

## Suggested spec surface (MVP)

- Add `Synthesis::CombFilterSynth { frequency: f64, decay: f64, excitation: CombExcitation }`
- Add `CombExcitation` enum: `impulse`, `noise`, `saw` (minimal set)

## Implementation notes

- Can reuse delay-line + feedback comb building blocks (may overlap with `Filter::Comb` implementation).
- Keep deterministic; seed any noise excitation from layer seed.

## Acceptance criteria

- Produces comb-resonant tone distinct from `karplus_strong` and `metallic`.
- Deterministic; docs/schema/tests updated.

