# Audio Spec Reference

> **SSOT:** The authoritative `audio_v1` parameter surface is the Rust type `AudioV1Params` in `crates/speccade-spec/src/recipe/audio/`.

**Asset Type:** `audio` | **Recipe:** `audio_v1` | **Output:** WAV (exactly one `primary` output)

## Recipe Params

| Param | Type | Required | Default | Notes |
|------:|------|:--------:|---------|-------|
| `duration_seconds` | number | yes | — | Total rendered length |
| `sample_rate` | integer | no | 44100 | Sample rate in Hz |
| `layers` | array | yes | — | Synthesis layers |
| `base_note` | string/int | no | — | MIDI note for tracker pitch correction |
| `pitch_envelope` | object | no | — | Global pitch modulation |
| `generate_loop_points` | bool | no | false | Set loop at attack+decay |
| `master_filter` | object | no | — | Post-mix filter |
| `effects` | array | no | [] | Post-mix effect chain |
| `post_fx_lfos` | array | no | [] | LFO modulation of effects |

After mixing, the backend normalizes to **-3 dB peak headroom**.

## Audio Layers

| Field | Type | Required |
|------:|------|:--------:|
| `synthesis` | object | yes |
| `envelope` | object | yes |
| `volume` | number | yes |
| `pan` | number | yes |
| `delay` | number | no |
| `filter` | object | no |
| `lfo` | object | no |

## Synthesis Types

All synthesis types are tagged unions with `type`. For full field details, see `crates/speccade-spec/src/recipe/audio/synthesis.rs`.

| Type | Description |
|------|-------------|
| `oscillator` | Basic waveform (sine, square, sawtooth, triangle, pulse) with optional sweep/detune |
| `fm_synth` | 2-operator FM |
| `am_synth` | Amplitude modulation |
| `feedback_fm` | Self-modulating FM (DX7-style) |
| `ring_mod_synth` | Ring modulation (sum/difference frequencies) |
| `noise_burst` | Noise (white, pink, brown) with optional filter |
| `karplus_strong` | Plucked string |
| `additive` | Harmonic series |
| `multi_oscillator` | Multiple mixed oscillators |
| `supersaw_unison` | Detuned unison voices with stereo spread |
| `wavetable` | Wavetable morphing (basic, analog, digital, pwm, formant, organ) |
| `granular` | Grain-based synthesis |
| `pd_synth` | Phase distortion (Casio CZ-style) |
| `modal` | Struck/bowed resonant modes |
| `metallic` | Inharmonic partials |
| `membrane_drum` | Drum membrane modes (Bessel zeros) |
| `comb_filter_synth` | Comb filter resonance |
| `vocoder` | Vocoder filter bank |
| `formant` | Vowel/voice formant synthesis |
| `vector` | 4-source 2D crossfade |
| `waveguide` | Wind instrument physical model |
| `bowed_string` | Bowed string physical model |
| `pulsar` | Synchronized grain trains |
| `vosim` | Voice simulation (squared-sine pulses) |
| `spectral_freeze` | Frozen FFT spectrum |
| `pitched_body` | Impact frequency sweep |

## Filters

| Type | Key Params | Sweep |
|------|-----------|-------|
| `lowpass` | cutoff, resonance | cutoff_end |
| `highpass` | cutoff, resonance | cutoff_end |
| `bandpass` | center, resonance | center_end |
| `notch` | center, resonance | center_end |
| `allpass` | frequency, resonance | frequency_end |
| `comb` | delay_ms, feedback, wet | — |
| `formant` | vowel, intensity | — |
| `ladder` | cutoff, resonance | cutoff_end |
| `shelf_low` | frequency, gain_db | — |
| `shelf_high` | frequency, gain_db | — |

## Effects Chain

Effects in `effects[]` are processed in order. Tagged unions with `type`.

| Type | Key Params |
|------|-----------|
| `reverb` | room_size, damping, wet, width |
| `delay` | time_ms, feedback, wet, ping_pong |
| `multi_tap_delay` | taps[] (time_ms, feedback, pan, level, filter_cutoff) |
| `chorus` | rate, depth, wet, voices |
| `phaser` | rate, depth, stages, wet |
| `flanger` | rate, depth, feedback, delay_ms, wet |
| `bitcrush` | bits, sample_rate_reduction |
| `waveshaper` | drive, curve, wet |
| `tape_saturation` | drive, bias, wow_rate, flutter_rate, hiss_level |
| `compressor` | threshold_db, ratio, attack_ms, release_ms, makeup_db |
| `limiter` | threshold_db, release_ms, lookahead_ms, ceiling_db |
| `gate_expander` | threshold_db, ratio, attack_ms, hold_ms, release_ms, range_db |
| `parametric_eq` | bands[] (frequency, gain_db, q, band_type) |
| `stereo_widener` | width, mode, delay_ms |
| `transient_shaper` | attack, sustain, output_gain_db |
| `auto_filter` | sensitivity, attack_ms, release_ms, depth, base_frequency |
| `cabinet_sim` | cabinet_type, mic_position |
| `rotary_speaker` | rate, depth, wet |
| `ring_modulator` | frequency, mix |
| `granular_delay` | time_ms, feedback, grain_size_ms, pitch_semitones, wet |

## Post-FX LFO Targets

| Target | Valid Effects | Amount Field |
|--------|---------------|-------------|
| `delay_time` | delay, multi_tap_delay, flanger, granular_delay, stereo_widener (haas) | `amount_ms` |
| `reverb_size` | reverb | `amount` |
| `distortion_drive` | waveshaper, tape_saturation | `amount` |

## Layer LFO Targets

pitch, volume, filter_cutoff, pan, pulse_width, fm_index, grain_size, grain_density.

## See Also

- [Starlark stdlib audio functions](../stdlib-audio.md)
- Rust SSOT: `crates/speccade-spec/src/recipe/audio/`
