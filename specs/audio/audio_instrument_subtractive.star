# Golden test for subtractive synthesis - exercises multi_oscillator with volume, detune, phase, duty, and freq_sweep

spec(
    asset_id = "audio-instrument-subtractive-golden",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 999012,
    description = "Golden test for subtractive synthesis - exercises multi_oscillator with volume, detune, phase, duty, and freq_sweep",
    outputs = [output("audio_instrument_subtractive.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "base_note": "A4",
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "multi_oscillator",
                        "frequency": 440.0,
                        "oscillators": [
                            {
                                "waveform": "sawtooth",
                                "volume": 1.0,
                                "detune": 0.0,
                                "phase": 0.0
                            },
                            {
                                "waveform": "sawtooth",
                                "volume": 0.8,
                                "detune": 7.0,
                                "phase": 0.5
                            },
                            {
                                "waveform": "square",
                                "volume": 0.5,
                                "detune": -7.0,
                                "phase": 0.25,
                                "duty": 0.25
                            },
                            {
                                "waveform": "sine",
                                "volume": 0.6,
                                "detune": -1200.0,
                                "phase": 0.0
                            }
                        ],
                        "freq_sweep": {
                            "end_freq": 220.0,
                            "curve": "exponential"
                        }
                    },
                    envelope = envelope(0.05, 0.2, 0.7, 0.3),
                    volume = 1.0,
                    pan = 0.0
                )
            ]
        }
    }
)
