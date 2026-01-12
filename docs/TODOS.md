# SpecCade TODOs (Curated Backlog)

## Music (Compose / RFC-0003)

- Implement `music.tracker_song_compose_v1` end-to-end (spec types + schema + backend expander + CLI dispatch).
- Add `speccade expand --spec <spec.json>` for compose specs (prints expanded `music.tracker_song_v1` params JSON for review/tests).
- Implement Pattern IR operators + limits per RFC-0003:
  - structural: `stack`, `concat`, `repeat`, `shift`, `slice`, `ref`
  - emit: `emit`, `emit_seq`
  - time: `range`, `list`, `euclid`, `pattern`
  - variation: `prob`, `choose`
  - transform: `transpose_semitones`, `vol_mul`, `set`
- Determinism + safety: PCG32 seed derivation, recursion/cycle detection, resource limits, stable error codes.
- Tests:
  - unit tests for operator semantics + merge policies
  - integration tests: compose → expanded JSON snapshot; XM/IT bytes identical vs expanded

## Music (Musical Helpers / RFC-0004)

- Add optional `channel_ids` / `instrument_ids` alias maps and allow cell templates to reference `channel`/`inst` by name.
- Add `timebase` + pattern `bars` and implement `beat_range` / `beat_list` time expressions.
- Add `harmony` + `ChordSpec` parsing (see `docs/music-chord-spec.md`) + `pitch_seq` for `emit_seq` (`scale_degree` / `chord_tone`).
- Add deterministic “humanize” helpers (e.g., `humanize_vol`) and pick a minimal, deterministic swing strategy (row-shift vs note-delay effect).

## Music (Content / Workflow)

- Prefer `TrackerInstrument.ref` (external `audio_v1` specs) in music examples/goldens so instruments are reusable and specs stay small; keep inline synthesis only when demonstrating a feature.
- Add a higher-quality drum example (filtered noise / metallic synthesis and/or WAV drum samples) and tune it by ear so kick/snare/hat sound correct in common XM/IT players.
- Add a small “XM vs IT parity” listening checklist and automate basic structural checks (loop flags/envelope flags) in tests.
- Add “genre kits” as data packages: curated compose `defs` + instrument refs + timebase/harmony defaults (with docs/workflows that focus edits on patterns, not instruments).

## Audio

- Create a small preset library of reusable `audio_v1` specs (kick/snare/hat/clap/bass/lead/pad/etc.) tuned by ear + validated by metrics (no clipping, sane RMS, no DC offset).
- Ensure the library covers `docs/audio-preset-library-master-list.md` (used by the music kits).
- Add an “audio audit” report (peak/RMS/DC offset) to catch broken presets and regressions.
- Improve loop-point generation + click-free envelope defaults for tracker instrument baking; document best practices.

## Tooling / QA

- Grow the Tier-1 golden corpus (`golden/`) for audio/music/textures and run it in CI.
- Add “expand/inspect” style commands/flags where helpful for review (compose → expanded JSON, packed-texture intermediate maps).
