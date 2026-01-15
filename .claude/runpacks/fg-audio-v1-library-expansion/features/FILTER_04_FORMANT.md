# [FILTER] Formant

Source: `docs/FUTURE_GENERATORS.md` → “Missing Filter Types”.

## Goal

Expose a `formant` filter type (vowel filtering) usable as a layer/master filter.

## Required spec surface

- Add `Filter::Formant { vowel: FormantVowel, intensity: f64 }`
  - Serde tag: `"type": "formant"`
  - Reuse `FormantVowel` if appropriate, or define a small filter-specific vowel enum.
  - `intensity` mixes dry → vowel-shaped.

If reuse creates confusing naming collisions, prefer `FilterFormantVowel` / `FormantFilter`.

## Implementation notes

- Spec types live in `crates/speccade-spec/src/recipe/audio/synthesis/**`.
- Backend:
  - Implement as a small bank of deterministic resonant bandpass/peaking filters.
  - Ensure stable, bounded gains (avoid runaway resonance).

## Acceptance criteria

- Serde roundtrip tests.
- Filter produces audible vowel-ish shaping and remains stable.
- Schema + docs updated.
