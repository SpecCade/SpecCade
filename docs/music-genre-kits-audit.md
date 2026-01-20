# Game Music Genre Kits - Audit Checklist

**Location:** `packs/preset_library_v1/music/`

Checklist for reviewing `kit_*.json` files.

Conventions (recommended role taxonomy, naming): `docs/music-genre-kits-master-list.md`

## What "audit" means here

A kit is considered healthy when:

- It validates: `speccade validate --spec <kit.json>`
- It expands if it uses compose IR: `speccade expand --spec <kit.json>`
- It generates into a playable XM/IT: `speccade generate --spec <kit.json> --out-root <out>`
- Naming is consistent (`instrument_ids`, `channel_ids`) and follows the role taxonomy
- Referenced `audio_v1` presets exist and are reasonable (no clipping/DC offset, sane `base_note`)

## Suggested per-kit checklist

For each `packs/preset_library_v1/music/kit_*.json`:

### 1) Contract + validation

- [ ] `asset_id` matches file name intent
- [ ] `license` present and appropriate
- [ ] `outputs[]` include at least one `primary` output (XM and/or IT)
- [ ] `speccade validate` passes with no errors

### 2) Pattern authoring (compose kits)

- [ ] `speccade expand` succeeds
- [ ] Expanded output is reviewable
- [ ] If `harmony`/`pitch_seq` is used, defaults keep parts in-key

### 3) Roles and naming

- [ ] Core drum roles mapped (`kick`, `snare`/`clap`, `hat_closed`, etc.)
- [ ] Bass roles mapped (`bass_sub` at minimum)
- [ ] At least one melodic/harmonic role mapped (`pad`/`keys`/`lead_1`, depending on style)
- [ ] Game-use roles available when relevant (`impact`, `riser`, `downlifter`, `stinger`, `pickup`)

### 4) Audio preset references

- [ ] All referenced preset paths exist
- [ ] Pitched instruments have sensible `base_note`
- [ ] One-shots have click-free envelopes and no obvious DC offset

### 5) Generate + listen

- [ ] Generation produces the expected XM/IT
- [ ] Quick listen check in a known player
