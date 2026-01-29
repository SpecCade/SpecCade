# Synthesized kick drum - pitched body with noise layer

spec(
    asset_id = "kick_drum",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 1006,
    description = "Synthesized kick drum - pitched body with noise layer",
    outputs = [output("kick_drum.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.35,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "oscillator",
                        "waveform": "sine",
                        "frequency": 180.0,
                        "freq_sweep": {
                            "end_freq": 45.0,
                            "curve": "exponential"
                        }
                    },
                    envelope = envelope(0.001, 0.15, 0.1, 0.15),
                    volume = 1.0,
                    pan = 0.0
                ),
                audio_layer(
                    synthesis = {
                        "type": "noise_burst",
                        "noise_type": "white",
                        "filter": {
                            "type": "highpass",
                            "cutoff": 2000.0,
                            "resonance": 0.5
                        }
                    },
                    envelope = envelope(0.001, 0.02, 0.0, 0.02),
                    volume = 0.2,
                    pan = 0.0
                )
            ]
        }
    }
)
