# [SYNTH P3] Spectral synthesis

Source: `docs/FUTURE_GENERATORS.md` → “Missing Synthesis Types (Priority 3)”.

## Goal

Add a minimal, deterministic FFT-based **spectral freeze** synthesis type.

## Required spec surface

- Add `Synthesis::SpectralFreeze { source: SpectralSource }`
  - Serde tag: `"type": "spectral_freeze"`
- Add `SpectralSource` enum with exactly:
  - `noise { noise_type: NoiseType }`
  - `tone { waveform: Waveform, frequency: f64 }`

## Implementation notes

- Add an FFT dependency (`rustfft`) and keep it deterministic.
- Use fixed DSP constants (no tunable FFT size in this runpack):
  - `fft_size = 2048`
  - `hop_size = 512`
  - Hann window
- Algorithm (freeze):
  1. Generate one deterministic source frame of length `fft_size` (noise or tone) using the layer seed.
  2. Window it and compute FFT, storing the complex spectrum (magnitude + phase).
  3. Render the full output by repeating:
     - inverse FFT of the stored spectrum
     - window
     - overlap-add at `hop_size`
  4. Normalize by the overlap-add window sum so amplitude is stable.
- Validate bounds to avoid runaway allocations (duration is already capped globally; still avoid per-frame realloc churn).

## Acceptance criteria

- Works end-to-end (no stubs) and deterministic.
- Docs/schema/tests updated; scope stays in audio crates.
