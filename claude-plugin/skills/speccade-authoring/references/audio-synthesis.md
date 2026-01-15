# Audio Synthesis Types

SpecCade supports 16+ synthesis methods for audio generation. Each layer in an audio spec has a `synthesis` object specifying the type and parameters.

## Basic Synthesis

### Oscillator
Simple waveform generator with optional frequency sweep.

```json
{
  "type": "oscillator",
  "waveform": "sine",       // sine, square, saw, triangle
  "frequency": 440.0,       // Hz
  "duty_cycle": 0.5,        // For pulse waves (0.0-1.0)
  "frequency_sweep": {      // Optional
    "end_frequency": 220.0,
    "duration": 0.5,
    "curve": "exponential"  // linear, exponential, logarithmic
  }
}
```

### Multi-Oscillator
Stack multiple detuned oscillators for thicker sound.

```json
{
  "type": "multi_oscillator",
  "waveform": "saw",
  "frequency": 220.0,
  "voices": 4,              // Number of oscillators
  "detune_cents": 10.0,     // Spread in cents
  "stereo_spread": 0.5      // Pan spread (0.0-1.0)
}
```

### Noise
White, pink, or brown noise generation.

```json
{
  "type": "noise",
  "noise_type": "white"     // white, pink, brown
}
```

## FM Synthesis

### FM Synth
Frequency modulation with carrier and modulator.

```json
{
  "type": "fm",
  "carrier_frequency": 440.0,
  "carrier_waveform": "sine",
  "modulator_frequency": 880.0,
  "modulator_waveform": "sine",
  "modulation_index": 2.0,
  "index_envelope": {        // Optional - modulate index over time
    "start": 5.0,
    "end": 1.0,
    "curve": "exponential"
  }
}
```

### AM Synth
Amplitude modulation.

```json
{
  "type": "am",
  "carrier_frequency": 440.0,
  "carrier_waveform": "sine",
  "modulator_frequency": 5.0,
  "modulator_waveform": "sine",
  "modulation_depth": 0.5    // 0.0-1.0
}
```

### Ring Modulation
Carrier × Modulator for metallic/robotic sounds.

```json
{
  "type": "ring_mod",
  "carrier_frequency": 440.0,
  "carrier_waveform": "sine",
  "modulator_frequency": 100.0,
  "modulator_waveform": "sine"
}
```

## Physical Modeling

### Karplus-Strong
Plucked string simulation.

```json
{
  "type": "karplus_strong",
  "frequency": 220.0,
  "decay": 0.996,           // Feedback amount (0.9-0.999)
  "blend": 0.5,             // Noise vs pitched (0.0-1.0)
  "stretch": 1.0            // String stiffness
}
```

### Modal Synthesis
Resonant modes for bells, chimes, marimbas.

```json
{
  "type": "modal",
  "fundamental": 440.0,
  "modes": [
    { "ratio": 1.0, "amplitude": 1.0, "decay": 0.5 },
    { "ratio": 2.0, "amplitude": 0.6, "decay": 0.4 },
    { "ratio": 3.5, "amplitude": 0.3, "decay": 0.3 }
  ],
  "excitation": "impulse"   // impulse, noise, bow
}
```

### Pitched Body
Impact sounds with body resonance.

```json
{
  "type": "pitched_body",
  "frequency": 200.0,
  "body_resonance": 0.8,
  "frequency_sweep": {
    "end_frequency": 80.0,
    "duration": 0.1,
    "curve": "exponential"
  }
}
```

## Spectral Synthesis

### Additive
Build sounds from individual harmonics.

```json
{
  "type": "additive",
  "fundamental": 220.0,
  "harmonics": [1.0, 0.5, 0.33, 0.25, 0.2]  // Amplitudes per harmonic
}
```

### Harmonics
Similar to additive with more control.

```json
{
  "type": "harmonics",
  "fundamental": 440.0,
  "partials": [
    { "harmonic": 1, "amplitude": 1.0, "phase": 0.0 },
    { "harmonic": 2, "amplitude": 0.5, "phase": 0.25 },
    { "harmonic": 3, "amplitude": 0.33, "phase": 0.0 }
  ]
}
```

### Metallic
Inharmonic partials for bells and metallic timbres.

```json
{
  "type": "metallic",
  "fundamental": 440.0,
  "inharmonicity": 0.02,    // Amount of inharmonic stretch
  "partials": 12,           // Number of partials
  "decay_factor": 0.8       // Per-partial decay
}
```

## Complex Synthesis

### Granular
Grain-based texture synthesis.

```json
{
  "type": "granular",
  "base_frequency": 440.0,
  "grain_size_ms": 50.0,    // Grain duration
  "grain_density": 20.0,    // Grains per second
  "pitch_spread": 0.1,      // Random pitch variation
  "pan_spread": 0.5,        // Stereo spread
  "waveform": "sine"
}
```

### Wavetable
Morphing between waveforms.

```json
{
  "type": "wavetable",
  "frequency": 440.0,
  "tables": ["sine", "saw", "square"],
  "position": 0.5,          // Initial table position
  "position_envelope": {    // Optional - morph over time
    "start": 0.0,
    "end": 1.0,
    "curve": "linear"
  },
  "unison_voices": 4,
  "unison_detune": 10.0
}
```

### Phase Distortion
Casio CZ-style synthesis.

```json
{
  "type": "pd",
  "frequency": 440.0,
  "distortion_type": "resonant",  // sawtooth, square, pulse, resonant
  "distortion_amount": 0.8,
  "distortion_decay": 0.5   // Decay over note duration
}
```

### Vector
2D crossfade between 4 sources.

```json
{
  "type": "vector",
  "frequency": 440.0,
  "sources": [
    { "waveform": "sine" },
    { "waveform": "saw" },
    { "waveform": "square" },
    { "waveform": "triangle" }
  ],
  "x": 0.5,                 // X position (0.0-1.0)
  "y": 0.5,                 // Y position (0.0-1.0)
  "path": [                 // Optional animation
    { "time": 0.0, "x": 0.0, "y": 0.0 },
    { "time": 0.5, "x": 1.0, "y": 0.5 },
    { "time": 1.0, "x": 0.5, "y": 1.0 }
  ]
}
```

## Voice Synthesis

### Vocoder
Filter bank with formant animation.

```json
{
  "type": "vocoder",
  "carrier": "sawtooth",    // sawtooth, pulse, noise
  "carrier_frequency": 110.0,
  "bands": 16,              // Number of filter bands
  "formants": [
    { "time": 0.0, "vowel": "a" },
    { "time": 0.5, "vowel": "e" },
    { "time": 1.0, "vowel": "o" }
  ],
  "bandwidth": 100.0
}
```

### Formant
Vowel synthesis using formant filters.

```json
{
  "type": "formant",
  "frequency": 220.0,
  "vowel": "a",             // a, e, i, o, u
  "voice_type": "tenor",    // bass, tenor, alto, soprano
  "vowel_morph": {          // Optional
    "start_vowel": "a",
    "end_vowel": "o",
    "duration": 0.5
  }
}
```

## Layer Modulation

### LFO
Apply LFO to layer parameters.

```json
{
  "lfo": {
    "rate": 5.0,            // Hz
    "depth": 0.2,           // Modulation amount
    "waveform": "sine",     // sine, triangle, square, saw
    "target": "pitch"       // pitch, volume, pan, filter_cutoff
  }
}
```

### Per-Layer Filter
Apply filter to individual layer.

```json
{
  "filter": {
    "type": "lowpass",      // lowpass, highpass, bandpass, notch
    "cutoff": 2000.0,       // Hz
    "resonance": 0.5,       // Q factor
    "envelope": {           // Optional filter envelope
      "amount": 4000.0,
      "attack": 0.01,
      "decay": 0.2
    }
  }
}
```
