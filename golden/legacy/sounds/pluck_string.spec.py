# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Karplus-Strong plucked string sound

SOUND = {
    "name": "pluck_string",
    "duration": 0.8,
    "sample_rate": 44100,
    "layers": [
        {
            "type": "karplus",
            "freq": 330,
            "damping": 0.995,
            "brightness": 0.8,
            "amplitude": 1.0,
            "envelope": {
                "attack": 0.001,
                "decay": 0.3,
                "sustain": 0.4,
                "release": 0.3
            }
        }
    ],
    "normalize": True,
    "peak_db": -2.0
}
