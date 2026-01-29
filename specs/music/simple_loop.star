# Simple 4-channel loop - basic XM module

lead_inst = tracker_instrument(
    name = "lead",
    synthesis = instrument_synthesis("triangle"),
    envelope = envelope(0.001, 0.1, 0.5, 0.2)
)

bass_inst = tracker_instrument(
    name = "bass",
    synthesis = instrument_synthesis("sawtooth"),
    envelope = envelope(0.01, 0.1, 0.6, 0.15)
)

intro_pattern = tracker_pattern(64, notes = {
    "0": [
        pattern_note(0, "C4", 0, vol = 64),
        pattern_note(16, "E4", 0, vol = 64),
        pattern_note(32, "G4", 0, vol = 64),
        pattern_note(48, "C5", 0, vol = 64)
    ],
    "1": [
        pattern_note(0, "C2", 1, vol = 48),
        pattern_note(32, "G2", 1, vol = 48)
    ]
})

music_spec(
    asset_id = "simple_loop",
    seed = 3001,
    output_path = "simple_loop.xm",
    format = "xm",
    bpm = 120,
    speed = 6,
    channels = 4,
    instruments = [lead_inst, bass_inst],
    patterns = {"intro": intro_pattern},
    arrangement = [arrangement_entry("intro", 4)],
    description = "Simple 4-channel loop - basic XM module",
    license = "CC0-1.0"
)
