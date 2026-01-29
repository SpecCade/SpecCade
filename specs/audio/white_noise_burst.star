# Pure white noise burst - useful for percussion/static

spec(
    asset_id = "white_noise_burst",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 1005,
    description = "Pure white noise burst - useful for percussion/static",
    outputs = [output("white_noise_burst.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.15,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "noise_burst",
                        "noise_type": "white"
                    },
                    envelope = envelope(0.001, 0.05, 0.3, 0.08),
                    volume = 1.0,
                    pan = 0.0
                )
            ]
        }
    }
)
