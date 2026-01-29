# Golden test for additive synthesis - exercises base_freq and harmonics array

spec(
    asset_id = "audio-instrument-additive-golden",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 999013,
    description = "Golden test for additive synthesis - exercises base_freq and harmonics array",
    outputs = [output("audio_instrument_additive.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "base_note": "E4",
            "duration_seconds": 1.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = additive(440.0, [1.0, 0.5, 0.33, 0.25, 0.2, 0.167, 0.143, 0.125]),
                    envelope = envelope(0.1, 0.3, 0.5, 0.4),
                    volume = 1.0,
                    pan = 0.0
                )
            ]
        }
    }
)
