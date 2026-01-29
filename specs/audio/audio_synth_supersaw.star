# Supersaw/Unison synthesis example
#
# Supersaw creates thick sounds by layering detuned sawtooth oscillators.
# Classic trance/EDM lead sound.
# Covers: supersaw_unison()

spec(
    asset_id = "stdlib-audio-supersaw-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/supersaw.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = supersaw_unison(440, 7, 20, 0.8),
                    envelope = envelope(0.01, 0.2, 0.7, 0.4),
                    volume = 0.6,
                    filter = lowpass(3000, 0.707, 1000)
                )
            ],
            "effects": [reverb(0.5, 0.2)]
        }
    }
)
