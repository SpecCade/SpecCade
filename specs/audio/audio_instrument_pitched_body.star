# Golden test for pitched body synthesis - exercises start_freq and end_freq for impact sounds

spec(
    asset_id = "audio-instrument-pitched-body-golden",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 999018,
    description = "Golden test for pitched body synthesis - exercises start_freq and end_freq for impact sounds",
    outputs = [output("audio_instrument_pitched_body.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "base_note": "C4",
            "duration_seconds": 0.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = pitched_body(200.0, 50.0),
                    envelope = envelope(0.001, 0.1, 0.3, 0.35),
                    volume = 1.0,
                    pan = 0.0
                )
            ]
        }
    }
)
