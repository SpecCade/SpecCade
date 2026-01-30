# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Sawtooth lead instrument - subtractive synthesis

INSTRUMENT = {
    "name": "saw_lead",
    "base_note": "C4",
    "sample_rate": 44100,
    "synthesis": {
        "type": "subtractive",
        "oscillators": [
            {"waveform": "saw", "detune": 0},
            {"waveform": "saw", "detune": 7}
        ],
        "filter": {
            "type": "lowpass",
            "cutoff": 4000,
            "q": 1.5
        }
    },
    "envelope": {
        "attack": 0.05,
        "decay": 0.1,
        "sustain": 0.7,
        "release": 0.3
    },
    "output": {
        "duration": 1.0,
        "bit_depth": 16
    }
}
