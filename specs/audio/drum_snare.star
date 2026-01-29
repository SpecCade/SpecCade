# Snare drum - Pitched body with white noise for snare rattle

spec(
    asset_id = "drum_snare",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 2021,
    description = "Snare drum - Pitched body with white noise for snare rattle",
    tags = ["drums", "percussion", "snare"],
    outputs = [output("drum_snare.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.3,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "oscillator",
                        "waveform": "triangle",
                        "frequency": 180.0,
                        "freq_sweep": {
                            "end_freq": 120.0,
                            "curve": "linear"
                        }
                    },
                    envelope = envelope(0.001, 0.08, 0.0, 0.15),
                    volume = 1.0,
                    pan = 0.0
                )
            ]
        }
    }
)
