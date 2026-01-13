# Game Music Genre Kits — Master Inventory

**Last updated:** 2026-01-14
**Location:** `packs/preset_library_v1/music/`

This document is the single source of truth for genre kits in the preset library.

## Implementation Status Summary

| Status | Count |
|--------|-------|
| Implemented | 26 |
| Not yet implemented | 16 |
| **Total** | **42** |

---

## Contents

1. What a genre kit includes
2. Standard role taxonomy
3. Kit requirements (quality, licensing, variation)
4. Master inventory (game-focused kits)
5. Quick coverage map (game modes to kits)
6. Next curation steps

---

## 1) What a "Genre Kit" means in SpecCade

A **genre kit** includes:

- `audio_v1` presets (original synthesis; optionally WAV-only for explicitly licensed packs)
- `music.tracker_song_compose_v1` templates (`defs`, patterns, and defaults)
- consistent channel + instrument naming (`channel_ids`, `instrument_ids`)
- default `timebase` (and optional `harmony`)
- (recommended) short audition/demo specs for listening + regression tests

Related docs:

- Core IR: `docs/rfcs/RFC-0003-music-pattern-ir.md`
- Musical helpers: `docs/rfcs/RFC-0004-music-compose-musical-helpers.md`
- Chords: `docs/music-chord-spec.md`

---

## 2) Standard role taxonomy (use across all kits)

Use these **role keys** across all kits.

Kits may also provide style-specific preset names (e.g., `acid_bass`, `gated_snare`), but they should include these
standard role aliases in `instrument_ids` (and/or `channel_ids`) so templates can stay consistent across kits.

Minimal example (aliases):

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

### 2.1 Drums (core)

- `kick`
- `snare` or `clap` (or both)
- `hat_closed`
- `hat_open` (optional for some styles)
- `perc_1`, `perc_2` (toms, shakers, clicks, rims, etc.)
- `cymbal` (`crash`/`ride`)

### 2.2 Bass (core)

- `bass_sub` (or one bass preset that cleanly covers sub range)
- `bass_mid` (optional in minimal styles)

### 2.3 Harmony (common)

- `pad` (sustained harmony / ambience)
- `keys` / `chords` (stabs, EP, organ, etc.)
- `arp` / `pluck`

### 2.4 Melody (common)

- `lead_1` (primary lead)
- `lead_2` / `counter` (optional)

### 2.5 FX (game-focused)

- `impact` (hit / slam)
- `riser` (upward build)
- `downlifter` (release)
- `whoosh` (movement)
- `stinger` (short musical punctuation)
- `pickup` (UI/powerup blip; can be shared with SFX packs)

### 2.6 Atmos (exploration/menu)

- `drone`
- `texture` (noise beds, granular-ish, evolving layer)

---

## 3) Kit requirements

### 3.1 Sound quality requirements (recommended)

- No clipping, no obvious DC offset, no harsh aliasing in exposed leads.
- Stable perceived loudness across presets (within a reasonable style range).
- Click-free envelopes (attack/release) for one-shots; clean loop points for sustain instruments.
- Clear pitch mapping (`base_note`) for any pitched preset intended for tracker playback.

### 3.2 Musicality guardrails (recommended defaults)

- Prefer `harmony` + `pitch_seq` (`scale_degree` / `chord_tone`) for pitched templates to keep parts in key.
- Provide "safe" chord progressions for templates, but allow easy swapping (no baked-in melodies).

### 3.3 Licensing / originality (selling-ready)

- Prefer fully-synthesized content (SpecCade `audio_v1`) for clean licensing.
- If a kit ships WAVs, include explicit license provenance and avoid third-party rips.

### 3.4 Pack deliverables (optional)

- `audio_v1` preset specs (source of truth)
- rendered WAVs (one-shots and/or loopable instruments)
- loop demos (XM/IT and/or rendered WAV loops)
- pattern templates (compose `defs` / snippets; optional MIDI exports)
- FX set (`riser`, `downlifter`, `impact`, `whoosh`, `stinger`, `pickup`)
- short audition/demo songs

### 3.5 Preset glossary (selected targets)

- **808 kick/sub:** sine-based sub with fast pitch drop + short click; long tail option; mild saturation.
- **909 drum kit:** punchy kick; snappy snare; metallic hats/ride; short, bright transients.
- **Acid bass (303-style):** saw/square with resonant lowpass filter movement; slide/glide and accent behavior (authored via pattern effects/automation).
- **DX7-style keys/tones:** FM electric piano, bell, marimba, bass, brass; clean transients and stable pitch.
- **OPL/DOS FM style:** 2-operator FM timbres (hollow/bright); classic "AdLib" brass/bass/lead silhouettes.

---

## 4) Variation requirements

- Do not encode a single signature riff as a required template.

- Provide **variants per role** (e.g., `kick_a`, `kick_b`, `kick_c`) and choose deterministically.
- Provide **pattern templates** (grooves, fills, transitions) with knobs:
  - `choose` between groove variants
  - `prob` for fills/ghosts
  - optional deterministic `humanize_vol`
- Encourage cross-kit mixing (e.g., "Synthwave drums + Ambient pads").

---

## 5) Master inventory (game-focused kits)

Legend: `[x]` = implemented, `[ ]` = not yet implemented

### 5.1 Retro / Tracker-first

#### `[x]` Kit: **Chiptune 8-bit (NES-ish)** — `kit_chiptune_8bit.json`

- Use cases: retro platformers, UI/menus, puzzle.
- Typical tempo: 110–180 BPM
- Required roles:
  - leads: `pulse_lead`, `pulse_chords`
  - bass: `triangle_bass`
  - drums: `noise_hat`, `noise_snare`, `kick`
  - FX: `stinger`, `pickup`
- Optional roles: `arp`, `drone`

#### `[x]` Kit: **Chiptune 16-bit / Amiga Tracker Retro** — `kit_chiptune_16bit.json`

- Use cases: retro shooters, racers, demoscene vibes.
- Typical tempo: 120–170 BPM
- Required roles:
  - drums: `kick`, `snare`, `hat_closed`, `hat_open`, `perc_1`, `cymbal`
  - bass: `bass_sub` or `bass_pluck`
  - harmony: `pad` or `chord_stab`
  - melody: `lead_1`
  - FX: `riser`, `impact`, `stinger`

#### `[x]` Kit: **FM / Arcade (Genesis-ish)** — `kit_fm_arcade.json`

- Use cases: arcade action, score attack.
- Typical tempo: 120–190 BPM
- Required roles:
  - FM: `fm_bass`, `fm_keys`, `fm_bell`, `fm_lead`
  - drums: `kick`, `snare`, `hat_closed`, `perc_1`
  - FX: `impact`, `pickup`
- Optional roles: `pad`, `arp`

#### `[x]` Kit: **OPL / AdLib / DOS FM** — `kit_opl_dos.json`

- Use cases: DOS retro, arcade throwbacks.
- Typical tempo: 100–180 BPM
- Required roles:
  - FM: `opl_bass`, `opl_brass`, `opl_lead`, `opl_bell`
  - drums: `kick`, `snare`, `hat_closed`
  - FX: `stinger`, `pickup`

#### `[ ]` Kit: **DX7 / FM Pop Keys**

- Use cases: towns/shops, menus, nostalgic themes.
- Typical tempo: 80–140 BPM
- Required roles:
  - FM: `dx7_ep`, `dx7_bass`, `dx7_bell`, `dx7_marimba`, `dx7_lead`
  - drums: `kick`, `snare`, `hat_closed`
  - FX: `stinger`, `pickup`

### 5.2 Modern synth genres (common in games)

#### `[x]` Kit: **Synthwave / Outrun** — `kit_synthwave.json`

- Use cases: racing, neon menus, retro-future city.
- Typical tempo: 90–120 BPM (or 140–180 for driving variants)
- Required roles:
  - drums: `kick`, `gated_snare`, `hat_closed`, `hat_open`, `cymbal`
  - bass: `bass_sub`, `bass_mid`
  - harmony: `poly_pad`, `chord_stab`, `arp`
  - melody: `lead_1`
  - FX: `riser`, `impact`, `downlifter`

#### `[ ]` Kit: **Cyberpunk / Industrial**

- Use cases: hacking, dystopian city, combat.
- Typical tempo: 90–160 BPM
- Required roles:
  - drums: `kick_dist`, `snare_noise`, `hat_metallic`, `perc_metal`
  - bass: `bass_aggressive` (+ optional `bass_sub`)
  - harmony: `pad_cold` or `drone`
  - FX: `impact`, `riser`, `whoosh`, `glitch`

#### `[x]` Kit: **DnB / Breakbeat** — `kit_dnb_breakbeat.json`

- Use cases: speed levels, chase sequences, high-skill gameplay.
- Typical tempo: 160–180 BPM
- Required roles:
  - drums: `kick`, `snare`, `hat_closed`, `hat_open`, `perc_break_layer`
  - bass: `bass_reese` (+ `bass_sub`)
  - harmony: `stab`, `pad_dark`
  - FX: `riser`, `impact`, `downlifter`

#### `[x]` Kit: **House / Techno** — `kit_house_techno.json`

- Use cases: club levels, menus, futuristic facilities.
- Typical tempo: 120–140 BPM (techno variants 135–150)
- Required roles:
  - drums: `kick_4`, `clap`, `hat_closed`, `hat_open`, `perc_1`
  - bass: `bass_sub` or `bass_pluck`
  - harmony: `stab`, `pad`
  - FX: `riser`, `downlifter`

#### `[x]` Kit: **Trap / 808** — `kit_trap_808.json`

- Use cases: modern menus, character themes, action setpieces.
- Typical tempo: 120–160 BPM (often half-time feel)
- Required roles:
  - drums: `kick_808`, `snare`/`clap`, `hat_closed`, `perc_1` (rim/cowbell), `cymbal`
  - bass: `bass_808`
  - harmony: `pad_dark` or `pluck`
  - melody: `lead_1`
  - FX: `riser`, `impact`, `downlifter`, `stinger`

#### `[x]` Kit: **909 / Acid / Rave** — `kit_909_acid.json`

- Use cases: arcade racing, club levels, high-energy loops.
- Typical tempo: 125–150 BPM
- Required roles:
  - drums: `kick_909`, `snare_909`, `hat_closed`, `hat_open`, `ride`, `toms`
  - bass: `acid_bass`
  - harmony: `rave_stab`
  - FX: `riser`, `impact`, `downlifter`

#### `[x]` Kit: **Action EDM / Boss Fight** — `kit_boss_action.json`

- Use cases: boss fights, high intensity encounters.
- Typical tempo: 128–170 BPM
- Required roles:
  - drums: `big_kick`, `snare`, `cymbal`, `toms`
  - bass: `bass_growl` (+ `bass_sub`)
  - harmony: `brass_stab` or `stacked_saw_chords`
  - melody: `lead_1`, optional `lead_2`
  - FX: `riser`, `impact`, `downlifter`, `whoosh`

#### `[ ]` Kit: **Bass Music / Halftime (Wobble)**

- Use cases: boss fights, arenas, sci-fi combat, "heavy" setpieces.
- Typical tempo: 70–90 (or 140–180 in halftime feel)
- Required roles:
  - drums: `big_kick`, `snare_heavy`, `hat_closed`, `perc_1`
  - bass: `bass_wobble` (+ `bass_sub`)
  - harmony: `stab_dark` or `pad_dark`
  - melody: `lead_1` (optional, often sparse)
  - FX: `impact`, `riser`, `downlifter`, `whoosh`

### 5.3 Atmos / exploration / horror

#### `[x]` Kit: **Ambient / Exploration** — `kit_ambient_exploration.json`

- Use cases: exploration, menus, narrative scenes.
- Typical tempo: free / 60–120 (often minimal drums)
- Required roles:
  - `pad_warm`, `drone`, `texture_air`
  - optional `pluck_soft`, `lead_sparse`
  - FX: `swell`, `downlifter_soft`, `stinger_soft`

#### `[x]` Kit: **Ethereal / Underwater / Dream** — `kit_ethereal_dream.json`

- Use cases: underwater levels, dream sequences, magic realms, calm "floaty" exploration.
- Typical tempo: free / 50–120 (often minimal drums)
- Required roles:
  - `pad_shimmer`, `drone_soft`, `texture_bubbles`
  - optional `pluck_glass`, `bell_glass`, `lead_whistle`
  - FX: `swell`, `downlifter_soft`, `stinger_soft`

#### `[x]` Kit: **Dark Ambient / Horror** — `kit_dark_ambient.json`

- Use cases: horror, stealth, tension ramps.
- Typical tempo: free / 50–110
- Required roles:
  - `drone_dark`, `texture_noise`, `sub_rumble`
  - hits: `impact_metal`, `stinger_horror`
  - FX: `riser_tension`, `downlifter_tension`

#### `[ ]` Kit: **Stealth / Spy (stylized)**

- Use cases: stealth levels, infiltration, heists, suspense (non-horror).
- Typical tempo: 80–130 (often sparse)
- Required roles:
  - percussion: `kick_soft`, `snare_brush` or `rim`, `hat_tick`, `perc_click`
  - bass: `bass_muted` or `bass_round`
  - harmony: `chord_pluck_muted` or `pad_cold`
  - melody: `lead_muted`
  - FX: `stinger`, `riser_tension`, `whoosh_soft`

### 5.4 Stylized "cinematic" (avoid fake realism)

#### `[x]` Kit: **Fantasy Cinematic (Synth-Orchestral, stylized)** — `kit_epic_cinematic.json`

- Use cases: overworld, quests, heroic beats.
- Typical tempo: 70–140
- Required roles:
  - `string_pad`, `choir_pad`, `bell`, `brass_stab`, `timp_hit`, `cym_swell`
  - melody: `lead_heroic`
  - FX: `impact`, `riser`, `stinger`

#### `[x]` Kit: **Sci-Fi Hybrid Cinematic** — `kit_scifi.json`

- Use cases: sci-fi story, space travel, labs.
- Typical tempo: 70–140
- Required roles:
  - `low_hit`, `metallic_perc`, `arp_pulse`, `pad_cold`, `lead_alarm`
  - FX: `impact`, `riser`, `downlifter`

#### `[x]` Kit: **Military / March / Strategy (stylized)** — `kit_military_march.json`

- Use cases: tactical maps, RTS/4X, war zones, "mission start" themes.
- Typical tempo: 90–140
- Required roles:
  - percussion: `snare_march`, `toms`, `big_drum`, `cymbal`
  - harmony: `brass_low`, `brass_stab`, `string_stacc`
  - optional `choir_pad` or `pad_cold`
  - FX: `riser_tension`, `impact`, `stinger`

### 5.5 Style color kits (useful for variety)

#### `[ ]` Kit: **Chiprock / Arcade Rock**

- Use cases: arcade action, sports, "fun combat".
- Typical tempo: 120–180
- Required roles:
  - drums: `kick`, `snare`, `hat_closed`, `toms`, `cymbal`
  - `bass_pick`, `guitar_lead` (synth), `power_chord_stab`
  - FX: `impact`, `stinger`

#### `[ ]` Kit: **Funk / Jazz-lite (stylized)**

- Use cases: shops, city hubs, character themes.
- Typical tempo: 85–120
- Required roles:
  - drums: `kick`, `snare`, `hat_closed`, `perc_shaker`
  - `bass_round`, `keys_ep`, `lead_muted`, `brass_stab`
  - FX: `stinger`, `pickup`

#### `[ ]` Kit: **Noir / Detective / Mystery (stylized)**

- Use cases: investigation, dialogue-heavy scenes, low-key tension, "case" menus.
- Typical tempo: 70–120
- Required roles:
  - drums: `kick_soft`, `snare_brush`, `hat_tick` (optional)
  - bass: `bass_upright` or `bass_round`
  - harmony: `piano`, `vibes` (vibraphone-ish) or `keys_ep`
  - melody: `lead_muted` (or `sax_lead` stylized)
  - FX: `stinger_soft`, `whoosh_soft`

#### `[x]` Kit: **Lo-Fi / Chillhop (stylized)** — `kit_lofi_chillhop.json`

- Use cases: menus, hubs, downtime, "cozy" scenes.
- Typical tempo: 70–100 BPM
- Required roles:
  - drums: `kick_soft`, `snare_soft`, `hat_closed`, `perc_shaker`
  - `bass_round`, `keys_ep`, `pad_warm`, `texture_vinyl`
  - FX: `stinger_soft`, `pickup`

#### `[ ]` Kit: **Tribal / World Perc (stylized)**

- Use cases: dungeons, rituals, deserts/jungles.
- Typical tempo: 80–150
- Required roles:
  - `big_drum`, `frame_drum`, `shaker`, `clacks`, `tom`
  - `drone`, optional `flute_lead`
  - FX: `impact`, `riser`

### 5.6 Acoustic / Hybrid (stylized)

#### `[ ]` Kit: **Orchestral Adventure (stylized)**

- Use cases: RPG overworld, story scenes, exploration/combat hybrids.
- Typical tempo: 60–140
- Required roles:
  - harmony: `string_pad`, `string_stacc`, `brass_stab`, `woodwind_lead` (or `lead_1`)
  - percussion: `timp_hit`, `cym_swell`
  - FX: `impact`, `riser`, `stinger`

#### `[ ]` Kit: **Piano Minimal / Narrative**

- Use cases: dialogue, introspection, endings, credits.
- Typical tempo: 50–110
- Required roles:
  - `piano`, `string_pad` (or `pad_warm`), `bell_soft` (optional)
  - FX: `stinger_soft`, `downlifter_soft`

#### `[x]` Kit: **Metal / Heavy (stylized)** — `kit_metal.json`

- Use cases: combat arenas, boss fights, action shooters.
- Typical tempo: 110–200
- Required roles:
  - drums: `kick`, `snare`, `hat_closed`, `toms`, `cymbal`
  - `bass_pick`, `guitar_rhythm`, `guitar_lead`
  - FX: `impact`, `stinger`

#### `[x]` Kit: **Western / Desert (stylized)** — `kit_western.json`

- Use cases: deserts, bounty hunts, frontier towns.
- Typical tempo: 70–150
- Required roles:
  - `guitar_twang`, `bass_round`, `perc_shaker`, `perc_clack`
  - optional `harmonica_lead` (or `lead_1`)
  - FX: `stinger`, `whoosh`, `impact`

#### `[ ]` Kit: **Pirate / Nautical (stylized)**

- Use cases: ports, ships, pirate fights, sea maps.
- Typical tempo: 90–170
- Required roles:
  - percussion: `frame_drum`, `perc_clack`, `shaker`
  - bass: `bass_upright` or `bass_round`
  - harmony: `accordion`/`concertina`, `guitar_pluck`
  - melody: `fiddle_lead`, optional `whistle_lead`
  - FX: `stinger`, `impact`, `whoosh`

#### `[x]` Kit: **Celtic / Medieval Folk (stylized)** — `kit_celtic_medieval.json`

- Use cases: medieval villages, taverns, fantasy exploration.
- Typical tempo: 80–160
- Required roles:
  - `harp`/`lute_pluck`, `fiddle_lead` (or `lead_1`), `flute_lead`
  - percussion: `frame_drum`, `shaker`
  - FX: `stinger`, `downlifter_soft`

### 5.7 Casual / Whimsical / Pop

#### `[ ]` Kit: **Kawaii / Whimsical**

- Use cases: cozy games, platformers, character themes.
- Typical tempo: 90–160
- Required roles:
  - `toy_kick`, `toy_snare`, `hat_closed`
  - `pluck_bright`, `pad_warm`, `lead_cute`
  - FX: `pickup`, `stinger`

#### `[ ]` Kit: **Puzzle / Minimal (Mallet & Plucks)**

- Use cases: puzzle loops, UI/menu backgrounds, strategy overlays.
- Typical tempo: 70–130
- Required roles:
  - `mallet` (marimba/vibraphone-ish), `pluck_soft`, `bass_round`
  - percussion: `perc_click`, `perc_shaker`
  - FX: `pickup`, `stinger_soft`

#### `[ ]` Kit: **Uplifting Pop / Indie Pop (stylized)**

- Use cases: credits, menus, upbeat overworld.
- Typical tempo: 90–150
- Required roles:
  - drums: `kick`, `snare`/`clap`, `hat_closed`, `cymbal`
  - `bass_round`, `chord_pluck`, `pad_warm`, `lead_1`
  - FX: `riser`, `downlifter`, `stinger`

#### `[ ]` Kit: **Stadium / Sports Hype (stylized)**

- Use cases: sports menus, match start, victory themes, arcade sports.
- Typical tempo: 95–160
- Required roles:
  - drums: `big_kick`, `snare`, `clap`, `hat_closed`, `toms`, `cymbal`
  - bass: `bass_saw` or `bass_round`
  - harmony: `brass_stab`, `chord_pluck`
  - melody: `lead_1`
  - FX: `riser`, `impact`, `stinger`

### 5.8 Regional flavor (stylized)

#### `[x]` Kit: **Latin / Tropical (stylized)** — `kit_latin.json`

- Use cases: beach towns, festivals, upbeat hubs.
- Typical tempo: 85–140
- Required roles:
  - percussion: `conga`, `bongo`, `claves`, `shaker`
  - `bass_round`, `chord_pluck`, optional `brass_stab`
  - FX: `stinger`, `pickup`

#### `[x]` Kit: **Afrobeat / Afrofusion (stylized)** — `kit_afrobeat.json`

- Use cases: vibrant towns, open-world travel, celebration scenes.
- Typical tempo: 90–130
- Required roles:
  - percussion: `kick`, `snare`, `hat_closed`, `perc_shaker`, `perc_conga`
  - bass: `bass_round` (syncopated)
  - harmony: `guitar_chops` or `keys_organ`, optional `brass_stab`
  - melody: `lead_1` (light, motif-driven)
  - FX: `stinger`, `pickup`

#### `[x]` Kit: **Dub / Reggae / Caribbean (stylized)** — `kit_caribbean_dub.json`

- Use cases: island hubs, relaxed traversal, beach levels, comedic downtime.
- Typical tempo: 70–110
- Required roles:
  - drums: `kick`, `snare_rim`, `hat_closed`, `perc_shaker`
  - bass: `bass_dub` (round, long)
  - harmony: `keys_organ`, `guitar_skank`
  - FX: `stinger_soft`, `whoosh_soft`

#### `[x]` Kit: **East Asian (stylized)** — `kit_east_asian.json`

- Use cases: temples, city districts, character themes.
- Typical tempo: 70–150
- Required roles:
  - `koto_pluck`, `shakuhachi_lead` (or `flute_lead`), `taiko_hit`
  - `drone`, `pad_cold` (optional)
  - FX: `stinger`, `downlifter_soft`

#### `[x]` Kit: **Middle Eastern / Desert (stylized)** — `kit_middle_eastern.json`

- Use cases: deserts, bazaars, ancient ruins.
- Typical tempo: 70–150
- Required roles:
  - `oud_pluck`, `ney_lead` (or `lead_1`), `darbuka`
  - `drone`, optional `perc_clack`
  - FX: `stinger`, `whoosh`

#### `[x]` Kit: **South Asian (stylized)** — `kit_south_asian.json`

- Use cases: palaces/temples, bustling markets, story sequences.
- Typical tempo: 70–150
- Required roles:
  - percussion: `tabla`, `frame_drum`, `shaker`
  - `tanpura_drone` (or `drone`), `sitar_pluck`, optional `harmonium_keys`
  - melody: `bansuri_lead` (or `lead_1`)
  - FX: `stinger`, `downlifter_soft`

### 5.9 Experimental

#### `[ ]` Kit: **Glitch / IDM / Experimental Electronica**

- Use cases: sci-fi puzzles, abstract worlds, "weird tech" areas.
- Typical tempo: 80–170
- Required roles:
  - percussion: `perc_glitch`, `perc_click`, `hat_metallic`
  - `bass_sub`, `lead_bleep`, `texture_noise`
  - FX: `whoosh`, `impact`, `downlifter`

---

## 6) Quick coverage map (game modes to kits)

- **Menu / UI:** Lo-Fi / Chillhop, DX7 / FM Pop Keys, Puzzle / Minimal, Ambient / Exploration, Kawaii / Whimsical
- **Exploration / overworld:** Ambient / Exploration, Orchestral Adventure, Fantasy Cinematic, Celtic / Medieval Folk
- **Town / shop / hub:** DX7 / FM Pop Keys, Funk / Jazz-lite, Lo-Fi / Chillhop, Uplifting Pop / Indie Pop, Afrobeat / Afrofusion, Dub / Reggae
- **Combat (general):** Action EDM / Boss Fight, Metal / Heavy, DnB / Breakbeat, Cyberpunk / Industrial, Chiprock / Arcade Rock
- **Boss / setpiece:** Action EDM / Boss Fight, Bass Music / Halftime, Metal / Heavy, Cyberpunk / Industrial, Sci-Fi Hybrid Cinematic
- **Stealth / infiltration:** Stealth / Spy, Noir / Detective / Mystery, Dark Ambient / Horror (tension variants), Cyberpunk / Industrial (minimal variants)
- **Horror:** Dark Ambient / Horror
- **Racing / high speed:** Synthwave / Outrun, 909 / Acid / Rave, DnB / Breakbeat, FM / Arcade
- **Puzzle / strategy overlay:** Puzzle / Minimal, Ambient / Exploration, Chiptune 8-bit (light), Lo-Fi / Chillhop, Noir / Detective / Mystery
- **Strategy / tactics:** Military / March / Strategy, Dark Ambient / Horror (tension), Ambient / Exploration (minimal), Sci-Fi Hybrid Cinematic
- **Nautical / sea:** Pirate / Nautical, Dub / Reggae, Western / Desert (travel), Celtic / Medieval Folk (folk)
- **Underwater / dream:** Ethereal / Underwater / Dream, Ambient / Exploration, Fantasy Cinematic (soft)
- **Sports / event:** Stadium / Sports Hype, Chiprock / Arcade Rock, House / Techno

---

## 7) Next curation steps (for the public-facing "pack")

1. Implement remaining kits (16 remaining)
2. For each kit:
   - build presets (original synthesis)
   - build compose templates (`defs`) with variation knobs
   - create a short audition spec that cycles grooves, fills, bass, lead
3. Tune by ear in common players (XM/IT), and lock goldens.
4. Publish with clear licensing and a consistent naming/versioning scheme.

---

## 8) Gaps for future expansion

Potential kits not yet in the inventory:

- Nordic / Viking (chants, war drums, lyre/tagelharpa-ish, "raid" energy)
- Anime / J-Rock (bright leads, tight rock kit, power chords, fast hooks)
- Bluegrass / Appalachian (banjo/mandolin/fiddle, stomp/clap)
- Big Band / Swing (brass sections, walking bass, swing kit)
- Minimal Tech / Tension Pulses (ultra-minimal synth pulses + impacts for overlays)
