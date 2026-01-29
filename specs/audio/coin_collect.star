# Coin collect sound - FM with harmonics

spec(
    asset_id = "coin_collect",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 1004,
    description = "Coin collect sound - FM with harmonics",
    outputs = [output("coin_collect.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.4,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = fm_synth(1500.0, 3000.0, 3.0),
                    envelope = envelope(0.001, 0.15, 0.4, 0.2),
                    volume = 0.7,
                    pan = 0.0
                ),
                audio_layer(
                    synthesis = {
                        "type": "oscillator",
                        "waveform": "sine",
                        "frequency": 3000.0,
                        "freq_sweep": {
                            "end_freq": 2000.0,
                            "curve": "linear"
                        }
                    },
                    envelope = envelope(0.001, 0.1, 0.2, 0.15),
                    volume = 0.3,
                    pan = 0.0
                )
            ]
        }
    }
)
