# [EFFECT P3] Rotary speaker (Leslie)

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 3)”.

## Goal

Add `rotary_speaker` effect for organ/psychedelic motion.

## Suggested spec surface (MVP)

- Add `Effect::RotarySpeaker { rate: f64, depth: f64, wet: f64 }`
  - Optionally add `accel` later; keep MVP minimal.

## Implementation notes

- Implement as deterministic amplitude + pan + mild doppler (short modulated delay) if feasible.

## Acceptance criteria

- Audible rotating motion; deterministic; docs/schema/tests updated.

