# Audio Synthesis Types — Status

Status of `audio_v1` synthesis implementations. Updated: 2026-01-13

This is a quick snapshot. For the authoritative schema, see `crates/speccade-spec/src/recipe/audio/synthesis.rs`.
For parameter docs and examples, see `docs/spec-reference/audio.md`.

## Implemented

All currently supported `recipe.params.layers[].synthesis.type` variants are implemented in `speccade-backend-audio`:

- `oscillator` — DONE
- `multi_oscillator` — DONE
- `fm_synth` — DONE
- `feedback_fm` — DONE
- `am_synth` — DONE
- `ring_mod_synth` — DONE
- `karplus_strong` — DONE
- `bowed_string` — DONE
- `noise_burst` — DONE
- `additive` — DONE
- `pitched_body` — DONE
- `metallic` — DONE
- `wavetable` — DONE
- `granular` — DONE
- `pd_synth` — DONE
- `modal` — DONE
- `vocoder` — DONE
- `formant` — DONE
- `vector` — DONE
- `supersaw_unison` — DONE
- `waveguide` — DONE
- `membrane_drum` — DONE
- `comb_filter_synth` — DONE
- `pulsar` — DONE
- `vosim` — DONE
- `spectral_freeze` — DONE
