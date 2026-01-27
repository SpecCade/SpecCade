# Music Pattern IR (Compose) — Examples (Draft)

These examples are written against the proposed `music.tracker_song_compose_v1` recipe kind from:

- `crates/speccade-spec/src/recipe/music/` (Rust SSOT, implements completed RFC-0003/0004)

They are intended to be used as:

- implementation targets (unit tests + integration tests)
- “golden” inputs for future corpus expansion
- templates for authors

---

## Example 1: Minimal Beat (16 rows) + Expected Expansion

Compose spec:

- `docs/examples/music/compose_minimal_16rows.json`

Expected expanded `music.tracker_song_v1` params:

- `docs/examples/music/compose_minimal_16rows.expanded.params.json`

This example is deliberately tiny so it can be used as a snapshot test for:

- `range`
- `emit`
- `stack`
- merge behavior across channels

---

## Example 2: Bassline With `emit_seq` (Cycle)

Goal: place an 8-step bassline on every 4th row (16th-notes grid).

```json
{
  "rows": 64,
  "program": {
    "op": "emit_seq",
    "at": { "op": "range", "start": 0, "step": 4, "count": 16 },
    "cell": { "channel": 2, "inst": 3, "vol": 56 },
    "note_seq": {
      "mode": "cycle",
      "values": ["F1", "F1", "C2", "C2", "G1", "G1", "D2", "D2"]
    }
  }
}
```

Notes:

- For variation, wrap the body in `transform` (`transpose_semitones`) and reuse the same `note_seq`.

---

## Example 3: Eurobeat-ish 4 Bars (Layered Drums + Bass + Lead)

Full compose spec:

- `docs/examples/music/compose_eurobeat_4bars.json`

What this example demonstrates:

- `defs` + `ref` for reuse (drum layers, fills)
- `emit` for regular grids (hats, backbeat snare)
- `emit_seq` for bass/lead motifs
- `choose` for an end-of-phrase fill
- nested `stack` with `merge: "last_wins"` for intentional overrides (e.g., ghost hats overriding volume)

Suggested tests for this example (once implemented):

- Expand and ensure no merge conflicts with `merge: "error"` at top-level.
- Generate XM from compose spec and compare bytes to XM from the expanded spec.

---

## Example 4: Harmony + Chord Tones + Octave Bass (Draft)

Compose spec:

- `docs/examples/music/compose_harmony_octave_bass_4bars.json`

What this example demonstrates:

- `timebase` + pattern length in bars
- `channel_ids` / `instrument_ids` aliases
- `harmony` chord progression
- chord-tone authoring + octave doubling via transposition
