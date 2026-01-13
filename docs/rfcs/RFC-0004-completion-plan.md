# RFC-0004 Completion Plan (SpecCade)

- **Status:** Draft (plan)
- **Last updated:** 2026-01-13

## Goal

Bring RFC-0004 (“Musical Helpers for Music Compose”) to 100% compliance with the current `music.tracker_song_compose_v1` implementation (types, schema, expansion engine, CLI, tests), then mark the RFC **COMPLETED**.

## Current State (as of 2026-01-13)

- RFC-0003 (`music.tracker_song_compose_v1` Pattern IR) is implemented end-to-end (spec types, schema, expander, `speccade expand`, integration tests).
- RFC-0004 helpers are not present in the compose surface:
  - no `channel_ids` / `instrument_ids` aliases
  - no beat/bar timebase or beat-based time expressions
  - no harmony block or degree/chord-tone pitch sequencing

## Definition of Done

- Compose specs can use all RFC-0004 features and still produce deterministic, byte-identical XM/IT (Tier 1).
- JSON schema + serde types + validation + expander behavior are aligned.
- Unit tests cover helpers + error cases; integration tests ensure compose generation == expanded generation.

---

## 1) Named Channels and Instruments (Aliases)

### Spec surface

- Add optional alias maps to `MusicTrackerSongComposeV1Params`:
  - `channel_ids: { <name>: <u8> }`
  - `instrument_ids: { <name>: <u8> }`
- Introduce `ChannelRef` / `InstrumentRef` (untagged `u8 | String`) and update:
  - `CellTemplate.channel`
  - `CellTemplate.inst` (and any `*_seq` variants that should support aliases)
- Decide whether manual overrides (`ComposePattern.data` / `ComposePattern.notes`) also accept aliases; RFC-0004 does not require it, but the decision should be explicit.

### Expansion

- Resolve aliases in `speccade/crates/speccade-backend-music/src/compose.rs`:
  - unknown alias → new `MUSIC_COMPOSE_0xx` error code
  - bounds checks (`channel < params.channels`, `inst < instruments.len()`)
- Ensure resolution is deterministic and performed before merge/conflict checks.

### Schema + validation

- Update `speccade/schemas/speccade-spec-v1.schema.json` so compose `channel` / `inst` fields accept `integer | string` where applicable.
- Add spec-level validation in `speccade/crates/speccade-spec/src/validation/mod.rs` (optional); otherwise ensure expander errors surface cleanly via CLI.

### Tests

- Unit tests: alias resolution, unknown alias, bounds errors.
- Integration test: a compose spec using aliases expands to a stable snapshot.

---

## 2) Musical Time (Bars/Beats → Rows)

### Spec surface

- Add optional `timebase` (RFC-0004) to compose params and/or patterns:
  - `{ "beats_per_bar": <u16>, "rows_per_beat": <u16> }`
- Allow `ComposePattern` to declare `bars` (mutually exclusive with `rows`), expanded as:
  - `rows = bars * beats_per_bar * rows_per_beat`

### Pattern IR time ops

- Extend `TimeExpr` with:
  - `beat_list`
  - `beat_range`
- Implement mapping per RFC-0004:
  - `row = ((bar * beats_per_bar) + beat) * rows_per_beat + sub`
  - `delta_rows = step.beats * rows_per_beat + step.sub`

### Expansion

- Add conversions in `speccade/crates/speccade-backend-music/src/compose.rs`:
  - validate beat positions are within pattern length
  - reject negative/overflow rows deterministically

### Schema + tests

- Update JSON schema for the new structs and ops.
- Add unit tests for mapping + edge cases (invalid bar/beat/sub, out-of-bounds).

---

## 3) Harmony Helpers (Keys, Chords, Degrees)

### Spec surface

- Add optional `harmony` block:
  - `key` (root + scale)
  - `chords[]` with beat positions and chord symbols (plus interval-form escape hatch)
- Extend `emit_seq` to accept exactly one of:
  - `note_seq` (existing)
  - `pitch_seq` (new)
- Define `PitchSeq` supporting at least:
  - `scale_degree`
  - `chord_tone`

### Implementation

- Implement chord parsing per `speccade/docs/music-chord-spec.md` (including the interval-form escape hatch).
- During expansion, resolve `pitch_seq` to concrete note names using:
  - current key
  - current chord at event time (most recent chord whose `at` is `<=` the event time)
- Keep determinism guarantees (avoid floating-point; define stable error codes).

### Tests

- Unit tests: chord parsing, chord selection over time, `pitch_seq` mapping.
- Integration test: pitch-seq compose spec generates identical bytes to an expanded note-seq equivalent.

---

## 4) Docs + Close-out

- Update docs to match behavior:
  - `speccade/docs/rfcs/RFC-0004-music-compose-musical-helpers.md`
  - `speccade/docs/spec-reference/music.md`
- When all items above pass:
  - set RFC-0004 `Status: COMPLETED`
  - set `Last reviewed: YYYY-MM-DD`

