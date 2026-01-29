# Golden comprehensive test - exercises multiple synthesis types: fm_synth, multi_oscillator, additive,
# karplus_strong, and pitch_envelope with all ADSR

{
    "spec_version": 1,
    "asset_id": "audio-instrument-comprehensive-golden",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 999002,
    "description": "Golden comprehensive test - exercises multiple synthesis types: fm_synth, multi_oscillator, additive, karplus_strong, and pitch_envelope with all ADSR",
    "outputs": [
        {
            "kind": "primary",
            "format": "wav",
            "path": "audio_instrument_comprehensive.wav"
        }
    ],
    "variants": [
        {"variant_id": "c3", "seed_offset": 0},
        {"variant_id": "c4", "seed_offset": 12},
        {"variant_id": "c5", "seed_offset": 24}
    ],
    "recipe": {
        "kind": "audio_v1",
        "params": {
            "base_note": "C4",
            "duration_seconds": 2.0,
            "sample_rate": 44100,
            "layers": [
                # Layer 1: FM synthesis
                audio_layer(
                    synthesis = fm_synth(440.0, 880.0, 4.0),
                    envelope = envelope(0.01, 0.2, 0.7, 0.5),
                    volume = 0.6,
                    pan = 0.0
                ),
                # Layer 2: Multi-oscillator (5 oscillators with various waveforms)
                audio_layer(
                    synthesis = {
                        "type": "multi_oscillator",
                        "frequency": 440.0,
                        "oscillators": [
                            {
                                "waveform": "sawtooth",
                                "volume": 1.0,
                                "detune": 0.0,
                                "phase": 0.0,
                                "duty": 0.5
                            },
                            {
                                "waveform": "square",
                                "volume": 0.8,
                                "detune": 7.0,
                                "phase": 0.25,
                                "duty": 0.4
                            },
                            {
                                "waveform": "sawtooth",
                                "volume": 0.8,
                                "detune": -7.0,
                                "phase": 0.5,
                                "duty": 0.5
                            },
                            {
                                "waveform": "sine",
                                "volume": 0.5,
                                "detune": -1200.0,
                                "phase": 0.0
                            },
                            {
                                "waveform": "triangle",
                                "volume": 0.6,
                                "detune": 3.0,
                                "phase": 0.75
                            }
                        ]
                    },
                    envelope = envelope(0.01, 0.15, 0.6, 0.4),
                    volume = 0.5,
                    pan = -0.2
                ),
                # Layer 3: Additive synthesis
                audio_layer(
                    synthesis = additive(440.0, [1.0, 0.5, 0.33, 0.25, 0.2, 0.167, 0.143, 0.125]),
                    envelope = envelope(0.02, 0.3, 0.5, 0.4),
                    volume = 0.4,
                    pan = 0.2
                ),
                # Layer 4: Karplus-Strong (using raw dict due to stdlib damping/decay field mismatch)
                audio_layer(
                    synthesis = {"type": "karplus_strong", "frequency": 220.0, "decay": 0.995, "blend": 0.8},
                    envelope = envelope(0.001, 0.5, 0.3, 0.6),
                    volume = 0.5,
                    pan = 0.0
                )
            ],
            "pitch_envelope": {
                "attack": 0.01,
                "decay": 0.1,
                "sustain": 0.0,
                "release": 0.05,
                "depth": 12.0
            },
            "master_filter": {
                "type": "lowpass",
                "cutoff": 4000.0,
                "resonance": 2.0,
                "cutoff_end": 800.0
            }
        }
    }
}
