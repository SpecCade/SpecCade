# Karplus-Strong plucked string sound

spec(
    asset_id = "pluck_string",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 1010,
    description = "Karplus-Strong plucked string sound",
    outputs = [output("pluck_string.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.8,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = karplus_strong(330.0, 0.995, 0.8),
                    envelope = envelope(0.001, 0.3, 0.4, 0.3),
                    volume = 1.0,
                    pan = 0.0
                )
            ]
        }
    }
)
