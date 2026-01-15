# [LFO] Target: `reverb_size`

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing LFO Targets”.

## Goal

Add an LFO target to modulate reverb “room size” over time (evolving spaces).

## Design note

Reverb lives in the global effects chain, so this likely needs recipe-level modulation (see `LFO_05_DELAY_TIME.md`).

## Suggested spec surface (minimal)

- Add `AudioV1Params.master_lfo: Option<LfoModulation>` (or equivalent)
- Add `ModulationTarget::ReverbSize { amount: f64 }` (unit interval delta)

## Implementation notes

- Backend:
  - Apply modulation to `Effect::Reverb.room_size` over time.
  - Prefer deterministic block updates if per-sample is too invasive.

## Acceptance criteria

- Target is not a no-op: reverb size varies over time.
- Deterministic behavior; schema/docs updated.

