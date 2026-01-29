# Impact sound with delayed layers - demonstrates layer delay feature

spec(
    asset_id = "delayed_impact",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 1017,
    description = "Impact sound with delayed layers - demonstrates layer delay feature",
    outputs = [output("delayed_impact.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.8,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "oscillator",
                        "waveform": "sine",
                        "frequency": 200.0,
                        "freq_sweep": {
                            "end_freq": 60.0,
                            "curve": "exponential"
                        }
                    },
                    envelope = envelope(0.001, 0.2, 0.0, 0.15),
                    volume = 1.0,
                    pan = -0.3
                ),
                audio_layer(
                    synthesis = {
                        "type": "noise_burst",
                        "noise_type": "white",
                        "filter": {
                            "type": "highpass",
                            "cutoff": 2000.0,
                            "resonance": 0.5,
                            "cutoff_end": 6000.0
                        }
                    },
                    envelope = envelope(0.001, 0.05, 0.0, 0.08),
                    volume = 0.4,
                    pan = 0.3,
                    delay = 0.02
                ),
                audio_layer(
                    synthesis = oscillator(150.0, "sine"),
                    envelope = envelope(0.01, 0.3, 0.2, 0.2),
                    volume = 0.5,
                    pan = 0.0,
                    delay = 0.1
                )
            ]
        }
    }
)
