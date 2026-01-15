# [SYNTH P2] Comb filter synthesis

Source: `docs/FUTURE_GENERATORS.md` → “Missing Synthesis Types (Priority 2)”.

## Goal

Add `comb_filter_synth` for resonant metallic tones / Karplus-Strong variants.

## Required spec surface

- Add `Synthesis::CombFilterSynth { frequency: f64, decay: f64, excitation: CombExcitation }`
- Add `CombExcitation` enum: `impulse`, `noise`, `saw` (minimal set)

## Implementation notes

- Reuse the same deterministic building blocks as the `comb` filter: delay-line + feedback comb (do not invent a second unrelated comb algorithm).
- Keep deterministic; seed any noise excitation from layer seed.

## Acceptance criteria

- Produces comb-resonant tone distinct from `karplus_strong` and `metallic`.
- Deterministic; docs/schema/tests updated.
