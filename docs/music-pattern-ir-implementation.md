# Music Pattern IR (Compose) — Implementation Checklist (Draft)

This document is a “build guide” for implementing `music.tracker_song_compose_v1` as specified in:

- `docs/rfcs/RFC-0003-music-pattern-ir.md`
- `docs/rfcs/RFC-0004-music-compose-musical-helpers.md` (optional)

It is written so an implementation agent can follow it end-to-end and land a working system with tests.

---

## 1. High-Level Acceptance Criteria

1) A spec with `recipe.kind: "music.tracker_song_compose_v1"` expands deterministically into a valid `music.tracker_song_v1` params JSON.
2) `speccade generate` can generate XM/IT from compose specs.
3) An expansion view exists:
   - `speccade expand <spec.json>` → prints expanded `music.tracker_song_v1` params JSON
4) Unit tests cover expansion correctness + determinism.
5) Integration tests confirm generating from compose == generating from expanded (byte-identical XM/IT).

---

## 2. Data Model (Rust Types)

### 2.1 New recipe kind

Add:

- `music.tracker_song_compose_v1`

Changes:

- `speccade/crates/speccade-spec/src/recipe/mod.rs`:
  - extend `RecipeKind`
  - extend `Recipe::parse_kind`
  - add `Recipe::as_music_tracker_song_compose()` helper

### 2.2 New params struct

In `speccade/crates/speccade-spec/src/recipe/music/` add:

- `MusicTrackerSongComposeV1Params`
  - mostly mirrors `MusicTrackerSongV1Params`
  - replaces `patterns: HashMap<String, ComposePattern>`
  - adds `defs: HashMap<String, PatternExpr>` (optional)

Add supporting structs/enums (all `#[serde(deny_unknown_fields)]`):

- `ComposePattern { rows, program, notes?, data? }`
- `PatternExpr` (tagged union by `op`)
- `TimeExpr` (tagged union by `op`)
- `Seq<T> { mode, values }` for `emit_seq`
- `MergePolicy` enum (`error`, `merge_fields`, `last_wins`)

Optional (RFC-0004) additions:

- `ChannelRef` / `InstrumentRef` (untagged: `u8 | String`) + alias maps in params
- `TimeBase { beats_per_bar, rows_per_beat }` + `ComposePattern.bars`
- `BeatPos { bar, beat, sub }` + `BeatDelta { beats, sub }` + `TimeExpr::beat_range`, `TimeExpr::beat_list`
- `Harmony { key, chords[] }` + `ChordSpec` (see `docs/music-chord-spec.md`)
- `PitchSeq` for `emit_seq` (`scale_degree` / `chord_tone`)

Recommendation: keep the operator set minimal for v1:

- structural: `stack`, `concat`, `repeat`, `shift`, `slice`, `ref`
- emit: `emit`, `emit_seq`
- time: `range`, `list`, `euclid`, `pattern`
- variation: `prob`, `choose`
- (optional v1): `transform` with `transpose_semitones`, `vol_mul`, `set`

---

## 3. JSON Schema Updates

Update `speccade/schemas/speccade-spec-v1.schema.json`:

- allow recipe kind `music.tracker_song_compose_v1`
- define schema for `MusicTrackerSongComposeV1Params` and Pattern IR nodes
- keep schemas strict (disallow unknown fields), matching serde `deny_unknown_fields`

Also update docs tables if desired (already drafted in `docs/spec-reference/music.md`).

---

## 4. Expansion Implementation (Core Logic)

### 4.1 Where the expander should live

Recommended location:

- `speccade/crates/speccade-backend-music/src/compose.rs` (new)

Rationale:

- music-specific logic stays in the music backend
- CLI can remain a thin dispatcher

### 4.2 Public API shape

Add a function that turns compose params into canonical params:

- `fn expand_compose(params: &MusicTrackerSongComposeV1Params, seed: u32) -> Result<MusicTrackerSongV1Params, ExpandError>`

Then modify the existing entrypoint:

- `speccade-backend-music::generate::generate_music(...)`

…to dispatch:

- `tracker_song_v1` → generate directly
- `tracker_song_compose_v1` → expand → generate

### 4.3 Merge semantics (important)

Implement a cell type internally:

- key: `(row: u16, channel: u8)`
- value: `Cell { note, inst, vol?, effect?, param?, effect_name?, effect_xy? }`

Merging:

- `merge_fields`: fieldwise merge; error on conflicts (see RFC-0003)
- `last_wins`: deterministic by evaluation order (documented)
- `error`: error on any double-write to the same cell

Expansion should produce a `TrackerPattern { rows, data: Some(Vec<PatternNote>) }` with:

- one `PatternNote` per occupied cell
- `PatternNote.channel = Some(channel)`

### 4.4 Deterministic RNG

Use PCG32 seeded via RFC-0001’s derivation style:

- base: `spec.seed`
- include: pattern name + `seed_salt` (required for `prob` / `choose`)

Recommendation: derive a per-node seed via `truncate_u32(BLAKE3(spec_seed || pattern_name || seed_salt))` and initialize a fresh PCG32 stream from that seed.

Avoid floating-point probabilities:

- `prob.p_permille: u16` (0..=1000)

### 4.5 Resource limits

Enforce at expansion time:

- recursion depth limit
- max expanded cells per pattern
- max time list sizes

Return typed errors with stable error codes.

---

## 5. CLI Wiring

### 5.1 Dispatch

Update `speccade/crates/speccade-cli/src/dispatch.rs` so `generate` supports the new kind.

Implementation path:

1) Parse `recipe.params` as `MusicTrackerSongComposeV1Params`
2) Expand to `MusicTrackerSongV1Params`
3) Reuse existing music generation pipeline (XM/IT)

### 5.2 `speccade expand`

Add a new CLI subcommand:

- `speccade expand <spec.json>`

Behavior:

- if spec kind is `music.tracker_song_compose_v1`, print expanded `music.tracker_song_v1` **params** JSON
- for other kinds: either error or print “not supported”

This command enables:

- snapshot tests (stable expanded JSON)
- PR review diffs

---

## 6. Tests

### 6.1 Unit tests (expansion)

Add unit tests in one of:

- `speccade/crates/speccade-backend-music/src/compose.rs` (module tests)
- `speccade/crates/speccade-spec/src/recipe/music/tests_advanced.rs` (if expansion is placed in spec crate)

Minimum unit coverage:

- `range` / `list` / `euclid` / `pattern`
- `emit` / `emit_seq` (`cycle` vs `once`)
- merge policies
- `defs` + `ref` resolution and missing-ref errors
- deterministic `prob` / `choose`

### 6.2 Integration tests

Add tests under `speccade/crates/speccade-tests/tests/`:

- Expand `docs/examples/music/compose_minimal_16rows.json` and compare to:
  - `docs/examples/music/compose_minimal_16rows.expanded.params.json`
- Generate XM from the compose spec and from the expanded params-as-v1 spec and assert:
  - bytes identical (Tier 1 determinism)

---

## 7. Golden Corpus (Optional, After Implementation)

Once compose is implemented and stable:

- add a compose spec to `golden/speccade/specs/music/`
- add an expected expanded JSON snapshot to `golden/.../reports/` or a dedicated golden-expansion folder

---

## 8. Recommended Implementation Order

1) Spec types + schema + parsing (`speccade-spec`)
2) Expansion engine + unit tests (`speccade-backend-music`)
3) CLI dispatch + `speccade expand` (`speccade-cli`)
4) Integration tests (`speccade-tests`)
5) Docs polish + golden corpus
