# Golden comprehensive test - exercises audio_v1 synthesis types: oscillator, fm_synth, karplus_strong,
# noise_burst, pitched_body, metallic, additive, and multi_oscillator

spec(
    asset_id = "audio-sfx-comprehensive-golden",
    asset_type = "audio",
    license = "CC0-1.0",
    seed = 999001,
    description = "Golden comprehensive test - exercises audio_v1 synthesis types: oscillator, fm_synth, karplus_strong, noise_burst, pitched_body, metallic, additive, and multi_oscillator",
    outputs = [output("audio_sfx_comprehensive.wav", "wav")],
    variants = [
        variant("loud", 100),
        variant("quiet", 200)
    ],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "base_note": "C4",
            "duration_seconds": 2.0,
            "sample_rate": 44100,
            "layers": [
                # Oscillator: sine with freq sweep
                audio_layer(
                    synthesis = {
                        "type": "oscillator",
                        "waveform": "sine",
                        "frequency": 440.0,
                        "freq_sweep": {
                            "end_freq": 220.0,
                            "curve": "exponential"
                        }
                    },
                    envelope = envelope(0.01, 0.1, 0.7, 0.2),
                    volume = 0.5,
                    pan = 0.0,
                    delay = 0.0
                ),
                # Oscillator: square with duty
                audio_layer(
                    synthesis = {
                        "type": "oscillator",
                        "waveform": "square",
                        "frequency": 330.0,
                        "duty": 0.25
                    },
                    envelope = envelope(0.005, 0.2, 0.5, 0.3),
                    volume = 0.4,
                    pan = -0.3,
                    delay = 0.05
                ),
                # Oscillator: sawtooth with linear sweep
                audio_layer(
                    synthesis = {
                        "type": "oscillator",
                        "waveform": "sawtooth",
                        "frequency": 220.0,
                        "freq_sweep": {
                            "end_freq": 110.0,
                            "curve": "linear"
                        }
                    },
                    envelope = envelope(0.02, 0.15, 0.6, 0.25),
                    volume = 0.35,
                    pan = 0.3,
                    delay = 0.1
                ),
                # Oscillator: triangle
                audio_layer(
                    synthesis = oscillator(550.0, "triangle"),
                    envelope = envelope(0.001, 0.3, 0.4, 0.4),
                    volume = 0.3,
                    pan = 0.0,
                    delay = 0.0
                ),
                # FM synthesis
                audio_layer(
                    synthesis = fm_synth(440.0, 880.0, 5.0),
                    envelope = envelope(0.001, 0.4, 0.2, 0.6),
                    volume = 0.45,
                    pan = 0.1,
                    delay = 0.0
                ),
                # Karplus-Strong
                audio_layer(
                    synthesis = karplus_strong(196.0, 0.996, 0.7),
                    envelope = envelope(0.001, 0.5, 0.3, 0.8),
                    volume = 0.5,
                    pan = -0.2,
                    delay = 0.2
                ),
                # Noise burst: white with bandpass
                audio_layer(
                    synthesis = {
                        "type": "noise_burst",
                        "noise_type": "white",
                        "filter": {
                            "type": "bandpass",
                            "center": 2000.0,
                            "resonance": 1.0
                        }
                    },
                    envelope = envelope(0.001, 0.05, 0.0, 0.1),
                    volume = 0.25,
                    pan = 0.0,
                    delay = 0.0
                ),
                # Noise burst: pink with lowpass
                audio_layer(
                    synthesis = {
                        "type": "noise_burst",
                        "noise_type": "pink",
                        "filter": {
                            "type": "lowpass",
                            "cutoff": 3000.0,
                            "resonance": 0.5
                        }
                    },
                    envelope = envelope(0.01, 0.2, 0.1, 0.3),
                    volume = 0.2,
                    pan = 0.4,
                    delay = 0.3
                ),
                # Noise burst: brown with lowpass sweep
                audio_layer(
                    synthesis = {
                        "type": "noise_burst",
                        "noise_type": "brown",
                        "filter": {
                            "type": "lowpass",
                            "cutoff": 800.0,
                            "resonance": 1.2,
                            "cutoff_end": 200.0
                        }
                    },
                    envelope = envelope(0.005, 0.3, 0.2, 0.7),
                    volume = 0.6,
                    pan = 0.0,
                    delay = 0.0
                ),
                # Pitched body
                audio_layer(
                    synthesis = pitched_body(200.0, 50.0),
                    envelope = envelope(0.01, 0.4, 0.1, 0.5),
                    volume = 0.7,
                    pan = 0.0,
                    delay = 0.01
                ),
                # Metallic
                audio_layer(
                    synthesis = metallic_synth(800.0, 6, 1.414),
                    envelope = envelope(0.001, 0.6, 0.1, 1.0),
                    volume = 0.35,
                    pan = -0.1,
                    delay = 0.15
                ),
                # Additive
                audio_layer(
                    synthesis = additive_synth(220.0, [1.0, 0.5, 0.33, 0.25, 0.2]),
                    envelope = envelope(0.02, 0.3, 0.5, 0.4),
                    volume = 0.4,
                    pan = 0.2,
                    delay = 0.0
                ),
                # Oscillator: sine (late onset)
                audio_layer(
                    synthesis = oscillator(1000.0, "sine"),
                    envelope = envelope(0.001, 0.1, 0.5, 0.2),
                    volume = 0.2,
                    pan = 0.0,
                    delay = 1.0
                )
            ],
            "master_filter": {
                "type": "lowpass",
                "cutoff": 16000.0,
                "resonance": 0.707
            }
        }
    }
)
