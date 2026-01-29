# Simple oscillator with envelope - demonstrates audio stdlib
#
# This example shows basic oscillator synthesis using the stdlib helpers.
# The spec() and output() functions create the spec structure,
# while oscillator(), envelope(), and audio_layer() create the synthesis parameters.

spec(
    asset_id = "stdlib-audio-osc-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/oscillator.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = oscillator(440, "sine"),
                    envelope = envelope(0.01, 0.1, 0.5, 0.2),
                    volume = 0.8
                )
            ]
        }
    }
)
