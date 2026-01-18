# Profiling

SpecCade includes built-in profiling support to help identify performance bottlenecks during asset generation.

## Usage

Add `--profile` to the `generate` command to collect per-stage timing information:

```bash
# Human-readable output
speccade generate --spec my_sound.json --out-root ./out --profile

# JSON output with timing data
speccade generate --spec my_sound.json --out-root ./out --profile --json
```

When profiling is enabled:
- Stage timings are collected for major generation phases
- Timings are included in the generated report file (`*.report.json`)
- Cache is bypassed (profiling implies a fresh generation run)

## Report Structure

When `--profile` is used, the report includes a `stages` array:

```json
{
  "report_version": 1,
  "spec_hash": "abc123...",
  "ok": true,
  "stages": [
    { "stage": "parse_params", "duration_ms": 2 },
    { "stage": "render_audio", "duration_ms": 145 },
    { "stage": "encode_output", "duration_ms": 12 },
    { "stage": "generate_waveform", "duration_ms": 8 }
  ],
  "duration_ms": 167,
  ...
}
```

Without `--profile`, the `stages` field is omitted from the report.

## Instrumented Stages

### Audio (`audio_v1`)

| Stage | Description |
|-------|-------------|
| `render_audio` | Audio synthesis and mixing |
| `encode_output` | WAV file encoding and writing |
| `generate_waveform` | Waveform preview PNG generation |

### Texture (`texture.procedural_v1`)

| Stage | Description |
|-------|-------------|
| `parse_params` | Recipe parameter parsing and validation |
| `render_graph` | Node graph evaluation and rendering |
| `encode_outputs` | PNG encoding for all output nodes |

### Music (`music.tracker_song_v1`, `music.tracker_song_compose_v1`)

| Stage | Description |
|-------|-------------|
| `parse_params` | Recipe parameter parsing |
| `expand_compose` | Compose expansion (compose recipe only) |
| `render_music` | Tracker module generation |
| `encode_output` | XM/IT file encoding and writing |

For multi-output music specs (both XM and IT), render stages are named `render_music_xm` and `render_music_it`.

## Limitations

- Profiling uses `std::time::Instant` for wall-clock timing
- No memory tracking is included (keep overhead minimal)
- Blender-based backends (static_mesh, skeletal_mesh, skeletal_animation) are not instrumented

## Example

```bash
# Generate with profiling and view timing in the report
speccade generate --spec golden/speccade/specs/audio/simple_beep.json \
    --out-root ./tmp/profile --profile --json

# The report will include timing breakdown
cat ./tmp/profile/simple_beep.report.json | jq '.stages'
```
