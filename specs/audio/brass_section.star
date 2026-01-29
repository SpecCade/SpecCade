# Brass section - FM synthesis with bright harmonics and punchy attack

spec(
    asset_id = "brass_section",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 2013,
    description = "Brass section - FM synthesis with bright harmonics and punchy attack",
    style_tags = ["orchestral", "brass", "big-band", "jazz"],
    outputs = [output("brass_section.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "base_note": "C4",
            "duration_seconds": 1.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = fm_synth(261.63, 523.26, 4.0),
                    envelope = envelope(0.03, 0.1, 0.75, 0.2),
                    volume = 1.0,
                    pan = 0.0
                )
            ],
            "generate_loop_points": True
        }
    }
)
