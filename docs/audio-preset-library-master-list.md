# Audio Preset Library — Master Inventory

**Last updated:** 2026-01-15
**Location:** `packs/preset_library_v1/audio/`
**Feature Analysis:** See [`FEATURE_ANALYSIS.md`](../packs/preset_library_v1/FEATURE_ANALYSIS.md) for detailed statistics

This document inventories the preset-spec JSON files under `packs/preset_library_v1/audio/`.

Notes:
- Counts and tables include preset specs (`*.json`) and exclude derived `*.report.json`.
- Files named `test_*.json` at the `audio/` root are integration/dev test specs, not presets.
- Run `python analyze_presets.py` to regenerate feature analysis for this or future packs.

---

## Feature Summary

| Metric | Value |
|--------|-------|
| Total Presets | 255 |
| Presets with LFO modulation | 159 (62.4%) |
| Presets with filters | 186 (72.9%) |
| Presets with effects | 244 (95.7%) |

### Top Synthesis Types
| Type | Usage Count | % of Layers |
|------|-------------|-------------|
| noise_burst | 231 | 19.0% |
| oscillator | 187 | 15.4% |
| fm_synth | 142 | 11.7% |
| multi_oscillator | 103 | 8.5% |
| modal | 98 | 8.1% |
| metallic | 90 | 7.4% |
| additive | 75 | 6.2% |
| karplus_strong | 74 | 6.1% |
| granular | 60 | 4.9% |
| wavetable | 53 | 4.4% |

### Effects Usage
| Effect | Usage Count | % of Effects |
|--------|-------------|--------------|
| reverb | 221 | 27.8% |
| compressor | 183 | 23.0% |
| chorus | 160 | 20.2% |
| delay | 95 | 12.0% |
| waveshaper | 75 | 9.4% |
| phaser | 41 | 5.2% |
| bitcrush | 19 | 2.4% |

### Quality Metrics
- **Average layers per preset:** 4.8
- **Most common layer count:** 4 (29% of presets)
- **Most common effect count:** 3 (34% of presets)
- **Most used waveform:** sine (501 uses)
- **Most used filter:** lowpass (490 uses)
- **Most used LFO target:** filter_cutoff (175 uses)

---

## Directory Structure

```
audio/
├── bass/           (24 presets)
├── bells/          (9 presets)
├── drums/
│   ├── cymbals/    (5 presets)
│   ├── hats/       (6 presets)
│   ├── kicks/      (8 presets)
│   ├── percussion/ (31 presets)
│   ├── snares/     (17 presets)
│   └── toms/       (6 presets)
├── fx/             (27 presets)
├── keys/           (15 presets)
├── leads/
│   ├── acoustic/   (13 presets)
│   └── synth/      (23 presets)
├── pads/           (24 presets)
├── plucks/         (35 presets)
└── textures/       (12 presets)
```

**Total: 255 presets**

---

## 1) Drums

### 1.1 Kicks (`drums/kicks/`)

| Preset | Description |
|--------|-------------|
| `big_kick` | Cinematic impact kick - epic thunderous layered sound with long tail, trailer-style |
| `kick` | Punchy kick drum - sine body with pitch sweep + transient click |
| `kick_4` | Four-on-the-floor house kick - tight punchy club-ready sound with transient click |
| `kick_808` | Classic 808 kick - long sub bass decay |
| `kick_909` | Classic 909 kick - punchy with click transient |
| `kick_dist` | Distorted kick drum - aggressive saturated punch with grit, industrial character |
| `kick_soft` | Soft kick drum - warm pillowy body with gentle attack, ideal for lo-fi and ambient |
| `toy_kick` | Toy kick drum - playful cute plastic sound, chiptune-adjacent with bright tone |

### 1.2 Snares & Claps (`drums/snares/`)

| Preset | Description |
|--------|-------------|
| `clap` | Electronic clap - layered noise bursts with slight delays |
| `clap_909` | Classic 909 clap - layered noise bursts with 3-5ms offsets |
| `clap_vinyl` | Vinyl clap - warmer, filtered clap with lo-fi character |
| `finger_snap` | Finger snap - very short, bright transient with bandpass |
| `gated_snare` | Gated snare - hard gate at 0.15s, synthwave style with big impact |
| `rimshot` | Rimshot - short, bright, metallic click with high-pitched body |
| `snare` | Punchy snare - tonal body + filtered noise for snare wires |
| `snare_909` | Classic 909 snare - noise burst + resonant body at 180Hz |
| `snare_brush` | Brush snare - noise-based with longer envelope, swishing character |
| `snare_dist` | Distorted snare - aggressive, crunchy character with high resonance |
| `snare_heavy` | Heavy rock/metal snare - deep body, powerful crack, aggressive character |
| `snare_march` | Marching snare drum - tight, crisp, military character with pronounced snares |
| `snare_noise` | Noise snare - pure filtered noise, no tonal body |
| `snare_rim` | Rim-focused snare - emphasis on the rim click/crack with less body |
| `snare_soft` | Soft snare - gentle, brushed character with filtered highs |
| `snare_tight` | Tight snare - very short, punchy with fast decay |
| `toy_snare` | Toy snare - playful, plastic-sounding with higher pitch |

### 1.3 Hats (`drums/hats/`)

| Preset | Description |
|--------|-------------|
| `hat_closed` | Closed hi-hat - metallic inharmonic partials + noise |
| `hat_metallic` | Metallic hi-hat - emphasizes inharmonic partials for pronounced metal character |
| `hat_open` | Open hi-hat - longer decay metallic sound |
| `hat_pedal` | Pedal hi-hat - foot pedal character, slightly darker than closed hat |
| `hat_tick` | Tick hi-hat - very short, bright click with metallic character |
| `noise_hat` | Noise hi-hat - pure filtered noise, chiptune digital character |

### 1.4 Cymbals (`drums/cymbals/`)

| Preset | Description |
|--------|-------------|
| `china` | China cymbal - trashy, highly inharmonic character with quick attack |
| `crash` | Crash cymbal - explosive attack with long decay, wide spectrum |
| `cym_swell` | Cymbal swell - building cymbal wash that crescendos |
| `cymbal` | Cymbal - long decay with complex metallic partials |
| `ride` | Ride cymbal - bell-like attack with sustained wash |

### 1.5 Toms (`drums/toms/`)

| Preset | Description |
|--------|-------------|
| `electronic_tom` | Electronic tom - synth body with triangle wave for harmonics |
| `tom_floor` | Floor tom - very low pitched body with deep resonance |
| `tom_high` | High tom - pitched body with fast decay |
| `tom_low` | Low tom - deep pitched body with long decay |
| `tom_mid` | Mid tom - pitched body with medium resonance |
| `toms` | Multi-tom hit - layered tom sound covering multiple frequency ranges |

### 1.6 Percussion (`drums/percussion/`)

| Preset | Description |
|--------|-------------|
| `big_drum` | Big orchestral bass drum - massive low-end impact with cinematic character |
| `bongo_high` | High bongo - bright short pitched body |
| `bongo_low` | Low bongo - warm short pitched body |
| `cajon` | Cajon - box drum with mix of kick and snare character |
| `clacks` | Clacks - double wooden click/clack sound like castanets or stick clicks |
| `claves` | Claves - bright hard wood click |
| `conga_high` | High conga - bright pitched body with quick pitch drop |
| `conga_low` | Low conga - deep pitched body with moderate resonance |
| `cowbell` | Cowbell - two inharmonic metallic partials, classic 808 style |
| `darbuka` | Darbuka/doumbek - Middle Eastern goblet drum with tight, punchy character |
| `djembe` | Djembe - pitched body with sharp slap character |
| `frame_drum` | Frame drum - deep resonant hand drum with warm body and natural decay |
| `guiro` | Guiro - raspy noise texture with rhythmic character |
| `perc_1` | Generic percussion hit 1 - versatile mid-frequency percussion with quick decay |
| `perc_break_layer` | Breakbeat percussion layer - punchy mid-freq hit for layering with kicks/snares |
| `perc_clack` | Percussion clack - woody mid-frequency clacking sound like sticks hitting |
| `perc_click` | Percussion click - sharp, high-frequency transient for rhythmic accents |
| `perc_conga` | Alternative conga - mid-range pitched hand drum with Latin character |
| `perc_glitch` | Glitchy percussion - digital/electronic glitch percussion with artifacts |
| `perc_metal` | Metallic percussion - high-frequency metallic hit with shimmer |
| `perc_shaker` | Percussion shaker variant - alternative shaker texture with different character |
| `shaker` | Shaker - filtered white noise with fast attack-release |
| `sidestick` | Sidestick - very short bright click/pop |
| `tabla_high` | High tabla - pitched body with specific harmonic ratios |
| `tabla_low` | Low tabla - deep resonant body with rich harmonics |
| `taiko` | Taiko - large pitched body with sub emphasis and long decay |
| `tambourine` | Tambourine - metallic jingles with noise shake |
| `timbale` | Timbale - metallic bright pitched body with long sustain |
| `timp_hit` | Timpani hit - orchestral kettle drum with resonant pitched tone |
| `triangle_perc` | Triangle - high metallic with long sustain |
| `woodblock` | Woodblock - short bright click with wood-like character |

---

## 2) Bass (`bass/`)

| Preset | Description |
|--------|-------------|
| `bass_808` | TR-808 style sub bass with long decay, classic boomy character |
| `bass_acid` | Acid bass - TB-303 style with high resonance lowpass filter and dramatic envelope sweep for squelchy acid character |
| `bass_aggressive` | Aggressive bass - sawtooth through waveshaper distortion for growling, in-your-face character |
| `bass_cz` | Casio CZ-style bass with resonant phase distortion and envelope decay |
| `bass_dist` | Distorted bass - sawtooth through heavy waveshaper for crunchy, saturated distortion character |
| `bass_dub` | Dub reggae bass - warm, round, and deep with slight filtering |
| `bass_dubstep` | Fast aggressive dubstep bass - detuned sawtooths with rapid LFO and waveshaper for gritty character |
| `bass_dx7` | DX7 bass - classic Yamaha DX7 FM bass with high modulation index for glassy, complex harmonic spectrum |
| `bass_fm` | FM bass - 2-operator FM synthesis with 1:1 ratio for punchy, percussive bass with metallic attack |
| `bass_growl` | Growling wobble bass - slower LFO rate with high resonance for deep, aggressive movement |
| `bass_mid` | Mid-focused bass - warm sawtooth filtered at 800Hz for rich harmonic content with controlled highs |
| `bass_muted` | Muted bass - heavily filtered at 300Hz with short decay for dull, muffled palm-muted character |
| `bass_opl` | OPL bass - FM synthesis mimicking Yamaha OPL chip with bright digital character and harmonic richness |
| `bass_pick` | Picked bass - Karplus-Strong synthesis with fast attack and moderate decay for articulate plucked character |
| `bass_pluck` | Plucky bass - Karplus-Strong for organic attack (loop-safe post-decay) |
| `bass_reese` | Reese bass - 4-voice detuned sawtooths creating characteristic phasing and movement for DnB and dubstep |
| `bass_reese_mod` | Modulated Reese bass - 4 detuned sawtooths with slow filter LFO for evolving DnB movement |
| `bass_round` | Round bass - warm sine foundation with gentle saw harmonics, heavily filtered for smooth character |
| `bass_saw` | Saw bass - raw sawtooth with dramatic filter envelope sweep on attack for classic synth bass movement |
| `bass_sub` | Sub bass - pure sine for clean low-end (loop-safe) |
| `bass_talking` | Talking bass - fast filter LFO with high resonance creating vowel-like formant movement |
| `bass_triangle` | Triangle bass - pure triangle wave for clean chiptune sub bass with soft odd harmonics |
| `bass_upright` | Upright bass - Karplus-Strong with long decay and warm blend for acoustic double bass character |
| `bass_wobble` | Classic dubstep wobble bass - sawtooth oscillator with LFO modulating filter cutoff for rhythmic movement |

---

## 3) Keys (`keys/`)

| Preset | Description |
|--------|-------------|
| `celesta` | Celesta/music box - high metallic synthesis with crystalline bell-like quality |
| `dx7_ep` | DX7-style FM electric piano - classic 80s FM EP sound |
| `dx7_marimba` | DX7-style FM marimba - warm wooden mallet FM sound |
| `fm_keys` | FM keyboard - general purpose FM synthesis keyboard sound |
| `harmonium` | Warm harmonium or pump organ character with chorus for breath-like quality |
| `harmonium_keys` | Harmonium keyboard variant - reed organ with rich, breathy character |
| `keys_ep` | Electric piano - Rhodes/Fender Rhodes style FM synthesis with bell-like attack and warm decay |
| `keys_organ` | Classic rock organ with middle registration blend using wavetable synthesis |
| `keys_organ_full` | Full powerful organ with rich registration and subtle unison for body |
| `keys_wurli` | Wurlitzer electric piano - brighter FM synthesis with more bark and shorter decay |
| `mallet` | Generic mallet instrument - Karplus-Strong with clear tone and medium decay |
| `piano` | Acoustic piano - Karplus-Strong physical modeling with hammer noise transient |
| `piano_bright` | Bright honky-tonk piano - percussive attack, slight detuning for character |
| `piano_soft` | Soft acoustic piano - warmer Karplus-Strong with gentle attack and subtle hammer |
| `vibes` | Vibraphone - metallic synthesis with low inharmonicity, long sustain and spacious reverb |

---

## 4) Bells (`bells/`)

| Preset | Description |
|--------|-------------|
| `bell` | Standard bell - medium-sized bell with clear fundamental and metallic overtones |
| `bell_glass` | Glass bell - crystalline, bright bell with high inharmonicity and quick decay |
| `bell_soft` | Soft bell - muted, gentle bell with reduced overtones and warm character |
| `bell_tubular` | Tubular bell - deep, resonant chime with long sustain and moderate inharmonicity |
| `dx7_bell` | DX7 bell - classic FM synthesis bell with high modulation index and 1:1 ratio |
| `fm_bell` | FM bell - experimental FM synthesis bell with 1:3.5 ratio for unique timbre |
| `modal_bell` | Modal synthesis bell with physically-modeled resonant modes - crystalline and realistic |
| `music_box` | Music box - delicate, high-pitched bell with toy-like character and quick decay |
| `opl_bell` | OPL/AdLib style FM bell - retro PC game bell sound with metallic YM3812 character |

---

## 5) Leads

### 5.1 Synth Leads (`leads/synth/`)

| Preset | Description |
|--------|-------------|
| `brass_swell` | Swelling brass lead - sawtooth stack with filter LFO for breathing, dynamic brass character |
| `dx7_lead` | DX7-style FM lead - classic 80s FM synth lead with bright, cutting tone |
| `fm_lead` | FM synthesis lead - bright and cutting with harmonic complexity for modern electronic |
| `lead_1` | Classic synth lead - detuned sawtooths (loop-safe) |
| `lead_2` | Second lead voice - bright saw with resonant filter and subtle vibrato |
| `lead_acid` | Acid lead - TB-303 style sawtooth with high resonance filter for squelchy acid lines |
| `lead_alarm` | Alarm lead - bright square wave that demands attention |
| `lead_bleep` | Clean digital bleep - simple sine and triangle for pure tones |
| `lead_bright` | Bright unfiltered saw stack - cuts through any mix with full harmonic content |
| `lead_cute` | Cute playful lead - soft triangle and pulse for charming melodies |
| `lead_heroic` | Heroic epic lead - thick detuned saw stack with chorus for triumphant themes |
| `lead_hoover` | Hoover lead - heavily detuned saws with pitch down-sweep for mentasm/rave character |
| `lead_muted` | Muted soft lead - heavily filtered for warm background melodies |
| `lead_sparse` | Sparse minimal lead - single naked sawtooth oscillator for raw character |
| `lead_trance` | Classic trance lead - supersaw style with fast square wave LFO for gated tremolo effect |
| `lead_vibrato` | Expressive vibrato lead - sawtooth with pitch LFO for musical vibrato character |
| `lead_wavetable` | Cutting evolving lead with digital wavetable character and slow position modulation |
| `lead_whistle` | Synth whistle lead - pure, airy whistle-like lead sound |
| `opl_brass` | OPL/AdLib style FM brass - retro PC game brass sound |
| `opl_lead` | OPL chip lead - FM synthesis mimicking Yamaha OPL for retro PC game character |
| `pulse_chords` | Pulse chord lead - thick pulse wave stack optimized for chord voicings |
| `pulse_lead` | Pulse wave lead - classic PWM character with narrow duty cycle for nasal tone |
| `sax_lead` | Synth saxophone lead - breathy, expressive sax-like synth |

### 5.2 Acoustic Leads (`leads/acoustic/`)

| Preset | Description |
|--------|-------------|
| `accordion_lead` | Accordion melody voice - reed character with detuned voices and bellows vibrato |
| `bagpipe_chanter` | Bagpipe chanter - nasal, buzzy with strong odd harmonics and drone-like sustain |
| `bansuri_lead` | Bansuri flute lead - Indian bamboo flute with warm, flowing character |
| `fiddle_lead` | Fiddle/violin lead - folk-style bowed string lead |
| `flute` | Concert flute - pure sine with breath noise and vibrato |
| `flute_pan` | Pan flute - airy character with prominent breath noise |
| `harmonica` | Blues harmonica - reedy FM synthesis with expressive vibrato |
| `ney_lead` | Ney flute lead - Middle Eastern end-blown flute with haunting, breathy tone |
| `ocarina` | Ocarina - pure, round sine tone with subtle vibrato (Zelda-esque) |
| `recorder` | Recorder/block flute - bright with subtle odd harmonics |
| `shakuhachi_lead` | Shakuhachi flute lead - Japanese bamboo flute with breathy, meditative character |
| `whistle` | Tin whistle - pure, bright tone with fast attack and vibrato |
| `woodwind_lead` | Generic woodwind lead - breathy wind instrument sound |

---

## 6) Pads (`pads/`)

| Preset | Description |
|--------|-------------|
| `atmosphere` | Ambient atmosphere - layered pink noise and low sine with large grains and wide stereo spread |
| `choir_ethereal` | Ghostly vocal / choir texture - formant source with medium grains and subtle pitch variation |
| `choir_pad` | Choir pad - vocal-like pad with 'aah' character |
| `chord_stab` | Bright chord stab - multi-osc stack (loop-safe) |
| `formant_choir` | Formant synthesis choir pad with breathy vowel character |
| `pad` | Basic pad - simple, neutral synth pad for chord playing |
| `pad_breathing` | Breathing meditation pad - soft pad with very slow combined volume and filter modulation for calming, ambient character |
| `pad_cold` | Cold pad - icy, ethereal pad with high-frequency shimmer |
| `pad_dark` | Dark pad - low, ominous pad with minimal high frequencies |
| `pad_digital` | Bright harsh digital pad with cutting high-frequency character |
| `pad_evolving` | Constantly morphing pad that sweeps through entire wavetable for evolving texture |
| `pad_granular` | Textured granular pad - sawtooth with medium grains and pitch spread for shimmer |
| `pad_shimmer` | Shimmering, sparkling pad - higher octave sine with small grains and high pitch spread |
| `pad_slow_lfo` | Evolving pad - warm detuned oscillators with very slow filter LFO for breathing, ambient movement |
| `pad_sweep` | Trance-style sweep pad with fast position modulation for rhythmic movement |
| `pad_tremolo` | Pulsing tremolo pad - warm pad with volume LFO for rhythmic, organic pulsing character |
| `pad_vocal` | Choir-like vocal pad using formant wavetable for vowel-based texture |
| `pad_warm` | Warm pad - detuned oscillators with static lowpass (loop-safe) |
| `poly_pad` | Thick polyphonic analog pad with slow morphing and wide unison spread |
| `string_pad` | String pad - lush string ensemble pad for sustained chords |
| `string_stacc` | Staccato strings - short, punchy bowed string hits with quick decay for rhythmic patterns |
| `strings_ensemble` | Lush string ensemble - multi-oscillator strings with slow pitch vibrato and filter movement for orchestral character |
| `supersaw_pad` | Classic supersaw pad with massive unison spread for thick saw-based texture |
| `vector_pad` | Vector synthesis evolving pad morphing between four timbres |

---

## 7) Plucks (`plucks/`)

| Preset | Description |
|--------|-------------|
| `accordion` | Accordion - reed instrument with bellows character |
| `arp` | Basic arp - simple arpeggiator-friendly pluck sound |
| `arp_bright` | Bright short pluck optimized for arpeggios - fast attack and decay with crystalline character |
| `arp_filter_sweep` | Rhythmic arp pluck - plucky sound with synced-feel filter LFO for dynamic, moving arpeggios |
| `arp_pulse` | Pulse wave arp lead - bright and snappy (loop-safe) |
| `arp_soft` | Soft gentle arpeggio pluck with triangle waveform - smooth melodic character |
| `brass_low` | Low brass stab - deep brass/trombone-like stab sound with rich harmonics |
| `brass_stab` | Brass stab - punchy brass section hit for stab patterns |
| `chord_pluck` | Chord pluck - full, rich pluck designed for chord stabs |
| `chord_pluck_muted` | Muted chord pluck - filtered, softer chord pluck sound |
| `concertina` | Concertina - small accordion-like reed instrument with brighter tone |
| `guitar_chops` | Guitar chops - funky muted guitar scratch/chop sound |
| `guitar_lead` | Lead guitar - electric guitar lead with sustain |
| `guitar_pluck` | Guitar pluck - single clean guitar note pluck |
| `guitar_rhythm` | Rhythm guitar - strummed acoustic guitar chord character |
| `guitar_skank` | Skank guitar - reggae/ska offbeat guitar stab |
| `guitar_twang` | Twangy guitar - country/surf style bright guitar with characteristic spring reverb twang |
| `harp` | Harp - plucked string harp sound |
| `koto_pluck` | Koto pluck - Japanese zither pluck with bright, clear attack |
| `lute_pluck` | Lute pluck - medieval/renaissance plucked string sound with warm, rounded tone |
| `oud_pluck` | Oud pluck - Middle Eastern lute-like pluck with warm, defined character |
| `pluck` | Classic Karplus-Strong pluck with medium decay and neutral blend - versatile plucked string sound |
| `pluck_bright` | Bright pluck - shimmering, high-frequency rich pluck sound with sparkly character |
| `pluck_glass` | Bright crystalline glass pluck with low blend and short decay - shimmering character with subtle reverb |
| `pluck_nylon` | Warm nylon string pluck with medium-long decay - classic acoustic guitar character |
| `pluck_soft` | Soft mellow pluck with high blend and longer decay - warm nylon-like character |
| `pluck_steel` | Bright metallic steel string pluck with very low blend and fast decay - crisp attack |
| `power_chord_stab` | Power chord stab - heavy guitar-like power chord hit |
| `rave_stab` | Rave stab - classic 90s rave chord stab sound |
| `sitar_pluck` | Sitar pluck - Indian sitar-style pluck with sympathetic resonance |
| `stab` | Classic punchy stab with stacked sawtooth oscillators - fast attack and short decay |
| `stab_dark` | Dark filtered stab with lowpass filter - deep and moody character |
| `stab_rave` | Bright detuned rave stab with chorus effect - classic 90s dance character with width |
| `stacked_saw_chords` | Stacked saw chords - thick supersaw-style chord stab |
| `tanpura_drone` | Tanpura drone - Indian drone instrument with continuous resonance |

---

## 8) Textures (`textures/`)

| Preset | Description |
|--------|-------------|
| `drone` | Basic drone - sustained low-frequency drone texture |
| `drone_dark` | Dark, ominous drone - low sawtooth tone with large grains and high density |
| `drone_light` | Ethereal, bright drone - sine tone with medium grains and moderate pitch spread |
| `drone_pulsing` | Pulsating drone texture - triangle wave with rhythmic grain density variation |
| `drone_soft` | Soft drone - gentle, filtered drone texture |
| `sub_rumble` | Deep sub rumble / earthquake - brown noise and low sine with very large grains |
| `texture_air` | Airy, breathy texture - white noise with small grains and wide stereo spread |
| `texture_bubbles` | Bubble texture - underwater bubbling effect |
| `texture_digital` | Glitchy digital artifacts - square wave tone with very small grains and high pitch spread |
| `texture_noise` | Noise texture - filtered noise for atmospheric layers |
| `texture_organic` | Natural, organic texture - brown noise with medium grains and gentle modulation |
| `texture_vinyl` | Vinyl crackle / lo-fi character - pink noise with tiny grains and position randomization |

---

## 9) FX (`fx/`)

| Preset | Description |
|--------|-------------|
| `am_tremolo` | AM tremolo effect - pulsating amplitude modulation creating rhythmic volume fluctuation |
| `downlifter` | Downlifter - falling noise sweep for descending energy and tension release |
| `downlifter_impact` | Downlifter with impact finish - falling noise sweep with delayed hit at the end |
| `downlifter_soft` | Soft downlifter - gentle descending sweep for subtle transitions and energy release |
| `downlifter_tension` | Tension downlifter - dramatic descending sweep that builds unease |
| `glitch` | Glitch effect - digital glitch/malfunction sound effect |
| `impact` | Heavy impact - low pitched body with noise crunch |
| `impact_metal` | Metallic clang with weight - high inharmonicity burst with sub layer |
| `impact_reverse` | Reverse impact - 'sucking in' effect with slow attack and fast decay |
| `impact_sub` | Big sub impact with pitch drop - trailer-style deep hit |
| `low_hit` | Deep punchy low-end hit - 60Hz to 25Hz pitch drop |
| `pickup` | Pickup/collect - bright ascending tone |
| `reverse_cymbal` | Classic reverse cymbal build - metallic synthesis with slow attack and quick release |
| `ring_mod_metallic` | Metallic ring modulation with inharmonic sidebands - creates bell-like metallic timbres through frequency multiplication |
| `riser` | Rising sweep - pitch up with noise buildup |
| `riser_tension` | Building tension riser - noise with highpass filter sweep for rising pitch feel |
| `riser_white` | Classic white noise riser - bandpass sweep upward for tension build |
| `stinger` | Musical stinger - quick bright hit with tail |
| `stinger_horror` | Horror stinger - scary, dissonant accent for horror games |
| `stinger_soft` | Soft stinger - gentle musical accent/stinger |
| `swell` | Musical swell - crescendo effect that builds in volume and intensity |
| `swell_brass` | Bold cinematic brass swell - long crescendo attack with bright harmonic content |
| `swell_strings` | Orchestral string swell - multi-oscillator saw stack with long attack and reverb |
| `transition_sweep` | Versatile transition sweep - noise and oscillator sweeping together for smooth transitions |
| `vocoder_robot` | Classic robot voice vocoder with animated formant patterns - multi-band vocoder effect creating synthetic voice-like textures |
| `whoosh` | Whoosh sweep - filtered noise passing through |
| `whoosh_soft` | Soft whoosh - gentle passing/transition sound |

---

## Notes on `.report.json`

`*.report.json` files next to preset specs are derived artifacts produced by `speccade validate` / `speccade generate`.
They are useful for QA (hashes/metrics/warnings), but should be treated as regeneratable outputs rather than source-of-truth presets.
