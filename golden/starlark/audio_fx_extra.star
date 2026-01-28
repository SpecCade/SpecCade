# Audio effects extra coverage example
#
# Demonstrates audio stdlib functions for effects and special audio types.
# Covers: audio_spec, flanger, ring_modulator, true_peak_limiter,
#         oneshot_envelope, loop_envelope, impact_builder, whoosh_builder, with_loop_config

# oneshot_envelope optimized for one-shot attack sounds
# Parameters: attack_ms, decay_ms, sustain_level
oneshot_env = oneshot_envelope(5.0, 100.0, 0.0)

# loop_envelope optimized for looping sounds
loop_env = loop_envelope(
    attack_ms = 50.0,
    sustain_level = 0.8,
    release_ms = 100.0
)

# flanger effect creates jet-like sweeping sound
# Parameters: rate, depth, feedback, delay_ms, wet
flanger_fx = flanger(0.5, 0.7, 0.3, 5.0, 0.5)

# ring_modulator creates bell-like/metallic timbres
ring_fx = ring_modulator(
    frequency = 440.0,
    mix = 0.5
)

# true_peak_limiter for broadcast compliance
limiter_fx = true_peak_limiter(
    ceiling_db = -1.0,
    release_ms = 100.0
)

# audio_spec creates a complete audio spec
audio_spec(
    asset_id = "stdlib-audio-fx-extra-01",
    seed = 42,
    duration_seconds = 2.0,
    sample_rate = 44100,
    layers = [
        audio_layer(
            synthesis = oscillator(220, "sawtooth"),
            envelope = oneshot_envelope(10.0, 150.0, 0.0),
            volume = 0.5,
            filter = lowpass(2000)
        )
    ],
    effects = [
        flanger(0.3, 0.5, 0.2, 5.0, 0.4),
        ring_modulator(frequency = 880.0, mix = 0.3),
        true_peak_limiter(ceiling_db = -0.5, release_ms = 50.0)
    ],
    output_path = "sounds/fx_extra_coverage.wav",
    format = "wav"
)

# --------------------------------------------------------------------------
# Impact builder coverage
# --------------------------------------------------------------------------

# impact_builder creates a layered impact sound effect from three component layers
# Parameters: transient, body, tail (all are audio_layer dicts)
impact_transient = audio_layer(
    synthesis = noise("white"),
    envelope = oneshot_envelope(1.0, 20.0, 0.0),
    volume = 0.8,
    filter = highpass(2000)
)

impact_body = audio_layer(
    synthesis = oscillator(80, "sine"),
    envelope = oneshot_envelope(5.0, 100.0, 0.0),
    volume = 0.7
)

impact_tail = audio_layer(
    synthesis = noise("pink"),
    envelope = oneshot_envelope(10.0, 500.0, 0.0),
    volume = 0.3,
    filter = lowpass(500)
)

# impact_builder returns a list of three audio_layer dicts
impact_layers = impact_builder(
    transient = impact_transient,
    body = impact_body,
    tail = impact_tail
)

# --------------------------------------------------------------------------
# Whoosh builder coverage
# --------------------------------------------------------------------------

# whoosh_builder creates a whoosh sound effect with a filtered noise sweep
# Parameters: direction, duration_ms, start_freq, end_freq, noise_type (optional)
whoosh_layer = whoosh_builder(
    direction = "left_to_right",
    duration_ms = 500.0,
    start_freq = 200.0,
    end_freq = 4000.0,
    noise_type = "pink"
)

# whoosh with default noise type
whoosh_simple = whoosh_builder(
    direction = "right_to_left",
    duration_ms = 300.0,
    start_freq = 500.0,
    end_freq = 2000.0
)

# --------------------------------------------------------------------------
# With loop config coverage
# --------------------------------------------------------------------------

# with_loop_config adds loop configuration to an audio layer for seamless looping
# Parameters: layer, loop_start (optional), loop_end (optional), crossfade_samples (optional)
ambient_layer = audio_layer(
    synthesis = oscillator(220, "sine"),
    envelope = loop_envelope(50.0, 0.8, 100.0),
    volume = 0.5
)

# Add loop config to make the layer seamlessly loopable
looped_ambient = with_loop_config(
    layer = ambient_layer,
    loop_start = 0.1,
    loop_end = 1.9,
    crossfade_samples = 882
)

# Simple loop config with defaults
simple_looped = with_loop_config(
    layer = audio_layer(
        synthesis = noise("pink"),
        volume = 0.3
    )
)
