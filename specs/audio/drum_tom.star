# Tom drum - Mid-range pitched body with warm decay

spec(
    asset_id = "drum_tom",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 2023,
    description = "Tom drum - Mid-range pitched body with warm decay",
    tags = ["drums", "percussion", "tom"],
    outputs = [output("drum_tom.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.4,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "oscillator",
                        "waveform": "sine",
                        "frequency": 120.0,
                        "freq_sweep": {
                            "end_freq": 90.0,
                            "curve": "exponential"
                        }
                    },
                    envelope = envelope(0.001, 0.2, 0.0, 0.15),
                    volume = 1.0,
                    pan = 0.0
                )
            ]
        }
    }
)
