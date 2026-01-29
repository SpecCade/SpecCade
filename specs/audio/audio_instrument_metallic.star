# Golden test for metallic synthesis - exercises base_freq, num_partials, and inharmonicity

spec(
    asset_id = "audio-instrument-metallic-golden",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 999014,
    description = "Golden test for metallic synthesis - exercises base_freq, num_partials, and inharmonicity",
    outputs = [output("audio_instrument_metallic.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "base_note": "C5",
            "duration_seconds": 3.0,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = metallic(800.0, 8, 1.414),
                    envelope = envelope(0.001, 0.5, 0.2, 1.5),
                    volume = 1.0,
                    pan = 0.0
                )
            ]
        }
    }
)
