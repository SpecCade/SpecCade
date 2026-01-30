# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Drum-focused track - showcases percussion synthesis

SONG = {
    "name": "drum_pattern",
    "title": "Drum Pattern",
    "format": "xm",
    "bpm": 140,
    "speed": 6,
    "channels": 6,
    "restart_position": 0,
    "instruments": [
        {
            "name": "kick",
            "synthesis": {
                "type": "fm",
                "index": 8.0,
                "index_decay": 30.0
            },
            "envelope": {
                "attack": 0.001,
                "decay": 0.15,
                "sustain": 0.0,
                "release": 0.1
            }
        },
        {
            "name": "snare",
            "synthesis": {
                "type": "fm",
                "index": 12.0,
                "index_decay": 40.0
            },
            "envelope": {
                "attack": 0.001,
                "decay": 0.1,
                "sustain": 0.0,
                "release": 0.08
            }
        },
        {
            "name": "hihat",
            "synthesis": {
                "type": "fm",
                "index": 6.0,
                "index_decay": 50.0
            },
            "envelope": {
                "attack": 0.001,
                "decay": 0.03,
                "sustain": 0.0,
                "release": 0.02
            }
        }
    ],
    "patterns": {
        "beat_basic": {
            "rows": 64,
            "notes": {
                "0": [
                    {"row": 0, "note": "C3", "inst": 0, "vol": 64},
                    {"row": 16, "note": "C3", "inst": 0, "vol": 64},
                    {"row": 32, "note": "C3", "inst": 0, "vol": 64},
                    {"row": 48, "note": "C3", "inst": 0, "vol": 64}
                ],
                "1": [
                    {"row": 8, "note": "D3", "inst": 1, "vol": 56},
                    {"row": 24, "note": "D3", "inst": 1, "vol": 56},
                    {"row": 40, "note": "D3", "inst": 1, "vol": 56},
                    {"row": 56, "note": "D3", "inst": 1, "vol": 56}
                ],
                "2": [
                    {"row": 0, "note": "F#5", "inst": 2, "vol": 32},
                    {"row": 4, "note": "F#5", "inst": 2, "vol": 24},
                    {"row": 8, "note": "F#5", "inst": 2, "vol": 32},
                    {"row": 12, "note": "F#5", "inst": 2, "vol": 24},
                    {"row": 16, "note": "F#5", "inst": 2, "vol": 32},
                    {"row": 20, "note": "F#5", "inst": 2, "vol": 24},
                    {"row": 24, "note": "F#5", "inst": 2, "vol": 32},
                    {"row": 28, "note": "F#5", "inst": 2, "vol": 24},
                    {"row": 32, "note": "F#5", "inst": 2, "vol": 32},
                    {"row": 36, "note": "F#5", "inst": 2, "vol": 24},
                    {"row": 40, "note": "F#5", "inst": 2, "vol": 32},
                    {"row": 44, "note": "F#5", "inst": 2, "vol": 24},
                    {"row": 48, "note": "F#5", "inst": 2, "vol": 32},
                    {"row": 52, "note": "F#5", "inst": 2, "vol": 24},
                    {"row": 56, "note": "F#5", "inst": 2, "vol": 32},
                    {"row": 60, "note": "F#5", "inst": 2, "vol": 24}
                ]
            }
        }
    },
    "arrangement": [
        {"pattern": "beat_basic", "repeat": 4}
    ]
}
