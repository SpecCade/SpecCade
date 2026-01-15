# Audio Effects Chain

Effects are applied post-generation to the mixed audio. Add effects array to audio_v1 params. Effects process in order.

## Reverb

Room simulation with decay and diffusion.

```json
{
  "type": "reverb",
  "room_size": 0.7,         // 0.0-1.0 (small to large)
  "damping": 0.5,           // High-frequency absorption (0.0-1.0)
  "wet": 0.3,               // Effect mix (0.0-1.0)
  "dry": 0.7,               // Dry signal (0.0-1.0)
  "width": 1.0,             // Stereo width (0.0-1.0)
  "predelay_ms": 20.0       // Initial delay before reverb
}
```

**Use cases:**
- Room ambience: `room_size: 0.3-0.5`
- Hall reverb: `room_size: 0.7-0.8`
- Huge spaces: `room_size: 0.9+, damping: 0.3`

## Delay

Echo with feedback.

```json
{
  "type": "delay",
  "time_ms": 250.0,         // Delay time in milliseconds
  "feedback": 0.4,          // Repeat amount (0.0-1.0, <1.0 for stability)
  "wet": 0.3,
  "dry": 0.7,
  "ping_pong": true,        // Alternate L/R channels
  "lowpass_cutoff": 4000.0  // Optional - darken repeats
}
```

**Common settings:**
- Slapback: `time_ms: 80-120, feedback: 0.1`
- Rhythmic: `time_ms: 375 (8th at 80bpm), feedback: 0.5`
- Ambient: `time_ms: 500+, feedback: 0.6, ping_pong: true`

## Chorus

Detuned copies for thickness.

```json
{
  "type": "chorus",
  "rate": 1.5,              // LFO rate in Hz
  "depth": 0.5,             // Modulation depth (0.0-1.0)
  "wet": 0.5,
  "dry": 0.5,
  "voices": 2,              // 1-4 chorus voices
  "stereo_spread": 0.8      // Voice panning spread
}
```

**Use cases:**
- Subtle thickening: `depth: 0.2, voices: 2`
- Classic chorus: `rate: 0.8, depth: 0.5, voices: 3`
- Ensemble: `rate: 1.2, depth: 0.7, voices: 4`

## Phaser

Swept notch filters for sweeping effect.

```json
{
  "type": "phaser",
  "rate": 0.5,              // LFO rate in Hz
  "depth": 0.7,             // Sweep depth (0.0-1.0)
  "stages": 6,              // Number of allpass stages (2-12, even)
  "feedback": 0.3,          // Resonance (0.0-1.0)
  "wet": 0.5,
  "dry": 0.5,
  "center_frequency": 1000.0  // Center of sweep
}
```

**Use cases:**
- Subtle sweep: `stages: 4, depth: 0.4`
- Classic phaser: `stages: 6, depth: 0.7, feedback: 0.5`
- Intense jet: `stages: 12, depth: 0.9, feedback: 0.7`

## Bitcrusher

Lo-fi bit reduction and sample rate reduction.

```json
{
  "type": "bitcrush",
  "bit_depth": 8,           // Target bits (1-16)
  "sample_rate": 22050,     // Target sample rate
  "wet": 1.0,
  "dry": 0.0
}
```

**Use cases:**
- Subtle grit: `bit_depth: 12, sample_rate: 44100`
- Retro: `bit_depth: 8, sample_rate: 22050`
- Extreme: `bit_depth: 4, sample_rate: 8000`

## Waveshaper

Distortion with various curves.

```json
{
  "type": "waveshaper",
  "drive": 2.0,             // Input gain (1.0 = unity)
  "curve": "tanh",          // tanh, soft_clip, hard_clip, sine
  "wet": 0.7,
  "dry": 0.3,
  "output_gain": 0.8        // Post-shaping level
}
```

**Curves:**
- `tanh`: Smooth saturation
- `soft_clip`: Gentle clipping
- `hard_clip`: Aggressive clipping
- `sine`: Wavefolder-like

**Use cases:**
- Warmth: `drive: 1.5, curve: tanh, wet: 0.3`
- Overdrive: `drive: 3.0, curve: soft_clip, wet: 0.6`
- Distortion: `drive: 5.0, curve: hard_clip, wet: 0.8`

## Compressor

Dynamic range control.

```json
{
  "type": "compressor",
  "threshold_db": -12.0,    // Compression starts here
  "ratio": 4.0,             // Compression ratio (1:1 to inf:1)
  "attack_ms": 10.0,        // Attack time
  "release_ms": 100.0,      // Release time
  "makeup_gain_db": 3.0,    // Output gain compensation
  "knee_db": 3.0            // Soft knee width
}
```

**Use cases:**
- Gentle leveling: `ratio: 2.0, threshold: -6`
- Punch: `ratio: 4.0, attack: 30, threshold: -12`
- Limiting: `ratio: 20.0, threshold: -3`

## Effects Chain Examples

### Drum Bus
```json
"effects": [
  { "type": "compressor", "threshold_db": -10, "ratio": 4.0, "attack_ms": 5 },
  { "type": "waveshaper", "drive": 1.3, "curve": "tanh", "wet": 0.2 }
]
```

### Ambient Pad
```json
"effects": [
  { "type": "chorus", "rate": 0.8, "depth": 0.4, "voices": 3 },
  { "type": "reverb", "room_size": 0.85, "wet": 0.5, "damping": 0.4 },
  { "type": "delay", "time_ms": 500, "feedback": 0.3, "wet": 0.2, "ping_pong": true }
]
```

### Lo-Fi
```json
"effects": [
  { "type": "bitcrush", "bit_depth": 10, "sample_rate": 32000 },
  { "type": "waveshaper", "drive": 1.5, "curve": "tanh", "wet": 0.3 },
  { "type": "reverb", "room_size": 0.4, "wet": 0.2 }
]
```

### Lead Synth
```json
"effects": [
  { "type": "phaser", "rate": 0.3, "depth": 0.5, "stages": 6 },
  { "type": "delay", "time_ms": 375, "feedback": 0.4, "ping_pong": true, "wet": 0.25 },
  { "type": "reverb", "room_size": 0.6, "wet": 0.3 }
]
```
