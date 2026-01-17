# Music Tracker Song Example
# Demonstrates the music stdlib functions for creating tracker modules.

# Define instruments using synthesis
bass_inst = tracker_instrument(
    name = "bass",
    synthesis = instrument_synthesis("sawtooth"),
    envelope = envelope(0.01, 0.1, 0.8, 0.1),
    default_volume = 48
)

lead_inst = tracker_instrument(
    name = "lead",
    synthesis = instrument_synthesis("pulse", 0.25),
    envelope = envelope(0.01, 0.05, 0.6, 0.3)
)

# Define patterns with notes
intro_pattern = tracker_pattern(64, notes = {
    "0": [
        pattern_note(0, "C3", 0),
        pattern_note(16, "E3", 0),
        pattern_note(32, "G3", 0),
        pattern_note(48, "C4", 0)
    ],
    "1": [
        pattern_note(0, "E4", 1, vol = 40),
        pattern_note(8, "G4", 1, vol = 40),
        pattern_note(16, "C5", 1, vol = 40),
        pattern_note(24, "E5", 1, vol = 40),
        pattern_note(32, "G4", 1, vol = 40),
        pattern_note(40, "C5", 1, vol = 40),
        pattern_note(48, "E5", 1, vol = 40),
        pattern_note(56, "G5", 1, vol = 40)
    ]
})

verse_pattern = tracker_pattern(64, notes = {
    "0": [
        pattern_note(0, "A2", 0),
        pattern_note(16, "C3", 0),
        pattern_note(32, "E3", 0),
        pattern_note(48, "A3", 0)
    ],
    "1": [
        pattern_note(0, "A4", 1),
        pattern_note(16, "C5", 1),
        pattern_note(32, "E5", 1),
        pattern_note(48, "A5", 1)
    ]
})

# Create the complete music spec
music_spec(
    asset_id = "example-tracker-song-01",
    seed = 12345,
    output_path = "music/example.xm",
    format = "xm",
    bpm = 120,
    speed = 6,
    channels = 4,
    instruments = [bass_inst, lead_inst],
    patterns = {
        "intro": intro_pattern,
        "verse": verse_pattern
    },
    arrangement = [
        arrangement_entry("intro", 2),
        arrangement_entry("verse", 4),
        arrangement_entry("intro", 1)
    ],
    name = "Example Song",
    title = "SpecCade Example Tracker Song",
    description = "A simple tracker song demonstrating the music stdlib",
    tags = ["retro", "chiptune", "example"]
)
