# Golden test for simple FM synthesis - exercises carrier_freq, modulator_freq, modulation_index, and freq_sweep

spec(
    asset_id = "audio-instrument-fm-simple-golden",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 999015,
    description = "Golden test for simple FM synthesis - exercises carrier_freq, modulator_freq, modulation_index, and freq_sweep",
    outputs = [output("audio_instrument_fm_simple.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "base_note": "A4",
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "fm_synth",
                        "carrier_freq": 440.0,
                        "modulator_freq": 880.0,
                        "modulation_index": 2.5,
                        "freq_sweep": {
                            "end_freq": 110.0,
                            "curve": "exponential"
                        }
                    },
                    envelope = envelope(0.01, 0.2, 0.5, 0.3),
                    volume = 1.0,
                    pan = 0.0
                )
            ]
        }
    }
)
