# RFC-0004: Musical Helpers for Music Compose (Names, Beats, Harmony)

- **Status:** Draft
- **Author:** SpecCade Team
- **Created:** 2026-01-11
- **Target Version:** SpecCade v1.x (Compose extensions)
- **Dependencies:** RFC-0001 (Canonical Spec Architecture), RFC-0003 (Music Pattern IR)
- **Last reviewed:** 2026-01-13

## Summary

RFC-0003 defines a small JSON Pattern IR that expands deterministically into canonical `music.tracker_song_v1` tracker
events.

This RFC proposes **authoring helpers** on top of that IR so authors can write *musical structure* (beats, bars, keys,
chords) without:

- counting raw rows everywhere
- juggling channel/instrument indices
- manually spelling every pitched note (and drifting out of key)

The proposal is influenced by live-coding “pattern combinator” systems (e.g., Strudel/Tidal style composition):

- small, composable operators
- reuse via named fragments
- deterministic “sometimes” / “choose”
- rhythm-first authoring

All helpers compile away during expansion; the canonical output remains `music.tracker_song_v1`.

---

## 1. Goals / Non-Goals

### Goals

- Make music specs **more reviewable** and **less error-prone** than raw tracker event lists.
- Enable authoring in musical units:
  - “4 bars”
  - “kick on beats 1–4”
  - “bass plays chord tones”
- Reduce authoring errors by reducing:
  - token volume
  - numeric bookkeeping
  - pitch spelling mistakes
- Preserve RFC-0001 determinism guarantees (Tier 1: byte-identical XM/IT).

### Non-Goals

- A realtime DAW engine (no audio graph scheduling, no realtime clock).
- Perfect “theory correctness”; this is an authoring substrate with strong defaults and explicit opt-ins for dissonance.

---

## 2. Named Channels and Instruments (Aliases)

### 2.1 Motivation

Channel and instrument indices are a major source of mistakes (off-by-one, refactor churn, misrouting parts).

### 2.2 Proposal

Add optional alias maps to `music.tracker_song_compose_v1` params:

```json
{
  "channel_ids": { "kick": 0, "snare": 1, "bass": 2, "lead": 3, "hats": 5 },
  "instrument_ids": { "kick": 0, "snare": 1, "hat": 2, "bass": 3, "lead": 4 }
}
```

Then allow cell templates to reference channels/instruments by either:

- integer index (existing)
- string alias (new)

Example:

```json
{
  "op": "emit",
  "at": { "op": "range", "start": 0, "step": 4, "count": 4 },
  "cell": { "channel": "kick", "inst": "kick", "note": "C4", "vol": 64 }
}
```

Recommended Rust types:

- `ChannelRef = u8 | String`
- `InstrumentRef = u8 | String`

Resolution happens during expansion:

- unknown alias → error
- alias must map to a valid index (`channel < params.channels`, `inst < instruments.len()`).

---

## 3. Musical Time (Bars/Beats → Rows)

### 3.1 Motivation

Humans and musicians rarely think “row 57”. They think “last half-bar fill”, “eighth-note hats”, “3/4 for one section”.

### 3.2 TimeBase

Add an optional timebase:

```json
{ "beats_per_bar": 4, "rows_per_beat": 4 }
```

Meaning:

- `beats_per_bar`: time signature numerator (common: 3, 4, 5, 7)
- `rows_per_beat`: rhythmic grid resolution (common: 4 for 16ths, 3 for triplet feel)

### 3.3 Pattern length in bars

Allow `ComposePattern` to declare either:

- `rows` (existing) OR
- `bars` (new; expanded to rows using the timebase)

```json
{
  "bars": 4,
  "timebase": { "beats_per_bar": 4, "rows_per_beat": 4 },
  "program": { "op": "ref", "name": "verse" }
}
```

Expansion:

`rows = bars * beats_per_bar * rows_per_beat`

### 3.4 BeatPos and BeatDelta

Define:

```json
{ "bar": 0, "beat": 0, "sub": 0 }
```

Where:

- `bar`: 0-indexed bar number within the pattern
- `beat`: 0-indexed beat within the bar (`0..beats_per_bar-1`, default `0`)
- `sub`: 0-indexed subdivision within the beat (`0..rows_per_beat-1`, default `0`)

And a delta:

```json
{ "beats": 1, "sub": 0 }
```

### 3.5 TimeExpr additions

Add `TimeExpr` operators that compile to row indices:

#### `beat_list`

```json
{ "op": "beat_list", "beats": [ { "bar": 0, "beat": 0 }, { "bar": 0, "beat": 2, "sub": 2 } ] }
```

#### `beat_range`

```json
{
  "op": "beat_range",
  "start": { "bar": 0, "beat": 0, "sub": 0 },
  "step": { "beats": 1, "sub": 0 },
  "count": 4
}
```

Mapping:

`row = ((bar * beats_per_bar) + beat) * rows_per_beat + sub`

Step semantics (recommended):

- Convert `start` to `row_start` using the mapping above.
- Convert `step` to a row delta:
  - `delta_rows = (step.beats * rows_per_beat) + step.sub`
- Emit rows: `row_i = row_start + i * delta_rows` for `i in 0..count`.

This keeps `beat_range` simple and avoids “carry” rules for `(bar, beat, sub)` arithmetic.

### 3.6 Why this supports “custom conventions”

Different songs/genres can choose different:

- `rows_per_beat` (straight vs triplet grid)
- `bars` per pattern (2, 4, 8…)
- even `beats_per_bar` per pattern (e.g., 7/8 break pattern)

The convention is explicit in the spec and compiles to concrete rows.

---

## 4. Harmony Helpers (Keys, Chords, Degrees)

### 4.1 Motivation

Out-of-key pitches are a common failure mode when authoring many absolute note names.

Musicians often think in:

- chord progressions (“this bar is Am”)
- scale degrees (“melody goes 1–2–3–5”)
- chord tones on strong beats (stable) with passing tones elsewhere (motion)

### 4.2 Harmony block

Add an optional harmony block in compose params:

```json
{
  "harmony": {
    "key": { "root": "A", "scale": "minor" },
    "chords": [
      { "at": { "bar": 0 }, "chord": { "symbol": "Am" } },
      { "at": { "bar": 1 }, "chord": { "symbol": "F" } },
      { "at": { "bar": 2 }, "chord": { "symbol": "G" } },
      { "at": { "bar": 3 }, "chord": { "symbol": "E7" } }
    ]
  }
}
```

Chord selection rule: at any event time, use the most recent chord whose `at` is `<=` the event time.

### 4.3 ChordSpec

Chord parsing is specified in:

- `docs/music-chord-spec.md`

Critical requirement: `ChordSpec` MUST support an interval-form escape hatch so any chord can be represented.

### 4.4 Pitch sequences (degree/chord-tone authoring)

Extend `emit_seq` to allow **exactly one** of:

- `note_seq` (existing): absolute note names / MIDI numbers
- `pitch_seq` (new): degrees relative to key or chord

Proposed shape:

```json
{
  "kind": "scale_degree" | "chord_tone",
  "mode": "cycle" | "once",
  "values": ["1", "1", "5", "5", "b7", "b7", "4", "4"],
  "octave": 2
}
```

Rules:

- `scale_degree` maps `1..7` (and optional accidentals like `b3`, `#4`) to the active key scale.
  - default: accidentals are rejected unless explicitly enabled (opt-in dissonance).
- `chord_tone` maps degrees like `1`, `3`, `5`, `7`, `9`, `11`, `13` to tones present in the active chord.
  - default: requesting a tone not present in the chord is an error (strong “no accidental dissonance” default).

Pitch-to-note conversion (recommended):

1) Convert the chosen pitch (root + semitone offset) to an absolute MIDI number using the provided `octave`.
2) Convert MIDI → note name using a **canonical sharp spelling** (e.g., `C#4`, not `Db4`) for stable diffs.
3) Emit that note name into the expanded `music.tracker_song_v1` pattern.

### 4.5 Octave bass and doubling

Octave bass is naturally expressible:

- author the bassline once (often as chord tones)
- stack a transposed copy (+12 semitones)

```json
{
  "op": "stack",
  "merge": "merge_fields",
  "parts": [
    { "op": "ref", "name": "bassline" },
    { "op": "transform", "ops": [ { "op": "transpose_semitones", "semitones": 12 } ], "body": { "op": "ref", "name": "bassline" } }
  ]
}
```

---

## 5. Optional Strudel-Style Authoring Helpers (Non-blocking)

These are useful for “human feel” and for compressing common authoring patterns, but should be treated as optional
extensions.

### 5.0 Strudel-style mapping (informal)

This is an *informal* translation guide for authors coming from Strudel/Tidal-style workflows.

| Concept | Strudel/Tidal-style idea | Compose IR analogue |
|---|---|---|
| Layering | `stack(...)` | `stack` |
| Sequencing | `seq(...)` / `cat(...)` | `concat` |
| Repetition | `fast/slow` + repetition | `repeat` + `shift` (and/or bars/beats timebase) |
| Euclidean rhythms | `euclid(pulses, steps)` | `euclid` (as a `TimeExpr`) |
| Sometimes | `sometimesBy(p, ...)` | `prob` |
| Choose | `choose(...)` | `choose` |
| Mini drum strings | `"x..x..."` | `pattern` (as a `TimeExpr`) |

Compose remains JSON-only and deterministic, but it should be possible to build a Strudel-like *text frontend* that
compiles to the JSON IR later (not part of the spec contract).

### 5.1 Deterministic humanize (volume)

A transform that adjusts `vol` per event deterministically:

- `humanize_vol { min, max, seed_salt }`

This compiles to explicit `vol` values (no runtime randomness).

### 5.2 Swing (timing feel)

Swing can be approximated in a few ways depending on target format and chosen constraints:

- row-level reshaping (move offbeats later by some rows, if grid allows)
- format-specific note-delay style effects (if implemented)

The authoring API should be explicit about the chosen strategy to preserve determinism.

---

## 6. Documentation / Workflow Requirements

To keep this system reviewable:

1) Provide `speccade expand --spec <spec.json>` to show the fully expanded `music.tracker_song_v1` params.
2) Keep examples for:
   - named channels/instruments
   - beat-range rhythms
   - chord-tone bassline + octave doubling
3) Encourage authoring patterns in `defs` and reusing them (Strudel-style composition).

Suggested starter example:

- `docs/examples/music/compose_harmony_octave_bass_4bars.json`
