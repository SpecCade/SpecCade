# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Chiptune square wave instrument - retro 8-bit style

INSTRUMENT = {
    "name": "square_chip",
    "base_note": "C4",
    "sample_rate": 44100,
    "synthesis": {
        "type": "subtractive",
        "oscillators": [
            {"waveform": "square", "duty": 0.5, "detune": 0}
        ],
        "filter": {
            "type": "lowpass",
            "cutoff": 8000,
            "q": 0.7
        }
    },
    "envelope": {
        "attack": 0.001,
        "decay": 0.1,
        "sustain": 0.6,
        "release": 0.15
    },
    "output": {
        "duration": 0.8,
        "bit_depth": 16
    }
}
