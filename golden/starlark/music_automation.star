# Golden test: Music automation
# Tests: volume_fade, tempo_change, it_options

inst = tracker_instrument(
    name = "lead",
    synthesis = instrument_synthesis("sawtooth"),
    default_volume = 64,
)

pattern = tracker_pattern(
    rows = 64,
    notes = {
        "0": [
            pattern_note(0, "C-4", 0, vol = 64),
            pattern_note(16, "E-4", 0, vol = 64),
            pattern_note(32, "G-4", 0, vol = 64),
            pattern_note(48, "C-5", 0, vol = 64)
        ]
    }
)

# Create tracker song with automation and it_options
song = tracker_song(
    format = "it",
    bpm = 120,
    speed = 6,
    channels = 4,
    instruments = [inst],
    patterns = {"main": pattern},
    arrangement = [
        arrangement_entry("main", 2)
    ],
    automation = [
        volume_fade("main", 0, 0, 32, 64, 32),
        tempo_change("main", 32, 140),
    ],
    it_options = it_options(stereo = True, global_volume = 128, mix_volume = 80),
)

# Create the complete spec
spec(
    asset_id = "stdlib-music-automation-01",
    asset_type = "music",
    seed = 42,
    outputs = [output("music/automation.it", "it")],
    recipe = song
)
