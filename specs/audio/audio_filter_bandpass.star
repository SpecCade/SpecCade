# Bandpass filter example
#
# Bandpass filter allows frequencies around a center frequency to pass.
# Useful for telephone/radio effects and resonant sweeps.
# Covers: bandpass()

spec(
    asset_id = "stdlib-audio-bandpass-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/bandpass.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.5,
            "sample_rate": 44100,
            "layers": [
                # White noise through bandpass filter with sweep
                audio_layer(
                    synthesis = noise_burst("white"),
                    envelope = envelope(0.05, 0.3, 0.7, 0.4),
                    volume = 0.7,
                    filter = bandpass(1000, 4.0, 4000)
                )
            ]
        }
    }
)
