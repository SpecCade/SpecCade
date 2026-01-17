# RFC-0008 Appendix: Audio Analysis Backend Specification

**Purpose**: Technical specification for S2-A (Audio Analysis Backend) enabling perceptual feedback loops.

---

## Overview

The audio analysis backend extracts structured metrics from generated audio, enabling:
1. LLM feedback loops for iterative refinement
2. Quality validation and regression detection
3. Semantic search based on perceptual similarity

---

## Command Interface

### Basic Usage

```bash
# Analyze a generated audio file
speccade analyze --input output.wav --output metrics.json

# Analyze directly from spec (generate + analyze)
speccade analyze --spec kick.star --output metrics.json

# Compare two outputs
speccade compare --a kick_v1.wav --b kick_v2.wav --output comparison.json

# Compare against reference
speccade compare --input generated.wav --reference target.wav
```

### Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--input` | Input audio file (WAV/FLAC) | required |
| `--spec` | Input spec (alternative to --input) | - |
| `--output` | Output JSON file | stdout |
| `--format` | Output format (json/yaml/summary) | json |
| `--detailed` | Include frame-level data | false |
| `--normalize` | Normalize metrics to 0-1 range | true |

---

## Output Schema

### Full Analysis Output

```json
{
  "version": "1.0",
  "input": "kick.wav",
  "sample_rate": 44100,
  "duration_ms": 450,
  "channels": 1,

  "temporal": {
    "attack_ms": 8.2,
    "decay_ms": 142.5,
    "release_ms": 45.3,
    "sustain_ratio": 0.0,
    "transient_count": 1,
    "zero_crossing_rate": 0.023
  },

  "spectral": {
    "centroid_hz": 124.5,
    "spread_hz": 89.2,
    "rolloff_85_hz": 312.0,
    "rolloff_95_hz": 1240.0,
    "flatness": 0.12,
    "flux_mean": 0.034,
    "dominant_frequency_hz": 55.0
  },

  "dynamics": {
    "peak_db": -3.2,
    "rms_db": -12.4,
    "lufs_integrated": -14.2,
    "lufs_short_term_max": -10.1,
    "crest_factor_db": 9.2,
    "dynamic_range_db": 18.4
  },

  "perceptual": {
    "brightness": 0.28,
    "warmth": 0.72,
    "punch": 0.85,
    "sharpness": 0.34,
    "roughness": 0.15,
    "depth": 0.65
  },

  "tonal": {
    "is_pitched": false,
    "fundamental_hz": null,
    "harmonicity": 0.18,
    "pitch_confidence": 0.23,
    "pitch_stability": null
  },

  "bands": {
    "sub_20_60": 0.45,
    "bass_60_250": 0.38,
    "low_mid_250_500": 0.12,
    "mid_500_2000": 0.04,
    "high_mid_2000_4000": 0.008,
    "high_4000_8000": 0.002,
    "air_8000_20000": 0.0
  },

  "quality": {
    "clipping_detected": false,
    "dc_offset": 0.001,
    "silence_ratio": 0.12,
    "noise_floor_db": -72.3
  }
}
```

### Comparison Output

```json
{
  "version": "1.0",
  "a": "kick_v1.wav",
  "b": "kick_v2.wav",

  "similarity": {
    "overall": 0.82,
    "spectral": 0.78,
    "temporal": 0.91,
    "dynamic": 0.85,
    "perceptual": 0.76
  },

  "differences": {
    "brightness_delta": 0.15,
    "punch_delta": -0.08,
    "centroid_delta_hz": 45.2,
    "attack_delta_ms": -2.1,
    "rms_delta_db": 1.2
  },

  "recommendation": {
    "preferred": "b",
    "confidence": 0.73,
    "reasons": [
      "Better transient definition (attack_ms: 6.1 vs 8.2)",
      "Tighter low end (sub_20_60 ratio more focused)"
    ]
  }
}
```

---

## Metric Definitions

### Temporal Metrics

| Metric | Definition | Calculation | Range |
|--------|------------|-------------|-------|
| `attack_ms` | Time from onset to peak | Envelope follower, 10% to 90% rise | 0.1 - 1000+ |
| `decay_ms` | Time from peak to sustain/silence | Envelope follower, 90% to 10% fall | 1 - 10000+ |
| `release_ms` | Time from sustain to silence | End of decay to -60dB | 1 - 10000+ |
| `sustain_ratio` | Proportion of sound at steady level | Plateau detection in envelope | 0.0 - 1.0 |
| `transient_count` | Number of attack transients | Peak detection with 50ms refractory | 0 - N |
| `zero_crossing_rate` | Rate of sign changes | Sum(sign(x[n]) != sign(x[n-1])) / N | 0.0 - 0.5 |

### Spectral Metrics

| Metric | Definition | Calculation | Range |
|--------|------------|-------------|-------|
| `centroid_hz` | "Center of mass" of spectrum | Σ(f * |X(f)|) / Σ|X(f)| | 20 - 20000 |
| `spread_hz` | Spectral dispersion | Std dev around centroid | 0 - 10000 |
| `rolloff_85_hz` | Frequency below which 85% energy | Cumulative sum threshold | 20 - 20000 |
| `rolloff_95_hz` | Frequency below which 95% energy | Cumulative sum threshold | 20 - 20000 |
| `flatness` | Noise vs tone ratio | Geometric mean / Arithmetic mean | 0.0 - 1.0 |
| `flux_mean` | Average spectral change rate | Mean(||X(t) - X(t-1)||) | 0.0 - 1.0 |
| `dominant_frequency_hz` | Loudest frequency component | argmax(|X(f)|) | 20 - 20000 |

### Dynamic Metrics

| Metric | Definition | Calculation | Range |
|--------|------------|-------------|-------|
| `peak_db` | Maximum sample value | 20 * log10(max(abs(x))) | -inf - 0 |
| `rms_db` | Root mean square level | 20 * log10(sqrt(mean(x²))) | -inf - 0 |
| `lufs_integrated` | EBU R128 loudness | ITU-R BS.1770-4 | -70 - 0 |
| `crest_factor_db` | Peak to RMS ratio | peak_db - rms_db | 0 - 30+ |
| `dynamic_range_db` | LRA or peak-floor difference | EBU R128 LRA or custom | 0 - 40+ |

### Perceptual Metrics

| Metric | Definition | Estimation Method | Range |
|--------|------------|-------------------|-------|
| `brightness` | Perceived high frequency content | Weighted centroid / 10000 | 0.0 - 1.0 |
| `warmth` | Perceived low-mid richness | Energy 100-400Hz / total, inverted brightness | 0.0 - 1.0 |
| `punch` | Transient impact | Attack slope × low-mid energy | 0.0 - 1.0 |
| `sharpness` | High frequency harshness | Zwicker sharpness model | 0.0 - 1.0 |
| `roughness` | Amplitude modulation sensation | Modulation depth 15-300Hz | 0.0 - 1.0 |
| `depth` | Low frequency presence | Sub-bass ratio × duration | 0.0 - 1.0 |

### Tonal Metrics

| Metric | Definition | Calculation | Range |
|--------|------------|-------------|-------|
| `is_pitched` | Whether fundamental is detectable | pitch_confidence > 0.7 | bool |
| `fundamental_hz` | Detected pitch | YIN / pYIN algorithm | 20 - 20000 / null |
| `harmonicity` | Harmonic vs inharmonic ratio | Autocorrelation clarity | 0.0 - 1.0 |
| `pitch_confidence` | Pitch detection confidence | Algorithm-specific | 0.0 - 1.0 |
| `pitch_stability` | Pitch variation over time | Std dev of f0 track | 0.0 - 1.0 |

---

## Perceptual Estimation Algorithms

### Brightness

```
brightness = clamp(spectral_centroid / 8000, 0, 1) * 0.7
           + clamp(energy_above_4khz / total_energy, 0, 1) * 0.3
```

### Warmth

```
warmth = clamp(energy_100_400hz / total_energy * 3, 0, 1) * 0.6
       + (1 - brightness) * 0.4
```

### Punch

```
punch = clamp(attack_slope * 2, 0, 1) * 0.5
      + clamp(energy_60_250hz / total_energy * 2, 0, 1) * 0.3
      + clamp(transient_energy / rms_energy, 0, 1) * 0.2
```

Where `attack_slope = (peak_amplitude - 0) / attack_ms`

### Calibration

These formulas are calibrated against human perceptual ratings:
- Training set: 500 sounds rated by 10 listeners on 1-10 scales
- Correlation targets: r > 0.8 for each perceptual dimension
- Periodic recalibration recommended as preset library grows

---

## Implementation Architecture

### Pipeline

```
Input (WAV) → Decode → Normalize → Window → FFT → Metrics → JSON
                 │                            │
                 └── Envelope Follower ───────┘
```

### Dependencies

| Component | Library | Purpose |
|-----------|---------|---------|
| Audio I/O | `hound` | WAV reading |
| FFT | `rustfft` | Spectral analysis |
| Resampling | `rubato` | Normalize sample rate |
| Loudness | Custom | EBU R128 implementation |
| Pitch | Custom / `pitch-detection` | F0 estimation |

### Performance Targets

| Audio Duration | Analysis Time | Memory |
|----------------|---------------|--------|
| 1 second | < 50ms | < 10MB |
| 10 seconds | < 200ms | < 50MB |
| 60 seconds | < 1s | < 100MB |

### Frame Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Window size | 2048 samples (46ms @ 44.1kHz) | Balance time/frequency resolution |
| Hop size | 512 samples (11.6ms) | 75% overlap for smooth features |
| Window type | Hann | Good spectral leakage suppression |
| FFT size | 2048 | Match window |

---

## LLM Integration Examples

### Agentic Refinement Loop

```python
# Pseudo-code for LLM agent workflow

def refine_sound(target_description, max_iterations=5):
    spec = generate_initial_spec(target_description)

    for i in range(max_iterations):
        # Generate and analyze
        audio = speccade_generate(spec)
        metrics = speccade_analyze(audio)

        # Check if target met
        if meets_target(metrics, target_description):
            return spec

        # Generate refinement based on metrics
        feedback = format_feedback(metrics, target_description)
        # e.g., "Current: brightness=0.28, target needs brightness>0.6"

        spec = llm_refine(spec, feedback)

    return spec
```

### Feedback Message Format

For LLM consumption, format analysis as natural language:

```
Current sound analysis:
- Brightness: 0.28 (LOW) - needs to be higher for "bright" target
- Punch: 0.85 (HIGH) - good, matches "punchy" target
- Attack: 8.2ms (MEDIUM) - could be faster for "snappy" feel
- Spectral centroid: 124 Hz - very bass-heavy

Suggested adjustments:
- Increase filter cutoff from 600Hz to 2000-4000Hz
- Reduce attack time from 8ms to 2-4ms
- Add high frequency content (noise layer or harmonic boost)
```

### Comparison for A/B Selection

```
Comparing kick_v1 vs kick_v2 against target "punchy 808":

kick_v1:
  - punch: 0.72, warmth: 0.65, sub: 0.38
  - Overall similarity to target: 0.71

kick_v2:
  - punch: 0.81, warmth: 0.71, sub: 0.45
  - Overall similarity to target: 0.84

Recommendation: kick_v2
Reason: Better punch (0.81 vs 0.72) and stronger sub-bass presence
```

---

## Validation and Testing

### Golden Audio Tests

Maintain test fixtures with known metrics:
```
tests/audio/
├── sine_440hz.wav      → centroid ≈ 440, flatness ≈ 0, harmonicity ≈ 1.0
├── white_noise.wav     → flatness ≈ 1, harmonicity ≈ 0
├── kick_reference.wav  → punch > 0.7, attack < 20ms
└── pad_reference.wav   → attack > 200ms, brightness < 0.4
```

### Metric Stability

Ensure deterministic output:
```bash
# Same input must produce identical metrics
speccade analyze --input test.wav --output a.json
speccade analyze --input test.wav --output b.json
diff a.json b.json  # Must be empty
```

### Cross-Platform Consistency

Metrics should match within tolerance across platforms:
- Temporal metrics: ±1ms
- Spectral metrics: ±2%
- Perceptual metrics: ±0.02

---

## Future Extensions

### Phase 2: Embedding Export

```bash
speccade analyze --input kick.wav --embed --model clap-base
```

Output includes 512-dim embedding vector for similarity search.

### Phase 3: Batch Analysis

```bash
speccade analyze --input-dir ./sounds/ --output-csv metrics.csv
```

Analyze entire directory, output as CSV for visualization/clustering.

### Phase 4: Real-Time Analysis

```bash
speccade analyze --stream --port 8080
```

WebSocket server for real-time analysis during iterative generation.

---

*End of Audio Analysis Specification*
