# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Simple 4-channel loop - basic XM module

SONG = {
    "name": "simple_loop",
    "title": "Simple Loop",
    "format": "xm",
    "bpm": 120,
    "speed": 6,
    "channels": 4,
    "restart_position": 0,
    "instruments": [
        {
            "name": "lead",
            "synthesis": {
                "type": "karplus_strong",
                "damping": 0.996,
                "brightness": 0.7
            },
            "envelope": {
                "attack": 0.001,
                "decay": 0.1,
                "sustain": 0.5,
                "release": 0.2
            }
        },
        {
            "name": "bass",
            "synthesis": {
                "type": "subtractive",
                "oscillators": [{"waveform": "saw"}],
                "filter": {"type": "lowpass", "cutoff": 800}
            },
            "envelope": {
                "attack": 0.01,
                "decay": 0.1,
                "sustain": 0.6,
                "release": 0.15
            }
        }
    ],
    "patterns": {
        "intro": {
            "rows": 64,
            "notes": {
                "0": [
                    {"row": 0, "note": "C4", "inst": 0, "vol": 64},
                    {"row": 16, "note": "E4", "inst": 0, "vol": 64},
                    {"row": 32, "note": "G4", "inst": 0, "vol": 64},
                    {"row": 48, "note": "C5", "inst": 0, "vol": 64}
                ],
                "1": [
                    {"row": 0, "note": "C2", "inst": 1, "vol": 48},
                    {"row": 32, "note": "G2", "inst": 1, "vol": 48}
                ]
            }
        }
    },
    "arrangement": [
        {"pattern": "intro", "repeat": 4}
    ]
}
