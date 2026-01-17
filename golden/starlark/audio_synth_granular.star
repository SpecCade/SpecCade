# Granular synthesis example
#
# Granular synthesis creates textures from many overlapping audio grains.
# Source material can be noise, tones, or formants.
# Covers: granular(), granular_source()

spec(
    asset_id = "stdlib-audio-granular-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/granular.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 2.0,
            "sample_rate": 44100,
            "layers": [
                # Granular texture from sine tone source
                audio_layer(
                    synthesis = granular(
                        granular_source("tone", "sine", 440),
                        50,   # grain_size_ms
                        20,   # grain_density
                        2.0,  # pitch_spread semitones
                        0.3,  # position_spread
                        0.6   # pan_spread
                    ),
                    envelope = envelope(0.2, 0.3, 0.7, 0.5),
                    volume = 0.6
                )
            ],
            "effects": [reverb(0.6, 0.3)]
        }
    }
)
