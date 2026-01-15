# [LFO] Target: `delay_time`

Source: `docs/FUTURE_GENERATORS.md` → “Missing LFO Targets”.

## Goal

Add a **post-FX** LFO target to modulate delay time for time-based effects.

This target is **not** a layer LFO target. It must be implemented via `AudioV1Params.post_fx_lfos` per:
- `.claude/runpacks/fg-audio-v1-library-expansion/DECISIONS.md`

## Required spec surface

1. Add `AudioV1Params.post_fx_lfos: Vec<LfoModulation>`:
   - File: `crates/speccade-spec/src/recipe/audio/mod.rs`
   - Serde: `#[serde(default, skip_serializing_if = "Vec::is_empty")]`

2. Add `ModulationTarget::DelayTime { amount_ms: f64 }`:
   - File: `crates/speccade-spec/src/recipe/audio/synthesis/modulation.rs`
   - Serde target object shape: `{ "target": "delay_time", "amount_ms": 25.0 }`

## Required behavior (no discretion)

1. Validation rules:
   - These post-FX targets are **invalid** on `AudioLayer.lfo`:
     - `delay_time`, `reverb_size`, `distortion_drive`
   - `AudioV1Params.post_fx_lfos` must only use those post-FX targets (it must reject layer-only targets like `pitch`, `pan`, etc).
   - `AudioV1Params.post_fx_lfos` must contain **max 1 entry per target** (duplicate targets must fail validation).
   - If any `post_fx_lfos[]` entry has `target == delay_time` and there are **zero matching effects** in `AudioV1Params.effects[]`, validation must fail (no silent no-op).
   - Implement the “matching effects” list exactly as defined in `.claude/runpacks/fg-audio-v1-library-expansion/DECISIONS.md` (apply to all matching effect instances, not just the first).

2. Backend rules:
   - Apply `post_fx_lfos[]` during post-mix effect processing (inside the effect chain application).
   - For each `post_fx_lfos[]` entry, generate its LFO curve **once** for the full render duration (length = `num_samples`) and reuse it for every matching effect.
   - Apply modulation using the formula and clamps in `.claude/runpacks/fg-audio-v1-library-expansion/DECISIONS.md`.

## Acceptance criteria

- Target is not a no-op: delay time varies over time in output audio, deterministically.
- If `post_fx_lfos` contains a `delay_time` entry but there is no matching effect in the chain, spec validation fails.
- Schema + docs updated to include `post_fx_lfos` and the `delay_time` target.
