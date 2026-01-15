# [LFO] Target: `distortion_drive`

Source: `docs/FUTURE_GENERATORS.md` → “Missing LFO Targets”.

## Goal

Add a **post-FX** LFO target to modulate distortion drive over time.

This target is **not** a layer LFO target. It must be implemented via `AudioV1Params.post_fx_lfos` per:
- `.claude/runpacks/fg-audio-v1-library-expansion/DECISIONS.md`

## Required spec surface

1. Add `ModulationTarget::DistortionDrive { amount: f64 }`:
   - File: `crates/speccade-spec/src/recipe/audio/synthesis/modulation.rs`
   - Serde target object shape: `{ "target": "distortion_drive", "amount": 10.0 }`

2. This target uses `AudioV1Params.post_fx_lfos` (not `AudioLayer.lfo`):
   - If `post_fx_lfos` is not present yet, add it exactly as specified in `.claude/runpacks/fg-audio-v1-library-expansion/DECISIONS.md`.

## Required behavior (no discretion)

1. Validation rules:
   - `ModulationTarget::DistortionDrive` is invalid on `AudioLayer.lfo`.
   - `AudioV1Params.post_fx_lfos` must contain **max 1 entry per target** (duplicate targets must fail validation).
   - If any `post_fx_lfos[]` entry has `target == distortion_drive` and there are **zero matching effects** in `AudioV1Params.effects[]`, validation must fail.
   - Implement the “matching effects” list exactly as defined in `.claude/runpacks/fg-audio-v1-library-expansion/DECISIONS.md` (apply to all matching effect instances).

2. Backend rules:
   - Apply modulation to distortion drive during post-mix effect processing.
   - Reuse the LFO curve for this entry across all matching effects.
   - Apply modulation using the formula and clamps in `.claude/runpacks/fg-audio-v1-library-expansion/DECISIONS.md`.

## Acceptance criteria

- Target is not a no-op: distortion drive varies over time in output audio, deterministically.
- If `post_fx_lfos` contains a `distortion_drive` entry but there is no matching effect in the chain, spec validation fails.
- Schema + docs updated to include the `distortion_drive` target.
