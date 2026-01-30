# Wavetable synthesis example
#
# Wavetable synthesis scans through a table of waveforms for evolving timbres.
# Position can sweep across the table over time.
# Covers: wavetable()

spec(
    asset_id = "stdlib-audio-wavetable-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/wavetable.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 2.0,
            "sample_rate": 44100,
            "layers": [
                # Sweeping through the analog wavetable
                audio_layer(
                    synthesis = wavetable("analog", 440, 0.0, 1.0),
                    envelope = envelope(0.1, 0.3, 0.6, 0.5),
                    volume = 0.7
                )
            ]
        }
    }
)
