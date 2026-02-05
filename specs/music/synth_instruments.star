# Music spec covering all synth_type enum values for coverage tracking

pulse_synth = tracker_instrument(
    name = "pulse_synth",
    synthesis = instrument_synthesis("pulse", duty_cycle = 0.25),
    envelope = envelope(0.01, 0.1, 0.7, 0.2)
)

square_synth = tracker_instrument(
    name = "square_synth",
    synthesis = instrument_synthesis("pulse", duty_cycle = 0.5),
    envelope = envelope(0.01, 0.1, 0.7, 0.2)
)

triangle_synth = tracker_instrument(
    name = "triangle_synth",
    synthesis = instrument_synthesis("triangle"),
    envelope = envelope(0.01, 0.1, 0.7, 0.2)
)

sawtooth_synth = tracker_instrument(
    name = "sawtooth_synth",
    synthesis = instrument_synthesis("sawtooth"),
    envelope = envelope(0.01, 0.1, 0.7, 0.2)
)

sine_synth = tracker_instrument(
    name = "sine_synth",
    synthesis = instrument_synthesis("sine"),
    envelope = envelope(0.01, 0.1, 0.7, 0.2)
)

noise_synth = tracker_instrument(
    name = "noise_synth",
    synthesis = instrument_synthesis("noise"),
    envelope = envelope(0.01, 0.05, 0.3, 0.1)
)

test_pattern = tracker_pattern(64, notes = {
    "0": [
        pattern_note(0, "C4", 0, vol = 64),
        pattern_note(8, "D4", 0, vol = 64)
    ],
    "1": [
        pattern_note(16, "E4", 1, vol = 64),
        pattern_note(24, "F4", 1, vol = 64)
    ],
    "2": [
        pattern_note(32, "G4", 2, vol = 64),
        pattern_note(40, "A4", 2, vol = 64)
    ],
    "3": [
        pattern_note(48, "B4", 3, vol = 64),
        pattern_note(56, "C5", 3, vol = 64)
    ]
})

spec(
    asset_id = "synth-instruments",
    asset_type = "music",
    seed = 42,
    license = "CC0-1.0",
    description = "Music spec covering all synth_type enum values for coverage tracking",
    outputs = [output("synth_instruments.xm", "xm")],
    recipe = {
        "kind": "music.tracker_v1",
        "params": {
            "bpm": 120,
            "speed": 6,
            "channels": 6,
            "instruments": [pulse_synth, square_synth, triangle_synth, sawtooth_synth, sine_synth, noise_synth],
            "patterns": {"test": test_pattern},
            "arrangement": [arrangement_entry("test", 1)]
        }
    }
)
