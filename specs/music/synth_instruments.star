# Music spec covering all synth_type enum values for coverage tracking

pulse_synth = tracker_instrument(
    name = "pulse_synth",
    synth_type = "pulse",
    synthesis = {"type": "pulse", "duty_cycle": 0.25},
    envelope = envelope(0.01, 0.1, 0.7, 0.2)
)

square_synth = tracker_instrument(
    name = "square_synth",
    synth_type = "square",
    synthesis = {"type": "square"},
    envelope = envelope(0.01, 0.1, 0.7, 0.2)
)

triangle_synth = tracker_instrument(
    name = "triangle_synth",
    synth_type = "triangle",
    synthesis = {"type": "triangle"},
    envelope = envelope(0.01, 0.1, 0.7, 0.2)
)

sawtooth_synth = tracker_instrument(
    name = "sawtooth_synth",
    synth_type = "sawtooth",
    synthesis = {"type": "sawtooth"},
    envelope = envelope(0.01, 0.1, 0.7, 0.2)
)

sine_synth = tracker_instrument(
    name = "sine_synth",
    synth_type = "sine",
    synthesis = {"type": "sine"},
    envelope = envelope(0.01, 0.1, 0.7, 0.2)
)

noise_synth = tracker_instrument(
    name = "noise_synth",
    synth_type = "noise",
    synthesis = {"type": "noise", "periodic": False},
    envelope = envelope(0.01, 0.05, 0.3, 0.1)
)

test_pattern = tracker_pattern(
    name = "test",
    rows = 64,
    data = [
        pattern_note(row = 0, channel = 0, note = "C4", instrument = 0, volume = 64),
        pattern_note(row = 8, channel = 1, note = "D4", instrument = 1, volume = 64),
        pattern_note(row = 16, channel = 2, note = "E4", instrument = 2, volume = 64),
        pattern_note(row = 24, channel = 3, note = "F4", instrument = 3, volume = 64),
        pattern_note(row = 32, channel = 4, note = "G4", instrument = 4, volume = 64),
        pattern_note(row = 40, channel = 5, note = "C4", instrument = 5, volume = 64)
    ]
)

music_spec(
    asset_id = "synth-instruments",
    seed = 42,
    output_path = "synth_instruments.xm",
    format = "xm",
    bpm = 120,
    speed = 6,
    channels = 6,
    loop = False,
    description = "Music spec covering all synth_type enum values for coverage tracking",
    license = "CC0-1.0",
    synth_type_coverage = ["pulse", "square", "triangle", "sawtooth", "sine", "noise"],
    instruments = [pulse_synth, square_synth, triangle_synth, sawtooth_synth, sine_synth, noise_synth],
    patterns = {"test": test_pattern},
    arrangement = [arrangement_entry("test", 1)]
)
