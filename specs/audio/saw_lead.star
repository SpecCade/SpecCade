# Sawtooth lead instrument - subtractive synthesis

spec(
    asset_id = "saw_lead",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 2003,
    description = "Sawtooth lead instrument - subtractive synthesis",
    outputs = [output("saw_lead.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "base_note": "C4",
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = oscillator(261.63, "sawtooth"),
                    envelope = envelope(0.05, 0.1, 0.7, 0.3),
                    volume = 1.0,
                    pan = 0.0
                )
            ]
        }
    }
)
