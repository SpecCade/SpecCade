# Additive synthesis example
#
# Additive synthesis builds complex sounds by summing harmonics.
# Each harmonic has a relative amplitude, creating various timbres.
# Covers: additive()

spec(
    asset_id = "stdlib-audio-additive-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/additive.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [
                # Organ-like timbre with fundamental + odd harmonics
                audio_layer(
                    synthesis = additive(220, [1.0, 0.0, 0.33, 0.0, 0.2, 0.0, 0.14]),
                    envelope = envelope(0.02, 0.1, 0.8, 0.3),
                    volume = 0.7
                )
            ]
        }
    }
)
