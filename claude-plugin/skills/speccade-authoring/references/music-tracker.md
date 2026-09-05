# Tracker music authoring

SpecCade can generate XM/IT tracker music. Reuse a published music example and confirm the complete `music.tracker_song_v1` or compose contract in `crates/speccade-spec/src/recipe/music/`. Do not infer JSON shapes or effect encoding from a tracker application's UI.

- Instrument/sample setup: `instrument.rs` and `common.rs` in that directory.
- Patterns and notes: `pattern.rs`.
- Effects: `effects/` (XM and IT encodings differ).
- Arrangement/compose: `arrangement.rs`, `compose.rs`; see [compose reference](music-compose-ir.md).
- Broader guide: `docs/spec-reference/music.md`; current helper discovery: `speccade stdlib dump --format json`.

Validate the complete song, generate into an explicit root, decode in the intended player, then listen to instrument mapping, note timing, effects and loop transitions. Technical success does not establish a finished composition.

## Nethercore samples

Nethercore first extracts embedded XM/IT samples, converts them through its tracker conversion path, and merges them with explicit sounds. Non-empty instrument references must resolve in that combined sound map; separate WAV declarations are not always required. Prefer stable lowercase IDs; inspect naming/deduplication diagnostics and reject conflicting same-ID content. Explicit WAV sources still need 22,050 Hz mono signed 16-bit PCM. See `tools/nether-cli/src/pack/assets/mod.rs` and `audio.rs` in Nethercore.

Pack raw XM/IT plus any necessary explicit WAVs via `nether.toml`. Verify `rom_tracker` handle resolution and actual music playback, not only SpecCade generation or a successful pack. See [ZX integration](nethercore-zx-integration.md).
