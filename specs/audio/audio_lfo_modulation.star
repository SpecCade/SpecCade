# LFO modulation example
#
# LFO (Low Frequency Oscillator) adds movement through slow modulation.
# Can target pitch (vibrato), volume (tremolo), filter cutoff, and more.
# Covers: lfo(), lfo_modulation()

spec(
    asset_id = "stdlib-audio-lfo-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/lfo.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 3.0,
            "sample_rate": 44100,
            "layers": [
                # Vibrato effect: pitch modulation
                audio_layer(
                    synthesis = oscillator(440, "sine"),
                    envelope = envelope(0.1, 0.2, 0.8, 0.5),
                    volume = 0.6,
                    lfo = lfo_modulation(
                        lfo("sine", 5.0, 0.3),
                        "pitch",
                        1.5  # semitones
                    )
                ),
                # Tremolo effect: volume modulation
                audio_layer(
                    synthesis = oscillator(880, "triangle"),
                    envelope = envelope(0.1, 0.2, 0.8, 0.5),
                    volume = 0.4,
                    lfo = lfo_modulation(
                        lfo("triangle", 6.0, 0.5),
                        "volume",
                        0.4  # modulation amount
                    )
                )
            ]
        }
    }
)
