# Tracker Effects Reference

This document provides a comprehensive reference for all tracker effects supported by SpecCade's music backend.

## Overview

SpecCade supports both XM (FastTracker II) and IT (Impulse Tracker) formats. Effects can be specified in two ways:

1. **Typed effects** - Using the `TrackerEffect` enum with validated parameters
2. **String-based effects** - Using effect names with `effect_name` and `param`/`effect_xy` fields

The typed effect system provides:
- Parameter validation with format-specific ranges
- Automatic encoding to XM/IT effect codes
- Clear documentation of valid parameter ranges

## Effect Categories

### Pitch Effects

#### Arpeggio
Rapidly cycles between note, note+x semitones, and note+y semitones.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | 0xy | x=first offset (0-15), y=second offset (0-15) |
| IT | Jxy | x=first offset (0-15), y=second offset (0-15) |

```json
{ "type": "arpeggio", "x": 3, "y": 7 }
```

#### Portamento Up
Slides pitch up continuously.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | 1xx | speed (0-255, 0 uses previous) |
| IT | Fxx | speed (0-255, 0 uses previous) |

```json
{ "type": "portamento_up", "speed": 16 }
```

#### Portamento Down
Slides pitch down continuously.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | 2xx | speed (0-255, 0 uses previous) |
| IT | Exx | speed (0-255, 0 uses previous) |

```json
{ "type": "portamento_down", "speed": 16 }
```

#### Fine Portamento Up
Precise pitch slide up (finer control than regular portamento).

| Format | Code | Parameter |
|--------|------|-----------|
| XM | E1x | speed (0-15) |
| IT | FFx | speed (0-15) |

```json
{ "type": "fine_portamento_up", "speed": 4 }
```

#### Fine Portamento Down
Precise pitch slide down.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | E2x | speed (0-15) |
| IT | EFx | speed (0-15) |

```json
{ "type": "fine_portamento_down", "speed": 4 }
```

#### Extra Fine Portamento Up/Down
Very precise pitch slides (XM only, IT uses fine portamento).

| Format | Code | Parameter |
|--------|------|-----------|
| XM | X1x/X2x | speed (0-15) |

```json
{ "type": "extra_fine_porta_up", "speed": 2 }
```

#### Tone Portamento
Slides from current pitch to target note.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | 3xx | speed (0-255) |
| IT | Gxx | speed (0-255) |

```json
{ "type": "tone_portamento", "speed": 32 }
```

#### Vibrato
Oscillates pitch up and down.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | 4xy | speed (0-15), depth (0-15) |
| IT | Hxy | speed (0-15), depth (0-15) |

```json
{ "type": "vibrato", "speed": 4, "depth": 8 }
```

#### Fine Vibrato (IT only)
Finer vibrato with smaller depth increments.

| Format | Code | Parameter |
|--------|------|-----------|
| IT | Uxy | speed (0-15), depth (0-15) |

```json
{ "type": "fine_vibrato", "speed": 4, "depth": 8 }
```

### Volume Effects

#### Volume Slide
Slides volume up or down.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | Axy | up (0-15), down (0-15) |
| IT | Dxy | up (0-15), down (0-15) |

Note: Cannot have both up and down non-zero except for fine slides.

```json
{ "type": "volume_slide", "up": 4, "down": 0 }
```

#### Set Volume (XM only)
Sets volume directly.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | Cxx | volume (0-64) |

IT uses the volume column instead.

```json
{ "type": "set_volume", "volume": 48 }
```

#### Tremolo
Oscillates volume up and down.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | 7xy | speed (0-15), depth (0-15) |
| IT | Rxy | speed (0-15), depth (0-15) |

```json
{ "type": "tremolo", "speed": 4, "depth": 8 }
```

#### Tremor
Rapidly toggles volume on/off.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | Txy | on_time (0-15), off_time (0-15) |
| IT | Ixy | on_time (0-15), off_time (0-15) |

```json
{ "type": "tremor", "on_time": 3, "off_time": 1 }
```

#### Global Volume
Sets the global module volume.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | Gxx | volume (0-64) |
| IT | Vxx | volume (0-128) |

```json
{ "type": "set_global_volume", "volume": 64 }
```

#### Global Volume Slide
Slides global volume.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | Hxy | up (0-15), down (0-15) |
| IT | Wxy | up (0-15), down (0-15) |

```json
{ "type": "global_volume_slide", "up": 2, "down": 0 }
```

#### Channel Volume (IT only)
Sets per-channel volume.

| Format | Code | Parameter |
|--------|------|-----------|
| IT | Mxx | volume (0-64) |

```json
{ "type": "set_channel_volume", "volume": 48 }
```

#### Channel Volume Slide (IT only)
Slides per-channel volume.

| Format | Code | Parameter |
|--------|------|-----------|
| IT | Nxy | up (0-15), down (0-15) |

```json
{ "type": "channel_volume_slide", "up": 2, "down": 0 }
```

### Panning Effects

#### Set Panning
Sets stereo position.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | 8xx | pan (0-255, 0=left, 128=center, 255=right) |
| IT | Xxx | pan (0-255) |

```json
{ "type": "set_panning", "pan": 128 }
```

#### Panning Slide
Slides panning position.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | Pxy | right (0-15), left (0-15) |
| IT | Pxy | left (0-15), right (0-15) |

Note: XM and IT have reversed parameter order.

```json
{ "type": "panning_slide", "left": 0, "right": 4 }
```

#### Panbrello (IT only)
Oscillates panning position.

| Format | Code | Parameter |
|--------|------|-----------|
| IT | Yxy | speed (0-15), depth (0-15) |

```json
{ "type": "panbrello", "speed": 4, "depth": 8 }
```

### Timing/Trigger Effects

#### Retrigger
Retriggers the note at regular intervals with optional volume change.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | Rxy | volume_change (0-15), interval (1-15) |
| IT | Qxy | volume_change (0-15), interval (1-15) |

Volume change values:
- 0: No change
- 1-5: Decrease (-1, -2, -4, -8, -16)
- 6-7: Multiply (2/3, 1/2)
- 8: No change
- 9-D: Increase (+1, +2, +4, +8, +16)
- E-F: Multiply (3/2, 2)

```json
{ "type": "retrigger", "volume_change": 0, "interval": 3 }
```

#### Note Delay
Delays note start by ticks.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | EDx | ticks (0-15) |
| IT | SDx | ticks (0-15) |

```json
{ "type": "note_delay", "ticks": 4 }
```

#### Note Cut
Cuts note after specified ticks.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | ECx | ticks (0-15) |
| IT | SCx | ticks (0-15) |

```json
{ "type": "note_cut", "ticks": 8 }
```

### Playback Control

#### Set Speed
Sets ticks per row.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | Fxx | speed (1-31) |
| IT | Axx | speed (1-255) |

```json
{ "type": "set_speed", "speed": 6 }
```

#### Set Tempo
Sets tempo in BPM.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | Fxx | bpm (32-255) |
| IT | Txx | bpm (32-255) |

Note: In XM, speed and tempo share the same effect. Values 1-31 set speed, 32-255 set BPM.

```json
{ "type": "set_tempo", "bpm": 140 }
```

#### Position Jump
Jumps to specified order position.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | Bxx | position (0-255) |
| IT | Bxx | position (0-255) |

```json
{ "type": "position_jump", "position": 0 }
```

#### Pattern Break
Jumps to row in next pattern.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | Dxx | row (0-63) |
| IT | Cxx | row (0-255) |

Note: XM limits row to 63 max.

```json
{ "type": "pattern_break", "row": 0 }
```

#### Pattern Loop
Creates a loop within the pattern.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | E6x | count (0=set start, 1-15=loop count) |
| IT | SBx | count (0=set start, 1-15=loop count) |

```json
{ "type": "pattern_loop", "count": 2 }
```

### Sample Effects

#### Sample Offset
Starts sample playback from offset.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | 9xx | offset in 256-byte units (0-255) |
| IT | Oxx | offset in 256-byte units (0-255) |

```json
{ "type": "sample_offset", "offset": 32 }
```

### Combined Effects

#### Tone Portamento + Volume Slide
Combines tone portamento with volume slide.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | 5xy | up (0-15), down (0-15) |
| IT | Lxy | up (0-15), down (0-15) |

```json
{ "type": "tone_porta_volume_slide", "up": 2, "down": 0 }
```

#### Vibrato + Volume Slide
Combines vibrato with volume slide.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | 6xy | up (0-15), down (0-15) |
| IT | Kxy | up (0-15), down (0-15) |

```json
{ "type": "vibrato_volume_slide", "up": 2, "down": 0 }
```

### Waveform Control

#### Set Vibrato Waveform
Sets the waveform used for vibrato.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | E4x | waveform (0-3) |
| IT | S3x | waveform (0-3) |

Waveforms: 0=sine, 1=ramp down, 2=square, 3=random

```json
{ "type": "set_vibrato_waveform", "waveform": 0 }
```

#### Set Tremolo Waveform
Sets the waveform used for tremolo.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | E7x | waveform (0-3) |
| IT | S4x | waveform (0-3) |

```json
{ "type": "set_tremolo_waveform", "waveform": 1 }
```

### XM-Only Effects

#### Key Off
Triggers key-off at specified tick.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | Kxx | tick (0-255) |

```json
{ "type": "key_off", "tick": 0 }
```

#### Set Envelope Position
Sets position in volume/panning envelope.

| Format | Code | Parameter |
|--------|------|-----------|
| XM | Lxx | position (0-255) |

```json
{ "type": "set_envelope_position", "position": 16 }
```

## Using String-Based Effects

For backward compatibility, effects can also be specified using effect names:

```json
{
  "row": 0,
  "note": "C4",
  "inst": 1,
  "effect_name": "vibrato",
  "param": 72
}
```

Or using effect_xy for x/y nibble effects:

```json
{
  "row": 0,
  "note": "C4",
  "inst": 1,
  "effect_name": "arpeggio",
  "effect_xy": [3, 7]
}
```

## Validation

The `TrackerEffect` enum provides validation methods:

- `validate_xm()` - Validates for XM format constraints
- `validate_it()` - Validates for IT format constraints

Common validation checks:
- Nibble parameters (0-15)
- Volume ranges (0-64 for XM, 0-128 for IT global volume)
- Speed range (1-31 for XM)
- Tempo range (32-255)
- Format-specific effect support

## Format Differences Summary

| Feature | XM | IT |
|---------|----|----|
| Max channels | 32 | 64 |
| Speed range | 1-31 | 1-255 |
| Global volume max | 64 | 128 |
| Pattern break row max | 63 | 255 |
| Channel volume | No | Yes (Mxx/Nxy) |
| Fine vibrato | No | Yes (Uxy) |
| Panbrello | No | Yes (Yxy) |
| Key off effect | Yes (Kxx) | No |
| Envelope position | Yes (Lxx) | No |
| Set volume effect | Yes (Cxx) | No (use volume column) |

See also: [XM/IT Differences](xm-it-differences.md)
