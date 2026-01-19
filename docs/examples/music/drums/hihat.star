# Closed Hi-Hat - Crisp, Short
#
# A professional-quality closed hi-hat with metallic character.
#
# ## Design Rationale
#
# Hi-hats are characterized by their metallic, inharmonic sound. This is achieved
# through two complementary layers:
#
# 1. **Metallic Layer**: Uses the metallic() synthesis which generates inharmonic
#    partials (frequencies that don't follow the harmonic series). The high base
#    frequency (5kHz) and inharmonicity factor (1.4) create the characteristic
#    shimmer of metal cymbals.
#
# 2. **Noise Layer**: White noise filtered through a steep highpass at 7kHz adds
#    the "air" and brightness that makes hi-hats cut through a mix.
#
# ## Tuning Notes
#
# - Base frequency at 5kHz places the fundamental in the "presence" range
# - 6 partials provide enough complexity without being harsh
# - Inharmonicity 1.4 creates the metallic, bell-like quality
# - Very short duration (80ms total) keeps the hat closed and tight
# - Envelope decays are extremely fast for a crisp, defined sound
#
# ## Quality Gates
#
# This spec is designed to pass:
# - peak_db < 0 (no clipping)
# - dc_offset < 0.01 (minimal DC offset)
# - rms_db between -24 and -6 dBFS (appropriate loudness)

audio_spec(
    asset_id = "drum-kit-hihat-01",
    seed = 42,
    output_path = "drums/hihat.wav",
    format = "wav",
    duration_seconds = 0.1,
    sample_rate = 44100,
    description = "Crisp, short closed hi-hat with metallic character",
    tags = ["drums", "hihat", "percussion", "cymbal"],
    layers = [
        # Metallic layer: inharmonic partials for cymbal character
        audio_layer(
            synthesis = metallic(5000, 6, 1.4),
            envelope = envelope(
                attack = 0.001,  # Instant attack
                decay = 0.03,    # 30ms decay - very fast
                sustain = 0.1,   # Minimal sustain
                release = 0.04   # 40ms release
            ),
            volume = 0.5  # Primary layer
        ),
        # Noise layer: filtered white noise for air and brightness
        audio_layer(
            synthesis = noise_burst("white", highpass(7000, 0.7)),
            envelope = envelope(
                attack = 0.001,  # Instant attack
                decay = 0.02,    # 20ms decay - extremely fast
                sustain = 0.05,  # Tiny sustain
                release = 0.03   # 30ms release
            ),
            volume = 0.35  # Secondary layer for air
        )
    ]
)
