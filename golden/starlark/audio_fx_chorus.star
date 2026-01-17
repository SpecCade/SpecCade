# Chorus effect example
#
# Chorus creates a thickening effect by modulating delayed copies of the signal.
# Classic for guitars, synths, and vocals.
# Covers: chorus()

spec(
    asset_id = "stdlib-audio-chorus-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/chorus.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 2.0,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = oscillator(440, "sawtooth"),
                    envelope = envelope(0.05, 0.2, 0.7, 0.4),
                    volume = 0.6,
                    filter = lowpass(3000)
                )
            ],
            "effects": [
                chorus(1.5, 0.4, 0.5, 3)  # rate, depth, wet, voices
            ]
        }
    }
)
