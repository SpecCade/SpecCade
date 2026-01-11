# Music Pattern IR (Compose) — Quickstart (Draft)

This quickstart describes the proposed JSON “Pattern IR” authoring layer for tracker music described in:

- `docs/rfcs/RFC-0003-music-pattern-ir.md`

The goal is to keep `music.tracker_song_v1` as the canonical, fully-expanded event format, while providing a compact, safe, deterministic way to author patterns (for humans and LLMs).

## Mental Model

- A tracker pattern is a grid: `(row, channel) -> cell`.
- Pattern IR builds that grid by composing small generators (hits, sequences) using a few operators (`stack`, `repeat`, `shift`, …).
- The backend expands Pattern IR into a normal `TrackerPattern.data[]` list, sorted by `(row, channel)`.

## 5-Minute Workflow

1) Start with a working `music.tracker_song_v1` (or a small hand-made tracker idea).
2) Switch to `music.tracker_song_compose_v1`.
3) Turn the biggest repeated blocks into:
   - `emit` + `range` (regular grids)
   - `emit` + `euclid` (polyrhythms / syncopation)
   - `emit_seq` + `range` (melodies/basslines)
4) Extract reusable parts into `defs` (kick pattern, hat pattern, bass ostinato, fill).
5) Use `transform` for transposition and volume scaling instead of duplicating note lists.

## Debugging & Review (Required Tooling)

To keep this system “reviewable”, implement at least one expansion view:

- `speccade expand <spec.json>` → prints expanded `music.tracker_song_v1` params JSON

Recommended PR workflow:

- reviewers look at both the compact compose spec and the expanded JSON snapshot

## Authoring Tips (Human + LLM)

- Prefer `defs` for drum layers and common riffs.
- Prefer `emit_seq` for pitched material (bass/lead) and keep sequences short + cyclic.
- Keep “sound design” (instruments) separate from “music logic” (Pattern IR).
- Use `merge: "error"` at top-level stacks so you find collisions early.
- Use deterministic randomness (`choose`, `prob`) only for *small* variations (ghost notes, fills), not core structure.

## Reference Examples

- Minimal example + expected expansion: `docs/examples/music/compose_minimal_16rows.json`
- Larger example (eurobeat-ish 4 bars): `docs/examples/music/compose_eurobeat_4bars.json`

