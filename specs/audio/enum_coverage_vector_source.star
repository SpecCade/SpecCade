# Vector synthesis coverage for source_type enum values

spec(
    asset_id = "enum-coverage-vector-source",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 42,
    description = "Vector synthesis coverage for source_type enum values",
    outputs = [output("enum_coverage_vector_source.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 4.0,
            "sample_rate": 44100,
            "layers": [
                # Vector layer 1: noise, sine, saw, square
                audio_layer(
                    synthesis = {
                        "type": "vector",
                        "frequency": 440.0,
                        "sources": [
                            {"source_type": "noise", "frequency_ratio": 1.0},
                            {"source_type": "sine", "frequency_ratio": 1.0},
                            {"source_type": "saw", "frequency_ratio": 1.0},
                            {"source_type": "square", "frequency_ratio": 1.0}
                        ],
                        "position_x": 0.5,
                        "position_y": 0.5
                    },
                    envelope = envelope(0.1, 0.2, 0.6, 0.5),
                    volume = 0.5,
                    pan = 0.0,
                    delay = 0.0
                ),
                # Vector layer 2: triangle, wavetable, noise, saw
                audio_layer(
                    synthesis = {
                        "type": "vector",
                        "frequency": 330.0,
                        "sources": [
                            {"source_type": "triangle", "frequency_ratio": 1.0},
                            {"source_type": "wavetable", "frequency_ratio": 1.0},
                            {"source_type": "noise", "frequency_ratio": 0.5},
                            {"source_type": "saw", "frequency_ratio": 2.0}
                        ],
                        "position_x": 0.25,
                        "position_y": 0.75
                    },
                    envelope = envelope(0.05, 0.15, 0.5, 0.4),
                    volume = 0.4,
                    pan = 0.0,
                    delay = 2.0
                )
            ]
        }
    }
)
