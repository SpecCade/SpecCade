# Golden test: Advanced exotic synthesis variations
# Tests advanced features: vector path animation, spectral_freeze with tone, vocoder with pulse/noise
#
# This test file demonstrates advanced variations of exotic synthesis functions.

spec(
    asset_id = "stdlib-audio-exotic-advanced-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/exotic_advanced.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 5.0,
            "sample_rate": 44100,
            "layers": [
                # Test 1: Vocoder with pulse carrier
                audio_layer(
                    synthesis = vocoder(
                        carrier_freq = 110.0,
                        carrier_type = "pulse",
                        num_bands = 8,
                        band_spacing = "linear",
                        envelope_attack = 0.02,
                        envelope_release = 0.1,
                        formant_rate = 3.0
                    ),
                    envelope = envelope(0.01, 0.1, 0.7, 0.2),
                    volume = 0.6,
                    delay = 0.0
                ),

                # Test 2: Vocoder with noise carrier
                audio_layer(
                    synthesis = vocoder(
                        carrier_freq = 220.0,
                        carrier_type = "noise",
                        num_bands = 32,
                        band_spacing = "logarithmic",
                        envelope_attack = 0.005,
                        envelope_release = 0.03,
                        formant_rate = 1.5
                    ),
                    envelope = envelope(0.01, 0.1, 0.7, 0.2),
                    volume = 0.5,
                    delay = 1.0
                ),

                # Test 3: Vector synthesis with animated path
                audio_layer(
                    synthesis = vector_synth(
                        frequency = 330.0,
                        sources = [
                            vector_source("sine", 1.0),
                            vector_source("saw", 2.0),
                            vector_source("square", 0.5),
                            vector_source("triangle", 1.5),
                        ],
                        position_x = 0.0,
                        position_y = 0.0,
                        path = [
                            vector_path_point(0.0, 0.0, 0.25),
                            vector_path_point(1.0, 0.0, 0.25),
                            vector_path_point(1.0, 1.0, 0.25),
                            vector_path_point(0.0, 1.0, 0.25),
                        ],
                        path_loop = True,
                        path_curve = "exponential"
                    ),
                    envelope = envelope(0.01, 0.1, 0.8, 0.2),
                    volume = 0.6,
                    delay = 2.0
                ),

                # Test 4: Spectral freeze with tone source (sawtooth)
                audio_layer(
                    synthesis = spectral_freeze(
                        source = spectral_source("tone", "sawtooth", 440.0)
                    ),
                    envelope = envelope(0.01, 0.15, 0.8, 0.3),
                    volume = 0.5,
                    delay = 3.0
                ),

                # Test 5: Formant synthesis with custom formant configs
                audio_layer(
                    synthesis = formant_synth(
                        frequency = 165.0,
                        formants = [
                            formant_config(800.0, 1.0, 100.0),
                            formant_config(1200.0, 0.8, 120.0),
                            formant_config(2400.0, 0.5, 150.0),
                        ],
                        breathiness = 0.2
                    ),
                    envelope = envelope(0.02, 0.1, 0.7, 0.2),
                    volume = 0.6,
                    delay = 4.0
                ),
            ]
        }
    }
)
