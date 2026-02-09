# Music spec stdlib coverage
#
# Covers the music_spec() function from music.song module

# Create instruments for the spec
bass_instrument = tracker_instrument(
    name = "bass_coverage",
    synthesis = instrument_synthesis("sawtooth"),
    envelope = envelope(0.01, 0.1, 0.8, 0.1),
    default_volume = 48
)

lead_instrument = tracker_instrument(
    name = "lead_coverage",
    synthesis = instrument_synthesis("pulse", 0.25),
    envelope = envelope(0.01, 0.05, 0.6, 0.3),
    default_volume = 52
)

# Create pattern
main_pattern = tracker_pattern(32, notes = {
    "0": [
        pattern_note(0, "C3", 0),
        pattern_note(8, "E3", 0),
        pattern_note(16, "G3", 0),
        pattern_note(24, "C4", 0)
    ],
    "1": [
        pattern_note(0, "E4", 1, vol = 40),
        pattern_note(8, "G4", 1, vol = 40),
        pattern_note(16, "C5", 1, vol = 40),
        pattern_note(24, "E5", 1, vol = 40)
    ]
})

# Example usage of music_spec()
music_spec(
    asset_id = "stdlib-music-spec-coverage-01",
    seed = 42,
    output_path = "music/coverage.xm",
    format = "xm",
    bpm = 120,
    speed = 6,
    channels = 4,
    instruments = [bass_instrument, lead_instrument],
    patterns = {
        "main": main_pattern
    },
    arrangement = [
        arrangement_entry("main", 4)
    ],
    name = "CoverageSong",
    title = "Music Spec Coverage Test",
    loop = True,
    description = "Coverage test for music_spec stdlib function",
    tags = ["coverage", "test"]
)
