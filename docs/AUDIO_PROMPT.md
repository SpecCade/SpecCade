# SpecCade Audio & Music Content Development Prompt

**Purpose:** Self-contained workflow prompt for developing the audio preset library and genre kits.
**Usage:** Feed this document to Claude to execute batch development, validation, and human review workflows.

---

## Context

You are working on SpecCade's audio preset library and music genre kits. This prompt enables you to:
- **Implement engine enhancements first** (wavetable, granular, modulators, effects)
- Develop audio presets efficiently in batches (after engine features are ready)
- Validate and export generated audio
- Coordinate with human reviewers for quality assurance
- Build genre kit templates with proper instrument mappings

> **IMPORTANT:** Follow the implementation order: Engine Enhancements → Audio Presets → Genre Kits.
> New presets may require engine features. Check Phase 0 dependencies before implementing complex presets.

---

## Workspace Reference

| Resource | Path |
|----------|------|
| Spec Reference | `docs/spec-reference/audio.md` |
| Preset Library | `packs/preset_library_v1/audio/` |
| Work Tracker | `docs/AUDIO_WORK_TRACKER.md` |
| Master Preset List | `docs/audio-preset-library-master-list.md` |
| Genre Kits Spec | `docs/music-genre-kits-master-list.md` |
| Genre Kits Audit | `docs/music-genre-kits-audit.md` |

### CLI Commands

```bash
# Validate a spec (checks schema, constraints, determinism)
speccade validate packs/preset_library_v1/audio/<name>.json

# Generate audio from spec (outputs WAV + report)
speccade generate packs/preset_library_v1/audio/<name>.json

# Validate all presets in library
for f in packs/preset_library_v1/audio/*.json; do speccade validate "$f"; done

# Generate all presets
for f in packs/preset_library_v1/audio/*.json; do speccade generate "$f"; done
```

---

## Phase 0: Audio Engine Enhancements (IMPLEMENT FIRST)

Before developing presets, implement engine features that presets will depend on.

### 0.1 Engine Feature Priority Order

Implement in this order to maximize value:

| Priority | Features | Why First |
|----------|----------|-----------|
| **Q (Quick wins)** | Wavetable oscillator, Granular synth, LFO modulators | Enables rich pads, textures, wobble bass |
| **I (Isolated)** | Effect chain, Loudness targets, Loop points, One-shot/loop pairing | Improves all preset quality |
| **G (Gap-fillers)** | Foley helpers, Convolution reverb, Batch variations | Enables complex FX and variations |

### 0.2 Engine Feature → Preset Dependencies

Check this before implementing presets that require new features:

| Engine Feature | Presets That Need It |
|----------------|---------------------|
| Wavetable oscillator | pad_shimmer, poly_pad, complex leads, choir_pad |
| Granular synth | texture_*, drone_*, atmospheric sounds |
| LFO modulators | bass_wobble, vibrato effects, tremolo |
| Effect chain (reverb) | pad_*, ambient sounds, spatial FX |
| Effect chain (delay) | echoed leads, dub bass |
| Effect chain (chorus) | poly_pad, detuned sounds |
| Foley helpers | impact_*, whoosh_*, complex FX layers |

### 0.3 Engine Development Workflow

```
1. SELECT    → Choose next engine feature from Section 1 of AUDIO_WORK_TRACKER.md
2. DESIGN    → Document API/schema changes needed
3. IMPLEMENT → Add feature to audio generator
4. TEST      → Create test preset using new feature
5. VALIDATE  → Ensure determinism, quality metrics
6. UPDATE    → Mark complete in AUDIO_WORK_TRACKER.md
```

### 0.4 Minimum Engine Features Before Preset Work

You can begin basic preset development (drums, simple synths) immediately. However, wait for these features before developing complex presets:

| Preset Category | Required Engine Features |
|-----------------|-------------------------|
| Basic drums (kicks, snares, hats) | None - can proceed now |
| Simple bass (sub, pluck) | None - can proceed now |
| Simple leads | None - can proceed now |
| Wobble bass | LFO modulators |
| Textured pads | Wavetable or Granular |
| Atmospheric drones | Granular synth |
| Complex FX (impacts, whooshes) | Foley helpers (recommended) |
| Spatial sounds | Effect chain (reverb) |

---

## Phase 1: Audio Preset Development

> **Note:** Complete relevant engine features from Phase 0 before implementing presets that need them.

### 1.1 Development Loop (Single Preset)

```
0. CHECK DEPS → Verify required engine features are implemented (see Phase 0)
1. SELECT     → Choose next preset from AUDIO_WORK_TRACKER.md (prioritize Tier 1)
2. RESEARCH   → Read similar existing presets for synthesis patterns
3. WRITE      → Create JSON spec following audio_v1 format
4. VALIDATE   → Run: speccade validate <spec.json>
5. GENERATE   → Run: speccade generate <spec.json>
6. REPORT     → Check .report.json for quality metrics
7. QUEUE      → Add to "Ready to Validate" in AUDIO_WORK_TRACKER.md
8. AWAIT      → Human reviews and marks APPROVED or NEEDS IMPROVEMENT
```

### 1.2 Parallel Dispatch Strategy

Batch presets by synthesis type for efficient development:

| Batch | Synthesis Type | Example Presets |
|-------|----------------|-----------------|
| **OSC** | Oscillator-based | kicks, snares, basses (sub, mid, round) |
| **FM** | FM synthesis | dx7_*, fm_*, opl_* |
| **KS** | Karplus-Strong | plucks, guitars, harps, strings |
| **NOISE** | Noise-based | hats, claps, textures, noise_snare |
| **METAL** | Metallic | bells, cymbals, metallic_perc |
| **SWEEP** | Pitch/filter sweeps | risers, downlifters, impacts, whooshes |

**Recommended batch size:** 5-10 presets per session

### 1.3 Model Selection for Tasks

Use the appropriate model for each task type to optimize speed and cost:

| Task | Model | Rationale |
|------|-------|-----------|
| **Write preset specs** | Sonnet | Structured creativity, follows templates |
| **Run validation/generation** | Haiku | Simple command execution |
| **Read/check reports** | Haiku | Simple file parsing |
| **Update tracker** | Haiku | Simple edits |
| **Process feedback & revise** | Sonnet | Requires understanding and creative adjustment |
| **Batch preset creation** | Sonnet | Parallel spec writing |
| **Genre kit template design** | Sonnet | Music theory, pattern design |
| **Complex kit (orchestral, regional)** | Opus | Nuanced creative decisions |
| **Research existing presets** | Haiku | Simple lookups |
| **Architectural decisions** | Opus | Complex trade-offs |

**Cost/Speed Summary:**
- **Haiku** — Fastest, cheapest. Use for mechanical tasks.
- **Sonnet** — Balanced. Default for preset/kit development.
- **Opus** — Most capable. Reserve for complex creative work.

### 1.4 Preset JSON Template

```json
{
  "spec_version": 1,
  "asset_id": "preset_<name>",
  "asset_type": "audio",
  "license": "CC0-1.0",
  "seed": <unique_number>,
  "description": "<brief description of sound character>",
  "outputs": [{ "kind": "primary", "format": "wav", "path": "<name>.wav" }],
  "recipe": {
    "kind": "audio_v1",
    "params": {
      "duration_seconds": <float>,
      "sample_rate": 44100,
      "base_note": "<C4 for pitched instruments>",
      "layers": [
        {
          "synthesis": { "type": "...", ... },
          "envelope": { "attack": 0.01, "decay": 0.1, "sustain": 0.5, "release": 0.1 },
          "volume": 1.0,
          "pan": 0.0
        }
      ]
    }
  }
}
```

---

## Phase 2: Export & Validation

### 2.1 Validation Pipeline

After generating a preset, check the `.report.json` file:

```bash
# Generate preset
speccade generate packs/preset_library_v1/audio/kick_soft.json

# Check report (created alongside the spec)
cat packs/preset_library_v1/audio/kick_soft.report.json
```

### 2.2 Automated Quality Checks

| Metric | Pass Criteria | Notes |
|--------|---------------|-------|
| Peak Level | < 0 dB | No clipping |
| RMS Level | -24 to -6 dB | Appropriate for one-shots |
| DC Offset | < 0.01 | No audible DC bias |
| Duration | Matches spec | ± 0.01s tolerance |

### 2.3 Manual Quality Checklist

For each preset before marking "Ready to Validate":

```
[ ] Spec validates without errors
[ ] WAV generates successfully
[ ] Report shows no clipping (peak < 0 dB)
[ ] Report shows reasonable RMS (-24 to -6 dB)
[ ] Report shows no DC offset
[ ] Attack/release are click-free (listen test)
[ ] base_note is set correctly (pitched instruments)
[ ] Sound matches intended role (e.g., "kick" sounds like a kick)
```

---

## Phase 3: Human Review Workflow

### 3.1 Review States

| State | Meaning | Action |
|-------|---------|--------|
| **PENDING** | Not yet generated | Claude develops preset |
| **READY TO VALIDATE** | Generated, awaiting human listen | Human reviews audio |
| **APPROVED** | Passed human review | Move to "Recently Approved" |
| **NEEDS IMPROVEMENT** | Failed review | Claude revises based on feedback |

### 3.2 Updating the Work Tracker

When Claude generates a preset:

```markdown
## Ready to Validate

| Preset | Generated | Notes |
|--------|-----------|-------|
| kick_soft | 2026-01-13 | Softer attack than kick, gentle body |
| snare_soft | 2026-01-13 | Quieter, less transient |
```

When human reviews:

**If APPROVED:**
```markdown
## Recently Approved

| Preset | Approved Date | Validator |
|--------|---------------|-----------|
| kick_soft | 2026-01-13 | @rdave |
```

**If NEEDS IMPROVEMENT:**
```markdown
## Needs Improvement

| Preset | Issue | Assigned |
|--------|-------|----------|
| snare_soft | Too much high-end click, needs filter | Claude |
```

### 3.3 Revision Loop

When a preset needs improvement:

1. Human describes the issue in "Needs Improvement" table
2. Claude reads feedback and revises the spec
3. Claude regenerates and runs validation
4. Claude moves preset back to "Ready to Validate" with revision notes
5. Human re-reviews

---

## Phase 4: Genre Kit Development

### 4.1 Kit Structure

Each genre kit includes:

1. **Audio presets** — All `audio_v1` specs for the kit
2. **Compose template** — `music.tracker_song_compose_v1` with defs + patterns
3. **Role aliasing** — `instrument_ids` / `channel_ids` mapping
4. **Audition spec** — Short demo for listening test

### 4.2 Standard Role Mapping

All kits must provide these standard role aliases:

```json
{
  "instrument_ids": {
    "kick": 0,
    "snare": 1,
    "hat_closed": 2,
    "bass_sub": 3,
    "lead_1": 4,
    "pad": 5,

    // Style-specific aliases point to same indices
    "kick_808": 0,  // -> kick
    "acid_bass": 3  // -> bass_sub
  }
}
```

### 4.3 Kit Completion Checklist

For each kit:

```
[ ] All required presets exist in preset library
[ ] Compose template with defs + pattern examples
[ ] Standard role aliases in instrument_ids
[ ] Default timebase set appropriately
[ ] Audition spec generates valid XM/IT
[ ] Demo sounds correct in common tracker players
```

---

## Quality Guidelines by Category

### Drums — Kicks

| Variant | Synthesis | Duration | Character |
|---------|-----------|----------|-----------|
| `kick` | Sine + noise click | 0.3-0.5s | Punchy, versatile |
| `kick_soft` | Sine, slower attack | 0.3-0.5s | Gentle, round |
| `kick_808` | Sine, long tail | 0.5-0.8s | Sub-heavy, tunable |
| `kick_909` | Sine + pitch sweep | 0.3-0.4s | Punchy transient |
| `kick_dist` | Sine + saturation | 0.3-0.5s | Gritty, aggressive |
| `big_kick` | Layered sine + body | 0.5-0.8s | Cinematic impact |
| `toy_kick` | FM, bright | 0.2-0.3s | Cute, plastic |

### Drums — Snares

| Variant | Synthesis | Duration | Character |
|---------|-----------|----------|-----------|
| `snare` | Noise + tonal body | 0.2-0.4s | Standard snare |
| `snare_soft` | Filtered noise | 0.2-0.3s | Quieter, subtle |
| `snare_noise` | Pure noise burst | 0.1-0.2s | Chiptune style |
| `snare_909` | Noise + resonant body | 0.2-0.3s | Punchy, bright |
| `gated_snare` | Noise + hard gate | 0.15-0.25s | Synthwave |

### Drums — Hats

| Variant | Synthesis | Duration | Character |
|---------|-----------|----------|-----------|
| `hat_closed` | Metallic/noise HP | 0.05-0.15s | Tight, bright |
| `hat_open` | Metallic/noise | 0.3-0.8s | Sustained decay |
| `hat_metallic` | Metallic partials | 0.1-0.2s | Industrial |
| `noise_hat` | HP noise burst | 0.05-0.1s | Chiptune |

### Bass

| Variant | Synthesis | Duration | Character |
|---------|-----------|----------|-----------|
| `bass_sub` | Sine/triangle | 0.5-1.0s | Pure low end |
| `bass_mid` | Saw + LP filter | 0.3-0.5s | Presence, cut |
| `bass_round` | Sine + harmonics | 0.3-0.5s | Warm, full |
| `bass_pluck` | Saw + fast decay | 0.2-0.4s | Articulate |
| `bass_808` | Sine, long decay | 0.8-2.0s | Tunable sub |
| `bass_reese` | Detuned saws | 0.5-1.0s | DnB character |
| `bass_wobble` | LFO-modulated | 0.5-1.0s | Dubstep |

### Pads

| Variant | Synthesis | Duration | Character |
|---------|-----------|----------|-----------|
| `pad` | Multi-osc | 2.0-4.0s | Generic pad |
| `pad_warm` | Detuned saws + LP | 2.0-4.0s | Analog warmth |
| `pad_cold` | Saw + HP + reverb | 2.0-4.0s | Sterile, distant |
| `pad_dark` | Filtered, low | 2.0-4.0s | Ominous |
| `pad_shimmer` | High harmonics | 2.0-4.0s | Ethereal |

### Leads

| Variant | Synthesis | Duration | Character |
|---------|-----------|----------|-----------|
| `lead_1` | Saw/square | 0.5-1.0s | Versatile |
| `lead_muted` | Filtered saw | 0.5-1.0s | Subtle, background |
| `lead_heroic` | Bright, layered | 0.5-1.0s | Epic, bold |
| `lead_cute` | Triangle/sine | 0.5-1.0s | Soft, playful |

### FX

| Variant | Synthesis | Duration | Character |
|---------|-----------|----------|-----------|
| `riser` | Noise/pitch sweep up | 1.0-4.0s | Building tension |
| `downlifter` | Noise/pitch sweep down | 1.0-4.0s | Release |
| `impact` | Layered transient + sub | 0.5-1.5s | Hit |
| `whoosh` | Noise + BP sweep | 0.3-1.0s | Movement |
| `stinger` | Short musical phrase | 0.3-1.0s | Punctuation |

---

## Batch Commands

Use these prompts to execute common workflows. Each includes the recommended model.

### Engine Feature Development (Opus)

```
[Model: Opus]
Implement the next engine feature from Section 1 of AUDIO_WORK_TRACKER.md:
1. Review the feature specification
2. Design the API/schema changes
3. Implement the feature in the audio generator
4. Create a test preset demonstrating the feature
5. Validate determinism and quality
6. Mark complete in tracker
```

### Implement Wavetable Oscillator (Opus)

```
[Model: Opus]
Implement the wavetable oscillator synthesis type:
1. Define wavetable format (single-cycle waves, interpolation method)
2. Add "wavetable" synthesis type to audio_v1 schema
3. Implement wavetable playback with:
   - Table selection/morphing
   - Interpolation (linear/cubic)
   - Detune and unison voices
4. Create test presets: pad_shimmer, poly_pad
5. Update docs/spec-reference/audio.md
```

### Implement LFO Modulators (Opus)

```
[Model: Opus]
Implement LFO modulators for the audio engine:
1. Define modulator schema (rate, depth, shape, target)
2. Support targets: pitch, amplitude, filter cutoff
3. Support shapes: sine, triangle, square, sample-and-hold
4. Implement modulator routing in synthesis pipeline
5. Create test preset: bass_wobble
6. Update docs/spec-reference/audio.md
```

### Implement Effect Chain (Opus)

```
[Model: Opus]
Implement the effect chain for audio_v1:
1. Define effect chain schema in recipe params
2. Implement effects: delay, reverb, chorus, phaser, bitcrush, waveshaper, compressor
3. Support effect ordering and wet/dry mix
4. Create test presets demonstrating each effect
5. Update docs/spec-reference/audio.md
```

---

### Create Next Batch of Presets (Sonnet)

```
[Model: Sonnet]
Create the next 5 Tier 1 presets from AUDIO_WORK_TRACKER.md that are not yet implemented.
For each:
1. Write the JSON spec
2. Run validation
3. Run generation
4. Check report
5. Add to "Ready to Validate" queue
```

### Focus on Specific Category (Sonnet)

```
[Model: Sonnet]
Implement all remaining kick variants (kick_soft, kick_dist, kick_4, big_kick, toy_kick).
```

```
[Model: Sonnet]
Implement all FM/DX7 presets (fm_bass, fm_keys, fm_bell, fm_lead, dx7_ep, dx7_bass, dx7_bell, dx7_marimba, dx7_lead).
```

### Validate All Presets (Haiku)

```
[Model: Haiku]
Run validation on all presets in packs/preset_library_v1/audio/ and report any failures.
```

### Update Tracker After Session (Haiku)

```
[Model: Haiku]
Update AUDIO_WORK_TRACKER.md:
1. Move completed presets from pending to "Ready to Validate"
2. Update progress counts in dashboard
3. Note any issues encountered
```

### Process Human Feedback (Sonnet)

```
[Model: Sonnet]
Review the "Needs Improvement" section of AUDIO_WORK_TRACKER.md.
For each entry:
1. Read the issue description
2. Revise the spec to address the feedback
3. Regenerate and validate
4. Move back to "Ready to Validate" with revision notes
```

### Start a Genre Kit (Sonnet, or Opus for complex kits)

```
[Model: Sonnet]
Start implementing the Synthwave / Outrun genre kit:
1. List all required presets from music-genre-kits-master-list.md
2. Check which presets already exist in the library
3. Create missing presets (in batches)
4. Once all presets exist, create the compose template
5. Create the audition spec
```

```
[Model: Opus - use for complex regional/orchestral kits]
Start implementing the South Asian genre kit:
1. Research the specific instruments (tabla, sitar, bansuri, tanpura)
2. Design synthesis approaches for each regional instrument
3. Create presets with authentic character
4. Build compose template with appropriate timebase and harmony
```

### Parallel Batch Dispatch

When running multiple preset batches in parallel, use this pattern:

```
Launch 3 parallel agents (Sonnet) to create presets:
- Agent 1: OSC batch (kick_soft, kick_dist, kick_4, big_kick)
- Agent 2: FM batch (fm_bass, fm_keys, fm_bell, fm_lead)
- Agent 3: NOISE batch (hat_tick, hat_metallic, noise_hat, noise_snare)

Each agent writes specs, validates, generates, and queues for review.
```

---

## Reference: Synthesis Types

### Oscillator

```json
{
  "type": "oscillator",
  "waveform": "sine|sawtooth|square|triangle",
  "frequency": 440.0,
  "freq_sweep": { "end_freq": 220.0, "curve": "exponential|linear" },
  "detune": 7.0,
  "duty": 0.5
}
```

### FM Synth

```json
{
  "type": "fm_synth",
  "carrier_freq": 440.0,
  "modulator_freq": 880.0,
  "modulation_index": 4.0,
  "freq_sweep": { "end_freq": 110.0, "curve": "linear" }
}
```

### Karplus-Strong

```json
{
  "type": "karplus_strong",
  "frequency": 110.0,
  "decay": 0.996,
  "blend": 0.7
}
```

### Noise Burst

```json
{
  "type": "noise_burst",
  "noise_type": "white|pink",
  "filter": { "type": "lowpass|highpass|bandpass", "cutoff": 2000.0, "resonance": 0.7 }
}
```

### Additive

```json
{
  "type": "additive",
  "base_freq": 220.0,
  "harmonics": [1.0, 0.5, 0.33, 0.25]
}
```

### Multi-Oscillator

```json
{
  "type": "multi_oscillator",
  "frequency": 220.0,
  "oscillators": [
    { "waveform": "sawtooth", "volume": 1.0, "detune": 0.0 },
    { "waveform": "square", "volume": 0.8, "detune": 7.0, "duty": 0.4 }
  ]
}
```

### Metallic

```json
{
  "type": "metallic",
  "base_freq": 220.0,
  "num_partials": 8,
  "inharmonicity": 1.6
}
```

### Pitched Body

```json
{
  "type": "pitched_body",
  "start_freq": 600.0,
  "end_freq": 120.0
}
```

---

## Reference: Filter Types

```json
{ "type": "lowpass", "cutoff": 2000.0, "resonance": 0.7, "cutoff_end": 500.0 }
{ "type": "highpass", "cutoff": 200.0, "resonance": 0.7, "cutoff_end": 2000.0 }
{ "type": "bandpass", "center": 800.0, "resonance": 0.7, "center_end": 1200.0 }
```

---

## Session Checklist

Before ending a development session:

```
[ ] Engine features: Check if any were completed, update Section 1 of tracker
[ ] Presets: Verified required engine features exist before implementation
[ ] All new presets validated without errors
[ ] All new presets added to "Ready to Validate" queue
[ ] AUDIO_WORK_TRACKER.md dashboard updated (both engine and preset counts)
[ ] Any blockers noted in tracker
[ ] Git status shows expected changes
```
