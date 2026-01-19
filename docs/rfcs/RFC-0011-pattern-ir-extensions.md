# RFC-0011: Pattern IR Extensions for Music Composition

- **Status:** COMPLETED
- **Author:** SpecCade Team
- **Created:** 2026-01-20
- **Target Version:** SpecCade v1.x
- **Dependencies:** RFC-0003 (Music Pattern IR), RFC-0004 (Musical Helpers)
- **Last reviewed:** 2026-01-20

## Summary

This RFC extends the Pattern IR with additional operators for music composition, maintaining determinism and reviewability. The new operators enable more expressive pattern manipulation including reversing, mirroring, interleaving, channel remapping, and conditional note filtering.

**Design principles (from RFC-0003):**

- **Pure data:** JSON only; no embedded scripting
- **Deterministic:** same spec + seed + backend version = identical output
- **Composable:** orthogonal operators that combine cleanly
- **Bounded:** explicit hard limits prevent pathological specs

---

## 1. Motivation

RFC-0003 established the core Pattern IR with operators like `stack`, `concat`, `repeat`, `shift`, `slice`, `emit`, `emit_seq`, `transform`, `prob`, and `choose`. While these cover common cases, several musical patterns require verbose workarounds:

- **Retrograde:** Reversing note order (common in counterpoint and variation)
- **Inversion:** Mirroring pitch around an axis (melodic inversion)
- **Interleaving:** Weaving multiple streams together (call-and-response)
- **Channel remapping:** Moving patterns between channels
- **Conditional filtering:** Selecting notes by pitch, volume, or effect

This RFC adds operators to handle these patterns directly, reducing spec complexity and improving review clarity.

---

## 2. New Pattern Operators

### 2.1 `reverse`

Reverse the time ordering of events within a pattern, keeping each event's relative position.

```json
{
  "op": "reverse",
  "len_rows": 16,
  "body": { /* PatternExpr */ }
}
```

**Semantics:**
- Evaluate `body` to produce cells at rows `r`
- Transform each row: `new_row = (len_rows - 1) - r`
- Cells at row 0 move to row `len_rows - 1`, etc.

**Limits:**
- `len_rows` must be >= 1

### 2.2 `mirror`

Mirror (retrograde inversion) pattern in time - alias for reverse that may later add pitch inversion.

```json
{
  "op": "mirror",
  "len_rows": 16,
  "axis": "time",
  "body": { /* PatternExpr */ }
}
```

**Semantics:**
- `axis: "time"` is equivalent to `reverse`
- Future: `axis: "pitch"` for melodic inversion around a reference note

**Limits:**
- `len_rows` must be >= 1 when `axis: "time"`

### 2.3 `interleave`

Interleave events from multiple parts based on row position.

```json
{
  "op": "interleave",
  "stride": 4,
  "parts": [
    { /* PatternExpr - takes rows 0, 8, 16... */ },
    { /* PatternExpr - takes rows 4, 12, 20... */ }
  ]
}
```

**Semantics:**
- Each part handles rows at its index offset within each stride-sized block
- Part 0 handles rows where `(row / stride) % parts.len() == 0`
- Part 1 handles rows where `(row / stride) % parts.len() == 1`
- etc.

**Use cases:**
- Call-and-response patterns
- Alternating drum voices
- Polyrhythmic textures

**Limits:**
- `stride` must be >= 1
- `parts` must be non-empty, max 16 parts

### 2.4 `remap_channel`

Remap events to different channels.

```json
{
  "op": "remap_channel",
  "from": 0,
  "to": 3,
  "body": { /* PatternExpr */ }
}
```

**Semantics:**
- All cells in `body` with `channel == from` are changed to `channel == to`
- Cells with other channels are unchanged

**Use cases:**
- Moving a pattern to a different instrument/voice
- Creating harmonies by duplicating + remapping + transposing

**Limits:**
- `from` and `to` must be valid channel indices (0-63)

### 2.5 `filter`

Filter events based on criteria.

```json
{
  "op": "filter",
  "criteria": {
    "min_row": 8,
    "max_row": 24,
    "channel": 0,
    "has_note": true
  },
  "body": { /* PatternExpr */ }
}
```

**Criteria fields (all optional, combined with AND):**
- `min_row`: Include only events at row >= value
- `max_row`: Include only events at row < value
- `channel`: Include only events on this channel
- `has_note`: If true, include only events with a note set
- `has_effect`: If true, include only events with an effect set

**Semantics:**
- Evaluate `body`
- Keep only cells matching all specified criteria
- Unspecified criteria are not checked

**Use cases:**
- Extracting subsets of patterns for variation
- Creating ghost notes (filter high-velocity + reduce volume)
- Separating melodic content from percussion

**Limits:**
- At least one criterion must be specified

---

## 3. New Transform Operators

These extend the existing `transform` op's `ops` array.

### 3.1 `invert_pitch`

Melodic inversion around a pivot note.

```json
{
  "op": "invert_pitch",
  "pivot": "C4"
}
```

**Semantics:**
- For each cell with a valid note (not `---`, `...`, `OFF`, etc.)
- Calculate semitone distance from pivot
- Reflect: `new_note = pivot - distance`
- Example: If pivot is C4 and note is E4 (+4 semitones), result is G#3 (-4 semitones)

**Limits:**
- Resulting notes are clamped to valid MIDI range (C-1 to G9)

### 3.2 `quantize_pitch`

Snap notes to a scale.

```json
{
  "op": "quantize_pitch",
  "scale": "major",
  "root": "C"
}
```

**Semantics:**
- For each cell with a valid note
- If note is not in the scale, snap to nearest scale degree
- Ties snap down (toward lower pitch)

**Supported scales:**
- `major`, `minor` (natural minor)
- `harmonic_minor`, `melodic_minor`
- `pentatonic_major`, `pentatonic_minor`
- `chromatic` (no-op)

**Use cases:**
- Ensuring generated melodies fit a key
- "Correcting" pitch sequences to diatonic

### 3.3 `ratchet`

Add retriggering (ratchet) effect to notes.

```json
{
  "op": "ratchet",
  "divisions": 4,
  "seed_salt": "ratchet_fill"
}
```

**Semantics:**
- For cells at rows determined by seed-based selection
- Add a note-retrigger effect (E9x in XM, equivalent in IT)
- `divisions` determines the retrigger speed (1-16)

**Use cases:**
- Drum fills
- Electronic music stutters
- Glitchy textures

**Limits:**
- `divisions` must be 1-16

### 3.4 `arpeggiate`

Apply arpeggio effect to chords/notes.

```json
{
  "op": "arpeggiate",
  "semitones_up": 3,
  "semitones_down": 0
}
```

**Semantics:**
- Add arpeggio effect (0xy in XM/IT)
- `semitones_up`: x nibble (0-15)
- `semitones_down`: y nibble (0-15)

**Use cases:**
- Classic tracker arpeggios
- Chiptune textures

**Limits:**
- Both values must be 0-15

---

## 4. Hard Limits

All new operators respect these limits to prevent pathological specs:

| Limit | Value | Rationale |
|-------|-------|-----------|
| Max recursion depth | 64 | Prevent stack overflow |
| Max cells per pattern | 50,000 | Memory bound |
| Max interleave parts | 16 | Reasonable parallelism |
| Max filter criteria | 5 | Prevent combinatorial explosion |
| Stride range | 1-65535 | u16 compatible |
| Row range | i32 | Signed for negative shifts |
| Channel range | 0-63 | IT format max |

---

## 5. Determinism Guarantees

All new operators are fully deterministic:

- **reverse/mirror:** Pure row arithmetic
- **interleave:** Deterministic part selection based on row index
- **remap_channel:** Pure field rewrite
- **filter:** Deterministic predicate evaluation
- **invert_pitch:** Pure arithmetic on note values
- **quantize_pitch:** Deterministic scale lookup
- **ratchet:** Uses seed-based RNG per RFC-0003
- **arpeggiate:** Pure effect parameter setting

No new operators introduce non-deterministic behavior or external IO.

---

## 6. Schema Changes

Add to `music_pattern_expr` oneOf:

```json
{
  "type": "object",
  "additionalProperties": false,
  "required": ["op", "len_rows", "body"],
  "properties": {
    "op": { "const": "reverse" },
    "len_rows": { "type": "integer", "minimum": 1, "maximum": 65535 },
    "body": { "$ref": "#/definitions/music_pattern_expr" }
  }
}
```

(Similar entries for `mirror`, `interleave`, `remap_channel`, `filter`)

Add to `music_transform_op` oneOf:

```json
{
  "type": "object",
  "additionalProperties": false,
  "required": ["op", "pivot"],
  "properties": {
    "op": { "const": "invert_pitch" },
    "pivot": { "type": "string" }
  }
}
```

(Similar entries for `quantize_pitch`, `ratchet`, `arpeggiate`)

---

## 7. Test Plan

### 7.1 Unit Tests

- `reverse` produces correct row transformation
- `mirror` with `axis: "time"` equals `reverse`
- `interleave` distributes events correctly
- `remap_channel` changes channel field
- `filter` respects all criteria combinations
- `invert_pitch` reflects notes correctly
- `quantize_pitch` snaps to scale
- `ratchet` adds retrigger effect
- `arpeggiate` sets effect parameters

### 7.2 Determinism Tests

- Same seed + spec = identical output for all new operators
- Different seeds with `ratchet` produce different selection

### 7.3 Integration Tests

- Full compose spec using new operators expands correctly
- Generated XM/IT plays correctly in tracker software

---

## 8. Future Work

- `mirror` with `axis: "pitch"` (full retrograde inversion)
- Additional scales for `quantize_pitch`
- Time-stretching operators (half-time, double-time)
- Probability-weighted `filter` criteria
