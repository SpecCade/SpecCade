# Fast chiptune track - high tempo, square waves

square_lead = tracker_instrument(
    name = "square_lead",
    synthesis = instrument_synthesis("pulse", duty_cycle = 0.5),
    envelope = envelope(0.001, 0.05, 0.7, 0.1)
)

square_bass = tracker_instrument(
    name = "square_bass",
    synthesis = instrument_synthesis("pulse", duty_cycle = 0.25),
    envelope = envelope(0.001, 0.1, 0.5, 0.1)
)

noise_drum = tracker_instrument(
    name = "noise_drum",
    synthesis = instrument_synthesis("noise"),
    envelope = envelope(0.001, 0.05, 0.0, 0.05)
)

verse_pattern = tracker_pattern(32, notes = {
    "0": [
        pattern_note(0, "A4", 0, vol = 64),
        pattern_note(4, "A4", 0, vol = 48),
        pattern_note(8, "G4", 0, vol = 64),
        pattern_note(12, "G4", 0, vol = 48),
        pattern_note(16, "F4", 0, vol = 64),
        pattern_note(20, "E4", 0, vol = 64),
        pattern_note(24, "D4", 0, vol = 64),
        pattern_note(28, "C4", 0, vol = 64)
    ],
    "1": [
        pattern_note(0, "A2", 1, vol = 48),
        pattern_note(8, "G2", 1, vol = 48),
        pattern_note(16, "F2", 1, vol = 48),
        pattern_note(24, "C2", 1, vol = 48)
    ],
    "3": [
        pattern_note(0, "C4", 2, vol = 32),
        pattern_note(4, "C4", 2, vol = 32),
        pattern_note(8, "C4", 2, vol = 32),
        pattern_note(12, "C4", 2, vol = 32),
        pattern_note(16, "C4", 2, vol = 32),
        pattern_note(20, "C4", 2, vol = 32),
        pattern_note(24, "C4", 2, vol = 32),
        pattern_note(28, "C4", 2, vol = 32)
    ]
})

music_spec(
    asset_id = "chiptune_fast",
    seed = 3002,
    output_path = "chiptune_fast.xm",
    format = "xm",
    bpm = 160,
    speed = 4,
    channels = 4,
    instruments = [square_lead, square_bass, noise_drum],
    patterns = {"verse": verse_pattern},
    arrangement = [arrangement_entry("verse", 4)],
    description = "Fast chiptune track - high tempo, square waves",
    license = "CC0-1.0"
)
