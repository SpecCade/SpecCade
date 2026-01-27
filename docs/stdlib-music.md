# SpecCade Starlark Standard Library - Music Functions

[← Back to Index](stdlib-reference.md)

> **SSOT:** For complete parameter details, use `speccade stdlib dump --format json`
> or see the Rust types in `crates/speccade-spec/src/recipe/music/`.

Music uses a tracker-style model: instruments play notes in patterns, arranged into songs.

## Functions

| Function | Description |
|----------|-------------|
| `instrument_synthesis(synth_type, duty_cycle, periodic)` | Tracker instrument waveform |
| `tracker_instrument(name, synthesis, wav, ref, base_note, ...)` | Instrument definition |
| `pattern_note(row, note, inst, channel, vol, effect_name, ...)` | Note event |
| `tracker_pattern(rows, notes, data)` | Pattern with notes per channel |
| `arrangement_entry(pattern, repeat)` | Arrangement order entry |
| `it_options(stereo, global_volume, mix_volume)` | IT format options |
| `volume_fade(pattern, channel, start_row, end_row, start_vol, end_vol)` | Volume automation |
| `tempo_change(pattern, row, bpm)` | Tempo automation |
| `tracker_song(format, bpm, speed, channels, instruments, patterns, arrangement, ...)` | Complete song recipe |
| `music_spec(asset_id, seed, output_path, ...)` | Convenience wrapper for full spec |

## Cue Templates

| Function | Description |
|----------|-------------|
| `loop_low(name, bpm, measures, ...)` | Low-intensity loop (exploration, menus) |
| `loop_main(name, bpm, measures, ...)` | Main gameplay loop |
| `loop_hi(name, bpm, measures, ...)` | High-intensity loop (combat, boss) |
| `loop_cue(name, intensity, ...)` | Generic loop with explicit intensity |
| `stinger(name, stinger_type, duration_beats, ...)` | One-shot musical event (victory, pickup, etc.) |
| `transition(name, transition_type, from_intensity, to_intensity, ...)` | Bridge between music states |

[← Back to Index](stdlib-reference.md)
