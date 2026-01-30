# Bitcrusher effect example
#
# Bitcrusher reduces bit depth and/or sample rate for lo-fi digital distortion.
# Great for retro game sounds and harsh digital textures.
# Covers: bitcrush()

spec(
    asset_id = "stdlib-audio-bitcrush-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/bitcrush.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = oscillator(440, "sawtooth"),
                    envelope = envelope(0.01, 0.1, 0.6, 0.2),
                    volume = 0.7
                )
            ],
            "effects": [
                bitcrush(8, 4.0)  # 8-bit, 4x sample rate reduction
            ]
        }
    }
)
