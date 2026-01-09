# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Synthesized kick drum - pitched body with noise layer

SOUND = {
    "name": "kick_drum",
    "duration": 0.35,
    "sample_rate": 44100,
    "layers": [
        {
            "type": "pitched_body",
            "start_freq": 180,
            "end_freq": 45,
            "amplitude": 1.0,
            "envelope": {
                "attack": 0.001,
                "decay": 0.15,
                "sustain": 0.1,
                "release": 0.15
            }
        },
        {
            "type": "noise_burst",
            "color": "white",
            "amplitude": 0.2,
            "envelope": {
                "attack": 0.001,
                "decay": 0.02,
                "sustain": 0.0,
                "release": 0.02
            },
            "filter": {
                "type": "highpass",
                "cutoff": 2000,
                "q": 0.5
            }
        }
    ],
    "normalize": True,
    "peak_db": -1.0
}
