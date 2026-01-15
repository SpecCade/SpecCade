# [SYNTH P1] Supersaw / Unison engine

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Synthesis Types (Priority 1)”.

## Goal

Add a dedicated `supersaw_unison` synthesis type with proper unison controls (detune curve + stereo spread), beyond `multi_oscillator`.

## Suggested spec surface (MVP)

- Add `Synthesis::SupersawUnison { frequency: f64, voices: u8, detune_cents: f64, spread: f64, detune_curve: DetuneCurve }`
- Add `DetuneCurve` enum (e.g., `linear`, `exp`, `gaussian`), deterministic.

## Implementation notes

- Spec:
  - `speccade/crates/speccade-spec/src/recipe/audio/synthesis/**` (add a new variant + serde tag `supersaw_unison`)
- Backend:
  - Implement deterministically by generating N sub-voices with fixed detune offsets and pan offsets.
  - Current layer synthesis is mono; to support true stereo spread, consider expanding the layer into multiple mixer layers in `speccade/crates/speccade-backend-audio/src/generate/mod.rs` (each voice gets its own pan).
  - Keep the change localized; update validation max-layers if needed.

## Acceptance criteria

- Audible unison thickness; `spread` produces wider stereo image (not a no-op).
- Deterministic; schema/docs/tests updated.

