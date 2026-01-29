# Miscellaneous filters example
#
# Demonstrates various filter types: notch, allpass, comb, formant, highpass, and shelving filters.
# Covers: notch(), allpass(), comb_filter(), formant_filter(), shelf_low(), shelf_high(), highpass()

spec(
    asset_id = "stdlib-audio-filters-misc-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/filters_misc.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 2.5,
            "sample_rate": 44100,
            "layers": [
                # Layer 1: White noise with notch filter
                audio_layer(
                    synthesis = noise_burst("white"),
                    envelope = envelope(0.01, 0.5, 0.0, 0.1),
                    volume = 0.3,
                    filter = notch(1000.0, 2.0)  # center, resonance
                ),
                # Layer 2: Sawtooth with allpass filter
                audio_layer(
                    synthesis = oscillator(220, "sawtooth"),
                    envelope = envelope(0.01, 0.4, 0.5, 0.2),
                    volume = 0.25,
                    filter = allpass(500.0, 1.0)  # frequency, resonance
                ),
                # Layer 3: Sine wave with low shelf filter
                audio_layer(
                    synthesis = oscillator(330, "sine"),
                    envelope = envelope(0.02, 0.3, 0.6, 0.25),
                    volume = 0.25,
                    filter = shelf_low(200.0, 3.0)  # frequency, gain_db
                ),
                # Layer 4: Triangle wave with high shelf filter
                audio_layer(
                    synthesis = oscillator(440, "triangle"),
                    envelope = envelope(0.01, 0.35, 0.55, 0.3),
                    volume = 0.25,
                    filter = shelf_high(4000.0, -2.0)  # frequency, gain_db
                ),
                # Layer 5: Pulse wave with comb filter
                audio_layer(
                    synthesis = oscillator(110, "pulse"),
                    envelope = envelope(0.01, 0.3, 0.5, 0.2),
                    volume = 0.3,
                    filter = comb_filter(5.0, 0.7, 0.5)  # delay_ms, feedback, wet
                ),
                # Layer 6: Sawtooth with formant filter
                audio_layer(
                    synthesis = oscillator(150, "sawtooth"),
                    envelope = envelope(0.01, 0.2, 0.6, 0.3),
                    volume = 0.3,
                    filter = formant_filter("a", 0.8)  # vowel, intensity
                ),
                # Layer 7: Square wave with highpass filter
                audio_layer(
                    synthesis = oscillator(200, "square"),
                    envelope = envelope(0.01, 0.25, 0.5, 0.2),
                    volume = 0.25,
                    filter = highpass(500.0, 0.707)  # cutoff, resonance
                )
            ]
        }
    }
)
