# Bass thump - pitched body impact synthesis

spec(
    asset_id = "bass_thump",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 1013,
    description = "Bass thump - pitched body impact synthesis",
    outputs = [output("bass_thump.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "pitched_body",
                        "start_freq": 200.0,
                        "end_freq": 40.0
                    },
                    envelope = envelope(0.001, 0.25, 0.0, 0.2),
                    volume = 1.0,
                    pan = 0.0
                )
            ]
        }
    }
)
