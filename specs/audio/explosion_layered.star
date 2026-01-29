# Complex layered explosion - multiple synthesis types combined

spec(
    asset_id = "explosion_layered",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 1003,
    description = "Complex layered explosion - multiple synthesis types combined",
    outputs = [output("explosion_layered.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.2,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "noise_burst",
                        "noise_type": "brown",
                        "filter": {
                            "type": "lowpass",
                            "cutoff": 800.0,
                            "resonance": 1.2
                        }
                    },
                    envelope = envelope(0.005, 0.3, 0.2, 0.7),
                    volume = 1.0,
                    pan = 0.0
                ),
                audio_layer(
                    synthesis = {
                        "type": "oscillator",
                        "waveform": "sine",
                        "frequency": 150.0,
                        "freq_sweep": {
                            "end_freq": 30.0,
                            "curve": "exponential"
                        }
                    },
                    envelope = envelope(0.01, 0.4, 0.1, 0.5),
                    volume = 0.7,
                    pan = 0.0
                ),
                audio_layer(
                    synthesis = {
                        "type": "noise_burst",
                        "noise_type": "white",
                        "filter": {
                            "type": "highpass",
                            "cutoff": 4000.0,
                            "resonance": 0.7
                        }
                    },
                    envelope = envelope(0.001, 0.05, 0.0, 0.1),
                    volume = 0.3,
                    pan = 0.0
                )
            ]
        }
    }
)
