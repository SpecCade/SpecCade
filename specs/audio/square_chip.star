# Chiptune square wave instrument - retro 8-bit style

spec(
    asset_id = "square_chip",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 2005,
    description = "Chiptune square wave instrument - retro 8-bit style",
    outputs = [output("square_chip.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "base_note": "C4",
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = oscillator(261.63, "square"),
                    envelope = envelope(0.001, 0.1, 0.6, 0.15),
                    volume = 1.0,
                    pan = 0.0
                )
            ]
        }
    }
)
