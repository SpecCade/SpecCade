# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Bass pluck instrument - Karplus-Strong synthesis

INSTRUMENT = {
    "name": "bass_pluck",
    "base_note": "C2",
    "sample_rate": 44100,
    "synthesis": {
        "type": "karplus_strong",
        "damping": 0.998,
        "brightness": 0.5
    },
    "envelope": {
        "attack": 0.001,
        "decay": 0.2,
        "sustain": 0.3,
        "release": 0.4
    },
    "output": {
        "duration": 1.5,
        "bit_depth": 16
    }
}
