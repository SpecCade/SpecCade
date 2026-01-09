# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Pure white noise burst - useful for percussion/static

SOUND = {
    "name": "white_noise_burst",
    "duration": 0.15,
    "sample_rate": 44100,
    "layers": [
        {
            "type": "noise_burst",
            "color": "white",
            "amplitude": 1.0,
            "envelope": {
                "attack": 0.001,
                "decay": 0.05,
                "sustain": 0.3,
                "release": 0.08
            }
        }
    ],
    "normalize": True,
    "peak_db": -3.0
}
