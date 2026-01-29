# Simple sine wave beep - the most basic SFX

spec(
    asset_id = "simple_beep",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 1001,
    description = "Simple sine wave beep - the most basic SFX",
    outputs = [output("simple_beep.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.3,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = oscillator(880.0, "sine"),
                    envelope = envelope(0.01, 0.05, 0.6, 0.15),
                    volume = 0.8,
                    pan = 0.0
                )
            ]
        }
    }
)
