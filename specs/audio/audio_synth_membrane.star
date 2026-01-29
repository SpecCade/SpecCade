# Membrane drum synthesis example
#
# Membrane drum models the vibration of stretched membranes (drums).
# Parameters control the fundamental frequency, decay, tone, and strike intensity.
# Covers: membrane_drum()

spec(
    asset_id = "stdlib-audio-membrane-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/membrane.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.5,
            "sample_rate": 44100,
            "layers": [
                # Kick-like drum sound
                audio_layer(
                    synthesis = membrane_drum(60, 0.7, 0.3, 0.9),
                    envelope = envelope(0.001, 0.05, 0.0, 0.5),
                    volume = 0.9
                )
            ]
        }
    }
)
