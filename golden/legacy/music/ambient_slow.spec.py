# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Slow ambient track - IT format with more channels

SONG = {
    "name": "ambient_slow",
    "title": "Ambient Slow",
    "format": "it",
    "bpm": 70,
    "speed": 8,
    "channels": 8,
    "restart_position": 0,
    "instruments": [
        {
            "name": "pad_warm",
            "synthesis": {
                "type": "additive",
                "partials": [
                    (1.0, 1.0),
                    (2.0, 0.5),
                    (3.0, 0.25),
                    (4.0, 0.125)
                ]
            },
            "envelope": {
                "attack": 0.5,
                "decay": 0.3,
                "sustain": 0.8,
                "release": 1.0
            }
        },
        {
            "name": "bell_soft",
            "synthesis": {
                "type": "fm",
                "index": 3.0,
                "index_decay": 2.0
            },
            "envelope": {
                "attack": 0.01,
                "decay": 0.8,
                "sustain": 0.1,
                "release": 1.5
            }
        }
    ],
    "patterns": {
        "pad_intro": {
            "rows": 128,
            "notes": {
                "0": [
                    {"row": 0, "note": "C3", "inst": 0, "vol": 32},
                    {"row": 64, "note": "G3", "inst": 0, "vol": 32}
                ],
                "1": [
                    {"row": 0, "note": "E3", "inst": 0, "vol": 32},
                    {"row": 64, "note": "B3", "inst": 0, "vol": 32}
                ],
                "4": [
                    {"row": 32, "note": "C5", "inst": 1, "vol": 24},
                    {"row": 96, "note": "G5", "inst": 1, "vol": 24}
                ]
            }
        }
    },
    "arrangement": [
        {"pattern": "pad_intro", "repeat": 2}
    ],
    "it_options": {
        "stereo": True,
        "global_volume": 128,
        "mix_volume": 48
    }
}
