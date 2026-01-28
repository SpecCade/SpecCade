# Audio effects extra coverage example
#
# Demonstrates audio stdlib functions for effects and special audio types.
# Covers: audio_spec, flanger, ring_modulator, true_peak_limiter, impact_builder,
#         whoosh_builder, oneshot_envelope, loop_envelope, with_loop_config

# oneshot_envelope optimized for one-shot attack sounds
oneshot_env = oneshot_envelope(
    attack_ms = 5.0,
    decay_ms = 100.0,
    release_ms = 200.0
)

# loop_envelope optimized for looping sounds
loop_env = loop_envelope(
    attack_ms = 50.0,
    sustain_level = 0.8,
    release_ms = 100.0
)

# flanger effect creates jet-like sweeping sound
flanger_fx = flanger(
    rate = 0.5,
    depth = 0.7,
    feedback = 0.3,
    wet = 0.5
)

# ring_modulator creates bell-like/metallic timbres
ring_fx = ring_modulator(
    frequency = 440.0,
    mix = 0.5
)

# true_peak_limiter for broadcast compliance
limiter_fx = true_peak_limiter(
    ceiling_db = -1.0,
    lookahead_ms = 5.0,
    release_ms = 100.0
)

# impact_builder creates layered impact sounds
impact_layer = impact_builder(
    transient = "click",
    body = "thump",
    tail = "rumble"
)

# whoosh_builder creates sweep sounds
whoosh_layer = whoosh_builder(
    direction = "up",
    duration_ms = 500.0,
    start_freq = 200.0,
    end_freq = 2000.0
)

# Basic layer for demonstrating with_loop_config
base_layer = audio_layer(
    synthesis = oscillator(440, "sine"),
    envelope = envelope(0.01, 0.1, 0.7, 0.2),
    volume = 0.6
)

# with_loop_config adds loop points to a layer
looped_layer = with_loop_config(
    layer = base_layer,
    loop_start_ms = 100.0,
    loop_end_ms = 900.0,
    crossfade_ms = 50.0
)

# audio_spec creates a complete audio spec
audio_spec(
    asset_id = "stdlib-audio-fx-extra-01",
    duration_seconds = 2.0,
    sample_rate = 44100,
    layers = [
        audio_layer(
            synthesis = oscillator(220, "sawtooth"),
            envelope = oneshot_envelope(10.0, 150.0, 300.0),
            volume = 0.5,
            filter = lowpass(2000)
        )
    ],
    effects = [
        flanger(0.3, 0.5, 0.2, 0.4),
        ring_modulator(880.0, 0.3),
        true_peak_limiter(-0.5, 3.0, 50.0)
    ],
    output_path = "sounds/fx_extra_coverage.wav"
)
