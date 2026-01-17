# Dynamics effects example
#
# Demonstrates limiter, transient_shaper, reverb, and eq_band effects.
# Covers: limiter(), transient_shaper(), reverb(), eq_band(), parametric_eq()

spec(
    asset_id = "stdlib-audio-dynamics-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/dynamics.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 2.0,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = oscillator(220, "sawtooth"),
                    envelope = envelope(0.01, 0.1, 0.8, 0.3),
                    volume = 0.8
                )
            ],
            "effects": [
                reverb(0.5, 0.3, 0.7, 0.8),  # decay, wet, room_size, width
                transient_shaper(attack = 0.5, sustain = -0.2, output_gain_db = 3.0),
                parametric_eq([
                    eq_band(800.0, -3.0, 2.0, "notch"),      # frequency, gain_db, q, band_type
                    eq_band(2500.0, 4.0, 1.5, "peak"),
                    eq_band(200.0, 2.0, 0.7, "lowshelf"),
                    eq_band(6000.0, -2.0, 0.7, "highshelf")
                ]),
                limiter(-3.0, 100.0, 5.0, -0.5)  # threshold_db, release_ms, lookahead_ms, ceiling_db
            ]
        }
    }
)
