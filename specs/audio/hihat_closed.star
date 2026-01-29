# Synthesized closed hi-hat - metallic noise

spec(
    asset_id = "hihat_closed",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 1008,
    description = "Synthesized closed hi-hat - metallic noise",
    outputs = [output("hihat_closed.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.08,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "metallic",
                        "base_freq": 8000.0,
                        "num_partials": 6,
                        "inharmonicity": 2.14
                    },
                    envelope = envelope(0.001, 0.025, 0.0, 0.04),
                    volume = 0.6,
                    pan = 0.0
                ),
                audio_layer(
                    synthesis = {
                        "type": "noise_burst",
                        "noise_type": "white",
                        "filter": {
                            "type": "highpass",
                            "cutoff": 9000.0,
                            "resonance": 0.5
                        }
                    },
                    envelope = envelope(0.001, 0.02, 0.0, 0.025),
                    volume = 0.5,
                    pan = 0.0
                )
            ]
        }
    }
)
