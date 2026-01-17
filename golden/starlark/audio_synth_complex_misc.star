# Golden test: Complex miscellaneous synthesis
# Tests: ring_mod_synth, multi_oscillator + oscillator_config, granular_source

ring_mod_layer = audio_layer(
    synthesis = ring_mod_synth(carrier = 440.0, modulator = 110.0, mix = 0.5),
    envelope = envelope(attack = 0.01, decay = 0.2, sustain = 0.6, release = 0.3),
    volume = 0.7
)

ring_mod_sweep_layer = audio_layer(
    synthesis = ring_mod_synth(carrier = 880.0, modulator = 220.0, mix = 0.8, sweep_to = 440.0),
    envelope = envelope(attack = 0.02, decay = 0.25, sustain = 0.5, release = 0.35),
    volume = 0.65
)

multi_osc_layer = audio_layer(
    synthesis = multi_oscillator(
        frequency = 220.0,
        oscillators = [
            oscillator_config("sine", volume = 0.5),
            oscillator_config("sawtooth", volume = 0.3, detune = 5.0),
            oscillator_config("square", volume = 0.2, detune = -5.0),
        ]
    ),
    envelope = envelope(attack = 0.01, decay = 0.3, sustain = 0.6, release = 0.25),
    volume = 0.8
)

multi_osc_phase_layer = audio_layer(
    synthesis = multi_oscillator(
        frequency = 330.0,
        oscillators = [
            oscillator_config("sine", volume = 0.4, phase = 0.0),
            oscillator_config("triangle", volume = 0.3, phase = 0.33),
            oscillator_config("sawtooth", volume = 0.3, phase = 0.67),
        ]
    ),
    envelope = envelope(attack = 0.02, decay = 0.2, sustain = 0.5, release = 0.3),
    volume = 0.75
)

multi_osc_pulse_layer = audio_layer(
    synthesis = multi_oscillator(
        frequency = 440.0,
        oscillators = [
            oscillator_config("pulse", volume = 0.6, duty = 0.25),
            oscillator_config("pulse", volume = 0.4, duty = 0.75, detune = 10.0),
        ],
        sweep_to = 220.0
    ),
    envelope = envelope(attack = 0.01, decay = 0.15, sustain = 0.7, release = 0.2),
    volume = 0.7
)

granular_noise_layer = audio_layer(
    synthesis = granular(
        source = granular_source("noise", "white"),
        grain_size_ms = 50.0,
        grain_density = 20.0,
        pitch_spread = 0.0,
        position_spread = 0.0,
        pan_spread = 0.0
    ),
    envelope = envelope(attack = 0.05, decay = 0.2, sustain = 0.5, release = 0.3),
    volume = 0.6
)

granular_tone_layer = audio_layer(
    synthesis = granular(
        source = granular_source("tone", "sine", 440.0),
        grain_size_ms = 100.0,
        grain_density = 30.0,
        pitch_spread = 2.0,
        position_spread = 0.5,
        pan_spread = 0.8
    ),
    envelope = envelope(attack = 0.02, decay = 0.3, sustain = 0.6, release = 0.25),
    volume = 0.65
)

granular_formant_layer = audio_layer(
    synthesis = granular(
        source = granular_source("formant", 220.0, 880.0),
        grain_size_ms = 75.0,
        grain_density = 25.0,
        pitch_spread = 1.0,
        position_spread = 0.3,
        pan_spread = 0.6
    ),
    envelope = envelope(attack = 0.01, decay = 0.25, sustain = 0.55, release = 0.3),
    volume = 0.7
)

spec(
    asset_id = "stdlib-audio-complex-misc-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/complex_misc.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [
                ring_mod_layer,
                ring_mod_sweep_layer,
                multi_osc_layer,
                multi_osc_phase_layer,
                multi_osc_pulse_layer,
                granular_noise_layer,
                granular_tone_layer,
                granular_formant_layer,
            ]
        }
    }
)
