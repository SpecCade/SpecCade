# Game Music Genre Kits (preset_library_v1)

**Source of truth:** `packs/preset_library_v1/music/kit_*.json`

This doc describes conventions for authoring and reviewing music kit JSON files.

## Role taxonomy (recommended)

Use stable keys across kits so templates can stay consistent. Kits can also define style-specific aliases.

Minimal core:
- drums: `kick`, `snare`/`clap`, `hat_closed`
- bass: `bass_sub`
- harmony/melody: one of `pad`/`keys`/`lead_1`

Example (aliases):

```json
{
  "instrument_ids": {
    "kick": 0,
    "snare": 1,
    "hat_closed": 2,
    "bass_sub": 3,
    "lead_1": 4,

    "kick_808": 0,
    "acid_bass": 3
  }
}
```

## Review checklist

See `docs/music-genre-kits-audit.md`.

## Listing kits

- Windows: `Get-ChildItem packs/preset_library_v1/music -Filter kit_*.json`
- Cross-platform: list the directory and open the JSON files
