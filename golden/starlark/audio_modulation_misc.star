# Golden test: Miscellaneous modulation and effects
# Tests: pitch_envelope, waveshaper, auto_filter, rotary_speaker

# Note: pitch_envelope is added manually to layer dicts since audio_layer() doesn't expose it yet
base_layer_1 = audio_layer(
    synthesis = oscillator(440, "sine"),
    envelope = envelope(attack = 0.01, decay = 0.2, sustain = 0.6, release = 0.3),
    volume = 0.8
)
layer_with_pitch_env = base_layer_1 | {"pitch_envelope": pitch_envelope(attack = 0.01, decay = 0.1, sustain = 0.5, release = 0.2, depth = 12.0)}

base_layer_2 = audio_layer(
    synthesis = oscillator(330, "sawtooth"),
    envelope = envelope(attack = 0.005, decay = 0.15, sustain = 0.5, release = 0.25),
    volume = 0.75
)
layer_with_pitch_env_fast = base_layer_2 | {"pitch_envelope": pitch_envelope(attack = 0.005, decay = 0.05, sustain = 0.3, release = 0.1, depth = 24.0)}

base_layer_3 = audio_layer(
    synthesis = oscillator(220, "square"),
    envelope = envelope(attack = 0.02, decay = 0.3, sustain = 0.7, release = 0.4),
    volume = 0.7
)
layer_with_pitch_env_slow = base_layer_3 | {"pitch_envelope": pitch_envelope(attack = 0.02, decay = 0.2, sustain = 0.6, release = 0.3, depth = 7.0)}

waveshaper_layer = audio_layer(
    synthesis = oscillator(440, "sine"),
    envelope = envelope(attack = 0.01, decay = 0.2, sustain = 0.6, release = 0.3),
    volume = 0.65
)

waveshaper_soft_layer = audio_layer(
    synthesis = oscillator(330, "triangle"),
    envelope = envelope(attack = 0.02, decay = 0.25, sustain = 0.5, release = 0.35),
    volume = 0.6
)

waveshaper_hard_layer = audio_layer(
    synthesis = oscillator(220, "sawtooth"),
    envelope = envelope(attack = 0.01, decay = 0.15, sustain = 0.7, release = 0.25),
    volume = 0.55
)

waveshaper_sine_layer = audio_layer(
    synthesis = oscillator(550, "square"),
    envelope = envelope(attack = 0.015, decay = 0.2, sustain = 0.6, release = 0.3),
    volume = 0.7
)

auto_filter_layer = audio_layer(
    synthesis = oscillator(440, "sawtooth"),
    envelope = envelope(attack = 0.01, decay = 0.3, sustain = 0.5, release = 0.4),
    volume = 0.75
)

auto_filter_fast_layer = audio_layer(
    synthesis = oscillator(330, "square"),
    envelope = envelope(attack = 0.005, decay = 0.2, sustain = 0.6, release = 0.3),
    volume = 0.7
)

auto_filter_slow_layer = audio_layer(
    synthesis = oscillator(220, "triangle"),
    envelope = envelope(attack = 0.02, decay = 0.4, sustain = 0.7, release = 0.5),
    volume = 0.65
)

rotary_slow_layer = audio_layer(
    synthesis = oscillator(440, "sine"),
    envelope = envelope(attack = 0.01, decay = 0.3, sustain = 0.6, release = 0.4),
    volume = 0.7
)

rotary_fast_layer = audio_layer(
    synthesis = oscillator(330, "sawtooth"),
    envelope = envelope(attack = 0.02, decay = 0.25, sustain = 0.5, release = 0.35),
    volume = 0.75
)

rotary_medium_layer = audio_layer(
    synthesis = oscillator(550, "square"),
    envelope = envelope(attack = 0.015, decay = 0.2, sustain = 0.7, release = 0.3),
    volume = 0.65
)

spec(
    asset_id = "stdlib-audio-modulation-misc-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/modulation_misc.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [
                layer_with_pitch_env,
                layer_with_pitch_env_fast,
                layer_with_pitch_env_slow,
                waveshaper_layer,
                waveshaper_soft_layer,
                waveshaper_hard_layer,
                waveshaper_sine_layer,
                auto_filter_layer,
                auto_filter_fast_layer,
                auto_filter_slow_layer,
                rotary_slow_layer,
                rotary_fast_layer,
                rotary_medium_layer,
            ],
            "effects": [
                waveshaper(drive = 5.0, curve = "tanh", wet = 0.3),
                auto_filter(
                    sensitivity = 0.7,
                    attack_ms = 5.0,
                    release_ms = 100.0,
                    depth = 0.8,
                    base_frequency = 200.0
                ),
                rotary_speaker(rate = 1.5, depth = 0.6, wet = 0.4),
            ]
        }
    }
)
