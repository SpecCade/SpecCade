# Music functions coverage example
#
# Demonstrates music stdlib functions for cues and transforms.
# Covers: humanize_vol, swing, loop_cue, loop_hi, loop_low, loop_main, stinger, transition

# humanize_vol adds random volume variation to patterns
humanize = humanize_vol(
    min_vol = 40,
    max_vol = 64
)

# swing adds groove/shuffle feel to patterns
swing_transform = swing(
    amount_permille = 500,  # 50% swing
    affected_rows = 2
)

# loop_cue creates explicit loop cue with intensity level
cue = loop_cue(
    name = "battle_main",
    intensity = 0.7,
    tempo_bpm = 140
)

# loop_hi creates high intensity loop cue template
hi_cue = loop_hi(
    name = "combat_intense",
    tempo_bpm = 160
)

# loop_low creates low intensity loop cue template
low_cue = loop_low(
    name = "ambient_calm",
    tempo_bpm = 80
)

# loop_main creates main/standard intensity loop cue template
main_cue = loop_main(
    name = "exploration_normal",
    tempo_bpm = 110
)

# stinger creates one-shot musical event cue
victory_stinger = stinger(
    name = "victory_fanfare",
    duration_ms = 2000,
    tempo_bpm = 120
)

# transition creates bridge cue between music states
trans = transition(
    name = "calm_to_battle",
    from_state = "exploration",
    to_state = "combat",
    duration_ms = 1500
)

# Create a music spec that uses some of these
bass_inst = tracker_instrument(
    name = "bass_coverage",
    synthesis = instrument_synthesis("sawtooth"),
    envelope = envelope(0.01, 0.1, 0.8, 0.1),
    default_volume = 48
)

pattern_a = tracker_pattern(32, notes = {
    "0": [
        pattern_note(0, "C3", 0),
        pattern_note(8, "E3", 0),
        pattern_note(16, "G3", 0),
        pattern_note(24, "C4", 0)
    ]
})

music_spec(
    asset_id = "stdlib-music-functions-coverage-01",
    seed = 42,
    output_path = "music/coverage_track.xm",
    format = "xm",
    bpm = 120,
    speed = 6,
    channels = 2,
    instruments = [bass_inst],
    patterns = {
        "loop": pattern_a
    },
    arrangement = [
        arrangement_entry("loop", 4)
    ],
    name = "Coverage Track"
)
