# [SYNTH P3] Spectral synthesis

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Synthesis Types (Priority 3)”.

## Goal

Add `spectral` synthesis (FFT-based freeze/morph/filter style synthesis).

## Suggested spec surface (MVP)

- Keep MVP narrowly scoped, e.g.:
  - `Synthesis::SpectralFreeze { source: SpectralSource, smoothing: f64 }`
  - with a small set of deterministic sources (`noise`, `oscillator`).

## Implementation notes

- Adding an FFT dependency may be necessary; keep it deterministic.
- Avoid large CPU cost or unbounded allocations (validate sizes).

## Acceptance criteria

- Works end-to-end (no stubs) and deterministic.
- Docs/schema/tests updated; scope stays in audio crates.

