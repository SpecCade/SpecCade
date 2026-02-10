# Music Spec Reference

This document covers tracker module generation in SpecCade.

| Property | Value |
|----------|-------|
| Asset Type | `music` |
| Recipe Kinds | `music.tracker_song_v1` (canonical), `music.tracker_song_compose_v1` (authoring sugar; expanded to canonical) |
| Output Formats | `xm`, `it` |
| Determinism | Tier 1 (byte-identical outputs) |

## SSOT (Source Of Truth)

- Rust types: `crates/speccade-spec/src/recipe/music/`
- Output validation rules: `crates/speccade-spec/src/validation/recipe_outputs.rs`
- Starlark specs: `specs/music/`
- Example: `specs/music/music_tracker.star`

## Outputs

- `outputs[]` must contain at least one entry with `kind: "primary"`.
- For single-output specs, the `primary` output `format` must match `recipe.params.format`.
- You may declare up to two `primary` outputs: one `xm` and one `it`.

## Recipe: `music.tracker_song_v1`

This is the canonical tracker module recipe.

Key params (see `MusicTrackerSongV1Params` for the full schema):

- `format`: `xm` or `it`
- `bpm`: 32-255
- `speed`: 1-31
- `channels`: XM 1-32, IT 1-64
- `instruments`: list of `TrackerInstrument`
- `patterns`: map of pattern name -> `TrackerPattern`
- `arrangement`: list of `ArrangementEntry`
- `automation`: list of `AutomationEntry`
- `it_options`: optional `ItOptions` (IT only)

### Instruments

Each entry in `instruments[]` is a `TrackerInstrument`. You must set exactly one of:

- `ref`: path to an external `audio` spec with `recipe.kind: audio_v1`
- `synthesis_audio_v1`: inline `audio_v1` params baked into a tracker sample
- `wav`: path to a WAV sample file
- `synthesis`: deprecated inline tracker synth (prefer `ref` or `synthesis_audio_v1`)

## Recipe: `music.tracker_song_compose_v1`

This recipe is an authoring layer for dense music specs. It expands deterministically into the
canonical `music.tracker_song_v1` params before generation.

To inspect the expanded tracker params:

```bash
speccade expand --spec <path-to-spec.json>
```

References:

- `docs/music-pattern-ir-quickstart.md`
- `docs/music-pattern-ir-examples.md`
- `docs/music-chord-spec.md`
- `crates/speccade-spec/src/recipe/music/` (Rust SSOT)
- `docs/music-pattern-ir-quickstart.md`
