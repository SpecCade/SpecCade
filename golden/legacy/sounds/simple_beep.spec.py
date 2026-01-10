# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Simple sine wave beep - the most basic SFX

SOUND = {
    "name": "simple_beep",
    "duration": 0.3,
    "sample_rate": 44100,
    "layers": [
        {
            "type": "sine",
            "freq": 880,
            "amplitude": 0.8,
            "envelope": {
                "attack": 0.01,
                "decay": 0.05,
                "sustain": 0.6,
                "release": 0.15
            }
        }
    ],
    "normalize": True,
    "peak_db": -3.0
}
