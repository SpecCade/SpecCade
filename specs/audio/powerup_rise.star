# Power-up sound - rising FM sweep

spec(
    asset_id = "powerup_rise",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 1009,
    description = "Power-up sound - rising FM sweep",
    outputs = [output("powerup_rise.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.6,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = fm_synth(400.0, 600.0, 4.0),
                    envelope = envelope(0.02, 0.1, 0.7, 0.2),
                    volume = 0.8,
                    pan = 0.0
                ),
                audio_layer(
                    synthesis = {
                        "type": "oscillator",
                        "waveform": "sine",
                        "frequency": 400.0,
                        "freq_sweep": {
                            "end_freq": 1600.0,
                            "curve": "linear"
                        }
                    },
                    envelope = envelope(0.01, 0.1, 0.6, 0.2),
                    volume = 0.5,
                    pan = 0.0
                ),
                audio_layer(
                    synthesis = {
                        "type": "oscillator",
                        "waveform": "sine",
                        "frequency": 800.0,
                        "freq_sweep": {
                            "end_freq": 3200.0,
                            "curve": "linear"
                        }
                    },
                    envelope = envelope(0.02, 0.15, 0.5, 0.2),
                    volume = 0.3,
                    pan = 0.0
                )
            ]
        }
    }
)
