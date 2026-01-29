# Golden test for advanced FM synthesis - exercises fm_synth with carrier/modulator frequencies,
# modulation_index, freq_sweep, and pitch_envelope

spec(
    asset_id = "audio-instrument-fm-advanced-golden",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 999011,
    description = "Golden test for advanced FM synthesis - exercises fm_synth with carrier/modulator frequencies, modulation_index, freq_sweep, and pitch_envelope",
    outputs = [output("audio_instrument_fm_advanced.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "base_note": "A4",
            "duration_seconds": 2.0,
            "sample_rate": 48000,
            "layers": [
                # Layer 1: Basic 1:1 ratio FM
                audio_layer(
                    synthesis = fm_synth(440.0, 440.0, 3.0),
                    envelope = envelope(0.01, 0.2, 0.7, 0.3),
                    volume = 0.6,
                    pan = 0.0
                ),
                # Layer 2: 1:2 ratio with freq sweep
                audio_layer(
                    synthesis = {
                        "type": "fm_synth",
                        "carrier_freq": 440.0,
                        "modulator_freq": 880.0,
                        "modulation_index": 2.0,
                        "freq_sweep": {
                            "end_freq": 220.0,
                            "curve": "exponential"
                        }
                    },
                    envelope = envelope(0.005, 0.15, 0.5, 0.2),
                    volume = 0.4,
                    pan = -0.2
                ),
                # Layer 3: 1:3.5 ratio for metallic quality
                audio_layer(
                    synthesis = fm_synth(440.0, 1540.0, 1.0),
                    envelope = envelope(0.02, 0.3, 0.6, 0.4),
                    volume = 0.3,
                    pan = 0.2
                )
            ],
            "pitch_envelope": {
                "attack": 0.01,
                "decay": 0.1,
                "sustain": 0.0,
                "release": 0.0,
                "depth": 2.0
            },
            "generate_loop_points": True
        }
    }
)
