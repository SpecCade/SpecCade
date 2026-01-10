# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Power-up sound - rising FM sweep

SOUND = {
    "name": "powerup_rise",
    "duration": 0.6,
    "sample_rate": 44100,
    "layers": [
        {
            "type": "fm_synth",
            "carrier_freq": 400,
            "mod_ratio": 1.5,
            "mod_index": 4.0,
            "index_decay": 2.0,
            "amplitude": 0.8,
            "envelope": {
                "attack": 0.02,
                "decay": 0.1,
                "sustain": 0.7,
                "release": 0.2
            }
        },
        {
            "type": "sine",
            "freq": 400,
            "freq_end": 1600,
            "amplitude": 0.5,
            "envelope": {
                "attack": 0.01,
                "decay": 0.1,
                "sustain": 0.6,
                "release": 0.2
            }
        },
        {
            "type": "sine",
            "freq": 800,
            "freq_end": 3200,
            "amplitude": 0.3,
            "envelope": {
                "attack": 0.02,
                "decay": 0.15,
                "sustain": 0.5,
                "release": 0.2
            }
        }
    ],
    "normalize": True,
    "peak_db": -2.0
}
