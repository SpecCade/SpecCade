# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Synthesized snare drum - noise + tonal component

SOUND = {
    "name": "snare_hit",
    "duration": 0.25,
    "sample_rate": 44100,
    "layers": [
        {
            "type": "noise_burst",
            "color": "pink",
            "amplitude": 0.8,
            "envelope": {
                "attack": 0.001,
                "decay": 0.08,
                "sustain": 0.1,
                "release": 0.12
            },
            "filter": {
                "type": "bandpass",
                "cutoff_low": 800,
                "cutoff_high": 8000,
                "q": 0.7
            }
        },
        {
            "type": "sine",
            "freq": 200,
            "freq_end": 120,
            "amplitude": 0.5,
            "envelope": {
                "attack": 0.001,
                "decay": 0.04,
                "sustain": 0.0,
                "release": 0.05
            }
        }
    ],
    "normalize": True,
    "peak_db": -2.0
}
