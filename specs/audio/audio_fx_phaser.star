# Phaser effect example
#
# Phaser creates sweeping notches using modulated allpass filters.
# Classic psychedelic/space effect.
# Covers: phaser()

spec(
    asset_id = "stdlib-audio-phaser-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/phaser.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 3.0,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = oscillator(220, "sawtooth"),
                    envelope = envelope(0.1, 0.3, 0.7, 0.5),
                    volume = 0.6
                )
            ],
            "effects": [
                phaser(0.5, 0.7, 6, 0.6)  # rate, depth, stages, wet
            ]
        }
    }
)
