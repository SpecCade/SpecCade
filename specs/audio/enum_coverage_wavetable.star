# Wavetable synthesis covering all table enum values

spec(
    asset_id = "enum-coverage-wavetable",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 42,
    description = "Wavetable synthesis covering all table enum values",
    outputs = [output("enum_coverage_wavetable.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 7.0,
            "sample_rate": 44100,
            "layers": [
                # Table: basic
                audio_layer(
                    synthesis = {
                        "type": "wavetable",
                        "table": "basic",
                        "frequency": 220.0,
                        "position": 0.0,
                        "position_sweep": {"end_position": 0.5}
                    },
                    envelope = envelope(0.05, 0.2, 0.6, 0.3),
                    volume = 0.5,
                    pan = 0.0,
                    delay = 0.0
                ),
                # Table: analog
                audio_layer(
                    synthesis = {
                        "type": "wavetable",
                        "table": "analog",
                        "frequency": 330.0,
                        "position": 0.2,
                        "position_sweep": {"end_position": 0.8}
                    },
                    envelope = envelope(0.05, 0.2, 0.6, 0.3),
                    volume = 0.5,
                    pan = 0.0,
                    delay = 1.0
                ),
                # Table: digital
                audio_layer(
                    synthesis = {
                        "type": "wavetable",
                        "table": "digital",
                        "frequency": 440.0,
                        "position": 0.3,
                        "position_sweep": {"end_position": 0.9}
                    },
                    envelope = envelope(0.05, 0.2, 0.6, 0.3),
                    volume = 0.5,
                    pan = 0.0,
                    delay = 2.0
                ),
                # Table: pwm
                audio_layer(
                    synthesis = {
                        "type": "wavetable",
                        "table": "pwm",
                        "frequency": 550.0,
                        "position": 0.4,
                        "position_sweep": {"end_position": 1.0}
                    },
                    envelope = envelope(0.05, 0.2, 0.6, 0.3),
                    volume = 0.5,
                    pan = 0.0,
                    delay = 3.0
                ),
                # Table: formant
                audio_layer(
                    synthesis = {
                        "type": "wavetable",
                        "table": "formant",
                        "frequency": 660.0,
                        "position": 0.5,
                        "position_sweep": {"end_position": 1.0}
                    },
                    envelope = envelope(0.05, 0.2, 0.6, 0.3),
                    volume = 0.5,
                    pan = 0.0,
                    delay = 4.0
                ),
                # Table: organ
                audio_layer(
                    synthesis = {
                        "type": "wavetable",
                        "table": "organ",
                        "frequency": 770.0,
                        "position": 0.6,
                        "position_sweep": {"end_position": 1.0}
                    },
                    envelope = envelope(0.05, 0.2, 0.6, 0.3),
                    volume = 0.5,
                    pan = 0.0,
                    delay = 5.0
                )
            ]
        }
    }
)
