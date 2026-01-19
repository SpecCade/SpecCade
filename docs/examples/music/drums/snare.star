# Tuned Snare Drum - Bright, Tight
#
# A professional-quality snare drum with both tonal and noise components.
#
# ## Design Rationale
#
# The snare combines two layers to capture the essential characteristics:
#
# 1. **Noise Layer**: Pink noise filtered through a bandpass (800Hz-6kHz) creates
#    the "snare wire" rattling sound. Pink noise is used instead of white because
#    it has more natural frequency distribution (less harsh high end).
#
# 2. **Tonal Layer**: A sine wave sweeping from 200Hz to 120Hz provides the
#    drum body resonance. This gives the snare its pitched character.
#
# ## Tuning Notes
#
# - Bandpass center at 3kHz emphasizes the crack
# - Resonance (Q=0.7) provides focused but not harsh sound
# - Noise volume (0.6) dominates for that classic snare character
# - Tonal body (0.35) adds weight without muddiness
# - Fast decays keep the snare tight and punchy
#
# ## Quality Gates
#
# This spec is designed to pass:
# - peak_db < 0 (no clipping)
# - dc_offset < 0.01 (minimal DC offset)
# - rms_db between -24 and -6 dBFS (appropriate loudness)

audio_spec(
    asset_id = "drum-kit-snare-01",
    seed = 42,
    output_path = "drums/snare.wav",
    format = "wav",
    duration_seconds = 0.3,
    sample_rate = 44100,
    description = "Bright, tight snare drum with noise and tonal layers",
    tags = ["drums", "snare", "percussion", "acoustic"],
    layers = [
        # Noise layer: filtered pink noise for snare wire character
        audio_layer(
            synthesis = noise_burst("pink", bandpass(3000, 0.7)),
            envelope = envelope(
                attack = 0.001,  # Instant attack
                decay = 0.08,    # 80ms decay for body
                sustain = 0.1,   # Slight sustain for ring
                release = 0.12   # 120ms release for natural decay
            ),
            volume = 0.6  # Dominant layer
        ),
        # Tonal layer: pitched body for drum resonance
        audio_layer(
            synthesis = oscillator(200, "sine", sweep_to = 120, curve = "exponential"),
            envelope = envelope(
                attack = 0.001,  # Instant attack
                decay = 0.04,    # 40ms decay - quick
                sustain = 0.0,   # No sustain
                release = 0.06   # 60ms release
            ),
            volume = 0.35  # Supporting layer
        )
    ]
)
