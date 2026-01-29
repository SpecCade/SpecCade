# Electric bass - Karplus-Strong with filtered harmonics for warm bass tone

spec(
    asset_id = "bass_electric",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 2011,
    description = "Electric bass - Karplus-Strong with filtered harmonics for warm bass tone",
    tags = ["electric", "bass", "funk", "rock"],
    outputs = [output("bass_electric.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "base_note": "E1",
            "duration_seconds": 1.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {"type": "karplus_strong", "frequency": 41.2, "decay": 0.997, "blend": 0.3},
                    envelope = envelope(0.002, 0.15, 0.6, 0.4),
                    volume = 1.0,
                    pan = 0.0
                )
            ],
            "generate_loop_points": True
        }
    }
)
