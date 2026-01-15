# [LFO] Target: `grain_density`

Source: `docs/FUTURE_GENERATORS.md` → “Missing LFO Targets”.

## Goal

Add an LFO target to modulate granular grain density.

## Required spec surface

- Add `ModulationTarget::GrainDensity { amount: f64 }`
  - Serde: `{ "target": "grain_density", "amount": 20.0 }`
  - Clamp density to the existing granular constraints (1–100 grains/sec).

## Required behavior (no silent no-op)

- Validation:
  - This target is valid only for `Synthesis::Granular`.
  - Any other synthesis type + `grain_density` must fail spec validation.

## Implementation notes

- Backend: implement deterministically **per grain start** (not per arbitrary block size):
  - Generate the LFO curve once for the full layer duration (`num_samples`).
  - For each grain, sample the LFO at `grain_start` and compute:
    - `density = clamp(base + bipolar * amount, 1.0, 100.0)`
    - `grain_interval_samples = max(1, floor(sample_rate / density))`
  - Use that `grain_interval_samples` to schedule the next grain.

## Acceptance criteria

- Target parses/serializes.
- Granular density changes over time; deterministic; tests updated.
