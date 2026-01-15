# [SYNTH P2] Sample playback

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Synthesis Types (Priority 2)”.

## Goal

Add `sample_playback` synthesis for instruments requiring recorded samples.

## Suggested spec surface (minimal + deterministic)

- Add `Synthesis::SamplePlayback { source: SampleSource, gain: f64 }`
- Add `SampleSource` as either:
  - A small enum of built-in samples (preferred for determinism), OR
  - A `path` with strict constraints (must be inside pack/spec directory) + deterministic hashing of bytes.

## Implementation notes

- Backend likely needs WAV decode support; look for existing helpers in `speccade/crates/speccade-backend-audio/src/wav/**`.
- Determinism is the main risk: define and validate allowed paths + stable read semantics.

## Acceptance criteria

- Deterministic output given same sample bytes.
- Clear validation errors for missing/out-of-bounds paths.
- Docs/schema/tests updated.

