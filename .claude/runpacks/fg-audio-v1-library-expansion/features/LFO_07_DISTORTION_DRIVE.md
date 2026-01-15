# [LFO] Target: `distortion_drive`

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing LFO Targets”.

## Goal

Add an LFO target to modulate distortion drive over time (dynamic saturation).

## Design note

Waveshaper drive is part of the global effects chain. This likely needs recipe-level modulation (see `LFO_05_DELAY_TIME.md`).

## Suggested spec surface (minimal)

- Add `AudioV1Params.master_lfo: Option<LfoModulation>` (or equivalent)
- Add `ModulationTarget::DistortionDrive { amount: f64 }`

## Implementation notes

- Backend:
  - Apply modulation to `Effect::Waveshaper.drive` over time.
  - Deterministic block updates are acceptable.

## Acceptance criteria

- Target is not a no-op: drive varies over time.
- Deterministic behavior; schema/docs updated.

