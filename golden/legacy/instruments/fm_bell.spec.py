# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# FM bell instrument - classic FM synthesis bell tone

INSTRUMENT = {
    "name": "fm_bell",
    "base_note": "C5",
    "sample_rate": 44100,
    "synthesis": {
        "type": "fm",
        "operators": [
            {"ratio": 1.0, "amplitude": 1.0},
            {"ratio": 4.0, "amplitude": 0.8}
        ],
        "index": 5.0,
        "index_decay": 3.0
    },
    "envelope": {
        "attack": 0.001,
        "decay": 0.5,
        "sustain": 0.2,
        "release": 1.0
    },
    "output": {
        "duration": 2.0,
        "bit_depth": 16
    }
}
