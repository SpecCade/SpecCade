# Supersaw unison synthesis covering detune_curve enum values

spec(
    asset_id = "enum-coverage-supersaw",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 42,
    description = "Supersaw unison synthesis covering detune_curve enum values",
    outputs = [output("enum_coverage_supersaw.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 3.0,
            "sample_rate": 44100,
            "layers": [
                # Detune curve: linear
                audio_layer(
                    synthesis = {
                        "type": "supersaw_unison",
                        "frequency": 440.0,
                        "voices": 7,
                        "detune_cents": 25.0,
                        "spread": 0.8,
                        "detune_curve": "linear"
                    },
                    envelope = envelope(0.01, 0.2, 0.7, 0.4),
                    volume = 0.5,
                    pan = 0.0,
                    delay = 0.0
                ),
                # Detune curve: exp2
                audio_layer(
                    synthesis = {
                        "type": "supersaw_unison",
                        "frequency": 440.0,
                        "voices": 7,
                        "detune_cents": 25.0,
                        "spread": 0.8,
                        "detune_curve": "exp2"
                    },
                    envelope = envelope(0.01, 0.2, 0.7, 0.4),
                    volume = 0.5,
                    pan = 0.0,
                    delay = 1.5
                )
            ]
        }
    }
)
