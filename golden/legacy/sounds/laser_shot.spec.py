# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# FM synthesis laser shot - classic arcade sound

SOUND = {
    "name": "laser_shot",
    "duration": 0.25,
    "sample_rate": 44100,
    "layers": [
        {
            "type": "fm_synth",
            "carrier_freq": 1200,
            "mod_ratio": 2.5,
            "mod_index": 8.0,
            "index_decay": 20.0,
            "amplitude": 0.9,
            "envelope": {
                "attack": 0.001,
                "decay": 0.1,
                "sustain": 0.3,
                "release": 0.1
            }
        }
    ],
    "normalize": True,
    "peak_db": -1.0
}
