# Multi-layer sound with filter sweep - demonstrates layered synthesis
#
# This example combines multiple synthesis layers:
# 1. A sawtooth wave with a lowpass filter sweep
# 2. A filtered noise burst for attack transient
# Effects are applied to the mix.

spec(
    asset_id = "stdlib-audio-layered-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/layered.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [
                # Main body: sawtooth with filter sweep
                audio_layer(
                    synthesis = oscillator(440, "sawtooth"),
                    envelope = envelope(0.01, 0.2, 0.6, 0.3),
                    volume = 0.6,
                    filter = lowpass(2000, 0.707, 500)
                ),
                # Attack transient: filtered noise
                audio_layer(
                    synthesis = noise_burst("white", lowpass(5000)),
                    envelope = envelope(0.001, 0.05, 0.0, 0.1),
                    volume = 0.3,
                    delay = 0.0
                )
            ],
            "effects": [reverb(0.4, 0.2)]
        }
    }
)
