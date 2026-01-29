# Spatial effects example
#
# Demonstrates spatial and delay effects that create depth, width, and character.
# Covers: stereo_widener, delay_tap, multi_tap_delay, tape_saturation, cabinet_sim, granular_delay

spec(
    asset_id = "stdlib-audio-spatial-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/spatial.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 2.0,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = oscillator(220, "sawtooth"),
                    envelope = envelope(0.01, 0.1, 0.7, 0.3),
                    volume = 0.6
                )
            ],
            "effects": [
                stereo_widener(width = 1.5, mode = "haas", delay_ms = 15.0),
                multi_tap_delay(taps = [
                    delay_tap(time_ms = 150.0, feedback = 0.3, pan = -0.5, level = 0.8),
                    delay_tap(time_ms = 300.0, feedback = 0.2, pan = 0.5, level = 0.6),
                ]),
                tape_saturation(drive = 2.0, bias = 0.1, wow_rate = 0.5, flutter_rate = 5.0, hiss_level = 0.02),
                cabinet_sim(cabinet_type = "guitar_4x12", mic_position = 0.5),
                granular_delay(time_ms = 200.0, feedback = 0.4, grain_size_ms = 50.0, pitch_semitones = 7.0, wet = 0.3),
            ]
        }
    }
)
