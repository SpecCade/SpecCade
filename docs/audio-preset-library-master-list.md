# Audio Preset Library (preset_library_v1)

**Source of truth:** `packs/preset_library_v1/audio/` (`*.json` specs)

This pack contains `audio_v1` preset specs intended for reuse in gameplay SFX and as building blocks for music kits.

## How to browse

- Presets are grouped by folder (e.g. `drums/`, `bass/`, `pads/`).
- Each preset is a full SpecCade `audio` spec with `description` and `style_tags`.

## How to validate

- Validate all presets under the pack: `python validate_all.py` (repo helper)
- Spot-check a preset: `cargo run -p speccade-cli -- validate --spec <path-to-preset.json> --budget strict`

## Stats / inventory

Do not commit generated reports or derived inventories as documentation. If you need stats, run `python analyze_presets.py packs/preset_library_v1/audio` and consume the console output.
