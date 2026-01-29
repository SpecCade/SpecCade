# Drum-focused track - showcases percussion synthesis

kick_inst = tracker_instrument(
    name = "kick",
    ref = "../audio/kick_drum.json",
    envelope = envelope(0.0, 0.15, 0.0, 0.15)
)

snare_inst = tracker_instrument(
    name = "snare",
    ref = "../audio/snare_hit.json",
    envelope = envelope(0.0, 0.08, 0.0, 0.12)
)

hihat_inst = tracker_instrument(
    name = "hihat",
    ref = "../audio/hihat_closed.json",
    envelope = envelope(0.0, 0.02, 0.0, 0.04)
)

beat_pattern = tracker_pattern(64, notes = {
    "0": [
        pattern_note(0, None, 0, vol = 64),
        pattern_note(16, None, 0, vol = 64),
        pattern_note(32, None, 0, vol = 64),
        pattern_note(48, None, 0, vol = 64)
    ],
    "1": [
        pattern_note(8, None, 1, vol = 56),
        pattern_note(24, None, 1, vol = 56),
        pattern_note(40, None, 1, vol = 56),
        pattern_note(56, None, 1, vol = 56)
    ],
    "2": [
        pattern_note(0, None, 2, vol = 48),
        pattern_note(4, None, 2, vol = 40),
        pattern_note(8, None, 2, vol = 48),
        pattern_note(12, None, 2, vol = 40),
        pattern_note(16, None, 2, vol = 48),
        pattern_note(20, None, 2, vol = 40),
        pattern_note(24, None, 2, vol = 48),
        pattern_note(28, None, 2, vol = 40),
        pattern_note(32, None, 2, vol = 48),
        pattern_note(36, None, 2, vol = 40),
        pattern_note(40, None, 2, vol = 48),
        pattern_note(44, None, 2, vol = 40),
        pattern_note(48, None, 2, vol = 48),
        pattern_note(52, None, 2, vol = 40),
        pattern_note(56, None, 2, vol = 48),
        pattern_note(60, None, 2, vol = 40)
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
