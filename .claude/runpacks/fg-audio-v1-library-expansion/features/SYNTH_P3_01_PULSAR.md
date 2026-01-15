# [SYNTH P3] Pulsar synthesis

Source: `docs/FUTURE_GENERATORS.md` → “Missing Synthesis Types (Priority 3)”.

## Goal

Add `pulsar` synthesis (synchronized grain trains for rhythmic/tonal granular).

## Required spec surface

- Add `Synthesis::Pulsar { frequency: f64, pulse_rate: f64, grain_size_ms: f64, shape: Waveform }`

## Implementation notes

- Can be implemented using deterministic “grain bursts” at a fixed pulse rate.
- Keep RNG usage deterministic if any jitter is used (seeded, bounded).

## Acceptance criteria

- Audible pulsed/grain-train character; deterministic; docs/schema/tests updated.
