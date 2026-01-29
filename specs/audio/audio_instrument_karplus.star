# Golden test for Karplus-Strong synthesis - exercises decay, blend (brightness), notes array, and generate_loop_points

spec(
    asset_id = "audio-instrument-karplus-golden",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 999010,
    description = "Golden test for Karplus-Strong synthesis - exercises decay, blend (brightness), notes array, and generate_loop_points",
    outputs = [output("audio_instrument_karplus.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "base_note": "C4",
            "duration_seconds": 1.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = karplus_strong(440.0, 0.996, 0.7),
                    envelope = envelope(0.001, 0.05, 0.9, 0.5),
                    volume = 1.0,
                    pan = 0.0
                )
            ],
            "generate_loop_points": True
        }
    }
)
