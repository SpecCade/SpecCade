# Metallic bell strike - inharmonic partials synthesis

spec(
    asset_id = "bell_strike",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 1012,
    description = "Metallic bell strike - inharmonic partials synthesis",
    outputs = [output("bell_strike.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "metallic",
                        "base_freq": 800.0,
                        "num_partials": 8,
                        "inharmonicity": 1.414
                    },
                    envelope = envelope(0.001, 0.5, 0.1, 0.9),
                    volume = 0.8,
                    pan = 0.0
                )
            ]
        }
    }
)
