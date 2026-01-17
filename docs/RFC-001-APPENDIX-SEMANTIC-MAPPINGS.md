# RFC-001 Appendix: Semantic Parameter Mappings

**Purpose**: Concrete mappings from human/LLM-friendly descriptors to synthesis parameters.

---

## Overview

This appendix provides the curated mappings required for S1-B (Semantic Macro Functions). Each table maps a semantic descriptor to concrete parameter values, enabling abstractions like:

```python
sound = pluck_sound(character="bright", attack="snappy", body="wooden")
```

---

## 1. Character Descriptors

Maps timbral character to filter and harmonic configuration.

### Filter Mapping

| Character | Filter Type | Cutoff (Hz) | Resonance | Drive |
|-----------|-------------|-------------|-----------|-------|
| bright | lowpass | 8000-12000 | 0.1-0.3 | 0.0 |
| warm | lowpass | 400-1200 | 0.3-0.5 | 0.1 |
| dark | lowpass | 200-600 | 0.2-0.4 | 0.0 |
| hollow | bandpass | 800-2000 | 0.6-0.8 | 0.0 |
| nasal | bandpass | 1000-3000 | 0.7-0.9 | 0.0 |
| thin | highpass | 1500-3000 | 0.1-0.2 | 0.0 |
| aggressive | lowpass | 2000-4000 | 0.7-0.9 | 0.3-0.6 |
| smooth | lowpass | 3000-6000 | 0.0-0.1 | 0.0 |
| metallic | bandpass | 2000-5000 | 0.8-0.95 | 0.2 |
| airy | highpass | 4000-8000 | 0.1-0.2 | 0.0 |

### Harmonic Content Mapping

| Character | Synthesis Preference | Harmonics | Detune |
|-----------|---------------------|-----------|--------|
| pure | sine | 1 only | 0.0 |
| rich | sawtooth/additive | 8-16 | 0.0-0.05 |
| hollow | square/pulse | odd only | 0.0 |
| buzzy | sawtooth | 16+ | 0.02-0.08 |
| glassy | fm_synth | 4-8 (inharmonic) | 0.0 |
| noisy | noise blend | N/A | N/A |

---

## 2. Attack Descriptors

Maps attack character to envelope parameters.

### Envelope Mapping

| Attack Style | Attack (s) | Decay (s) | Sustain | Release (s) |
|--------------|------------|-----------|---------|-------------|
| instant | 0.0005-0.002 | 0.02-0.05 | varies | 0.05-0.1 |
| snappy | 0.001-0.005 | 0.05-0.1 | varies | 0.1-0.2 |
| punchy | 0.001-0.003 | 0.1-0.2 | 0.0-0.3 | 0.1-0.15 |
| soft | 0.05-0.15 | 0.1-0.3 | varies | 0.2-0.4 |
| swelling | 0.3-1.0 | 0.1-0.2 | 0.8-1.0 | 0.3-0.5 |
| plucky | 0.001-0.003 | 0.1-0.2 | 0.0 | 0.15-0.25 |
| percussive | 0.0005-0.002 | 0.05-0.15 | 0.0 | 0.02-0.08 |
| bowed | 0.1-0.3 | 0.05-0.1 | 0.7-0.9 | 0.2-0.4 |

### Transient Enhancement

| Attack Style | Click Layer | Noise Burst (ms) | Filter Env Amount |
|--------------|-------------|------------------|-------------------|
| clicky | yes | 2-5 | high (0.6-0.9) |
| smooth | no | 0 | low (0.0-0.2) |
| woody | yes | 5-15 | medium (0.3-0.5) |

---

## 3. Body/Resonance Descriptors

Maps body character to resonance and spatial parameters.

### Resonance Mapping

| Body | Resonance Type | Decay Character | Modal Count |
|------|----------------|-----------------|-------------|
| wooden | modal | medium (0.3-0.6s) | 4-8 |
| metallic | modal | long (1-3s) | 8-16 |
| string | karplus_strong | medium (0.2-0.5s) | N/A |
| membrane | membrane | short-medium | 3-6 |
| glass | modal (inharmonic) | long (1-2s) | 6-12 |
| synthetic | none | N/A | N/A |

### Spatial Mapping

| Body | Reverb Size | Reverb Decay | Early Reflections |
|------|-------------|--------------|-------------------|
| intimate | small (0.1-0.3) | short (0.3-0.8s) | close |
| room | medium (0.4-0.6) | medium (0.8-1.5s) | natural |
| hall | large (0.7-0.9) | long (1.5-3s) | distant |
| chamber | medium (0.5-0.7) | medium (1-2s) | focused |
| outdoor | none/minimal | N/A | none |
| synthetic | plate/spring | varies | artificial |

---

## 4. Dynamics Descriptors

Maps dynamic character to compression and amplitude parameters.

### Compression Mapping

| Dynamics | Threshold (dB) | Ratio | Attack (ms) | Release (ms) |
|----------|----------------|-------|-------------|--------------|
| punchy | -12 to -8 | 4:1-6:1 | 5-15 | 50-100 |
| squashed | -18 to -12 | 8:1-12:1 | 1-5 | 100-200 |
| breathing | -15 to -10 | 3:1-4:1 | 20-50 | 200-500 |
| natural | -20 to -15 | 2:1-3:1 | 10-30 | 100-300 |
| limited | -6 to -3 | 10:1+ | 0.1-1 | 50-100 |

---

## 5. Movement Descriptors

Maps movement/modulation character to LFO and automation.

### LFO Mapping

| Movement | Rate (Hz) | Depth | Shape | Target |
|----------|-----------|-------|-------|--------|
| subtle | 0.1-0.5 | 0.05-0.15 | sine | filter/pitch |
| vibrato | 4-7 | 0.1-0.3 | sine | pitch |
| tremolo | 4-8 | 0.3-0.6 | sine/triangle | amplitude |
| wobble | 0.5-4 | 0.4-0.8 | sine | filter |
| pulsing | 2-8 | 0.8-1.0 | square | amplitude |
| evolving | 0.05-0.2 | 0.2-0.5 | random/sine | multiple |
| static | 0 | 0 | N/A | none |

---

## 6. Size Descriptors

Maps perceived size to frequency content and spatial processing.

### Frequency Mapping

| Size | Low Cut (Hz) | High Boost | Sub Content |
|------|--------------|------------|-------------|
| tiny | 500-1000 | +3-6dB @ 4kHz+ | none |
| small | 200-400 | +2-4dB @ 2kHz+ | minimal |
| medium | 80-150 | flat | moderate |
| large | 40-80 | -2dB @ 8kHz+ | strong |
| massive | 20-40 | -4dB @ 6kHz+ | dominant |

### Stereo Width Mapping

| Size | Stereo Width | Chorus/Unison | Detuning |
|------|--------------|---------------|----------|
| mono | 0.0 | none | 0 |
| narrow | 0.2-0.4 | subtle | 0.01-0.02 |
| normal | 0.5-0.7 | moderate | 0.02-0.05 |
| wide | 0.8-1.0 | heavy | 0.05-0.1 |
| huge | 1.0 + haas | extreme | 0.1-0.2 |

---

## 7. Era/Style Descriptors

Maps stylistic era to processing chains.

### Processing Chain Mapping

| Style | Saturation | Filter Character | Effects Chain |
|-------|------------|------------------|---------------|
| vintage | tape (0.2-0.4) | analog-style (resonant) | chorus → tape delay → spring reverb |
| retro_digital | bitcrush (12-bit) | harsh cutoff | chorus → digital delay |
| modern | subtle (0.05-0.1) | clean | stereo widener → plate reverb |
| lo-fi | heavy (0.4-0.7) | low sample rate | bitcrush → vinyl noise → room |
| cinematic | none | smooth | convolution reverb (large) |
| chiptune | bitcrush (4-8 bit) | none | none/minimal |

---

## 8. Instrument Category Templates

High-level presets combining multiple descriptors.

### Percussion Templates

| Template | Attack | Body | Character | Size |
|----------|--------|------|-----------|------|
| kick_punchy | percussive | synthetic | warm | large |
| kick_808 | snappy | synthetic | dark | massive |
| snare_acoustic | snappy | membrane | bright | medium |
| snare_electronic | instant | synthetic | aggressive | medium |
| hihat_closed | instant | metallic | thin | tiny |
| hihat_open | instant | metallic | bright | small |

### Melodic Templates

| Template | Attack | Body | Character | Movement |
|----------|--------|------|-----------|----------|
| lead_analog | snappy | synthetic | rich | vibrato |
| lead_digital | instant | synthetic | bright | static |
| pad_warm | swelling | synthetic | warm | evolving |
| pad_ambient | swelling | synthetic | airy | subtle |
| bass_sub | snappy | synthetic | dark | static |
| bass_growl | punchy | synthetic | aggressive | wobble |
| pluck_soft | plucky | string | warm | static |
| pluck_bright | plucky | string | bright | subtle |

### Acoustic Emulation Templates

| Template | Attack | Body | Character | Resonance |
|----------|--------|------|-----------|-----------|
| piano_bright | snappy | wooden | bright | wooden |
| piano_mellow | soft | wooden | warm | wooden |
| guitar_acoustic | plucky | wooden | warm | string |
| guitar_electric | plucky | synthetic | rich | string |
| strings_ensemble | bowed | synthetic | warm | none |
| brass_stab | snappy | synthetic | aggressive | none |
| organ_drawbar | instant | synthetic | hollow | none |

---

## 9. Implementation Notes

### Mapping Resolution

When multiple descriptors conflict, use this priority:
1. Attack (defines transient)
2. Character (defines timbre)
3. Body (defines resonance)
4. Size (defines frequency range)
5. Movement (defines modulation)

### Randomization Ranges

For natural variation, apply ±10-20% randomization to:
- Filter cutoff
- LFO rate
- Reverb decay
- Detuning amount

Do NOT randomize:
- Attack time (changes character)
- Fundamental frequency
- Effect order

### Combining Descriptors

```python
# Valid combination
sound = synth_sound(
    attack = "snappy",
    character = "warm",
    body = "synthetic",
    size = "large"
)

# Invalid/conflicting - system should warn
sound = synth_sound(
    character = "bright",  # conflicts with
    character = "dark"     # (duplicate key)
)

# Unusual but valid - apply both
sound = synth_sound(
    body = "wooden",      # modal resonance
    character = "metallic" # but metallic filter
)
# Result: Modal synth with metallic-sounding filter
```

---

## 10. LLM Usage Examples

### Prompt → Spec Translation

**User prompt**: "A punchy bass with some wobble"

**Semantic call**:
```python
sound = bass_sound(
    attack = "punchy",
    character = "rich",
    movement = "wobble"
)
```

**Expands to** (approximately):
```python
audio_layer(
    synthesis = oscillator("sawtooth", 55, 0.9),
    envelope = envelope(0.002, 0.15, 0.2, 0.12),
    filter = filter("lowpass", 600, 0.45),
    lfo = lfo("sine", 2.0, 0.55, "filter_cutoff"),
    effects = [
        effect("distortion", drive=0.15),
        effect("compressor", threshold=-10, ratio=4.0)
    ]
)
```

---

*End of Semantic Mappings Appendix*
