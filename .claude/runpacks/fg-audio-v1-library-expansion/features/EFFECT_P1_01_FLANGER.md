# [EFFECT P1] Flanger

Source: `docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 1)”.

## Goal

Add a dedicated `flanger` effect (distinct from chorus/phaser).

## Required spec surface

- Add `Effect::Flanger { rate: f64, depth: f64, feedback: f64, delay_ms: f64, wet: f64 }`
  - Serde tag: `"type": "flanger"`
  - `depth` can be unit interval mapped to a small delay modulation range.

## Implementation notes

- Spec: `crates/speccade-spec/src/recipe/audio/effects.rs`
- Backend:
  - Add `crates/speccade-backend-audio/src/effects/flanger.rs`.
  - Wire it into `crates/speccade-backend-audio/src/effects/mod.rs` (`apply_effect_chain`).
  - Keep deterministic (no randomness).

## Acceptance criteria

- New variant is fully implemented + documented + in schema.
- Sounds distinct from chorus (shorter base delay, feedback path).
- Tests updated.
