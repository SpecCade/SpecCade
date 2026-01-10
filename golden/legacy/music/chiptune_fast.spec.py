# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Fast chiptune track - high tempo, square waves

SONG = {
    "name": "chiptune_fast",
    "title": "Chiptune Fast",
    "format": "xm",
    "bpm": 160,
    "speed": 4,
    "channels": 4,
    "restart_position": 0,
    "instruments": [
        {
            "name": "square_lead",
            "synthesis": {
                "type": "subtractive",
                "oscillators": [{"waveform": "square", "duty": 0.5}]
            },
            "envelope": {
                "attack": 0.001,
                "decay": 0.05,
                "sustain": 0.7,
                "release": 0.1
            }
        },
        {
            "name": "square_bass",
            "synthesis": {
                "type": "subtractive",
                "oscillators": [{"waveform": "square", "duty": 0.25}]
            },
            "envelope": {
                "attack": 0.001,
                "decay": 0.1,
                "sustain": 0.5,
                "release": 0.1
            }
        },
        {
            "name": "noise_drum",
            "synthesis": {
                "type": "fm",
                "index": 2.0,
                "index_decay": 10.0
            },
            "envelope": {
                "attack": 0.001,
                "decay": 0.05,
                "sustain": 0.0,
                "release": 0.05
            }
        }
    ],
    "patterns": {
        "verse": {
            "rows": 32,
            "notes": {
                "0": [
                    {"row": 0, "note": "A4", "inst": 0, "vol": 64},
                    {"row": 4, "note": "A4", "inst": 0, "vol": 48},
                    {"row": 8, "note": "G4", "inst": 0, "vol": 64},
                    {"row": 12, "note": "G4", "inst": 0, "vol": 48},
                    {"row": 16, "note": "F4", "inst": 0, "vol": 64},
                    {"row": 20, "note": "E4", "inst": 0, "vol": 64},
                    {"row": 24, "note": "D4", "inst": 0, "vol": 64},
                    {"row": 28, "note": "C4", "inst": 0, "vol": 64}
                ],
                "1": [
                    {"row": 0, "note": "A2", "inst": 1, "vol": 48},
                    {"row": 8, "note": "G2", "inst": 1, "vol": 48},
                    {"row": 16, "note": "F2", "inst": 1, "vol": 48},
                    {"row": 24, "note": "C2", "inst": 1, "vol": 48}
                ],
                "3": [
                    {"row": 0, "note": "C4", "inst": 2, "vol": 32},
                    {"row": 4, "note": "C4", "inst": 2, "vol": 32},
                    {"row": 8, "note": "C4", "inst": 2, "vol": 32},
                    {"row": 12, "note": "C4", "inst": 2, "vol": 32},
                    {"row": 16, "note": "C4", "inst": 2, "vol": 32},
                    {"row": 20, "note": "C4", "inst": 2, "vol": 32},
                    {"row": 24, "note": "C4", "inst": 2, "vol": 32},
                    {"row": 28, "note": "C4", "inst": 2, "vol": 32}
                ]
            }
        }
    },
    "arrangement": [
        {"pattern": "verse", "repeat": 4}
    ]
}
