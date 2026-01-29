# Slow ambient track - IT format with more channels

pad_warm = tracker_instrument(
    name = "pad_warm",
    synthesis = {"type": "sine"},
    envelope = envelope(0.5, 0.3, 0.8, 1.0)
)

bell_soft = tracker_instrument(
    name = "bell_soft",
    synthesis = {"type": "triangle"},
    envelope = envelope(0.01, 0.8, 0.1, 1.5)
)

pad_intro = tracker_pattern(
    name = "pad_intro",
    rows = 128,
    data = [
        pattern_note(row = 0, channel = 0, note = "C3", instrument = 0, volume = 32),
        pattern_note(row = 64, channel = 0, note = "G3", instrument = 0, volume = 32),
        pattern_note(row = 0, channel = 1, note = "E3", instrument = 0, volume = 32),
        pattern_note(row = 64, channel = 1, note = "B3", instrument = 0, volume = 32),
        pattern_note(row = 32, channel = 4, note = "C5", instrument = 1, volume = 24),
        pattern_note(row = 96, channel = 4, note = "G5", instrument = 1, volume = 24)
    ]
)

music_spec(
    asset_id = "ambient_slow",
    seed = 3003,
    output_path = "ambient_slow.it",
    format = "it",
    bpm = 70,
    speed = 8,
    channels = 8,
    loop = True,
    description = "Slow ambient track - IT format with more channels",
    license = "CC0-1.0",
    instruments = [pad_warm, bell_soft],
    patterns = {"pad_intro": pad_intro},
    arrangement = [arrangement_entry("pad_intro", 2)]
)
