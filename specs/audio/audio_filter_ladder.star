# Ladder filter example (Moog-style 4-pole lowpass)
#
# The ladder filter provides classic analog-style resonance.
# Famous for its warm, musical self-oscillation at high resonance.
# Covers: ladder()

spec(
    asset_id = "stdlib-audio-ladder-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/ladder.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 2.0,
            "sample_rate": 44100,
            "layers": [
                # Sawtooth through ladder filter with resonance sweep
                audio_layer(
                    synthesis = oscillator(110, "sawtooth"),
                    envelope = envelope(0.01, 0.2, 0.6, 0.4),
                    volume = 0.6,
                    filter = ladder(4000, 0.8, 200)
                )
            ]
        }
    }
)
