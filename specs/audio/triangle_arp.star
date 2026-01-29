# Triangle wave arpeggio with logarithmic sweep

spec(
    asset_id = "triangle_arp",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 1015,
    description = "Triangle wave arpeggio with logarithmic sweep - covers triangle waveform and logarithmic curve",
    outputs = [output("triangle_arp.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.6,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "oscillator",
                        "waveform": "triangle",
                        "frequency": 440.0,
                        "freq_sweep": {
                            "end_freq": 880.0,
                            "curve": "logarithmic"
                        }
                    },
                    envelope = envelope(0.01, 0.1, 0.6, 0.2),
                    volume = 0.8,
                    pan = 0.0
                )
            ]
        }
    }
)
