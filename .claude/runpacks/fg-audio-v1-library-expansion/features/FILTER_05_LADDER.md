# [FILTER] Ladder

Source: `speccade/docs/FUTURE_GENERATORS.md` → “Missing Filter Types”.

## Goal

Expose a `ladder` filter type (classic Moog-ish 4-pole with resonance) for layer/master filters.

## Suggested spec surface (MVP)

- Add `Filter::Ladder { cutoff: f64, resonance: f64, cutoff_end: Option<f64> }`
  - Serde tag: `"type": "ladder"`
  - MVP: no drive/nonlinearity knobs unless needed for stability.

## Implementation notes

- Prefer a deterministic implementation:
  - MVP can be a stable approximation (e.g., cascaded one-poles/biquads + resonance feedback) as long as it’s stable and documented.
- Watch file size: refactor `speccade/crates/speccade-backend-audio/src/filter.rs` into modules if needed.

## Acceptance criteria

- Serde roundtrip tests.
- Filter is stable across reasonable cutoff/resonance ranges; no NaNs.
- Schema + docs updated.

