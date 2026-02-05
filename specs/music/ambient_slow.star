# Slow ambient track - IT format with more channels

pad_warm = tracker_instrument(
    name = "pad_warm",
    synthesis = instrument_synthesis("sine"),
    envelope = envelope(0.5, 0.3, 0.8, 1.0)
)

bell_soft = tracker_instrument(
    name = "bell_soft",
    synthesis = instrument_synthesis("triangle"),
    envelope = envelope(0.01, 0.8, 0.1, 1.5)
)

pad_intro = tracker_pattern(128, notes = {
    "0": [
        pattern_note(0, "C3", 0, vol = 32),
        pattern_note(64, "G3", 0, vol = 32)
    ],
    "1": [
        pattern_note(0, "E3", 0, vol = 32),
        pattern_note(64, "B3", 0, vol = 32)
    ],
    "4": [
        pattern_note(32, "C5", 1, vol = 24),
        pattern_note(96, "G5", 1, vol = 24)
    ]
})

spec(
    asset_id = "ambient_slow",
    asset_type = "music",
    seed = 3003,
    license = "CC0-1.0",
    description = "Slow ambient track - IT format with more channels",
    outputs = [output("ambient_slow.it", "it")],
    recipe = {
        "kind": "music.tracker_v1",
        "params": {
            "bpm": 70,
            "speed": 8,
            "channels": 8,
            "instruments": [pad_warm, bell_soft],
            "patterns": {"pad_intro": pad_intro},
            "arrangement": [arrangement_entry("pad_intro", 2)]
        }
    }
)
