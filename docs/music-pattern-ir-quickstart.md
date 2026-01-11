# Music Pattern IR (Compose) — Quickstart (Draft)

This quickstart describes the proposed JSON “Pattern IR” authoring layer for tracker music described in:

- `docs/rfcs/RFC-0003-music-pattern-ir.md`
- `docs/rfcs/RFC-0004-music-compose-musical-helpers.md` (optional: names/beats/harmony)

The goal is to keep `music.tracker_song_v1` as the canonical, fully-expanded event format, while providing a compact, safe, deterministic way to author patterns.

## Mental Model

- A tracker pattern is a grid: `(row, channel) -> cell`.
- Pattern IR builds that grid by composing small generators (hits, sequences) using a few operators (`stack`, `repeat`, `shift`, …).
- The backend expands Pattern IR into a normal `TrackerPattern.data[]` list, sorted by `(row, channel)`.

## 5-Minute Workflow

1) Start with a working `music.tracker_song_v1` (or a small hand-made tracker idea).
2) Switch to `music.tracker_song_compose_v1`.
2a) (Recommended) Add `channel_ids` / `instrument_ids`, and a `timebase` so you can author by name + beats.
3) Turn the biggest repeated blocks into:
   - `emit` + `range` (regular grids)
   - `emit` + `euclid` (polyrhythms / syncopation)
   - `emit_seq` + `range` (melodies/basslines, absolute notes)
   - `emit_seq` + `pitch_seq` (melodies/basslines, degrees/chord tones; RFC-0004)
4) Extract reusable parts into `defs` (kick pattern, hat pattern, bass ostinato, fill).
5) Use `transform` for transposition and volume scaling instead of duplicating note lists.
5a) (Optional) Add `harmony` so pitched parts can be authored as chord tones / degrees.

## Debugging & Review (Required Tooling)

To keep this system “reviewable”, implement at least one expansion view:

- `speccade expand <spec.json>` → prints expanded `music.tracker_song_v1` params JSON

Recommended PR workflow:

- reviewers look at both the compact compose spec and the expanded JSON snapshot

## Authoring Tips

- Prefer `defs` for drum layers and common riffs.
- Prefer `emit_seq` for pitched material (bass/lead) and keep sequences short + cyclic.
- If using `harmony`, prefer chord tones for basslines and strong beats; allow accidentals only via explicit opt-in.
- Keep “sound design” (instruments) separate from “music logic” (Pattern IR).
- Use `merge: "error"` at top-level stacks so you find collisions early.
- When you *intentionally* overlap (e.g., “ghost notes” that override volume), use a nested `stack` with `merge: "last_wins"` for that layer.
- Use deterministic randomness (`choose`, `prob`) only for *small* variations (ghost notes, fills), not core structure.

## Workflow (Recommended)

1) Keep instruments stable: prefer editing **patterns/defs only** unless sound design is the goal.
2) Work in small chunks:
   - one pattern (e.g., 2–8 bars) at a time
   - one role at a time (drums, then bass, then lead)
3) Prefer musical helpers (RFC-0004):
   - `channel_ids` / `instrument_ids` to avoid index mistakes
   - `timebase` + pattern `bars` + `beat_range` to avoid row math
   - `harmony` + `pitch_seq` to avoid out-of-key note spelling
4) Always review the expanded output:
   - run `speccade expand <spec.json>` and sanity-check density, collisions, and note ranges

## Reference Examples

- Minimal example + expected expansion snapshot:
  - `docs/examples/music/compose_minimal_16rows.json`
  - `docs/examples/music/compose_minimal_16rows.expanded.params.json`
- Larger example (eurobeat-ish 4 bars): `docs/examples/music/compose_eurobeat_4bars.json`
- Harmony + octave bass example (RFC-0004): `docs/examples/music/compose_harmony_octave_bass_4bars.json`
