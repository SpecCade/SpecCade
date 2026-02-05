# Advanced features demo - showcases IT format with multiple patterns

lead_synth = tracker_instrument(
    name = "lead_synth",
    synthesis = instrument_synthesis("sawtooth"),
    envelope = envelope(0.01, 0.1, 0.7, 0.2),
    default_volume = 56
)

bass_pulse = tracker_instrument(
    name = "bass_pulse",
    synthesis = instrument_synthesis("pulse", duty_cycle = 0.25),
    envelope = envelope(0.005, 0.1, 0.6, 0.15),
    default_volume = 48
)

percussion = tracker_instrument(
    name = "percussion",
    synthesis = instrument_synthesis("noise"),
    envelope = envelope(0.001, 0.03, 0.0, 0.02)
)

intro_fade = tracker_pattern(64, notes = {
    "0": [
        pattern_note(0, "C4", 0, vol = 32),
        pattern_note(8, "E4", 0, vol = 40),
        pattern_note(16, "G4", 0, vol = 48),
        pattern_note(24, "C5", 0, vol = 56),
        pattern_note(32, "E5", 0, vol = 64)
    ],
    "1": [
        pattern_note(0, "C2", 1, vol = 48),
        pattern_note(32, "G2", 1, vol = 48)
    ]
})

main_section = tracker_pattern(64, notes = {
    "0": [
        pattern_note(0, "C4", 0, vol = 64),
        pattern_note(16, "D4", 0, vol = 64),
        pattern_note(32, "E4", 0, vol = 64),
        pattern_note(48, "G4", 0, vol = 64)
    ],
    "1": [
        pattern_note(0, "C2", 1, vol = 56),
        pattern_note(16, "D2", 1, vol = 56),
        pattern_note(32, "E2", 1, vol = 56),
        pattern_note(48, "G2", 1, vol = 56)
    ],
    "2": [
        pattern_note(0, "C4", 2, vol = 32),
        pattern_note(8, "C4", 2, vol = 28),
        pattern_note(16, "C4", 2, vol = 32),
        pattern_note(24, "C4", 2, vol = 28),
        pattern_note(32, "C4", 2, vol = 32),
        pattern_note(40, "C4", 2, vol = 28),
        pattern_note(48, "C4", 2, vol = 32),
        pattern_note(56, "C4", 2, vol = 28)
    ]
})

tempo_section = tracker_pattern(32, notes = {
    "0": [
        pattern_note(0, "G4", 0, vol = 64),
        pattern_note(8, "A4", 0, vol = 64),
        pattern_note(16, "B4", 0, vol = 64),
        pattern_note(24, "C5", 0, vol = 64)
    ],
    "1": [
        pattern_note(0, "G2", 1, vol = 56)
    ]
})

outro_fade = tracker_pattern(64, notes = {
    "0": [
        pattern_note(0, "C5", 0, vol = 64),
        pattern_note(16, "G4", 0, vol = 48),
        pattern_note(32, "E4", 0, vol = 32),
        pattern_note(48, "C4", 0, vol = 16)
    ],
    "1": [
        pattern_note(0, "C2", 1, vol = 48),
        pattern_note(32, "OFF", 1, vol = 0)
    ]
})

spec(
    asset_id = "advanced_features",
    asset_type = "music",
    seed = 3020,
    license = "CC0-1.0",
    description = "Advanced features demo - showcases IT format with multiple patterns",
    outputs = [output("advanced_features.it", "it")],
    recipe = {
        "kind": "music.tracker_v1",
        "params": {
            "bpm": 120,
            "speed": 6,
            "channels": 8,
            "instruments": [lead_synth, bass_pulse, percussion],
            "patterns": {
                "intro_fade": intro_fade,
                "main_section": main_section,
                "tempo_section": tempo_section,
                "outro_fade": outro_fade
            },
            "arrangement": [
                arrangement_entry("intro_fade", 1),
                arrangement_entry("main_section", 2),
                arrangement_entry("tempo_section", 1),
                arrangement_entry("outro_fade", 1)
            ]
        }
    }
)
