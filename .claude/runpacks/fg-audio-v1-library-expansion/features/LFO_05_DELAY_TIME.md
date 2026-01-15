# [LFO] Target: `delay_time`

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing LFO Targets”.

## Goal

Add an LFO target to modulate delay time for time-based effects (chorus/flanger/delay-style motion).

## Design note (important)

Current LFO lives on `AudioLayer` and runs during layer generation. Delay time exists in the **global effects chain** (`recipe.params.effects`). To make this target real (not a no-op), you likely need **recipe-level modulation**.

## Suggested spec surface (minimal)

- Add `AudioV1Params.master_lfo: Option<LfoModulation>` (or `post_fx_lfo`)
- Add `ModulationTarget::DelayTime { amount_ms: f64 }`

## Implementation notes

- Backend:
  - Apply `master_lfo` during `effects::apply_effect_chain`.
  - If per-sample modulation is too invasive, update parameters per fixed block size (deterministic).

## Acceptance criteria

- Target is not a no-op: delay time actually varies over time in output.
- Deterministic behavior.
- Schema + docs updated to describe the new `master_lfo` (if added).

