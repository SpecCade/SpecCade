# Demo spec for CLI variant expansion (base + variants)

spec(
    asset_id = "audio_variants_demo",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 424242,
    description = "Demo spec for CLI variant expansion (base + variants).",
    outputs = [output("demo.wav", "wav")],
    variants = [
        variant("soft", 0),
        variant("hard", 1)
    ],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.15,
            "sample_rate": 22050,
            "layers": [
                audio_layer(
                    synthesis = oscillator(440.0, "sine"),
                    envelope = envelope(0.01, 0.05, 0.6, 0.05),
                    volume = 0.8,
                    pan = 0.0
                )
            ]
        }
    }
)
