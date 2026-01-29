# Dual-format parity song - generates both XM and IT from one spec

lead_square = tracker_instrument(
    name = "lead_square",
    synthesis = instrument_synthesis("pulse", duty_cycle = 0.5),
    envelope = envelope(0.002, 0.08, 0.6, 0.12),
    default_volume = 56
)

bass_pulse = tracker_instrument(
    name = "bass_pulse",
    synthesis = instrument_synthesis("pulse", duty_cycle = 0.25),
    envelope = envelope(0.002, 0.12, 0.5, 0.15),
    default_volume = 48
)

main_pattern = tracker_pattern(64, notes = {
    "0": [
        pattern_note(0, "C4", 0, vol = 56),
        pattern_note(8, "E4", 0, vol = 56),
        pattern_note(16, "G4", 0, vol = 56),
        pattern_note(24, "C5", 0, vol = 56),
        pattern_note(32, "G4", 0, vol = 56),
        pattern_note(40, "E4", 0, vol = 56),
        pattern_note(48, "D4", 0, vol = 56),
        pattern_note(56, "C4", 0, vol = 56)
    ],
    "1": [
        pattern_note(0, "C2", 1, vol = 48),
        pattern_note(16, "C2", 1, vol = 48),
        pattern_note(32, "G1", 1, vol = 48),
        pattern_note(48, "C2", 1, vol = 48)
    ]
})

spec(
    asset_id = "xm-it-parity",
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
                    "synthesis": {"type": "pulse", "duty_cycle": 0.5},
                    "envelope": {"attack": 0.002, "decay": 0.08, "sustain": 0.6, "release": 0.12},
                    "default_volume": 56
                },
                {
                    "name": "bass_pulse",
                    "synthesis": {"type": "pulse", "duty_cycle": 0.25},
                    "envelope": {"attack": 0.002, "decay": 0.12, "sustain": 0.5, "release": 0.15},
                    "default_volume": 48
                }
            ],
            "patterns": {
                "main": {
                    "rows": 64,
                    "notes": {
                        "0": [
                            {"row": 0, "note": "C4", "inst": 0, "vol": 56},
                            {"row": 8, "note": "E4", "inst": 0, "vol": 56},
                            {"row": 16, "note": "G4", "inst": 0, "vol": 56},
                            {"row": 24, "note": "C5", "inst": 0, "vol": 56},
                            {"row": 32, "note": "G4", "inst": 0, "vol": 56},
                            {"row": 40, "note": "E4", "inst": 0, "vol": 56},
                            {"row": 48, "note": "D4", "inst": 0, "vol": 56},
                            {"row": 56, "note": "C4", "inst": 0, "vol": 56}
                        ],
                        "1": [
                            {"row": 0, "note": "C2", "inst": 1, "vol": 48},
                            {"row": 16, "note": "C2", "inst": 1, "vol": 48},
                            {"row": 32, "note": "G1", "inst": 1, "vol": 48},
                            {"row": 48, "note": "C2", "inst": 1, "vol": 48}
                        ]
                    }
                }
            },
            "arrangement": [{"pattern": "main", "repeat": 4}]
        }
    }
)
