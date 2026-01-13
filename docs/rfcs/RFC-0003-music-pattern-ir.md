# RFC-0003: Music Pattern IR ("Compose") for Tracker Songs

- **Status:** COMPLETED
- **Author:** SpecCade Team
- **Created:** 2026-01-11
- **Target Version:** SpecCade v1.x
- **Dependencies:** RFC-0001 (Canonical Spec Architecture)
- **Last reviewed:** 2026-01-13

## Summary

This RFC proposes a safe, deterministic, JSON-based **authoring layer** for tracker music. Instead of writing large, fully-expanded pattern event lists directly, spec authors write compact **Pattern IR** (a small JSON AST) that is expanded deterministically into the existing `music.tracker_song_v1` event format before XM/IT generation.

The intent is to make “assets-as-code” practical for dense symbolic assets like music, without executing untrusted code.

**Design principles:**

- **Pure data:** JSON only; no embedded scripting or evaluation of foreign code
- **Deterministic:** same spec + seed + backend version → identical expanded events → identical XM/IT bytes
- **Composable:** a small set of orthogonal operators (`stack`, `repeat`, `euclid`, …)
- **Debuggable:** expansion can be inspected and snapshot-tested

---

## 1. Motivation

Tracker music is “dense”: a 60-second song often contains hundreds to thousands of note events. Fully-expanded JSON (e.g. one object per hit) becomes:

- hard for humans to read/write/review
- expensive and failure-prone to produce reliably
- noisy in diffs (small musical changes → large line churn)

Previous “foreign Python” approaches offered expressiveness (loops, `%`, helper functions) but introduce high security and maintainability risks.

This RFC aims to capture the *useful parts* of “code” (reuse, repetition, variation, controlled randomness) while keeping specs safe and deterministic.

---

## 2. Goals / Non-Goals

### 2.1 Goals

- Shrink common music specs by **10–100×** versus fully-expanded event lists.
- Enable high-level patterns: repetition, fills, Euclidean rhythms, transposition, deterministic “sometimes”.
- Preserve `music.tracker_song_v1` as the **canonical expanded representation** (good for caching, reports, tooling).
- Make it easy to add new operators over time with minimal breakage.

### 2.2 Non-Goals

- A general-purpose programming language (no unbounded loops, recursion, IO, or user-defined functions).
- Micro-timing beyond tracker capabilities (future work may map “swing” to format-specific delay effects).
- Automatic “good taste” composition by itself; this is an authoring substrate, not an automatic composition system.

---

## 3. Architecture: Two-Layer Music Specs

### 3.1 Canonical Layer (unchanged)

`music.tracker_song_v1` remains the canonical, explicit event format (today’s generator input).

### 3.2 Authoring Layer (new)

Introduce a new recipe kind:

- `music.tracker_song_compose_v1`

This kind accepts compact Pattern IR and expands to `music.tracker_song_v1` internally before generating XM/IT.

Rationale: keep the canonical format stable while allowing iteration on authoring features.

---

## 4. Schema Design

### 4.1 Recipe Kind

```json
{
  "recipe": {
    "kind": "music.tracker_song_compose_v1",
    "params": { /* see below */ }
  }
}
```

### 4.2 Params: `MusicTrackerSongComposeV1Params`

`MusicTrackerSongComposeV1Params` mirrors `music.tracker_song_v1` with the following changes:

- `patterns` maps to `ComposePattern` instead of `TrackerPattern`
- adds optional `defs` for reusable fragments

```json
{
  "format": "xm",
  "bpm": 155,
  "speed": 6,
  "channels": 8,
  "instruments": [ /* unchanged TrackerInstrument[] */ ],
  "defs": {
    "four_on_floor": { "op": "emit", "at": { "op": "range", "start": 0, "step": 16, "count": 4 }, "cell": { "channel": 0, "note": "C4", "inst": 3, "vol": 64 } }
  },
  "patterns": {
    "intro": {
      "rows": 64,
      "program": { "op": "ref", "name": "four_on_floor" }
    }
  },
  "arrangement": [ { "pattern": "intro", "repeat": 2 } ]
}
```

### 4.3 `ComposePattern`

```json
{
  "rows": 64,
  "program": { /* PatternExpr */ },

  /* optional: allow “hand tweaks” on top of expanded output */
  "data": [ /* optional PatternNote[] (flat) */ ],
  "notes": { /* optional channel-keyed format */ }
}
```

**Expansion rule:** `program` expands first; then `data`/`notes` are merged **on top** as explicit overrides (equivalent to `merge: "last_wins"` with hand-authored cells applied last).

---

## 5. Pattern IR

### 5.1 Overview

Pattern IR is a small JSON AST. Each node has:

- `op`: operator tag
- operator-specific fields

Evaluation produces a set of **cells** indexed by `(row, channel)`, where each cell has up to the tracker fields:

- `note`, `inst`, `vol`, `effect`, `param`, `effect_name`, `effect_xy`

The result is serialized as a standard `TrackerPattern` (`data` form), sorted by `(row, channel)`.

### 5.2 Cell Templates

IR emission operators build **cell templates** that look like `PatternNote`, but without `row` (row comes from `TimeExpr`).

Recommended `CellTemplate` shape:

```json
{
  "channel": 5,
  "note": "C1",
  "inst": 4,
  "vol": 48,
  "effect": 0,
  "param": 0,
  "effect_name": "arpeggio",
  "effect_xy": [1, 2]
}
```

Rules:

- `channel` is required for `emit` / `emit_seq`.
- `note` may be omitted or empty to mean “trigger instrument base note” (mirrors `music.tracker_song_v1` behavior).
- `inst` is required for emitted cells in v1 (mirrors `music.tracker_song_v1` today).
- Special tracker tokens like `"---"`, `"..."`, `"OFF"` are preserved as-is.

### 5.3 Merge / Conflict Policy

When multiple sources target the same `(row, channel)`, merge is defined **fieldwise**:

- `merge_fields`: for each field, if both sides set a value:
  - if values are equal → OK
  - if values differ → error
  - if only one side sets a value → take it
- `last_wins`: same as `merge_fields`, except conflicts are resolved by taking the later writer’s value.
- `error`: error on any double-write to the same cell (even if fields would have merged).

Notes:

- In canonical `music.tracker_song_v1`, `note` and `inst` are always present on `PatternNote` (even if they default), so most “layering” in v1 should use either:
  - non-overlapping `(row, channel)` targets, or
  - `merge: "last_wins"` for explicit overrides (e.g., velocity shaping).

Operators that combine patterns (`stack`, `concat`, post-merge overrides) accept:

```json
{ "merge": "error" | "last_wins" | "merge_fields" }
```

Recommended default: `merge_fields` for internal layering + `error` at the top-level to surface mistakes early, with `last_wins` reserved for intentionally-overlapping override layers.

---

## 6. Operators (v1 Set)

This section defines the minimal operator set required to replace “write 1000 rows by hand”.

### 6.1 Structural operators

#### 6.1.1 `stack`

Overlay multiple expressions into one pattern.

```json
{ "op": "stack", "parts": [ /* PatternExpr[] */ ], "merge": "merge_fields" }
```

#### 6.1.2 `concat`

Concatenate parts in time. Each part is evaluated in its own time window and shifted forward.

```json
{
  "op": "concat",
  "parts": [
    { "len_rows": 16, "body": { /* PatternExpr */ } },
    { "len_rows": 16, "body": { /* PatternExpr */ } }
  ]
}
```

`concat` requires each part to declare `len_rows`. The total concatenated length must not exceed the pattern’s `rows`.

#### 6.1.3 `repeat`

Repeat a sub-expression `times` with an implicit time shift by `len_rows`.

```json
{ "op": "repeat", "times": 4, "len_rows": 16, "body": { /* PatternExpr */ } }
```

#### 6.1.4 `shift`

Shift all produced cells by `rows` (can be negative).

```json
{ "op": "shift", "rows": 8, "body": { /* PatternExpr */ } }
```

#### 6.1.5 `slice`

Keep only events in `[start, start+len)`.

```json
{ "op": "slice", "start": 0, "len": 32, "body": { /* PatternExpr */ } }
```

#### 6.1.6 `ref`

Reference a reusable fragment from `params.defs`.

```json
{ "op": "ref", "name": "four_on_floor" }
```

Rules:

- `name` must exist in `defs` (otherwise error).
- `ref` may point to another `ref` (nesting allowed).
- Cycles are errors (e.g., `a -> b -> a`).

---

### 6.2 Event emission

#### 6.2.1 `emit`

Emit a cell template at positions defined by a `TimeExpr`.

```json
{
  "op": "emit",
  "at": { /* TimeExpr */ },
  "cell": {
    "channel": 5,
    "note": "C1",
    "inst": 4,
    "vol": 48
  }
}
```

`cell.note` supports special tracker tokens like `"---"`, `"..."`, `"OFF"`, etc.

#### 6.2.2 `emit_seq`

Emit a sequence of values aligned to the generated time positions.

```json
{
  "op": "emit_seq",
  "at": { "op": "range", "start": 0, "step": 4, "count": 8 },
  "cell": { "channel": 3, "inst": 1, "vol": 56 },
  "note_seq": { "mode": "cycle", "values": ["F1", "C2", "G1", "D2"] }
}
```

The `*_seq` fields use:

```json
{ "mode": "cycle" | "once", "values": [ /* scalars */ ] }
```

Semantics:

- Evaluate `at` → `N` positions.
- `mode: "cycle"` uses `values[i % values.len()]` (error if `values` is empty).
- `mode: "once"` requires `values.len() == N` (otherwise error).

---

### 6.3 Time expressions (`TimeExpr`)

#### 6.3.1 `range`

```json
{ "op": "range", "start": 0, "step": 1, "count": 64 }
```

#### 6.3.2 `list`

```json
{ "op": "list", "rows": [0, 3, 8, 11, 16, 19] }
```

#### 6.3.3 `euclid`

Euclidean rhythm: `pulses` hits distributed across `steps`. `stride` scales steps to rows.

```json
{ "op": "euclid", "pulses": 5, "steps": 16, "rotate": 0, "stride": 4, "offset": 0 }
```

Suggested algorithm:

- Generate a Bjorklund pattern of length `steps` with `pulses` hits.
- Convert hit indices to row positions: `row = (idx * stride) + offset`.
- Apply rotation as a step-index rotate before stride/offset.

#### 6.3.4 `pattern` (mini-notation)

Optional convenience: interpret a compact string where:

- `x` = emit
- `.` = rest

```json
{ "op": "pattern", "pattern": "x...x...x...x...", "stride": 1, "offset": 0 }
```

This is intentionally tiny (not a general DSL) and compiles to a `list`.

---

### 6.4 Transforms and variation

#### 6.4.1 `transform`

Apply one or more transformations to produced cells.

```json
{
  "op": "transform",
  "ops": [
    { "op": "transpose_semitones", "semitones": 12 },
    { "op": "vol_mul", "mul": 3, "div": 4 }
  ],
  "body": { /* PatternExpr */ }
}
```

Required transforms in v1:

- `transpose_semitones` (skip special notes like `"---"`)
- `vol_mul` (integer ratio to avoid float nondeterminism)
- `set` (set `inst`, `vol`, `effect*` fields when missing)

#### 6.4.2 `prob`

Deterministically drop events with probability `p_permille` (0–1000).

```json
{ "op": "prob", "p_permille": 250, "seed_salt": "hat_ghosts", "body": { /* PatternExpr */ } }
```

#### 6.4.3 `choose`

Choose one branch deterministically based on RNG.

```json
{
  "op": "choose",
  "seed_salt": "fill",
  "choices": [
    { "weight": 3, "body": { "op": "ref", "name": "no_fill" } },
    { "weight": 1, "body": { "op": "ref", "name": "fill_1" } }
  ]
}
```

---

## 7. Determinism and RNG

All randomness uses PCG32 per RFC-0001.

For v1, `prob` and `choose` MUST include a stable `seed_salt`. RNG seeding is derived from:

- `spec.seed`
- `pattern_name`
- `seed_salt` (required for `prob` / `choose`)

Recommendation: implement a helper `derive_u32(spec_seed, pattern_name, seed_salt)` using BLAKE3 truncation.

---

## 8. Safety and Resource Limits

Backends MUST enforce limits to prevent pathological specs:

- Max expanded cells per pattern (e.g. 50,000)
- Max recursion depth (e.g. 64)
- Max `TimeExpr.list.rows` length (e.g. 50,000)
- Max `pattern` string length (e.g. 100,000 chars)

On violation: return a typed error with a stable error code.

---

## 9. CLI / Debuggability

Add at least one of:

1) `speccade expand --spec <spec.json>` → prints expanded `music.tracker_song_v1` params JSON
2) `speccade generate --emit-expanded-json <path>` → writes expanded params as metadata output

Expanded JSON is intended for:

- review diffs in PRs
- snapshot tests
- troubleshooting conflicts

---

## 10. Validation Strategy

Validation happens in two phases:

1) **Static validation:** schema shape, required fields, basic ranges
2) **Expansion validation:** expand with limits; validate:
   - `row < rows`
   - `channel < params.channels`
   - `inst < instruments.len()`
   - merge conflicts handled per policy

Errors should report enough context to locate the failing node (pattern name + node path).

---

## 11. Test Plan (Acceptance Criteria)

### 11.1 Unit tests (pure expansion)

- `range`, `list`, `euclid`, `pattern` produce expected row sets
- `emit` and `emit_seq` align sequences correctly (`cycle` vs `once`)
- `stack` merge policies: `error`, `merge_fields`, `last_wins`
- `transpose_semitones` note parsing + special-note passthrough
- RNG determinism: same seed → same output; different `seed_salt` → different output

### 11.2 Integration tests (end-to-end)

- A `music.tracker_song_compose_v1` spec expands to a known `music.tracker_song_v1` JSON snapshot.
- Generating XM/IT from compose spec matches generating from the expanded spec (byte-identical) for a small song.

---

## 12. Future Extensions (Non-blocking)

- Musical authoring helpers (named channels/instruments, bars/beats timebase, chord/degree pitch helpers): see RFC-0004.
- Scale/key-aware `degree_seq` and chord helpers
- “Swing” mapped to XM/IT delay effects when available
- Phrase libraries (“genre kits”) implemented as data packages, not code
- Optional text DSL frontend that compiles to this JSON IR (not part of the spec contract)
