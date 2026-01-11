# Music Chord Spec (Draft)

This document defines a deterministic, parseable chord representation for SpecCade’s music authoring layer.

It is intended for **authoring helpers** (e.g., “play chord tones”, “constrain melody to chord”), and is not a
runtime feature of XM/IT.

## Design Goals

- **Deterministic:** chord parsing must be unambiguous and stable.
- **Practical:** common chord symbols should work (`Am`, `F#m7b5`, `Cmaj7#11/G`).
- **Complete escape hatch:** any chord must be representable even if a symbol is not supported.

## ChordSpec

Chord specs should accept one of:

1) **Symbol form** (recommended):

```json
{ "symbol": "Cmaj7#11/G" }
```

2) **Interval form** (guarantees 100% coverage):

```json
{ "root": "C", "intervals": [0, 4, 7, 11, 18], "bass": "G" }
```

Where:

- `root` is a note name like `"C"`, `"F#"`, `"Bb"`.
- `intervals` are semitone offsets from the root (must include `0`).
- `bass` is an optional slash-bass note name (used for inversion / bassline targeting).

## Symbol Grammar (Suggested)

This is intentionally practical rather than “musicology complete”. It aims to cover common lead-sheet symbols with
predictable parsing.

### Root

`[A-G]` with optional accidental:

- `#` (sharp)
- `b` (flat)

Examples: `C`, `F#`, `Bb`.

Note spelling policy (recommended):

- The parser should accept both sharps and flats in input symbols.
- When emitting note names from intervals, use a **canonical sharp spelling** (`C#`/`D#`/...) for stable diffs.

### Quality (triad)

Defaults to major triad if omitted.

Supported qualities and base triad intervals:

| Quality token | Meaning | Triad intervals |
|---|---|---|
| *(none)* | major | `[0, 4, 7]` |
| `m` / `min` | minor | `[0, 3, 7]` |
| `dim` | diminished | `[0, 3, 6]` |
| `aug` / `+` | augmented | `[0, 4, 8]` |
| `sus2` | suspended 2 | `[0, 2, 7]` |
| `sus4` / `sus` | suspended 4 | `[0, 5, 7]` |
| `5` | power chord | `[0, 7]` |

### Sevenths

If a 7th is present:

| Token | Adds interval |
|---|---|
| `7` | `10` (dominant / minor 7th) |
| `maj7` | `11` (major 7th) |
| `m7` | `10` (minor 7th, implies minor triad) |
| `dim7` | `9` (diminished 7th, implies diminished triad) |
| `m7b5` / `ø7` | triad `[0,3,6]` + `10` (half-diminished) |

### Extensions and Adds

- `6` adds `9` (major 6th) unless already present.
- `9`, `11`, `13` add their *stacked* tones (by default):
  - `9` adds `14`
  - `11` adds `17`
  - `13` adds `21`

If you want “add without seventh”, use `add9`, `add11`, `add13`.

### Alterations

Alterations modify (or add) chord degrees:

| Token | Interval |
|---|---|
| `b5` / `#5` | `6` / `8` |
| `b9` / `#9` | `13` / `15` |
| `#11` | `18` |
| `b13` | `20` |

### Omissions

Omissions remove chord tones:

- `no3` removes `3`/`4`/`5` depending on the quality’s third.
- `no5` removes the fifth (6/7/8 depending on the quality’s fifth).

### Slash Bass

`/` followed by a note name indicates the bass note:

- `Cmaj7/G` (G in the bass)

This does not change `root`; it influences “bassline within chord” helpers.

## Parsing Rules (Recommended)

1) Parse `root`.
2) Parse quality → base triad intervals.
3) Parse seventh tokens (if any).
4) Parse extensions / adds.
5) Parse alterations / omissions.
6) Parse slash-bass (if any).
7) Normalize:
   - dedupe intervals
   - sort ascending
   - ensure `0` is present

## Examples (Symbol → Parsed Form)

These examples show the intended *deterministic* interpretation as `root + intervals (+ optional bass)`.

- `C` → `root: "C"`, `intervals: [0, 4, 7]`
- `Am` → `root: "A"`, `intervals: [0, 3, 7]`
- `G7` → `root: "G"`, `intervals: [0, 4, 7, 10]`
- `Dsus4` → `root: "D"`, `intervals: [0, 5, 7]`
- `F#m7b5` → `root: "F#"`, `intervals: [0, 3, 6, 10]`
- `Cmaj7#11/G` → `root: "C"`, `intervals: [0, 4, 7, 11, 18]`, `bass: "G"`

## Why “100% mapping of ALL chords” needs the interval form

Chord symbols in the wild are not a closed set; new spellings and conventions appear constantly.

The **interval form** guarantees that *any* chord can be represented exactly and deterministically even when the symbol
parser does not recognize a spelling.
