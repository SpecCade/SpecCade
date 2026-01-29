# Pulse wave bass with variable duty cycle and detune

spec(
    asset_id = "pulse_bass",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 1016,
    description = "Pulse wave bass with variable duty cycle and detune",
    outputs = [output("pulse_bass.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = {
                        "type": "oscillator",
                        "waveform": "pulse",
                        "frequency": 110.0,
                        "detune": 5.0,
                        "duty": 0.25
                    },
                    envelope = envelope(0.01, 0.15, 0.5, 0.15),
                    volume = 0.9,
                    pan = 0.0
                )
            ]
        }
    }
)
