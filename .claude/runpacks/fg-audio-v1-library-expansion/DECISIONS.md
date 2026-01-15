# FG Audio V1 Library Expansion — Decisions (Procedural-Only)

This runpack is intended to be **100% procedural**: all audio is generated from spec parameters + seed, with **no external audio assets**.

## Scope

In-scope (procedural):

- Filters, LFO targets, effects, and synthesis types that are purely algorithmic.
- Post-FX modulation (LFO applied over time to the effect chain), as long as it does not require external audio assets.

## Post-FX LFO (procedural, required for some targets)

Some LFO targets (e.g. `delay_time`, `reverb_size`, `distortion_drive`) operate on **post-mix effects**, not on per-layer synthesis.

### Contract

- Add `AudioV1Params.post_fx_lfos: Vec<LfoModulation>` (default empty).
- `post_fx_lfos[].target` variants are **post-FX only** (not valid on `AudioLayer.lfo`):
  - `delay_time { amount_ms: f64 }`
  - `reverb_size { amount: f64 }`
  - `distortion_drive { amount: f64 }`
- Each post-FX LFO applies to **all matching effects** in `AudioV1Params.effects[]`.
- No silent no-op:
  - For each entry in `post_fx_lfos`, if there are **zero matching effects**, spec validation must fail.
- Keep it simple: **max 1 post-FX LFO per target**:
  - If `post_fx_lfos` contains duplicate targets (e.g. two `delay_time` entries), spec validation must fail.

### Multiplicity (answering “how many LFOs?”)

- `AudioLayer.lfo` is **0 or 1** per layer (one target per layer).
- `AudioV1Params.post_fx_lfos` is **0..N** per recipe, with **max 1 entry per target**.
- Each `post_fx_lfos[]` entry targets one parameter, but modulates **all matching effect instances** in the chain.

### Timeline / determinism rule

Effect chains are applied sequentially. To keep modulation time-aligned across the whole chain:

- For each `post_fx_lfos[]` entry, generate its LFO curve **once** for the full render duration (length = `num_samples`) and reuse it for every matching effect.
- Do not re-initialize LFOs separately per effect instance.

### Modulation formula

Let:

- `base` be the parameter value from the effect instance,
- `amount` be the target’s amount field,
- `lfo_value` be the LFO sample in `[0.0, 1.0]`,
- `bipolar = (lfo_value - 0.5) * 2.0` in `[-1.0, 1.0]`.

Then:

- `delay_time`: `time_ms = clamp(base + bipolar * amount_ms, 1.0, 2000.0)`
- `reverb_size`: `room_size = clamp(base + bipolar * amount, 0.0, 1.0)`
- `distortion_drive`: `drive = clamp(base + bipolar * amount, 1.0, 100.0)`

### “Matching effects” definition

- `delay_time` applies to:
  - `delay`
  - `multi_tap_delay` (apply to every tap’s `time_ms`)
  - `flanger`
  - `stereo_widener` (its `delay_ms`)
  - `granular_delay` (its `time_ms`)
- `reverb_size` applies to: `reverb`
- `distortion_drive` applies to:
  - `waveshaper`
  - `tape_saturation`
