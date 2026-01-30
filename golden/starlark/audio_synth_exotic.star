# Golden test: Exotic synthesis functions
# Tests: vocoder, formant_synth, vector_synth, waveguide, bowed_string, pulsar, vosim, spectral_freeze, pitched_body
#
# This test file demonstrates the advanced/exotic synthesis capabilities available in the audio stdlib.
# Each synthesis method produces unique timbres and sound characteristics.

spec(
    asset_id = "stdlib-audio-exotic-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/exotic.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 9.0,  # 1 second per synthesis type
            "sample_rate": 44100,
            "layers": [
                # Test 1: Vocoder with sawtooth carrier
                audio_layer(
                    synthesis = vocoder(
                        carrier_freq = 220.0,
                        carrier_type = "sawtooth",
                        num_bands = 16,
                        band_spacing = "logarithmic",
                        envelope_attack = 0.01,
                        envelope_release = 0.05,
                        formant_rate = 2.0,
                        bands = [
                            vocoder_band(200.0, 80.0, [0.0, 0.5, 1.0, 0.5, 0.0]),
                            vocoder_band(400.0, 120.0),
                        ]
                    ),
                    envelope = envelope(0.01, 0.1, 0.7, 0.2),
                    volume = 0.6,
                    delay = 0.0
                ),

                # Test 2: Formant synthesis with vowel preset
                audio_layer(
                    synthesis = formant_synth(
                        frequency = 220.0,
                        vowel = "a",
                        breathiness = 0.1
                    ),
                    envelope = envelope(0.02, 0.1, 0.7, 0.2),
                    volume = 0.6,
                    delay = 1.0
                ),

                # Test 3: Formant synthesis with vowel morphing
                audio_layer(
                    synthesis = formant_synth(
                        frequency = 220.0,
                        vowel = "a",
                        vowel_morph = "i",
                        morph_amount = 0.5,
                        breathiness = 0.15
                    ),
                    envelope = envelope(0.02, 0.15, 0.7, 0.2),
                    volume = 0.6,
                    delay = 2.0
                ),

                # Test 4: Vector synthesis with 2D crossfading
                audio_layer(
                    synthesis = vector_synth(
                        frequency = 440.0,
                        sources = [
                            vector_source("sine"),
                            vector_source("saw", 1.0),
                            vector_source("square", 1.0),
                            vector_source("triangle", 1.0),
                        ],
                        position_x = 0.25,
                        position_y = 0.75
                    ),
                    envelope = envelope(0.01, 0.1, 0.7, 0.2),
                    volume = 0.6,
                    delay = 3.0
                ),

                # Test 5: Waveguide synthesis for wind/brass sounds
                audio_layer(
                    synthesis = waveguide(
                        frequency = 440.0,
                        breath = 0.7,
                        noise = 0.3,
                        damping = 0.5,
                        resonance = 0.8
                    ),
                    envelope = envelope(0.05, 0.1, 0.7, 0.2),
                    volume = 0.6,
                    delay = 4.0
                ),

                # Test 6: Bowed string synthesis for violin/cello sounds
                audio_layer(
                    synthesis = bowed_string(
                        frequency = 440.0,
                        bow_pressure = 0.6,
                        bow_position = 0.3,
                        damping = 0.2
                    ),
                    envelope = envelope(0.1, 0.2, 0.7, 0.3),
                    volume = 0.6,
                    delay = 5.0
                ),

                # Test 7: Pulsar synthesis (synchronized grain trains)
                audio_layer(
                    synthesis = pulsar(
                        frequency = 440.0,
                        pulse_rate = 20.0,
                        grain_size_ms = 50.0,
                        shape = "sine"
                    ),
                    envelope = envelope(0.01, 0.1, 0.7, 0.2),
                    volume = 0.5,
                    delay = 6.0
                ),

                # Test 8: VOSIM synthesis (voice simulation)
                audio_layer(
                    synthesis = vosim(
                        frequency = 220.0,
                        formant_freq = 880.0,
                        pulses = 4,
                        breathiness = 0.1
                    ),
                    envelope = envelope(0.01, 0.1, 0.7, 0.2),
                    volume = 0.6,
                    delay = 7.0
                ),

                # Test 9: Spectral freeze with noise source
                audio_layer(
                    synthesis = spectral_freeze(
                        source = spectral_source("noise", "pink")
                    ),
                    envelope = envelope(0.01, 0.1, 0.8, 0.2),
                    volume = 0.5,
                    delay = 8.0
                ),

                # Test 10: Pitched body (impact sounds with frequency sweep)
                audio_layer(
                    synthesis = pitched_body(
                        start_freq = 880.0,
                        end_freq = 110.0
                    ),
                    envelope = envelope(0.001, 0.2, 0.3, 0.5),
                    volume = 0.7,
                    delay = 8.5
                )
            ]
        }
    }
)
