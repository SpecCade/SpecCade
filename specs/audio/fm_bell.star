# FM bell instrument - classic FM synthesis bell tone

spec(
    asset_id = "fm_bell",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 2002,
    description = "FM bell instrument - classic FM synthesis bell tone",
    outputs = [output("fm_bell.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "base_note": "C5",
            "duration_seconds": 1.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = fm_synth(523.25, 2093.0, 5.0),
                    envelope = envelope(0.001, 0.5, 0.2, 1.0),
                    volume = 1.0,
                    pan = 0.0
                )
            ]
        }
    }
)
