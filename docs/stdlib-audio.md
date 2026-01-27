# SpecCade Starlark Standard Library - Audio Functions

[← Back to Index](stdlib-reference.md)

> **SSOT:** For complete parameter details, use `speccade stdlib dump --format json`
> or see the Rust types in `crates/speccade-spec/src/recipe/audio/`.

## Synthesis

| Function | Description |
|----------|-------------|
| `envelope(attack, decay, sustain, release)` | ADSR envelope |
| `oscillator(frequency, waveform, sweep_to, curve, detune, duty)` | Basic oscillator |
| `fm_synth(carrier, modulator, index, sweep_to)` | FM synthesis |
| `am_synth(carrier, modulator, depth, sweep_to)` | AM synthesis |
| `noise_burst(noise_type, filter)` | Noise burst |
| `karplus_strong(frequency, decay, blend)` | Plucked string |
| `additive(base_freq, harmonics)` | Additive harmonics |
| `supersaw_unison(frequency, voices, detune_cents, spread, detune_curve)` | Detuned unison |
| `wavetable(table, frequency, position, position_end, voices, detune)` | Wavetable morphing |
| `granular(source, grain_size_ms, grain_density, ...)` | Granular synthesis |
| `granular_source(source_type, param1, param2)` | Granular source config |
| `ring_mod_synth(carrier, modulator, mix, sweep_to)` | Ring modulation |
| `multi_oscillator(frequency, oscillators, sweep_to)` | Multi-oscillator |
| `oscillator_config(waveform, volume, detune, phase, duty)` | Oscillator for multi_oscillator |
| `membrane_drum(frequency, decay, tone, strike)` | Drum synthesis |
| `feedback_fm(frequency, feedback, modulation_index, sweep_to)` | Feedback FM |
| `pd_synth(frequency, distortion, distortion_decay, waveform, sweep_to)` | Phase distortion |
| `modal(frequency, modes, excitation, sweep_to)` | Modal synthesis |
| `modal_mode(freq_ratio, amplitude, decay_time)` | Modal mode config |
| `metallic(base_freq, num_partials, inharmonicity)` | Metallic/bell |
| `comb_filter_synth(frequency, decay, excitation)` | Comb filter |
| `vocoder(carrier_freq, carrier_type, num_bands, ...)` | Vocoder |
| `vocoder_band(center_freq, bandwidth, envelope_pattern)` | Vocoder band config |
| `formant_synth(frequency, formants, vowel, ...)` | Formant/voice |
| `formant_config(frequency, amplitude, bandwidth)` | Formant config |
| `vector_synth(frequency, sources, position_x, position_y, path, ...)` | Vector synthesis |
| `vector_source(source_type, frequency_ratio)` | Vector source config |
| `vector_path_point(x, y, duration)` | Vector path point |
| `waveguide(frequency, breath, noise, damping, resonance)` | Wind instrument |
| `bowed_string(frequency, bow_pressure, bow_position, damping)` | Bowed string |
| `pulsar(frequency, pulse_rate, grain_size_ms, shape)` | Pulsar grains |
| `vosim(frequency, formant_freq, pulses, breathiness)` | VOSIM voice |
| `spectral_freeze(source)` | Frozen spectrum |
| `spectral_source(source_type, param1, param2)` | Spectral source config |
| `pitched_body(start_freq, end_freq)` | Impact body |

## Filters

| Function | Description |
|----------|-------------|
| `lowpass(cutoff, resonance, sweep_to)` | Lowpass filter |
| `highpass(cutoff, resonance, sweep_to)` | Highpass filter |
| `bandpass(center, resonance, sweep_to)` | Bandpass filter |
| `notch(center, resonance, sweep_to)` | Notch (band-reject) |
| `allpass(frequency, resonance, sweep_to)` | Allpass filter |
| `comb_filter(delay_ms, feedback, wet)` | Comb filter |
| `formant_filter(vowel, intensity)` | Formant filter |
| `ladder(cutoff, resonance, sweep_to)` | Moog-style 4-pole LP |
| `shelf_low(frequency, gain_db)` | Low shelf |
| `shelf_high(frequency, gain_db)` | High shelf |

## Effects

| Function | Description |
|----------|-------------|
| `reverb(decay, wet, room_size, width)` | Reverb |
| `delay(time_ms, feedback, wet, ping_pong)` | Delay/echo |
| `compressor(threshold_db, ratio, attack_ms, release_ms, makeup_db)` | Compressor |
| `limiter(threshold_db, release_ms, lookahead_ms, ceiling_db)` | Brick-wall limiter |
| `chorus(rate, depth, wet, voices)` | Chorus |
| `phaser(rate, depth, stages, wet)` | Phaser |
| `flanger(rate, depth, feedback, delay_ms, wet)` | Flanger |
| `bitcrush(bits, sample_rate_reduction)` | Bitcrusher |
| `waveshaper(drive, curve, wet)` | Waveshaper distortion |
| `parametric_eq(bands)` | Parametric EQ |
| `eq_band(frequency, gain_db, q, band_type)` | EQ band config |
| `stereo_widener(width, mode, delay_ms)` | Stereo widener |
| `delay_tap(time_ms, feedback, pan, level, filter_cutoff)` | Delay tap config |
| `multi_tap_delay(taps)` | Multi-tap delay |
| `tape_saturation(drive, bias, wow_rate, flutter_rate, hiss_level)` | Tape saturation |
| `transient_shaper(attack, sustain, output_gain_db)` | Transient shaper |
| `auto_filter(sensitivity, attack_ms, release_ms, depth, base_frequency)` | Auto-filter |
| `cabinet_sim(cabinet_type, mic_position)` | Cabinet simulation |
| `rotary_speaker(rate, depth, wet)` | Rotary speaker (Leslie) |
| `ring_modulator(frequency, mix)` | Ring modulator effect |
| `granular_delay(time_ms, feedback, grain_size_ms, pitch_semitones, wet)` | Granular delay |

## Modulation

| Function | Description |
|----------|-------------|
| `lfo(waveform, rate, depth, phase)` | LFO config |
| `lfo_modulation(config, target, amount)` | LFO with target (pitch, volume, filter_cutoff, pan, etc.) |
| `pitch_envelope(attack, decay, sustain, release, depth)` | Pitch envelope |

## Layers

| Function | Description |
|----------|-------------|
| `audio_layer(synthesis, envelope, volume, pan, filter, lfo, delay)` | Complete audio layer |

[← Back to Index](stdlib-reference.md)
