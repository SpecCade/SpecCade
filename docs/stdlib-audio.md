# SpecCade Starlark Standard Library - Audio Functions

[← Back to Index](stdlib-reference.md)

## Audio Functions

Audio functions are organized into synthesis, filters, effects, modulation, and layers.

## Table of Contents
- [Synthesis](#synthesis)
- [Filters](#filters)
- [Effects](#effects)
- [Modulation](#modulation)
- [Layers](#layers)

---

## Synthesis

### envelope()

Creates an ADSR envelope.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| attack | f64 | No | 0.01 | Attack time in seconds |
| decay | f64 | No | 0.1 | Decay time in seconds |
| sustain | f64 | No | 0.5 | Sustain level (0.0-1.0) |
| release | f64 | No | 0.2 | Release time in seconds |

**Returns:** Dict matching the Envelope IR structure.

**Example:**
```python
envelope()  # Uses defaults
envelope(0.05, 0.2, 0.7, 0.3)
envelope(attack = 0.1, release = 0.5)
```

### oscillator()

Creates an oscillator synthesis block.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes | - | Frequency in Hz |
| waveform | str | No | "sine" | "sine", "square", "sawtooth", "triangle", "pulse" |
| sweep_to | f64 | No | None | Optional target frequency for sweep |
| curve | str | No | "linear" | "linear", "exponential", "logarithmic" |
| detune | f64 | No | None | Detune in cents (100 cents = 1 semitone) |
| duty | f64 | No | None | Duty cycle for pulse waves (0.0-1.0) |

**Returns:** Dict matching the Synthesis::Oscillator IR structure.

**Example:**
```python
oscillator(440)  # 440 Hz sine wave
oscillator(880, "sawtooth")  # 880 Hz sawtooth
oscillator(440, "sine", 220, "exponential")  # Sweep from 440 to 220 Hz
oscillator(440, "pulse", duty = 0.25)  # Pulse wave with 25% duty cycle
```

### fm_synth()

Creates an FM synthesis block.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| carrier | f64 | Yes | - | Carrier frequency in Hz |
| modulator | f64 | Yes | - | Modulator frequency in Hz |
| index | f64 | Yes | - | Modulation index |
| sweep_to | f64 | No | None | Optional target carrier frequency |

**Returns:** Dict matching the Synthesis::FmSynth IR structure.

**Example:**
```python
fm_synth(440, 880, 5.0)
fm_synth(440, 880, 5.0, 220)  # Sweep carrier to 220 Hz
```

### am_synth()

Creates an AM (Amplitude Modulation) synthesis block.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| carrier | f64 | Yes | - | Carrier frequency in Hz |
| modulator | f64 | Yes | - | Modulator frequency in Hz |
| depth | f64 | Yes | - | Modulation depth (0.0-1.0) |
| sweep_to | f64 | No | None | Optional target carrier frequency |

**Returns:** Dict matching the Synthesis::AmSynth IR structure.

**Example:**
```python
am_synth(440, 110, 0.5)
am_synth(440, 110, 0.5, 220)  # Sweep carrier to 220 Hz
```

### noise_burst()

Creates a noise burst synthesis block.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| noise_type | str | No | "white" | "white", "pink", or "brown" |
| filter | dict | No | None | Optional filter from lowpass()/highpass() |

**Returns:** Dict matching the Synthesis::NoiseBurst IR structure.

**Example:**
```python
noise_burst()  # White noise
noise_burst("pink")
noise_burst("white", lowpass(5000))
```

### karplus_strong()

Creates a Karplus-Strong plucked string synthesis block.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes | - | Base frequency in Hz |
| decay | f64 | No | 0.99 | Decay factor (0.0-1.0) |
| blend | f64 | No | 0.5 | Lowpass blend factor (0.0-1.0) |

**Returns:** Dict matching the Synthesis::KarplusStrong IR structure.

**Example:**
```python
karplus_strong(440)
karplus_strong(220, 0.996, 0.7)
```

### additive()

Creates an additive synthesis block with harmonics.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| base_freq | f64 | Yes | - | Base frequency in Hz |
| harmonics | list | Yes | - | List of harmonic amplitudes (index 0 = fundamental) |

**Returns:** Dict matching the Synthesis::Additive IR structure.

**Example:**
```python
additive(440, [1.0, 0.5, 0.25, 0.125])  # Fundamental + 3 harmonics
additive(220, [1.0, 0.0, 0.33, 0.0, 0.2])  # Odd harmonics only
```

### supersaw_unison()

Creates a Supersaw/Unison synthesis block.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes | - | Base frequency in Hz |
| voices | int | Yes | - | Number of unison voices (1-16) |
| detune_cents | f64 | Yes | - | Maximum detune in cents (100 = 1 semitone) |
| spread | f64 | Yes | - | Stereo spread (0.0-1.0) |
| detune_curve | str | No | "linear" | Detune distribution: "linear" or "exp2" |

**Returns:** Dict matching the Synthesis::SupersawUnison IR structure.

**Example:**
```python
supersaw_unison(440, 7, 20, 0.8)  # Classic supersaw
supersaw_unison(440, 5, 15, 1.0, "exp2")  # With exponential detune
```

### wavetable()

Creates a Wavetable synthesis block.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| table | str | Yes | - | Wavetable source: "basic", "analog", "digital", "pwm", "formant", "organ" |
| frequency | f64 | Yes | - | Base frequency in Hz |
| position | f64 | No | 0.0 | Position in wavetable (0.0-1.0) |
| position_end | f64 | No | None | Optional end position for sweep |
| voices | int | No | None | Number of unison voices (1-8) |
| detune | f64 | No | None | Detune amount in cents for unison |

**Returns:** Dict matching the Synthesis::Wavetable IR structure.

**Example:**
```python
wavetable("basic", 440)
wavetable("analog", 440, 0.5)  # Start at middle of wavetable
wavetable("digital", 440, 0.0, 1.0)  # Sweep through entire table
wavetable("basic", 440, 0.0, None, 4, 10)  # 4-voice unison
```

### granular()

Creates a Granular synthesis block.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| source | dict | Yes | - | Source material from granular_source() |
| grain_size_ms | f64 | Yes | - | Grain size in milliseconds (10-500) |
| grain_density | f64 | Yes | - | Grains per second (1-100) |
| pitch_spread | f64 | No | 0.0 | Random pitch variation in semitones |
| position_spread | f64 | No | 0.0 | Random position jitter (0.0-1.0) |
| pan_spread | f64 | No | 0.0 | Stereo spread (0.0-1.0) |

**Returns:** Dict matching the Synthesis::Granular IR structure.

**Example:**
```python
granular(granular_source("noise", "white"), 50, 20)
granular(granular_source("tone", "sine", 440), 100, 30, 2.0, 0.5, 0.8)
```

### granular_source()

Creates a granular source configuration.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| source_type | str | Yes | - | "noise", "tone", or "formant" |
| param1 | any | Yes | - | For noise: noise_type. For tone: waveform. For formant: frequency |
| param2 | any | No | None | For tone: frequency. For formant: formant_freq |

**Returns:** Dict matching the GranularSource IR structure.

**Example:**
```python
granular_source("noise", "white")
granular_source("tone", "sine", 440)
granular_source("formant", 220, 880)
```

### ring_mod_synth()

Creates a Ring Modulation synthesis block.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| carrier | f64 | Yes | - | Carrier frequency in Hz |
| modulator | f64 | Yes | - | Modulator frequency in Hz |
| mix | f64 | Yes | - | Wet/dry mix (0.0 = pure carrier, 1.0 = pure ring mod) |
| sweep_to | f64 | No | None | Optional target carrier frequency |

**Returns:** Dict matching the Synthesis::RingModSynth IR structure.

**Example:**
```python
ring_mod_synth(440, 880, 0.5)
ring_mod_synth(440, 880, 0.5, 220)
```

### multi_oscillator()

Creates a Multi-Oscillator synthesis block.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes | - | Base frequency in Hz |
| oscillators | list | Yes | - | List of oscillator config dicts from oscillator_config() |
| sweep_to | f64 | No | None | Optional target frequency for sweep |

**Returns:** Dict matching the Synthesis::MultiOscillator IR structure.

**Example:**
```python
multi_oscillator(440, [
    oscillator_config("sawtooth", 1.0),
    oscillator_config("square", 0.5, detune = 5.0)
])
```

### oscillator_config()

Creates an oscillator configuration for multi_oscillator().

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| waveform | str | Yes | - | Waveform type |
| volume | f64 | No | 1.0 | Volume (0.0-1.0) |
| detune | f64 | No | None | Detune in cents |
| phase | f64 | No | None | Phase offset (0.0-1.0) |
| duty | f64 | No | None | Duty cycle for pulse waves (0.0-1.0) |

**Returns:** Dict matching the OscillatorConfig IR structure.

**Example:**
```python
oscillator_config("sawtooth")
oscillator_config("square", 0.5, detune = 10.0)
oscillator_config("pulse", 0.8, duty = 0.25)
```

### membrane_drum()

Creates a Membrane Drum synthesis block.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes | - | Fundamental frequency in Hz |
| decay | f64 | Yes | - | Decay rate (0.0-1.0, higher = faster decay) |
| tone | f64 | Yes | - | Tone/brightness (0.0-1.0, low = fundamental, high = overtones) |
| strike | f64 | Yes | - | Strike strength (0.0-1.0) |

**Returns:** Dict matching the Synthesis::MembraneDrum IR structure.

**Example:**
```python
membrane_drum(80, 0.3, 0.5, 0.8)
```

### feedback_fm()

Creates a Feedback FM synthesis block.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes | - | Base frequency in Hz |
| feedback | f64 | Yes | - | Self-modulation amount (0.0-1.0) |
| modulation_index | f64 | Yes | - | Modulation depth/index |
| sweep_to | f64 | No | None | Optional target frequency for sweep |

**Returns:** Dict matching the Synthesis::FeedbackFm IR structure.

**Example:**
```python
feedback_fm(440, 0.5, 3.0)
feedback_fm(440, 0.7, 5.0, 220)
```

### pd_synth()

Creates a Phase Distortion synthesis block.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes | - | Base frequency in Hz |
| distortion | f64 | Yes | - | Distortion amount (0.0 = pure sine, higher = more harmonics) |
| distortion_decay | f64 | No | 0.0 | How fast distortion decays to pure sine |
| waveform | str | No | "resonant" | PD waveform: "resonant", "sawtooth", "pulse" |
| sweep_to | f64 | No | None | Optional target frequency for sweep |

**Returns:** Dict matching the Synthesis::PdSynth IR structure.

**Example:**
```python
pd_synth(440, 5.0)
pd_synth(440, 8.0, 2.0, "sawtooth")
```

### modal()

Creates a Modal synthesis block.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes | - | Base frequency in Hz |
| modes | list | Yes | - | List of mode dicts from modal_mode() |
| excitation | str | No | "impulse" | Excitation type: "impulse", "noise", "pluck" |
| sweep_to | f64 | No | None | Optional target frequency for sweep |

**Returns:** Dict matching the Synthesis::Modal IR structure.

**Example:**
```python
modal(440, [
    modal_mode(1.0, 1.0, 0.5),
    modal_mode(2.0, 0.5, 0.3),
    modal_mode(3.0, 0.25, 0.2)
])
```

### modal_mode()

Creates a modal mode configuration.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| freq_ratio | f64 | Yes | - | Frequency ratio relative to fundamental (1.0 = fundamental) |
| amplitude | f64 | Yes | - | Amplitude of this mode (0.0-1.0) |
| decay_time | f64 | Yes | - | Decay time in seconds |

**Returns:** Dict matching the ModalMode IR structure.

**Example:**
```python
modal_mode(1.0, 1.0, 0.5)  # Fundamental
modal_mode(2.7, 0.3, 0.2)  # Inharmonic partial
```

### metallic()

Creates a Metallic synthesis block.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| base_freq | f64 | Yes | - | Base frequency in Hz |
| num_partials | int | Yes | - | Number of inharmonic partials |
| inharmonicity | f64 | Yes | - | Inharmonicity factor (1.0 = harmonic, >1.0 = inharmonic) |

**Returns:** Dict matching the Synthesis::Metallic IR structure.

**Example:**
```python
metallic(200, 8, 1.4)  # Bell-like sound
```

### comb_filter_synth()

Creates a Comb Filter synthesis block.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes | - | Base frequency in Hz (determines delay line length) |
| decay | f64 | Yes | - | Feedback decay (0.0-1.0, higher = longer resonance) |
| excitation | str | No | "impulse" | Excitation type: "impulse", "noise", "saw" |

**Returns:** Dict matching the Synthesis::CombFilterSynth IR structure.

**Example:**
```python
comb_filter_synth(220, 0.9)
comb_filter_synth(440, 0.95, "noise")
```

### vocoder()

Creates a Vocoder synthesis block for speech-like tonal synthesis.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| carrier_freq | f64 | Yes (named) | - | Carrier frequency in Hz |
| carrier_type | str | Yes (named) | - | Carrier waveform: "sawtooth", "pulse", "noise" |
| num_bands | int | Yes (named) | - | Number of frequency bands (8-32) |
| band_spacing | str | Yes (named) | - | Band spacing: "linear" or "logarithmic" |
| envelope_attack | f64 | Yes (named) | - | Envelope attack time in seconds |
| envelope_release | f64 | Yes (named) | - | Envelope release time in seconds |
| formant_rate | f64 | Yes (named) | - | Formant modulation rate in Hz |
| bands | list | No (named) | None | Optional list of custom band configs from vocoder_band() |

**Returns:** Dict matching the Synthesis::Vocoder IR structure.

**Example:**
```python
vocoder(carrier_freq = 440, carrier_type = "sawtooth", num_bands = 16,
        band_spacing = "logarithmic", envelope_attack = 0.01, envelope_release = 0.05,
        formant_rate = 1.0)
```

### vocoder_band()

Creates a vocoder band configuration for custom vocoder band setup.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| center_freq | f64 | Yes | - | Center frequency of the band in Hz |
| bandwidth | f64 | Yes | - | Bandwidth (Q factor) of the band filter |
| envelope_pattern | list | No | None | Optional list of amplitude values over time (0.0-1.0) |

**Returns:** Dict matching the VocoderBand IR structure.

**Example:**
```python
vocoder_band(200, 100)
vocoder_band(400, 150, [0.0, 0.5, 1.0, 0.5, 0.0])
```

### formant_synth()

Creates a Formant/voice synthesis block for vowel-like sounds.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes (named) | - | Base frequency in Hz |
| formants | list | No (named) | None | Custom formant configs from formant_config() |
| vowel | str | No (named) | None | Vowel preset: "a", "i", "u", "e", "o" |
| vowel_morph | str | No (named) | None | Target vowel for morphing |
| morph_amount | f64 | No (named) | 0.0 | Morph blend amount (0.0-1.0) |
| breathiness | f64 | No (named) | 0.0 | Breathiness amount (0.0-1.0) |

**Returns:** Dict matching the Synthesis::FormantSynth IR structure.

**Example:**
```python
formant_synth(frequency = 220, vowel = "a")
formant_synth(frequency = 220, vowel = "a", vowel_morph = "i", morph_amount = 0.5)
formant_synth(frequency = 220, formants = [
    formant_config(800, 0.8, 100),
    formant_config(1200, 0.6, 120)
])
```

### formant_config()

Creates a formant configuration for custom formant synthesis.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes | - | Formant center frequency in Hz |
| amplitude | f64 | Yes | - | Formant amplitude (0.0-1.0) |
| bandwidth | f64 | Yes | - | Formant bandwidth in Hz |

**Returns:** Dict matching the FormantConfig IR structure.

**Example:**
```python
formant_config(800, 0.8, 100)
formant_config(1200, 0.6, 120)
```

### vector_synth()

Creates a Vector synthesis block with 4 sources arranged in a 2D space.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes (named) | - | Base frequency in Hz |
| sources | list | Yes (named) | - | List of exactly 4 vector sources from vector_source() |
| position_x | f64 | No (named) | 0.5 | X position in vector space (0.0-1.0) |
| position_y | f64 | No (named) | 0.5 | Y position in vector space (0.0-1.0) |
| path | list | No (named) | None | Optional animation path from vector_path_point() |
| path_loop | bool | No (named) | False | Loop the path animation |
| path_curve | str | No (named) | "linear" | Path interpolation: "linear", "exponential", "logarithmic" |

**Returns:** Dict matching the Synthesis::VectorSynth IR structure.

**Example:**
```python
vector_synth(
    frequency = 440,
    sources = [
        vector_source("sine"),
        vector_source("saw"),
        vector_source("square"),
        vector_source("triangle")
    ],
    position_x = 0.3,
    position_y = 0.7
)
```

### vector_source()

Creates a vector source configuration for vector synthesis.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| source_type | str | Yes | - | Source type: "sine", "saw", "square", "triangle", "noise", "wavetable" |
| frequency_ratio | f64 | No | 1.0 | Frequency ratio relative to base frequency |

**Returns:** Dict matching the VectorSource IR structure.

**Example:**
```python
vector_source("sine")
vector_source("saw", 2.0)  # One octave up
```

### vector_path_point()

Creates a vector path point for animated vector synthesis.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| x | f64 | Yes | - | X position (0.0-1.0) |
| y | f64 | Yes | - | Y position (0.0-1.0) |
| duration | f64 | Yes | - | Duration in seconds to reach this point |

**Returns:** Dict matching the VectorPathPoint IR structure.

**Example:**
```python
vector_path_point(0.0, 0.0, 0.5)  # Move to corner over 0.5s
vector_path_point(1.0, 1.0, 1.0)  # Move to opposite corner over 1s
```

### waveguide()

Creates a Waveguide synthesis block for wind instrument modeling.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes (named) | - | Base frequency in Hz |
| breath | f64 | Yes (named) | - | Breath pressure (0.0-1.0) |
| noise | f64 | Yes (named) | - | Noise/turbulence amount (0.0-1.0) |
| damping | f64 | Yes (named) | - | Damping factor (0.0-1.0) |
| resonance | f64 | Yes (named) | - | Resonance amount (0.0-1.0) |

**Returns:** Dict matching the Synthesis::Waveguide IR structure.

**Example:**
```python
waveguide(frequency = 440, breath = 0.7, noise = 0.1, damping = 0.3, resonance = 0.8)
```

### bowed_string()

Creates a Bowed String synthesis block for string instrument modeling.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes (named) | - | Base frequency in Hz |
| bow_pressure | f64 | Yes (named) | - | Bow pressure (0.0-1.0) |
| bow_position | f64 | Yes (named) | - | Bow position along string (0.0-1.0) |
| damping | f64 | Yes (named) | - | String damping (0.0-1.0) |

**Returns:** Dict matching the Synthesis::BowedString IR structure.

**Example:**
```python
bowed_string(frequency = 440, bow_pressure = 0.5, bow_position = 0.3, damping = 0.2)
```

### pulsar()

Creates a Pulsar synthesis block for rhythmic granular synthesis.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes (named) | - | Base frequency in Hz |
| pulse_rate | f64 | Yes (named) | - | Pulse rate in Hz |
| grain_size_ms | f64 | Yes (named) | - | Grain size in milliseconds |
| shape | str | Yes (named) | - | Grain shape: "sine", "square", "sawtooth", "triangle", "pulse" |

**Returns:** Dict matching the Synthesis::Pulsar IR structure.

**Example:**
```python
pulsar(frequency = 440, pulse_rate = 10, grain_size_ms = 50, shape = "sine")
```

### vosim()

Creates a VOSIM (Voice Simulation) synthesis block for voice-like sounds.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes (named) | - | Base frequency in Hz |
| formant_freq | f64 | Yes (named) | - | Formant frequency in Hz |
| pulses | int | Yes (named) | - | Number of pulses per period (1-16) |
| breathiness | f64 | No (named) | 0.0 | Breathiness amount (0.0-1.0) |

**Returns:** Dict matching the Synthesis::Vosim IR structure.

**Example:**
```python
vosim(frequency = 220, formant_freq = 880, pulses = 4)
vosim(frequency = 220, formant_freq = 880, pulses = 4, breathiness = 0.2)
```

### spectral_freeze()

Creates a Spectral Freeze synthesis block for frozen spectrum effects.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| source | dict | Yes (named) | - | Source configuration from spectral_source() |

**Returns:** Dict matching the Synthesis::SpectralFreeze IR structure.

**Example:**
```python
spectral_freeze(source = spectral_source("noise", "pink"))
spectral_freeze(source = spectral_source("tone", "sawtooth", 440))
```

### spectral_source()

Creates a spectral source configuration for spectral freeze synthesis.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| source_type | str | Yes | - | Source type: "noise" or "tone" |
| param1 | any | Yes | - | For noise: noise_type ("white", "pink", "brown"). For tone: waveform |
| param2 | any | No | None | For tone: frequency in Hz. Ignored for noise. |

**Returns:** Dict matching the SpectralSource IR structure.

**Example:**
```python
spectral_source("noise", "pink")
spectral_source("tone", "sawtooth", 440)
```

### pitched_body()

Creates a Pitched Body synthesis block for impact sounds with frequency sweep.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| start_freq | f64 | Yes (named) | - | Starting frequency in Hz |
| end_freq | f64 | Yes (named) | - | Ending frequency in Hz |

**Returns:** Dict matching the Synthesis::PitchedBody IR structure.

**Example:**
```python
pitched_body(start_freq = 200, end_freq = 50)  # Descending thump
pitched_body(start_freq = 100, end_freq = 400)  # Rising impact
```

---

## Filters

### lowpass()

Creates a lowpass filter.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| cutoff | f64 | Yes | - | Cutoff frequency in Hz |
| resonance | f64 | No | 0.707 | Q factor |
| sweep_to | f64 | No | None | Optional target cutoff |

**Returns:** Dict matching the Filter::Lowpass IR structure.

**Example:**
```python
lowpass(2000)
lowpass(5000, 1.5)
lowpass(5000, 0.707, 500)  # Sweep from 5000 to 500 Hz
```

### highpass()

Creates a highpass filter.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| cutoff | f64 | Yes | - | Cutoff frequency in Hz |
| resonance | f64 | No | 0.707 | Q factor |
| sweep_to | f64 | No | None | Optional target cutoff |

**Returns:** Dict matching the Filter::Highpass IR structure.

**Example:**
```python
highpass(100)
highpass(500, 1.0, 2000)  # Sweep from 500 to 2000 Hz
```

### bandpass()

Creates a bandpass filter.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| center | f64 | Yes | - | Center frequency in Hz |
| resonance | f64 | No | 1.0 | Q factor |
| sweep_to | f64 | No | None | Optional target center frequency |

**Returns:** Dict matching the Filter::Bandpass IR structure.

**Example:**
```python
bandpass(1000)
bandpass(2000, 2.0, 500)
```

### notch()

Creates a notch (band-reject) filter.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| center | f64 | Yes | - | Center frequency in Hz |
| resonance | f64 | No | 1.0 | Q factor |
| sweep_to | f64 | No | None | Optional target center frequency |

**Returns:** Dict matching the Filter::Notch IR structure.

**Example:**
```python
notch(1000)
notch(500, 2.0)
```

### allpass()

Creates an allpass filter.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes | - | Center frequency in Hz |
| resonance | f64 | No | 0.707 | Q factor |
| sweep_to | f64 | No | None | Optional target frequency |

**Returns:** Dict matching the Filter::Allpass IR structure.

**Example:**
```python
allpass(1000)
allpass(500, 1.0, 2000)
```

### comb_filter()

Creates a comb filter.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| delay_ms | f64 | Yes | - | Delay time in milliseconds |
| feedback | f64 | Yes | - | Feedback amount (0.0-0.99) |
| wet | f64 | Yes | - | Wet/dry mix (0.0-1.0) |

**Returns:** Dict matching the Filter::Comb IR structure.

**Example:**
```python
comb_filter(10, 0.8, 0.5)
```

### formant_filter()

Creates a formant filter.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| vowel | str | Yes | - | Vowel preset: "a", "i", "u", "e", "o" |
| intensity | f64 | Yes | - | Intensity (0.0-1.0, 0.0 = dry, 1.0 = full vowel shape) |

**Returns:** Dict matching the Filter::Formant IR structure.

**Example:**
```python
formant_filter("a", 0.8)
formant_filter("e", 1.0)
```

### ladder()

Creates a ladder filter (Moog-style 4-pole lowpass).

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| cutoff | f64 | Yes | - | Cutoff frequency in Hz |
| resonance | f64 | No | 0.0 | Resonance (0.0-1.0, maps to 0-4x feedback) |
| sweep_to | f64 | No | None | Optional target cutoff for sweep |

**Returns:** Dict matching the Filter::Ladder IR structure.

**Example:**
```python
ladder(1000)
ladder(2000, 0.7, 500)
```

### shelf_low()

Creates a low shelf filter.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes | - | Shelf frequency in Hz |
| gain_db | f64 | Yes | - | Gain in dB (positive for boost, negative for cut) |

**Returns:** Dict matching the Filter::ShelfLow IR structure.

**Example:**
```python
shelf_low(200, 6.0)  # Boost bass
shelf_low(300, -3.0)  # Cut bass
```

### shelf_high()

Creates a high shelf filter.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes | - | Shelf frequency in Hz |
| gain_db | f64 | Yes | - | Gain in dB (positive for boost, negative for cut) |

**Returns:** Dict matching the Filter::ShelfHigh IR structure.

**Example:**
```python
shelf_high(8000, 3.0)  # Boost highs
shelf_high(5000, -6.0)  # Cut highs
```

---

## Effects

### reverb()

Creates a reverb effect.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| decay | f64 | No | 0.5 | Reverb decay time |
| wet | f64 | No | 0.3 | Wet/dry mix (0.0-1.0) |
| room_size | f64 | No | 0.8 | Room size factor |
| width | f64 | No | 1.0 | Stereo width (0.0-1.0) |

**Returns:** Dict matching the Effect::Reverb IR structure.

**Example:**
```python
reverb()
reverb(0.8, 0.4, 0.9)
reverb(0.5, 0.3, 0.8, 0.5)  # Narrower stereo width
```

### delay()

Creates a delay effect.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| time_ms | f64 | No | 250 | Delay time in milliseconds |
| feedback | f64 | No | 0.4 | Feedback amount (0.0-1.0) |
| wet | f64 | No | 0.3 | Wet/dry mix (0.0-1.0) |
| ping_pong | bool | No | False | Enable stereo ping-pong mode |

**Returns:** Dict matching the Effect::Delay IR structure.

**Example:**
```python
delay()
delay(500, 0.5, 0.4)
delay(250, 0.4, 0.3, True)  # Ping-pong stereo delay
```

### compressor()

Creates a compressor effect.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| threshold_db | f64 | No | -12 | Threshold in dB |
| ratio | f64 | No | 4 | Compression ratio |
| attack_ms | f64 | No | 10 | Attack time in ms |
| release_ms | f64 | No | 100 | Release time in ms |
| makeup_db | f64 | No | 0 | Makeup gain in dB |

**Returns:** Dict matching the Effect::Compressor IR structure.

**Example:**
```python
compressor()
compressor(-18, 6, 5, 50)
compressor(-12, 4, 10, 100, 3.0)  # With 3dB makeup gain
```

### limiter()

Creates a limiter effect.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| threshold_db | f64 | Yes | - | Threshold in dB where limiting begins (-24 to 0) |
| release_ms | f64 | No | 100 | Release time in ms (10-500) |
| lookahead_ms | f64 | No | 5.0 | Lookahead time in ms (1-10) |
| ceiling_db | f64 | No | -0.3 | Maximum output level in dB (-6 to 0) |

**Returns:** Dict matching the Effect::Limiter IR structure.

**Example:**
```python
limiter(-6.0)
limiter(-3.0, 50.0, 3.0, -0.1)
```

### chorus()

Creates a chorus effect.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| rate | f64 | Yes | - | LFO rate in Hz |
| depth | f64 | Yes | - | Modulation depth (0.0-1.0) |
| wet | f64 | Yes | - | Wet/dry mix (0.0-1.0) |
| voices | int | No | 2 | Number of voices (1-4) |

**Returns:** Dict matching the Effect::Chorus IR structure.

**Example:**
```python
chorus(1.5, 0.5, 0.3)
chorus(2.0, 0.7, 0.4, 4)
```

### phaser()

Creates a phaser effect.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| rate | f64 | Yes | - | LFO rate in Hz |
| depth | f64 | Yes | - | Modulation depth (0.0-1.0) |
| stages | int | Yes | - | Number of allpass stages (2-12) |
| wet | f64 | Yes | - | Wet/dry mix (0.0-1.0) |

**Returns:** Dict matching the Effect::Phaser IR structure.

**Example:**
```python
phaser(0.5, 0.7, 4, 0.5)
phaser(1.0, 0.8, 8, 0.6)
```

### flanger()

Creates a flanger effect.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| rate | f64 | Yes | - | LFO rate in Hz (0.1-10.0) |
| depth | f64 | Yes | - | Modulation depth (0.0-1.0) |
| feedback | f64 | Yes | - | Feedback amount (-0.99 to 0.99) |
| delay_ms | f64 | Yes | - | Base delay time in ms (1-20) |
| wet | f64 | Yes | - | Wet/dry mix (0.0-1.0) |

**Returns:** Dict matching the Effect::Flanger IR structure.

**Example:**
```python
flanger(0.5, 0.7, 0.5, 5.0, 0.5)
```

### bitcrush()

Creates a bitcrusher effect.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| bits | int | Yes | - | Bit depth (1-16) |
| sample_rate_reduction | f64 | No | 1.0 | Sample rate reduction factor (1.0 = no reduction) |

**Returns:** Dict matching the Effect::Bitcrush IR structure.

**Example:**
```python
bitcrush(8)  # 8-bit sound
bitcrush(4, 4.0)  # Heavily crushed
```

### waveshaper()

Creates a waveshaper distortion effect.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| drive | f64 | Yes | - | Drive amount (1.0-100.0) |
| curve | str | No | "tanh" | Shaping curve: "tanh", "soft_clip", "hard_clip", "sine" |
| wet | f64 | No | 1.0 | Wet/dry mix (0.0-1.0) |

**Returns:** Dict matching the Effect::Waveshaper IR structure.

**Example:**
```python
waveshaper(5.0)
waveshaper(20.0, "hard_clip", 0.8)
```

### parametric_eq()

Creates a parametric EQ effect.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| bands | list | Yes | - | List of EQ band dicts from eq_band() |

**Returns:** Dict matching the Effect::ParametricEq IR structure.

**Example:**
```python
parametric_eq([
    eq_band(100, 3.0, 1.0, "lowshelf"),
    eq_band(1000, -2.0, 2.0, "peak"),
    eq_band(8000, 2.0, 0.7, "highshelf")
])
```

### eq_band()

Creates an EQ band configuration.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes | - | Center/corner frequency in Hz |
| gain_db | f64 | Yes | - | Gain in dB (-24 to +24) |
| q | f64 | Yes | - | Q factor (bandwidth), typically 0.1 to 10 |
| band_type | str | Yes | - | Band type: "lowshelf", "highshelf", "peak", "notch" |

**Returns:** Dict matching the EqBand IR structure.

**Example:**
```python
eq_band(100, 3.0, 1.0, "lowshelf")
eq_band(3000, -4.0, 2.0, "peak")
```

### stereo_widener()

Creates a stereo width effect for enhancing or narrowing the stereo field.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| width | f64 | Yes (named) | - | Stereo width (0.0-2.0, 1.0 = unchanged, >1.0 = wider) |
| mode | str | No (named) | "simple" | Width mode: "simple", "haas", "mid_side" |
| delay_ms | f64 | No (named) | 10.0 | Delay time for Haas effect (1-30 ms) |

**Returns:** Dict matching the Effect::StereoWidener IR structure.

**Example:**
```python
stereo_widener(width = 1.5)
stereo_widener(width = 1.8, mode = "haas", delay_ms = 15.0)
stereo_widener(width = 0.5, mode = "mid_side")  # Narrow the stereo field
```

### delay_tap()

Creates a delay tap configuration for multi-tap delay effects.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| time_ms | f64 | Yes (named) | - | Delay time in milliseconds (1-2000) |
| feedback | f64 | Yes (named) | - | Feedback amount (0.0-0.99) |
| pan | f64 | Yes (named) | - | Stereo pan position (-1.0 to 1.0) |
| level | f64 | Yes (named) | - | Tap output level (0.0-1.0) |
| filter_cutoff | f64 | No (named) | 0.0 | Optional lowpass filter cutoff (0 = disabled) |

**Returns:** Dict matching the DelayTap IR structure.

**Example:**
```python
delay_tap(time_ms = 250, feedback = 0.3, pan = -0.5, level = 0.8)
delay_tap(time_ms = 500, feedback = 0.4, pan = 0.5, level = 0.6, filter_cutoff = 2000)
```

### multi_tap_delay()

Creates a multi-tap delay effect with independent delay taps.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| taps | list | Yes (named) | - | List of delay tap configurations from delay_tap() |

**Returns:** Dict matching the Effect::MultiTapDelay IR structure.

**Example:**
```python
multi_tap_delay(taps = [
    delay_tap(time_ms = 200, feedback = 0.3, pan = -0.7, level = 0.8),
    delay_tap(time_ms = 400, feedback = 0.2, pan = 0.7, level = 0.6),
    delay_tap(time_ms = 600, feedback = 0.1, pan = 0.0, level = 0.4)
])
```

### tape_saturation()

Creates a tape saturation effect with warmth, wow/flutter, and hiss.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| drive | f64 | Yes (named) | - | Drive/saturation amount (1.0-20.0) |
| bias | f64 | Yes (named) | - | DC bias before saturation (-0.5 to 0.5). Affects harmonic content |
| wow_rate | f64 | Yes (named) | - | Wow rate in Hz (0.0-3.0). Low-frequency pitch modulation |
| flutter_rate | f64 | Yes (named) | - | Flutter rate in Hz (0.0-20.0). Higher-frequency pitch modulation |
| hiss_level | f64 | Yes (named) | - | Tape hiss level (0.0-0.1). Seeded noise added to output |

**Returns:** Dict matching the Effect::TapeSaturation IR structure.

**Example:**
```python
tape_saturation(drive = 3.0, bias = 0.1, wow_rate = 0.5, flutter_rate = 5.0, hiss_level = 0.02)
tape_saturation(drive = 8.0, bias = 0.0, wow_rate = 1.0, flutter_rate = 8.0, hiss_level = 0.05)
```

### transient_shaper()

Creates a transient shaper effect for controlling attack and sustain dynamics.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| attack | f64 | Yes (named) | - | Attack gain modifier (-1.0 to 1.0, positive = more punch) |
| sustain | f64 | Yes (named) | - | Sustain gain modifier (-1.0 to 1.0, positive = more body) |
| output_gain_db | f64 | Yes (named) | - | Output gain in dB (-12 to +12) |

**Returns:** Dict matching the Effect::TransientShaper IR structure.

**Example:**
```python
transient_shaper(attack = 0.5, sustain = -0.3, output_gain_db = 0.0)  # Punchier
transient_shaper(attack = -0.5, sustain = 0.5, output_gain_db = 0.0)  # Softer attack, more sustain
```

### auto_filter()

Creates an auto-filter / envelope follower effect for dynamic filter sweeps.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| sensitivity | f64 | Yes (named) | - | Envelope sensitivity (0.0-1.0) |
| attack_ms | f64 | Yes (named) | - | Envelope attack time in ms (0.1-100) |
| release_ms | f64 | Yes (named) | - | Envelope release time in ms (10-1000) |
| depth | f64 | Yes (named) | - | Filter sweep depth (0.0-1.0) |
| base_frequency | f64 | Yes (named) | - | Base filter frequency in Hz (100-8000) |

**Returns:** Dict matching the Effect::AutoFilter IR structure.

**Example:**
```python
auto_filter(sensitivity = 0.7, attack_ms = 10, release_ms = 100, depth = 0.8, base_frequency = 500)
auto_filter(sensitivity = 0.5, attack_ms = 5, release_ms = 200, depth = 1.0, base_frequency = 200)
```

### cabinet_sim()

Creates a cabinet simulation effect for amp/speaker modeling.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| cabinet_type | str | Yes (named) | - | Cabinet type: "guitar_1x12", "guitar_4x12", "bass_1x15", "radio", "telephone" |
| mic_position | f64 | No (named) | 0.0 | Microphone position (0.0-1.0, 0.0 = center, 1.0 = edge) |

**Returns:** Dict matching the Effect::CabinetSim IR structure.

**Example:**
```python
cabinet_sim(cabinet_type = "guitar_4x12")
cabinet_sim(cabinet_type = "guitar_1x12", mic_position = 0.5)
cabinet_sim(cabinet_type = "telephone")  # Lo-fi effect
```

### rotary_speaker()

Creates a rotary speaker (Leslie) effect with amplitude modulation and Doppler.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| rate | f64 | Yes (named) | - | Rotation rate in Hz (0.5-10.0, "slow" ~1 Hz, "fast" ~6 Hz) |
| depth | f64 | Yes (named) | - | Effect intensity (0.0-1.0) |
| wet | f64 | Yes (named) | - | Wet/dry mix (0.0-1.0) |

**Returns:** Dict matching the Effect::RotarySpeaker IR structure.

**Example:**
```python
rotary_speaker(rate = 1.0, depth = 0.8, wet = 0.7)  # Slow rotation
rotary_speaker(rate = 6.0, depth = 0.9, wet = 0.8)  # Fast rotation
```

### ring_modulator()

Creates a ring modulator effect that multiplies audio with a carrier oscillator.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| frequency | f64 | Yes (named) | - | Carrier frequency in Hz (20-2000) |
| mix | f64 | Yes (named) | - | Wet/dry mix (0.0-1.0) |

**Returns:** Dict matching the Effect::RingModulator IR structure.

**Example:**
```python
ring_modulator(frequency = 200, mix = 0.5)
ring_modulator(frequency = 800, mix = 1.0)  # Full effect
```

### granular_delay()

Creates a granular delay effect for shimmer and pitchy delays.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| time_ms | f64 | Yes (named) | - | Delay time in milliseconds (10-2000) |
| feedback | f64 | Yes (named) | - | Feedback amount (0.0-0.95) |
| grain_size_ms | f64 | Yes (named) | - | Grain window size in milliseconds (10-200) |
| pitch_semitones | f64 | Yes (named) | - | Pitch shift per grain pass in semitones (-24 to +24) |
| wet | f64 | Yes (named) | - | Wet/dry mix (0.0-1.0) |

**Returns:** Dict matching the Effect::GranularDelay IR structure.

**Example:**
```python
granular_delay(time_ms = 500, feedback = 0.6, grain_size_ms = 50, pitch_semitones = 12, wet = 0.5)
granular_delay(time_ms = 300, feedback = 0.7, grain_size_ms = 100, pitch_semitones = 7, wet = 0.4)
```

---

## Modulation

### lfo()

Creates an LFO (Low Frequency Oscillator) configuration.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| waveform | str | Yes | - | LFO waveform: "sine", "square", "sawtooth", "triangle" |
| rate | f64 | Yes | - | LFO rate in Hz (typically 0.1-20 Hz) |
| depth | f64 | Yes | - | Modulation depth (0.0-1.0) |
| phase | f64 | No | None | Initial phase offset (0.0-1.0) |

**Returns:** Dict matching the LfoConfig IR structure.

**Example:**
```python
lfo("sine", 5.0, 0.5)  # 5 Hz sine LFO at 50% depth
lfo("triangle", 2.0, 0.3, 0.25)  # With phase offset
```

### lfo_modulation()

Creates an LFO modulation with a target.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| config | dict | Yes | - | LFO configuration from lfo() |
| target | str | Yes | - | Modulation target (see below) |
| amount | f64 | Yes | - | Target-specific modulation amount |

**Valid targets:**
- `"pitch"` - Pitch modulation (amount in semitones)
- `"volume"` - Tremolo
- `"filter_cutoff"` - Filter modulation
- `"pan"` - Pan modulation
- `"pulse_width"` - PWM
- `"fm_index"` - FM depth
- `"grain_size"` - Granular grain size (amount in ms)
- `"grain_density"` - Granular density
- `"delay_time"` - Delay time (amount in ms)
- `"reverb_size"` - Reverb size
- `"distortion_drive"` - Distortion drive

**Returns:** Dict matching the LfoModulation IR structure.

**Example:**
```python
lfo_modulation(lfo("sine", 5.0, 0.5), "pitch", 2.0)  # 2 semitone vibrato
lfo_modulation(lfo("triangle", 4.0, 0.3), "volume", 0.5)  # Tremolo
lfo_modulation(lfo("sine", 2.0, 0.7), "filter_cutoff", 2000)  # Filter wobble
```

### pitch_envelope()

Creates a pitch envelope.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| attack | f64 | No | 0.01 | Attack time in seconds |
| decay | f64 | No | 0.1 | Decay time in seconds |
| sustain | f64 | No | 0.5 | Sustain level (0.0-1.0) |
| release | f64 | No | 0.2 | Release time in seconds |
| depth | f64 | No | 0.0 | Pitch depth in semitones (can be positive or negative) |

**Returns:** Dict matching the PitchEnvelope IR structure.

**Example:**
```python
pitch_envelope(0.0, 0.1, 0.0, 0.0, 12.0)  # Pitch drop from +12 semitones
pitch_envelope(0.01, 0.05, 0.0, 0.0, -24.0)  # Quick pitch dive
```

---

## Layers

### audio_layer()

Creates a complete audio synthesis layer.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| synthesis | dict | Yes | - | Synthesis from oscillator(), fm_synth(), etc. |
| envelope | dict | No | None | Optional envelope (uses default if None) |
| volume | f64 | No | 0.8 | Layer volume (0.0-1.0) |
| pan | f64 | No | 0.0 | Stereo pan (-1.0 to 1.0) |
| filter | dict | No | None | Optional filter |
| lfo | dict | No | None | Optional LFO modulation |
| delay | f64 | No | None | Optional layer start delay in seconds |

**Returns:** Dict matching the AudioLayer IR structure.

**Example:**
```python
audio_layer(oscillator(440))
audio_layer(
    synthesis = oscillator(440, "sawtooth"),
    envelope = envelope(0.01, 0.2, 0.6, 0.3),
    volume = 0.7,
    pan = -0.3,
    filter = lowpass(2000, 0.707, 500)
)
```

---

[← Back to Index](stdlib-reference.md)
