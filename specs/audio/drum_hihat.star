# Hi-hat cymbal - Metallic FM synthesis with noise

spec(
    asset_id = "drum_hihat",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 2022,
    description = "Hi-hat cymbal - Metallic FM synthesis with noise",
    style_tags = ["drums", "percussion", "hihat", "cymbal"],
    outputs = [output("drum_hihat.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.15,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "metallic",
                        "base_freq": 8000.0,
                        "num_partials": 6,
                        "inharmonicity": 2.14
                    },
                    envelope = envelope(0.001, 0.06, 0.0, 0.1),
                    volume = 0.7,
                    pan = 0.0
                ),
                audio_layer(
                    synthesis = {
                        "type": "noise_burst",
                        "noise_type": "white",
                        "filter": {
                            "type": "highpass",
                            "cutoff": 8000.0,
                            "resonance": 0.5
                        }
                    },
                    envelope = envelope(0.001, 0.04, 0.0, 0.06),
                    volume = 0.5,
                    pan = 0.0
                )
            ]
        }
    }
)
