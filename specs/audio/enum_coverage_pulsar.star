# Pulsar synthesis covering all shape enum values

spec(
    asset_id = "enum-coverage-pulsar",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 42,
    description = "Pulsar synthesis covering all shape enum values",
    outputs = [output("enum_coverage_pulsar.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 6.0,
            "sample_rate": 44100,
            "layers": [
                # Shape: sine
                audio_layer(
                    synthesis = {
                        "type": "pulsar",
                        "frequency": 220.0,
                        "pulse_rate": 15.0,
                        "grain_size_ms": 40.0,
                        "shape": "sine"
                    },
                    envelope = envelope(0.01, 0.1, 0.6, 0.2),
                    volume = 0.5,
                    pan = 0.0,
                    delay = 0.0
                ),
                # Shape: square
                audio_layer(
                    synthesis = {
                        "type": "pulsar",
                        "frequency": 330.0,
                        "pulse_rate": 18.0,
                        "grain_size_ms": 35.0,
                        "shape": "square"
                    },
                    envelope = envelope(0.01, 0.1, 0.6, 0.2),
                    volume = 0.5,
                    pan = 0.0,
                    delay = 1.0
                ),
                # Shape: sawtooth
                audio_layer(
                    synthesis = {
                        "type": "pulsar",
                        "frequency": 440.0,
                        "pulse_rate": 20.0,
                        "grain_size_ms": 45.0,
                        "shape": "sawtooth"
                    },
                    envelope = envelope(0.01, 0.1, 0.6, 0.2),
                    volume = 0.5,
                    pan = 0.0,
                    delay = 2.0
                ),
                # Shape: triangle
                audio_layer(
                    synthesis = {
                        "type": "pulsar",
                        "frequency": 550.0,
                        "pulse_rate": 22.0,
                        "grain_size_ms": 50.0,
                        "shape": "triangle"
                    },
                    envelope = envelope(0.01, 0.1, 0.6, 0.2),
                    volume = 0.5,
                    pan = 0.0,
                    delay = 3.0
                ),
                # Shape: pulse
                audio_layer(
                    synthesis = {
                        "type": "pulsar",
                        "frequency": 660.0,
                        "pulse_rate": 25.0,
                        "grain_size_ms": 55.0,
                        "shape": "pulse"
                    },
                    envelope = envelope(0.01, 0.1, 0.6, 0.2),
                    volume = 0.5,
                    pan = 0.0,
                    delay = 4.0
                )
            ]
        }
    }
)
