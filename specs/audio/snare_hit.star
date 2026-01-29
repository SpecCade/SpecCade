# Synthesized snare drum - noise + tonal component

spec(
    asset_id = "snare_hit",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 1007,
    description = "Synthesized snare drum - noise + tonal component",
    outputs = [output("snare_hit.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.25,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "noise_burst",
                        "noise_type": "pink",
                        "filter": {
                            "type": "bandpass",
                            "center": 4400.0,
                            "resonance": 0.7
                        }
                    },
                    envelope = envelope(0.001, 0.08, 0.1, 0.12),
                    volume = 0.8,
                    pan = 0.0
                ),
                audio_layer(
                    synthesis = {
                        "type": "oscillator",
                        "waveform": "sine",
                        "frequency": 200.0,
                        "freq_sweep": {
                            "end_freq": 120.0,
                            "curve": "exponential"
                        }
                    },
                    envelope = envelope(0.001, 0.04, 0.0, 0.05),
                    volume = 0.5,
                    pan = 0.0
                )
            ]
        }
    }
)
