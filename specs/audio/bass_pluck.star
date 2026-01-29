# Bass pluck instrument - Karplus-Strong synthesis

spec(
    asset_id = "bass_pluck",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 2001,
    description = "Bass pluck instrument - Karplus-Strong synthesis",
    outputs = [output("bass_pluck.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "base_note": "C2",
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = karplus_strong(65.41, 0.998, 0.5),
                    envelope = envelope(0.001, 0.2, 0.3, 0.4),
                    volume = 1.0,
                    pan = 0.0
                )
            ]
        }
    }
)
