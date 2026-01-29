# Acoustic piano - FM synthesis with complex harmonic structure

spec(
    asset_id = "piano_acoustic",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 2010,
    description = "Acoustic piano - FM synthesis with complex harmonic structure",
    style_tags = ["acoustic", "piano", "classical"],
    outputs = [output("piano_acoustic.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "base_note": "C4",
            "duration_seconds": 2.0,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = fm_synth(261.63, 523.26, 2.5),
                    envelope = envelope(0.001, 1.0, 0.2, 0.8),
                    volume = 1.0,
                    pan = 0.0
                ),
                audio_layer(
                    synthesis = fm_synth(523.26, 1046.52, 1.5),
                    envelope = envelope(0.001, 0.5, 0.1, 0.4),
                    volume = 0.5,
                    pan = 0.0
                )
            ]
        }
    }
)
