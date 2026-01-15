# [LFO] Target: `fm_index`

Source: `docs/FUTURE_GENERATORS.md` → “Missing LFO Targets”.

## Goal

Add an LFO target to modulate FM depth (`modulation_index`).

## Required spec surface

- Add `ModulationTarget::FmIndex { amount: f64 }`
  - Serde: `{ "target": "fm_index", "amount": 4.0 }`
  - `amount` is max delta; clamp index to `>= 0.0`.

## Required behavior (no silent no-op)

- Validation:
  - This target is valid only for `Synthesis::FmSynth`.
  - Any other synthesis type + `fm_index` must fail spec validation.

## Implementation notes

- Spec: `crates/speccade-spec/src/recipe/audio/synthesis/modulation.rs`
- Backend:
  - Add per-sample modulation support for `Synthesis::FmSynth` by re-synthesizing with a modulated `modulation_index`:
    - Generate the LFO curve once for the layer duration.
    - For each sample, compute `modulation_index = max(base + bipolar * amount, 0.0)` and synthesize with that value.

## Acceptance criteria

- Target parses/serializes.
- FM synth supports this modulation deterministically; tests updated.
