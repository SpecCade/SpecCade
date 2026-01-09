# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Coin collect sound - FM with harmonics

SOUND = {
    "name": "coin_collect",
    "duration": 0.4,
    "sample_rate": 44100,
    "layers": [
        {
            "type": "fm_synth",
            "carrier_freq": 1500,
            "mod_ratio": 2.0,
            "mod_index": 3.0,
            "index_decay": 8.0,
            "amplitude": 0.7,
            "envelope": {
                "attack": 0.001,
                "decay": 0.15,
                "sustain": 0.4,
                "release": 0.2
            }
        },
        {
            "type": "sine",
            "freq": 3000,
            "freq_end": 2000,
            "amplitude": 0.3,
            "envelope": {
                "attack": 0.001,
                "decay": 0.1,
                "sustain": 0.2,
                "release": 0.15
            }
        }
    ],
    "normalize": True,
    "peak_db": -2.0
}
