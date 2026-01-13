# Game Music Genre Kits — Audit Checklist (Draft)

**Last updated:** 2026-01-13  
**Location:** `packs/preset_library_v1/music/`

This document is a lightweight checklist for auditing **genre kits** in the preset library.
For the master inventory of kits (and the design targets for future kits), see `docs/music-genre-kits-master-list.md`.

## What “audit” means here

A kit is considered “healthy” when:

- The kit JSON validates (`speccade validate --spec <kit.json>`)
- The kit can be expanded (if it uses compose IR): `speccade expand --spec <kit.json>`
- The kit can be generated into a playable XM/IT (and ideally has at least one short audition/demo spec)
- Instrument and channel naming is consistent (`instrument_ids`, `channel_ids`) and follows the role taxonomy in the master list
- Referenced `audio_v1` presets exist (and are reasonable: no clipping/DC offset, sane base notes)

## Suggested per-kit checklist

For each `packs/preset_library_v1/music/kit_*.json`:

### 1) Contract + validation

- [ ] `asset_id` matches file name intent (stable, descriptive)
- [ ] `license` present and appropriate for the pack
- [ ] `outputs[]` include at least one `primary` output (XM and/or IT)
- [ ] `speccade validate --spec ...` passes with no errors

### 2) Pattern authoring (compose kits)

- [ ] `speccade expand --spec ...` succeeds
- [ ] Expanded output is reviewable (no accidental density, no merge collisions)
- [ ] If `harmony`/`pitch_seq` is used, defaults keep parts in-key

### 3) Roles and naming

- [ ] Core drum roles mapped (`kick`, `snare`/`clap`, `hat_closed`, etc.)
- [ ] Bass roles mapped (`bass_sub` at minimum)
- [ ] At least one melodic/harmonic role mapped (`pad`/`keys`/`lead_1`, depending on style)
- [ ] Game-use roles available when relevant (`impact`, `riser`, `downlifter`, `stinger`, `pickup`)

### 4) Audio preset references

- [ ] All referenced `audio_v1` preset paths exist
- [ ] Pitched instruments have sensible `base_note` and don’t alias badly in common ranges
- [ ] One-shots have click-free envelopes (attack/release) and no obvious DC offset

### 5) Generate + listen

- [ ] `speccade generate --spec ... --out-root <out>` produces the expected XM/IT
- [ ] Quick listen check in a known player (no silent channels, missing instruments, broken loops)
- [ ] Optional: render-to-WAV workflow documented (if used by the project)

