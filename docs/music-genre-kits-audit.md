# Game Music Genre Kits — Audit (Draft)

**Status:** Draft  
**Created:** 2026-01-11  
**Source:** `docs/music-genre-kits-master-list.md`  

## 1) Kit completeness (roles checklist)

Legend:

- `D` = drums / rhythm
- `B` = bass
- `H` = harmony (pads/keys/chords/ostinato)
- `M` = melody / lead
- `FX` = transitions / stingers / risers / impacts
- `A` = atmos / drones / textures

| Kit | D | B | H | M | FX | A | Notes |
|---|:--:|:--:|:--:|:--:|:--:|:--:|---|
| Chiptune 8-bit (NES-ish) | ✓ | ✓ | ✓ | ✓ | ✓ | ◻︎ | Ensure standard role alias mapping (`pulse_lead`, `triangle_bass`, `noise_hat`...) |
| Chiptune 16-bit / Amiga Tracker Retro | ✓ | ✓ | ✓ | ✓ | ✓ | ◻︎ | Ensure standard role alias mapping (`chord_stab` → `keys/chords`) |
| FM / Arcade (Genesis-ish) | ✓ | ✓ | ✓ | ✓ | ✓ | ◻︎ | Ensure standard role alias mapping (`fm_*` → standard roles) |
| OPL / AdLib / DOS FM | ✓ | ✓ | ✓ | ✓ | ✓ | ◻︎ | Add/alias a clear `pad` or `keys` role if needed |
| DX7 / FM Pop Keys | ✓ | ✓ | ✓ | ✓ | ✓ | ◻︎ | Ensure standard role alias mapping (`dx7_*` → standard roles) |
| Synthwave / Outrun | ✓ | ✓ | ✓ | ✓ | ✓ | ◻︎ | Alias `gated_snare` → `snare` |
| Cyberpunk / Industrial | ✓ | ✓ | ✓ | ◻︎ | ✓ | ✓ | Add a `lead_1` role (or mark as optional) |
| DnB / Breakbeat | ✓ | ✓ | ✓ | ◻︎ | ✓ | ◻︎ | Add a `lead_1` (or `arp`) role (or mark as optional) |
| House / Techno | ✓ | ✓ | ✓ | ◻︎ | ✓ | ◻︎ | Add a `lead_1`/`arp` role (or mark as optional) |
| Trap / 808 | ✓ | ✓ | ✓ | ✓ | ✓ | ◻︎ | — |
| 909 / Acid / Rave | ✓ | ✓ | ✓ | ◻︎ | ✓ | ◻︎ | Add a `lead_1` role (or mark as optional) |
| Action EDM / Boss Fight | ✓ | ✓ | ✓ | ✓ | ✓ | ◻︎ | — |
| Bass Music / Halftime (Wobble) | ✓ | ✓ | ✓ | ✓ | ✓ | ◻︎ | Ensure `bass_wobble` maps cleanly to `bass_mid` + `bass_sub` roles |
| Ambient / Exploration | ◻︎ | ◻︎ | ✓ | ◻︎ | ✓ | ✓ | Palette kit (intentionally sparse rhythm/bass) |
| Ethereal / Underwater / Dream | ◻︎ | ◻︎ | ✓ | ◻︎ | ✓ | ✓ | Palette kit (pads, bells, swells, textures) |
| Dark Ambient / Horror | ◻︎ | ✓ | ◻︎ | ◻︎ | ✓ | ✓ | Palette kit (hits, drones, sub) |
| Stealth / Spy | ✓ | ✓ | ✓ | ✓ | ✓ | ◻︎ | — |
| Fantasy Cinematic (Synth-Orchestral) | ✓ | ◻︎ | ✓ | ✓ | ✓ | ◻︎ | Add `bass_low`/`sub_rumble` (or mark optional) |
| Sci‑Fi Hybrid Cinematic | ✓ | ◻︎ | ✓ | ✓ | ✓ | ◻︎ | Add `bass_sub` (or mark optional) |
| Military / March / Strategy | ✓ | ◻︎ | ✓ | ◻︎ | ✓ | ◻︎ | Add `bass_low` and a clear `lead_1` (or keep motif in brass) |
| Chiprock / Arcade Rock | ✓ | ✓ | ✓ | ✓ | ✓ | ◻︎ | Ensure `power_chord_stab` maps to `chords` |
| Funk / Jazz-lite | ✓ | ✓ | ✓ | ✓ | ✓ | ◻︎ | — |
| Noir / Detective / Mystery | ✓ | ✓ | ✓ | ✓ | ✓ | ◻︎ | Ensure `snare_brush` / `bass_upright` are available or aliased |
| Lo‑Fi / Chillhop | ✓ | ✓ | ✓ | ◻︎ | ✓ | ✓ | Add `lead_1` (or mark optional) |
| Tribal / World Perc | ✓ | ◻︎ | ◻︎ | ◻︎ | ✓ | ✓ | Add `bass_low` + `pad/drone` defaults (or mark palette) |
| Orchestral Adventure | ✓ | ◻︎ | ✓ | ✓ | ✓ | ◻︎ | Add `bass_low` (or mark optional) |
| Piano Minimal / Narrative | ◻︎ | ◻︎ | ✓ | ◻︎ | ✓ | ◻︎ | Palette kit (piano-led, sparse) |
| Metal / Heavy | ✓ | ✓ | ◻︎ | ✓ | ✓ | ◻︎ | Add `pad`/`chords` support if desired (optional) |
| Western / Desert | ✓ | ✓ | ✓ | ◻︎ | ✓ | ◻︎ | Add `lead_1` (or keep harmonica optional) |
| Pirate / Nautical | ✓ | ✓ | ✓ | ✓ | ✓ | ◻︎ | Ensure `accordion`/`concertina` + `whistle_lead` are available |
| Celtic / Medieval Folk | ✓ | ◻︎ | ✓ | ✓ | ✓ | ◻︎ | Add `bass_low` (or mark optional) |
| Kawaii / Whimsical | ✓ | ◻︎ | ✓ | ✓ | ✓ | ◻︎ | Add `bass_round` (or mark optional) |
| Puzzle / Minimal (Mallet & Plucks) | ✓ | ✓ | ✓ | ◻︎ | ✓ | ◻︎ | Add `lead_1` (optional) |
| Uplifting Pop / Indie Pop | ✓ | ✓ | ✓ | ✓ | ✓ | ◻︎ | — |
| Stadium / Sports Hype | ✓ | ✓ | ✓ | ✓ | ✓ | ◻︎ | — |
| Latin / Tropical | ✓ | ✓ | ✓ | ◻︎ | ✓ | ◻︎ | Add `lead_1` (or mark optional) |
| Afrobeat / Afrofusion | ✓ | ✓ | ✓ | ✓ | ✓ | ◻︎ | Focus on groove templates + percussion variation |
| Dub / Reggae / Caribbean | ✓ | ✓ | ✓ | ◻︎ | ✓ | ◻︎ | Add `lead_1` (or keep melody sparse) |
| East Asian | ✓ | ◻︎ | ✓ | ✓ | ✓ | ✓ | Add `bass_low` (or mark optional) |
| Middle Eastern / Desert | ✓ | ◻︎ | ✓ | ✓ | ✓ | ✓ | Add `bass_low` (or mark optional) |
| South Asian | ✓ | ◻︎ | ✓ | ✓ | ✓ | ✓ | Add `bass_low` (optional) |
| Glitch / IDM / Experimental Electronica | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — |

## 2) Serious remaining gaps (high-impact additions)

- Nordic / Viking (chants, war drums, lyre/tagelharpa-ish, “raid” energy)
- Anime / J‑Rock (bright leads, tight rock kit, power chords, fast hooks)
- Bluegrass / Appalachian (banjo/mandolin/fiddle, stomp/clap)
- Big Band / Swing (brass sections, walking bass, swing kit)
- Minimal Tech / Tension Pulses (ultra-minimal synth pulses + impacts for overlays)

## 3) Coverage work that’s not “more kits”

- Standardize per-kit cue set templates:
  - `loop_main`, `loop_low_intensity`, `loop_high_intensity`
  - `transition_up`, `transition_down`
  - `stinger_victory`, `stinger_failure`, `stinger_secret`
- Enforce standard role alias mapping in every kit (`instrument_ids`/`channel_ids`), even when the kit uses style-specific preset IDs.
