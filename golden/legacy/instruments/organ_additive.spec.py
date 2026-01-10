# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Organ instrument - additive synthesis with drawbar-style partials

INSTRUMENT = {
    "name": "organ_additive",
    "base_note": "C4",
    "sample_rate": 44100,
    "synthesis": {
        "type": "additive",
        "partials": [
            (0.5, 0.8),    # sub-fundamental
            (1.0, 1.0),    # fundamental
            (2.0, 0.9),    # 2nd harmonic
            (3.0, 0.7),    # 3rd harmonic
            (4.0, 0.5),    # 4th harmonic
            (6.0, 0.3),    # 6th harmonic
            (8.0, 0.2)     # 8th harmonic
        ]
    },
    "envelope": {
        "attack": 0.01,
        "decay": 0.05,
        "sustain": 0.9,
        "release": 0.1
    },
    "output": {
        "duration": 1.0,
        "bit_depth": 16
    }
}
