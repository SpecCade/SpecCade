# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Complex layered explosion - multiple synthesis types combined

SOUND = {
    "name": "explosion_layered",
    "duration": 1.2,
    "sample_rate": 44100,
    "layers": [
        {
            "type": "noise_burst",
            "color": "brown",
            "amplitude": 1.0,
            "envelope": {
                "attack": 0.005,
                "decay": 0.3,
                "sustain": 0.2,
                "release": 0.7
            },
            "filter": {
                "type": "lowpass",
                "cutoff": 800,
                "cutoff_end": 200,
                "q": 1.2
            }
        },
        {
            "type": "pitched_body",
            "start_freq": 150,
            "end_freq": 30,
            "amplitude": 0.7,
            "delay": 0.01,
            "envelope": {
                "attack": 0.01,
                "decay": 0.4,
                "sustain": 0.1,
                "release": 0.5
            }
        },
        {
            "type": "noise_burst",
            "color": "white",
            "amplitude": 0.3,
            "delay": 0.0,
            "envelope": {
                "attack": 0.001,
                "decay": 0.05,
                "sustain": 0.0,
                "release": 0.1
            },
            "filter": {
                "type": "highpass",
                "cutoff": 4000,
                "q": 0.7
            }
        }
    ],
    "normalize": True,
    "peak_db": -0.5
}
