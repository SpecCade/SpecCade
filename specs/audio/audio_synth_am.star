# AM (Amplitude Modulation) synthesis example
#
# AM synthesis modulates the amplitude of a carrier wave with a modulator.
# This creates ring-modulation-like effects and tremolo.
# Covers: am_synth()

spec(
    asset_id = "stdlib-audio-am-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/am.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = am_synth(440, 110, 0.5),
                    envelope = envelope(0.05, 0.2, 0.6, 0.3),
                    volume = 0.7
                )
            ]
        }
    }
)
