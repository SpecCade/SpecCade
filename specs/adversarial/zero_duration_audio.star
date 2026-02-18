# Adversarial: Audio with zero-second duration
# Expected: validation rejects duration_seconds <= 0

spec(
    asset_id = "adv-zero-duration-audio",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 99901,
    outputs = [output("sounds/zero.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.0,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = oscillator(440.0, "sine"),
                    envelope = envelope(0.01, 0.05, 0.5, 0.1),
                    volume = 0.8,
                    pan = 0.0
                )
            ]
        }
    }
)
