# Fast chiptune track - high tempo, square waves

# Define instruments
square_lead = tracker_instrument(
    name = "square_lead",
    synthesis = {"type": "pulse", "duty_cycle": 0.5},
    envelope = envelope(0.001, 0.05, 0.7, 0.1)
)

square_bass = tracker_instrument(
    name = "square_bass",
    synthesis = {"type": "pulse", "duty_cycle": 0.25},
    envelope = envelope(0.001, 0.1, 0.5, 0.1)
)

noise_drum = tracker_instrument(
    name = "noise_drum",
    synthesis = {"type": "noise", "periodic": False},
    envelope = envelope(0.001, 0.05, 0.0, 0.05)
)

# Define verse pattern
verse_pattern = tracker_pattern(
    name = "verse",
    rows = 32,
    data = [
        # Lead melody
        pattern_note(row = 0, channel = 0, note = "A4", instrument = 0, volume = 64),
        pattern_note(row = 4, channel = 0, note = "A4", instrument = 0, volume = 48),
        pattern_note(row = 8, channel = 0, note = "G4", instrument = 0, volume = 64),
        pattern_note(row = 12, channel = 0, note = "G4", instrument = 0, volume = 48),
        pattern_note(row = 16, channel = 0, note = "F4", instrument = 0, volume = 64),
        pattern_note(row = 20, channel = 0, note = "E4", instrument = 0, volume = 64),
        pattern_note(row = 24, channel = 0, note = "D4", instrument = 0, volume = 64),
        pattern_note(row = 28, channel = 0, note = "C4", instrument = 0, volume = 64),
        # Bass line
        pattern_note(row = 0, channel = 1, note = "A2", instrument = 1, volume = 48),
        pattern_note(row = 8, channel = 1, note = "G2", instrument = 1, volume = 48),
        pattern_note(row = 16, channel = 1, note = "F2", instrument = 1, volume = 48),
        pattern_note(row = 24, channel = 1, note = "C2", instrument = 1, volume = 48),
        # Noise drums
        pattern_note(row = 0, channel = 3, note = "C4", instrument = 2, volume = 32),
        pattern_note(row = 4, channel = 3, note = "C4", instrument = 2, volume = 32),
        pattern_note(row = 8, channel = 3, note = "C4", instrument = 2, volume = 32),
        pattern_note(row = 12, channel = 3, note = "C4", instrument = 2, volume = 32),
        pattern_note(row = 16, channel = 3, note = "C4", instrument = 2, volume = 32),
        pattern_note(row = 20, channel = 3, note = "C4", instrument = 2, volume = 32),
        pattern_note(row = 24, channel = 3, note = "C4", instrument = 2, volume = 32),
        pattern_note(row = 28, channel = 3, note = "C4", instrument = 2, volume = 32)
    ]
)

music_spec(
    asset_id = "chiptune_fast",
    seed = 3002,
    output_path = "chiptune_fast.xm",
    format = "xm",
    bpm = 160,
    speed = 4,
    channels = 4,
    loop = True,
    description = "Fast chiptune track - high tempo, square waves",
    license = "CC0-1.0",
    instruments = [square_lead, square_bass, noise_drum],
    patterns = {"verse": verse_pattern},
    arrangement = [arrangement_entry("verse", 4)]
)
