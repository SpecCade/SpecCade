# Dual-format parity song - generates both XM and IT from one spec

lead_square = tracker_instrument(
    name = "lead_square",
    base_note = "C4",
    synthesis = {"type": "square"},
    envelope = envelope(0.002, 0.08, 0.6, 0.12),
    default_volume = 56
)

bass_pulse = tracker_instrument(
    name = "bass_pulse",
    base_note = "C4",
    synthesis = {"type": "pulse", "duty_cycle": 0.25},
    envelope = envelope(0.002, 0.12, 0.5, 0.15),
    default_volume = 48
)

main_pattern = tracker_pattern(
    name = "main",
    rows = 64,
    data = [
        pattern_note(row = 0, channel = 0, note = "C4", instrument = 0, volume = 56),
        pattern_note(row = 8, channel = 0, note = "E4", instrument = 0, volume = 56),
        pattern_note(row = 16, channel = 0, note = "G4", instrument = 0, volume = 56),
        pattern_note(row = 24, channel = 0, note = "C5", instrument = 0, volume = 56),
        pattern_note(row = 32, channel = 0, note = "G4", instrument = 0, volume = 56),
        pattern_note(row = 40, channel = 0, note = "E4", instrument = 0, volume = 56),
        pattern_note(row = 48, channel = 0, note = "D4", instrument = 0, volume = 56),
        pattern_note(row = 56, channel = 0, note = "C4", instrument = 0, volume = 56),
        pattern_note(row = 0, channel = 1, note = "C2", instrument = 1, volume = 48),
        pattern_note(row = 16, channel = 1, note = "C2", instrument = 1, volume = 48),
        pattern_note(row = 32, channel = 1, note = "G1", instrument = 1, volume = 48),
        pattern_note(row = 48, channel = 1, note = "C2", instrument = 1, volume = 48)
    ]
)

spec(
    asset_id = "xm_it_parity",
    asset_type = "music",
    license = "CC0-1.0",
    seed = 3010,
    description = "Dual-format parity song - generates both XM and IT from one spec",
    outputs = [
        output("xm_it_parity.xm", "xm"),
        output("xm_it_parity.it", "it")
    ],
    recipe = {
        "kind": "music.tracker_song_v1",
        "params": {
            "format": "xm",
            "bpm": 125,
            "speed": 6,
            "channels": 4,
            "loop": True,
            "instruments": [
                {
                    "name": "lead_square",
                    "base_note": "C4",
                    "synthesis": {"type": "square"},
                    "envelope": {"attack": 0.002, "decay": 0.08, "sustain": 0.6, "release": 0.12},
                    "default_volume": 56
                },
                {
                    "name": "bass_pulse",
                    "base_note": "C4",
                    "synthesis": {"type": "pulse", "duty_cycle": 0.25},
                    "envelope": {"attack": 0.002, "decay": 0.12, "sustain": 0.5, "release": 0.15},
                    "default_volume": 48
                }
            ],
            "patterns": {
                "main": {
                    "rows": 64,
                    "data": [
                        {"row": 0, "channel": 0, "note": "C4", "instrument": 0, "volume": 56},
                        {"row": 8, "channel": 0, "note": "E4", "instrument": 0, "volume": 56},
                        {"row": 16, "channel": 0, "note": "G4", "instrument": 0, "volume": 56},
                        {"row": 24, "channel": 0, "note": "C5", "instrument": 0, "volume": 56},
                        {"row": 32, "channel": 0, "note": "G4", "instrument": 0, "volume": 56},
                        {"row": 40, "channel": 0, "note": "E4", "instrument": 0, "volume": 56},
                        {"row": 48, "channel": 0, "note": "D4", "instrument": 0, "volume": 56},
                        {"row": 56, "channel": 0, "note": "C4", "instrument": 0, "volume": 56},
                        {"row": 0, "channel": 1, "note": "C2", "instrument": 1, "volume": 48},
                        {"row": 16, "channel": 1, "note": "C2", "instrument": 1, "volume": 48},
                        {"row": 32, "channel": 1, "note": "G1", "instrument": 1, "volume": 48},
                        {"row": 48, "channel": 1, "note": "C2", "instrument": 1, "volume": 48}
                    ]
                }
            },
            "arrangement": [{"pattern": "main", "repeat": 4}]
        }
    }
)
