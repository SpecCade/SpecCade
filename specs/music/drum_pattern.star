# Drum-focused track - showcases percussion synthesis

kick_inst = tracker_instrument(
    name = "kick",
    synthesis = instrument_synthesis("noise"),
    envelope = envelope(0.0, 0.15, 0.0, 0.15)
)

snare_inst = tracker_instrument(
    name = "snare",
    synthesis = instrument_synthesis("noise", periodic = True),
    envelope = envelope(0.0, 0.08, 0.0, 0.12)
)

hihat_inst = tracker_instrument(
    name = "hihat",
    synthesis = instrument_synthesis("noise"),
    envelope = envelope(0.0, 0.02, 0.0, 0.04)
)

beat_pattern = tracker_pattern(64, notes = {
    "0": [
        pattern_note(0, "C2", 0, vol = 64),
        pattern_note(16, "C2", 0, vol = 64),
        pattern_note(32, "C2", 0, vol = 64),
        pattern_note(48, "C2", 0, vol = 64)
    ],
    "1": [
        pattern_note(8, "C3", 1, vol = 56),
        pattern_note(24, "C3", 1, vol = 56),
        pattern_note(40, "C3", 1, vol = 56),
        pattern_note(56, "C3", 1, vol = 56)
    ],
    "2": [
        pattern_note(0, "C4", 2, vol = 48),
        pattern_note(4, "C4", 2, vol = 40),
        pattern_note(8, "C4", 2, vol = 48),
        pattern_note(12, "C4", 2, vol = 40),
        pattern_note(16, "C4", 2, vol = 48),
        pattern_note(20, "C4", 2, vol = 40),
        pattern_note(24, "C4", 2, vol = 48),
        pattern_note(28, "C4", 2, vol = 40),
        pattern_note(32, "C4", 2, vol = 48),
        pattern_note(36, "C4", 2, vol = 40),
        pattern_note(40, "C4", 2, vol = 48),
        pattern_note(44, "C4", 2, vol = 40),
        pattern_note(48, "C4", 2, vol = 48),
        pattern_note(52, "C4", 2, vol = 40),
        pattern_note(56, "C4", 2, vol = 48),
        pattern_note(60, "C4", 2, vol = 40)
    ]
})

music_spec(
    asset_id = "drum_pattern",
    seed = 3004,
    output_path = "drum_pattern.xm",
    format = "xm",
    bpm = 140,
    speed = 6,
    channels = 6,
    instruments = [kick_inst, snare_inst, hihat_inst],
    patterns = {"beat_basic": beat_pattern},
    arrangement = [arrangement_entry("beat_basic", 4)],
    description = "Drum-focused track - showcases percussion synthesis",
    license = "CC0-1.0"
)
