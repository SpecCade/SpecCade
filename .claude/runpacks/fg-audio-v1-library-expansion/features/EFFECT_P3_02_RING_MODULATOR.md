# [EFFECT P3] Ring modulator (effect)

Source: `docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 3)”.

## Goal

Add a `ring_modulator` effect that processes existing audio (distinct from `ring_mod_synth`).

## Required spec surface

- Add `Effect::RingModulator { frequency: f64, mix: f64 }`
  - Serde tag: `"type": "ring_modulator"`

## Implementation notes

- Deterministic oscillator multiplied with input (mono/stereo).
- Consider anti-aliasing only if already present; MVP can be naive but stable.

## Acceptance criteria

- Audible sidebands; deterministic; docs/schema/tests updated.
