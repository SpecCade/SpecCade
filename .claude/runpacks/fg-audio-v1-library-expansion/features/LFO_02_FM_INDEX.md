# [LFO] Target: `fm_index`

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing LFO Targets”.

## Goal

Add an LFO target to modulate FM depth (`modulation_index`).

## Suggested spec surface

- Add `ModulationTarget::FmIndex { amount: f64 }`
  - Serde: `{ "target": "fm_index", "amount": 4.0 }`
  - `amount` is max delta; clamp index to `>= 0.0`.

## Implementation notes

- Spec: `speccade/crates/speccade-spec/src/recipe/audio/synthesis/modulation.rs`
- Backend:
  - Add per-sample modulation support for `Synthesis::FmSynth` (and optionally `FeedbackFm` once added).
  - Prefer a dedicated “re-synth with modulated param” path (avoid post-processing).

## Acceptance criteria

- Target parses/serializes.
- FM synth supports this modulation deterministically; tests updated.

