# Parametric EQ effect example
#
# Parametric EQ allows precise frequency shaping with multiple bands.
# Essential for mixing and sound design.
# Covers: parametric_eq(), eq_band()

spec(
    asset_id = "stdlib-audio-eq-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/eq.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = noise_burst("white"),
                    envelope = envelope(0.05, 0.3, 0.7, 0.4),
                    volume = 0.7
                )
            ],
            "effects": [
                parametric_eq([
                    eq_band(80, 6.0, 0.7, "lowshelf"),     # Bass boost
                    eq_band(400, -4.0, 2.0, "notch"),      # Cut muddy frequencies
                    eq_band(3000, 3.0, 1.5, "peak"),       # Presence boost
                    eq_band(10000, -3.0, 0.7, "highshelf") # Tame highs
                ])
            ]
        }
    }
)
