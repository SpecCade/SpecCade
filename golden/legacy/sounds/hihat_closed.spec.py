# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Synthesized closed hi-hat - metallic noise

SOUND = {
    "name": "hihat_closed",
    "duration": 0.08,
    "sample_rate": 44100,
    "layers": [
        {
            "type": "metallic",
            "base_freq": 6000,
            "num_partials": 8,
            "inharmonicity": 1.5,
            "amplitude": 0.7,
            "envelope": {
                "attack": 0.001,
                "decay": 0.03,
                "sustain": 0.1,
                "release": 0.04
            }
        },
        {
            "type": "noise_burst",
            "color": "white",
            "amplitude": 0.4,
            "envelope": {
                "attack": 0.001,
                "decay": 0.02,
                "sustain": 0.05,
                "release": 0.03
            },
            "filter": {
                "type": "highpass",
                "cutoff": 8000,
                "q": 0.5
            }
        }
    ],
    "normalize": True,
    "peak_db": -3.0
}
