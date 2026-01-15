# [LFO] Target: `grain_size`

Source: `docs/FUTURE_GENERATORS.md` → “Missing LFO Targets”.

## Goal

Add an LFO target to modulate granular grain size.

## Required spec surface

- Add `ModulationTarget::GrainSize { amount_ms: f64 }`
  - Serde: `{ "target": "grain_size", "amount_ms": 30.0 }`
  - Clamp grain size to the existing granular constraints (10–500 ms).

## Required behavior (no silent no-op)

- Validation:
  - This target is valid only for `Synthesis::Granular`.
  - Any other synthesis type + `grain_size` must fail spec validation.

## Implementation notes

- Spec: `crates/speccade-spec/src/recipe/audio/synthesis/modulation.rs`
- Backend:
  - `Synthesis::Granular` lives in `crates/speccade-backend-audio/src/synthesis/granular.rs`.
  - Implement time-varying grain size deterministically **per grain start** (not per arbitrary block size):
    - Generate the LFO curve once for the full layer duration (`num_samples`).
    - For each grain, sample the LFO at `grain_start` and compute:
      - `grain_size_ms = clamp(base + bipolar * amount_ms, 10.0, 500.0)`
    - Use that `grain_size_ms` for that grain’s render/window.

## Acceptance criteria

- Target parses/serializes.
- Granular synthesis responds to modulation (audible); deterministic; tests updated.
