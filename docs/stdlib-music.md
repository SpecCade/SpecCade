# SpecCade Starlark Standard Library - Music Functions

[← Back to Index](stdlib-reference.md)

## Music Functions

Music functions provide tracker-style music composition with instruments, patterns, and arrangements.

## Table of Contents
- [Instrument Functions](#instrument-functions)
- [Pattern Functions](#pattern-functions)
- [Song Functions](#song-functions)
- [Cue Template Functions](#cue-template-functions)

---

## Instrument Functions

### instrument_synthesis()

Creates a tracker instrument synthesis configuration.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| synth_type | str | Yes | - | "pulse", "square", "triangle", "sawtooth", "sine", "noise" |
| duty_cycle | f64 | No | 0.5 | Duty cycle for pulse wave (0.0-1.0) |
| periodic | bool | No | False | For noise: use periodic noise for more tonal sound |

**Returns:** Dict matching the InstrumentSynthesis IR structure.

**Example:**
```python
instrument_synthesis("pulse", 0.25)
instrument_synthesis("square")
instrument_synthesis("noise", periodic = True)
```

### tracker_instrument()

Creates a tracker instrument definition.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| name | str | Yes (named) | - | Instrument name |
| synthesis | dict | No | None | Synthesis config (mutually exclusive with wav, ref) |
| wav | str | No | None | Path to WAV sample file |
| ref | str | No | None | Reference to external spec file |
| base_note | str | No | None | Base note for pitch correction (e.g., "C4", "A#3") |
| sample_rate | int | No | None | Sample rate for synthesized instruments |
| envelope | dict | No | None | ADSR envelope dict (from envelope()) |
| loop_mode | str | No | None | "auto", "none", "forward", "pingpong" |
| default_volume | int | No | None | Default volume (0-64) |
| comment | str | No | None | Optional comment |

**Returns:** Dict matching the TrackerInstrument IR structure.

**Example:**
```python
tracker_instrument(
    name = "bass",
    synthesis = instrument_synthesis("sawtooth"),
    envelope = envelope(0.01, 0.1, 0.7, 0.2)
)
tracker_instrument(name = "sample", wav = "samples/kick.wav")
```

---

## Pattern Functions

### pattern_note()

Creates a pattern note event.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| row | int | Yes | - | Row number (0-indexed) |
| note | str | Yes | - | Note name (e.g., "C4", "A#3") or "---" for note off |
| inst | int | Yes | - | Instrument index (0-indexed) |
| channel | int | No | None | Channel number (0-indexed, for flat array format) |
| vol | int | No | None | Volume (0-64) |
| effect | int | No | None | Effect command number |
| param | int | No | None | Effect parameter |
| effect_name | str | No | None | Named effect (e.g., "arpeggio", "portamento") |
| effect_xy | list | No | None | Effect XY parameters as [X, Y] |

**Returns:** Dict matching the PatternNote IR structure.

**Example:**
```python
pattern_note(0, "C4", 0)
pattern_note(4, "E4", 0, vol = 48)
pattern_note(8, "G4", 0, effect_name = "arpeggio", effect_xy = [3, 7])
```

### tracker_pattern()

Creates a tracker pattern definition.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| rows | int | Yes | - | Number of rows in the pattern (typically 64) |
| notes | dict | No | None | Dict of channel -> list of notes (channel-keyed format) |
| data | list | No | None | List of notes with channel field (flat array format) |

**Returns:** Dict matching the TrackerPattern IR structure.

**Example:**
```python
# Channel-keyed format (preferred)
tracker_pattern(64, notes = {
    "0": [pattern_note(0, "C4", 0), pattern_note(16, "E4", 0)],
    "1": [pattern_note(0, "G4", 1)]
})

# Flat array format
tracker_pattern(64, data = [
    pattern_note(0, "C4", 0, channel = 0),
    pattern_note(0, "G4", 1, channel = 1)
])
```

### arrangement_entry()

Creates an arrangement entry.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| pattern | str | Yes | - | Pattern name to play |
| repeat | int | No | 1 | Number of times to repeat |

**Returns:** Dict matching the ArrangementEntry IR structure.

**Example:**
```python
arrangement_entry("intro")
arrangement_entry("verse", 4)
arrangement_entry("chorus", repeat = 2)
```

---

## Song Functions

### it_options()

Creates IT-specific module options.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| stereo | bool | No | True | Enable stereo output |
| global_volume | int | No | 128 | Global volume (0-128) |
| mix_volume | int | No | 48 | Mix volume (0-128) |

**Returns:** Dict matching the ItOptions IR structure.

**Example:**
```python
it_options()
it_options(stereo = True, global_volume = 100)
```

### volume_fade()

Creates a volume fade automation entry.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| pattern | str | Yes | - | Target pattern name |
| channel | int | Yes | - | Target channel (0-indexed) |
| start_row | int | Yes | - | Start row |
| end_row | int | Yes | - | End row |
| start_vol | int | Yes | - | Start volume (0-64) |
| end_vol | int | Yes | - | End volume (0-64) |

**Returns:** Dict matching the AutomationEntry::VolumeFade IR structure.

**Example:**
```python
volume_fade("verse", 0, 0, 64, 64, 0)  # Fade out
```

### tempo_change()

Creates a tempo change automation entry.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| pattern | str | Yes | - | Target pattern name |
| row | int | Yes | - | Row for tempo change |
| bpm | int | Yes | - | New BPM (32-255) |

**Returns:** Dict matching the AutomationEntry::TempoChange IR structure.

**Example:**
```python
tempo_change("bridge", 32, 140)
```

### tracker_song()

Creates a complete tracker song recipe.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| format | str | Yes (named) | - | Tracker format: "xm" or "it" |
| bpm | int | Yes (named) | - | Beats per minute (30-300) |
| speed | int | Yes (named) | - | Tracker speed / ticks per row (1-31) |
| channels | int | Yes (named) | - | Number of channels (XM: 1-32, IT: 1-64) |
| instruments | list | Yes (named) | - | List of instrument dicts |
| patterns | dict | Yes (named) | - | Dict of pattern_name -> pattern dict |
| arrangement | list | Yes (named) | - | List of arrangement entries |
| name | str | No | None | Song internal name |
| title | str | No | None | Song display title |
| loop | bool | No | False | Whether the song should loop |
| restart_position | int | No | None | Order-table index to restart at when looping |
| automation | list | No | None | List of automation entries |
| it_options | dict | No | None | IT-specific options dict |

**Returns:** Dict matching the recipe structure for `music.tracker_song_v1`.

**Example:**
```python
tracker_song(
    format = "xm",
    bpm = 120,
    speed = 6,
    channels = 4,
    instruments = [
        tracker_instrument(name = "bass", synthesis = instrument_synthesis("sawtooth"))
    ],
    patterns = {
        "intro": tracker_pattern(64, notes = {"0": [pattern_note(0, "C4", 0)]})
    },
    arrangement = [arrangement_entry("intro", 4)]
)
```

### music_spec()

Creates a music spec with a tracker song recipe.

This is a convenience wrapper that combines `spec()` with a tracker song recipe.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| asset_id | str | Yes (named) | - | Kebab-case identifier for the asset |
| seed | int | Yes (named) | - | Deterministic seed (0 to 2^32-1) |
| output_path | str | Yes (named) | - | Output file path |
| format | str | Yes (named) | - | Tracker format: "xm" or "it" |
| bpm | int | Yes (named) | - | Beats per minute (30-300) |
| speed | int | Yes (named) | - | Tracker speed (1-31) |
| channels | int | Yes (named) | - | Number of channels |
| instruments | list | Yes (named) | - | List of instrument dicts |
| patterns | dict | Yes (named) | - | Dict of pattern_name -> pattern dict |
| arrangement | list | Yes (named) | - | List of arrangement entries |
| name | str | No | None | Song internal name |
| title | str | No | None | Song display title |
| loop | bool | No | False | Whether the song should loop |
| description | str | No | None | Asset description |
| tags | list | No | None | Style tags |
| license | str | No | "CC0-1.0" | SPDX license identifier |

**Returns:** A complete spec dict ready for serialization.

**Example:**
```python
music_spec(
    asset_id = "test-song-01",
    seed = 42,
    output_path = "music/test.xm",
    format = "xm",
    bpm = 120,
    speed = 6,
    channels = 4,
    instruments = [tracker_instrument(name = "bass")],
    patterns = {"intro": tracker_pattern(64)},
    arrangement = [arrangement_entry("intro")]
)
```

---

## Cue Template Functions

Cue templates provide structured helpers for creating common game audio patterns. They produce metadata and suggested parameters for use with `tracker_song()`.

### loop_low()

Creates a loop cue template with low intensity settings (exploration, menus, calm moments).

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| name | str | Yes (named) | - | Cue name |
| bpm | int | No | 90 | Beats per minute (30-300) |
| measures | int | No | 8 | Number of measures (1-64) |
| rows_per_beat | int | No | 4 | Rows per beat for pattern timing |
| channels | int | No | 4 | Number of channels |
| format | str | No | "xm" | Tracker format: "xm" or "it" |

**Returns:** Dict with cue metadata, timing info, song_params, and track_layout suggestions.

**Example:**
```python
loop_low(name = "explore_ambient")
loop_low(name = "menu_music", bpm = 80, measures = 16)
```

### loop_main()

Creates a loop cue template with main/standard intensity settings (general gameplay).

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| name | str | Yes (named) | - | Cue name |
| bpm | int | No | 120 | Beats per minute (30-300) |
| measures | int | No | 8 | Number of measures (1-64) |
| rows_per_beat | int | No | 4 | Rows per beat for pattern timing |
| channels | int | No | 8 | Number of channels |
| format | str | No | "xm" | Tracker format: "xm" or "it" |

**Returns:** Dict with cue metadata, timing info, song_params, and track_layout suggestions.

**Example:**
```python
loop_main(name = "gameplay_theme")
loop_main(name = "level_1", bpm = 128, channels = 8)
```

### loop_hi()

Creates a loop cue template with high intensity settings (combat, boss encounters).

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| name | str | Yes (named) | - | Cue name |
| bpm | int | No | 140 | Beats per minute (30-300) |
| measures | int | No | 8 | Number of measures (1-64) |
| rows_per_beat | int | No | 4 | Rows per beat for pattern timing |
| channels | int | No | 12 | Number of channels |
| format | str | No | "xm" | Tracker format: "xm" or "it" |

**Returns:** Dict with cue metadata, timing info, song_params, and track_layout suggestions.

**Example:**
```python
loop_hi(name = "boss_battle")
loop_hi(name = "combat_intense", bpm = 160, channels = 16)
```

### loop_cue()

Creates a loop cue with explicit intensity level (generic version).

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| name | str | Yes (named) | - | Cue name |
| intensity | str | Yes (named) | - | "low", "main", or "hi" |
| bpm | int | No | varies | BPM (defaults based on intensity) |
| measures | int | No | 8 | Number of measures (1-64) |
| rows_per_beat | int | No | 4 | Rows per beat for pattern timing |
| channels | int | No | varies | Channels (defaults based on intensity) |
| format | str | No | "xm" | Tracker format: "xm" or "it" |

**Returns:** Dict with cue metadata, timing info, song_params, and track_layout suggestions.

**Example:**
```python
loop_cue(name = "ambient", intensity = "low", bpm = 80)
loop_cue(name = "action", intensity = "hi", bpm = 150, channels = 16)
```

### stinger()

Creates a stinger cue template for one-shot musical events (victory, defeat, pickup, etc.).

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| name | str | Yes (named) | - | Cue name |
| stinger_type | str | No | "custom" | "victory", "defeat", "pickup", "levelup", "discovery", "danger", "alert", "custom" |
| duration_beats | int | No | 4 | Duration in beats (1-32) |
| bpm | int | No | 120 | Beats per minute (30-300) |
| rows_per_beat | int | No | 4 | Rows per beat for pattern timing |
| channels | int | No | 4 | Number of channels |
| format | str | No | "xm" | Tracker format: "xm" or "it" |
| tail_beats | int | No | 0 | Optional decay/reverb tail in beats (0-16) |

**Returns:** Dict with cue metadata, timing info, song_params, and stinger-specific track_layout suggestions.

**Example:**
```python
stinger(name = "coin_pickup", stinger_type = "pickup", duration_beats = 2)
stinger(name = "level_complete", stinger_type = "victory", duration_beats = 8)
stinger(name = "enemy_alert", stinger_type = "alert", duration_beats = 2, tail_beats = 2)
```

### transition()

Creates a transition cue template for bridging between music states.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| name | str | Yes (named) | - | Cue name |
| transition_type | str | No | "bridge" | "build", "breakdown", "bridge", "fill", "custom" |
| from_intensity | str | No | "main" | Starting intensity: "low", "main", "hi" |
| to_intensity | str | No | "main" | Target intensity: "low", "main", "hi" |
| measures | int | No | 2 | Number of measures (1-8) |
| bpm | int | No | 120 | Beats per minute (30-300) |
| rows_per_beat | int | No | 4 | Rows per beat for pattern timing |
| channels | int | No | 8 | Number of channels |
| format | str | No | "xm" | Tracker format: "xm" or "it" |

**Returns:** Dict with cue metadata, timing info, song_params, track_layout, and automation_hints.

**Example:**
```python
transition(name = "to_combat", transition_type = "build", from_intensity = "main", to_intensity = "hi")
transition(name = "combat_end", transition_type = "breakdown", from_intensity = "hi", to_intensity = "low")
transition(name = "drum_fill", transition_type = "fill", measures = 1)
```

---

[← Back to Index](stdlib-reference.md)
