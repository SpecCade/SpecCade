# Golden test for simple oscillator synthesis - exercises waveform, frequency, freq_sweep, detune, and duty cycle

spec(
    asset_id = "audio-instrument-oscillator-golden",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 999016,
    description = "Golden test for simple oscillator synthesis - exercises waveform, frequency, freq_sweep, detune, and duty cycle",
    outputs = [output("audio_instrument_oscillator.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "base_note": "C4",
            "duration_seconds": 0.5,
            "sample_rate": 22050,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "oscillator",
                        "waveform": "pulse",
                        "frequency": 440.0,
                        "freq_sweep": {
                            "end_freq": 880.0,
                            "curve": "linear"
                        },
                        "detune": 50.0,
                        "duty": 0.25
                    },
                    envelope = envelope(0.01, 0.1, 0.7, 0.15),
                    volume = 1.0,
                    pan = 0.0
                )
            ],
            "pitch_envelope": {
                "attack": 0.05,
                "decay": 0.1,
                "sustain": 0.5,
                "release": 0.1,
                "depth": -12.0
            }
        }
    }
)
