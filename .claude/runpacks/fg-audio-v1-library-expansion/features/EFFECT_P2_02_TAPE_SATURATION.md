# [EFFECT P2] Tape saturation

Source: `docs/FUTURE_GENERATORS.md` → “Missing Effects (Priority 2)”.

## Goal

Add `tape_saturation` (warmth that differs from basic waveshaping).

## Required spec surface

- Add `Effect::TapeSaturation { drive: f64, bias: f64, wow_rate: f64, flutter_rate: f64, hiss_level: f64 }`

## Implementation notes

- Determinism rule: hiss must be seed-driven and bit-identical.
  - Thread the audio `seed: u32` into effect-chain processing (pass it down from `generate_from_unified_params` into `effects::apply_effect_chain`).
  - Derive a dedicated RNG stream for tape hiss (e.g., `derive_component_seed(seed, "tape_saturation_hiss")`) and generate per-sample noise from it.
- Wow/flutter can be implemented as deterministic low-frequency pitch/time modulation in the delay-line domain.

## Acceptance criteria

- Deterministic (bit-identical) output for same spec/seed.
- No new stubs; tests updated.
