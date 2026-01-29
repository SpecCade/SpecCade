# Coverage-only spec for source_type enum values (tone, formant). Uses granular synthesis which has these values.

spec(
    asset_id = "enum-coverage-source-type",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 42,
    description = "Coverage-only spec for source_type enum values (tone, formant). Uses granular synthesis which has these values.",
    outputs = [output("enum_coverage_source_type.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 3.0,
            "sample_rate": 44100,
            "layers": [
                # Source type: tone
                audio_layer(
                    synthesis = {
                        "type": "granular",
                        "source": {
                            "type": "tone",
                            "waveform": "sine",
                            "frequency": 440.0
                        },
                        "grain_size_ms": 50.0,
                        "grain_density": 20.0
                    },
                    envelope = envelope(0.05, 0.2, 0.5, 0.3),
                    volume = 0.4,
                    pan = 0.0,
                    delay = 0.0
                ),
                # Source type: formant
                audio_layer(
                    synthesis = {
                        "type": "granular",
                        "source": {
                            "type": "formant",
                            "frequency": 220.0,
                            "formant_freq": 880.0
                        },
                        "grain_size_ms": 60.0,
                        "grain_density": 25.0
                    },
                    envelope = envelope(0.05, 0.2, 0.5, 0.3),
                    volume = 0.4,
                    pan = 0.0,
                    delay = 1.0
                )
            ],
            "_coverage_tone": {
                "source_type": "tone"
            },
            "_coverage_formant": {
                "source_type": "formant"
            }
        }
    }
)
