# Karplus-Strong plucked string synthesis example
#
# Karplus-Strong models plucked strings using a delay line with filtering.
# Great for guitar, harp, and bell-like sounds.
# Covers: karplus_strong()

spec(
    asset_id = "stdlib-audio-karplus-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/karplus.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 2.0,
            "sample_rate": 44100,
            "layers": [
                # Plucked string at A220
                audio_layer(
                    synthesis = karplus_strong(220, 0.996, 0.5),
                    envelope = envelope(0.001, 0.05, 0.0, 0.5),
                    volume = 0.8
                )
            ]
        }
    }
)
