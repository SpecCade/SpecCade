# [EFFECT P3] Convolution reverb

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 3)”.

## Goal

Add `convolution_reverb` using impulse responses.

## Suggested spec surface (MVP)

- Add `Effect::ConvolutionReverb { ir: ConvolutionIr, wet: f64 }`
- Add `ConvolutionIr` as either:
  - A small enum of built-in IRs (preferred for determinism), OR
  - An inline IR array (bounded length, validated).

## Implementation notes

- Deterministic convolution:
  - For MVP, time-domain convolution is acceptable for short IRs.
  - If FFT is used, keep it deterministic (fixed plan sizes, stable math).

## Acceptance criteria

- Deterministic; docs/schema/tests updated.
- Distinct “space” presets via built-in IRs (preferred).

