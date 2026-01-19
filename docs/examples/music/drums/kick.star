# Tuned Kick Drum - Punchy, No Clipping
#
# A professional-quality kick drum designed for electronic and acoustic styles.
#
# ## Design Rationale
#
# The kick combines two layers:
# 1. **Body Layer**: A sine wave with pitch sweep from 150Hz down to 50Hz creates
#    the characteristic "thump" of a kick drum. The fast attack (1ms) provides punch,
#    while the moderate decay (150ms) gives body without being muddy.
#
# 2. **Click Layer**: A short noise burst filtered through a highpass at 2kHz adds
#    the initial transient "click" that helps the kick cut through a mix.
#    The very short decay (20ms) keeps it tight.
#
# ## Tuning Notes
#
# - Start frequency (150Hz) provides initial attack punch
# - End frequency (50Hz) gives low-end weight without being subsonic
# - Volume levels (0.7 body, 0.15 click) are balanced to avoid clipping
# - Total headroom leaves peak around -2dB for safe mastering
#
# ## Quality Gates
#
# This spec is designed to pass:
# - peak_db < 0 (no clipping)
# - dc_offset < 0.01 (minimal DC offset)
# - rms_db between -24 and -6 dBFS (appropriate loudness)

audio_spec(
    asset_id = "drum-kit-kick-01",
    seed = 42,
    output_path = "drums/kick.wav",
    format = "wav",
    duration_seconds = 0.4,
    sample_rate = 44100,
    description = "Punchy kick drum with body and click layers",
    tags = ["drums", "kick", "percussion", "electronic"],
    layers = [
        # Body layer: pitched sine sweep for the low-end thump
        audio_layer(
            synthesis = oscillator(150, "sine", sweep_to = 50, curve = "exponential"),
            envelope = envelope(
                attack = 0.001,   # 1ms attack for punch
                decay = 0.15,    # 150ms decay for body
                sustain = 0.0,   # No sustain - percussive
                release = 0.2    # 200ms release for natural tail
            ),
            volume = 0.7  # Leave headroom for click layer
        ),
        # Click layer: filtered noise for transient attack
        audio_layer(
            synthesis = noise_burst("white", highpass(2000, 1.0)),
            envelope = envelope(
                attack = 0.001,  # Instant attack
                decay = 0.02,    # 20ms decay - very short
                sustain = 0.0,   # No sustain
                release = 0.02   # Short release
            ),
            volume = 0.15  # Subtle click, not overwhelming
        )
    ]
)
