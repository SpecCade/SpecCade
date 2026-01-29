# String ensemble - Rich additive synthesis with vibrato-like modulation

spec(
    asset_id = "strings_ensemble",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 2012,
    description = "String ensemble - Rich additive synthesis with vibrato-like modulation",
    style_tags = ["orchestral", "strings", "classical", "ambient"],
    outputs = [output("strings_ensemble.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "base_note": "C4",
            "duration_seconds": 2.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "additive",
                        "base_freq": 261.63,
                        "harmonics": [1.0, 0.7, 0.5, 0.4, 0.35, 0.3, 0.25, 0.2, 0.15, 0.12, 0.1, 0.08]
                    },
                    envelope = envelope(0.15, 0.2, 0.85, 0.5),
                    volume = 1.0,
                    pan = 0.0
                )
            ],
            "pitch_envelope": {
                "attack": 0.0,
                "decay": 0.0,
                "sustain": 1.0,
                "release": 0.0,
                "depth": 0.15
            },
            "generate_loop_points": True
        }
    }
)
