# FM synthesis laser shot - classic arcade sound

spec(
    asset_id = "laser_shot",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 1002,
    description = "FM synthesis laser shot - classic arcade sound",
    outputs = [output("laser_shot.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.25,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "fm_synth",
                        "carrier_freq": 1200.0,
                        "modulator_freq": 3000.0,
                        "modulation_index": 8.0,
                        "freq_sweep": {
                            "end_freq": 300.0,
                            "curve": "exponential"
                        }
                    },
                    envelope = envelope(0.001, 0.1, 0.3, 0.1),
                    volume = 0.9,
                    pan = 0.0
                )
            ]
        }
    }
)
