# Kick drum - Low frequency pitched body with noise transient

spec(
    asset_id = "drum_kick",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 2020,
    description = "Kick drum - Low frequency pitched body with noise transient",
    style_tags = ["drums", "percussion", "kick", "808"],
    outputs = [output("drum_kick.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "oscillator",
                        "waveform": "sine",
                        "frequency": 60.0,
                        "freq_sweep": {
                            "end_freq": 40.0,
                            "curve": "exponential"
                        }
                    },
                    envelope = envelope(0.001, 0.15, 0.0, 0.2),
                    volume = 1.0,
                    pan = 0.0
                )
            ]
        }
    }
)
