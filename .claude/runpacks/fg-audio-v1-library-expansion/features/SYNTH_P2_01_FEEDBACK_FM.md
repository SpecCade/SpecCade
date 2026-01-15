# [SYNTH P2] Feedback FM

Source: `docs/FUTURE_GENERATORS.md` → “Missing Synthesis Types (Priority 2)”.

## Goal

Add `feedback_fm` synthesis: self-modulating operator (distinct timbres vs 2-op FM).

## Required spec surface

- Add `Synthesis::FeedbackFm { frequency: f64, feedback: f64, modulation_index: f64, freq_sweep: Option<FreqSweep> }`
  - Serde tag: `"type": "feedback_fm"`

## Implementation notes

- Backend: implement in `crates/speccade-backend-audio/src/synthesis/` (new module).
- Ensure stability: clamp feedback and avoid NaNs.

## Acceptance criteria

- Distinct sound vs `fm_synth`.
- Deterministic; docs/schema/tests updated.
