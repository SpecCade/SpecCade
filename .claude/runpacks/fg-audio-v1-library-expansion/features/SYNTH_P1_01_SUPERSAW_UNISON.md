# [SYNTH P1] Supersaw / Unison engine

Source: `docs/FUTURE_GENERATORS.md` → “Missing Synthesis Types (Priority 1)”.

## Goal

Add a dedicated `supersaw_unison` synthesis type with proper unison controls (detune curve + stereo spread), beyond `multi_oscillator`.

## Required spec surface

- Add `Synthesis::SupersawUnison { frequency: f64, voices: u8, detune_cents: f64, spread: f64, detune_curve: DetuneCurve }`
  - Serde tag: `"type": "supersaw_unison"`
- Add `DetuneCurve` enum with exactly these variants:
  - `linear`
  - `exp2` (a steeper curve)

## Implementation notes

- Spec:
  - `crates/speccade-spec/src/recipe/audio/synthesis/**` (add a new variant + serde tag `supersaw_unison`)
- Backend:
  - Implement deterministically by **expanding the layer into N “virtual layers”** in `crates/speccade-backend-audio/src/generate/mod.rs` (required; no per-layer stereo support exists today).
  - For `voices = N`, define a per-voice normalized position `x`:
    - If `N == 1`: `x = 0.0`
    - Else: `x = -1.0 + 2.0 * (voice_idx as f64) / ((N - 1) as f64)` (so `x ∈ [-1, 1]`)
  - Detune cents offset per voice:
    - `linear`: `detune_offset_cents = x * detune_cents`
    - `exp2`: `detune_offset_cents = sign(x) * (x.abs() * x.abs()) * detune_cents`
  - Pan offset per voice:
    - `voice_pan = clamp(layer.pan + x * spread, -1.0, 1.0)`
  - Voice synthesis (per virtual layer):
    - Synthesize a sawtooth oscillator voice (`Waveform::Sawtooth`) at `frequency * 2^(detune_offset_cents/1200)`.
    - Reuse existing oscillator synthesis paths (do not introduce a new oscillator implementation).
  - Voice gain:
    - Set each virtual layer’s volume to `layer.volume / N` (keeps loudness stable vs `voices`).
  - Deterministic seeding:
    - Derive a unique seed per virtual voice from the original layer seed (e.g., via `derive_component_seed(layer_seed, "supersaw_voice_{voice_idx}")`).
  - Validation:
    - Enforce a hard cap so that the total number of expanded layers (sum of `voices` across all `supersaw_unison` layers, plus non-expanded layers) stays within the existing 32-layer limit.

## Acceptance criteria

- Audible unison thickness; `spread` produces wider stereo image (not a no-op).
- Deterministic; schema/docs/tests updated.
