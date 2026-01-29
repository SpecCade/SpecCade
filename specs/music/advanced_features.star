# Advanced features demo - showcases automation, IT options, effects, and all keywords

lead_synth = tracker_instrument(
    name = "lead_synth",
    synthesis = {"type": "sawtooth"},
    envelope = envelope(0.01, 0.1, 0.7, 0.2),
    default_volume = 56
)

bass_pulse = tracker_instrument(
    name = "bass_pulse",
    synthesis = {"type": "pulse", "duty_cycle": 0.25},
    envelope = envelope(0.005, 0.1, 0.6, 0.15),
    default_volume = 48
)

percussion = tracker_instrument(
    name = "percussion",
    synthesis = {"type": "noise", "periodic": False},
    envelope = envelope(0.001, 0.03, 0.0, 0.02)
)

intro_fade = tracker_pattern(
    name = "intro_fade",
    rows = 64,
    data = [
        pattern_note(row = 0, channel = 0, note = "C4", instrument = 0, volume = 32),
        pattern_note(row = 8, channel = 0, note = "E4", instrument = 0, volume = 40),
        pattern_note(row = 16, channel = 0, note = "G4", instrument = 0, volume = 48),
        pattern_note(row = 24, channel = 0, note = "C5", instrument = 0, volume = 56),
        pattern_note(row = 32, channel = 0, note = "E5", instrument = 0, volume = 64),
        pattern_note(row = 0, channel = 1, note = "C2", instrument = 1, volume = 48),
        pattern_note(row = 32, channel = 1, note = "G2", instrument = 1, volume = 48)
    ]
)

main_with_effects = tracker_pattern(
    name = "main_with_effects",
    rows = 64,
    data = [
        pattern_note(row = 0, channel = 0, note = "C4", instrument = 0, volume = 64, effect_name = "vibrato", param = 72),
        pattern_note(row = 16, channel = 0, note = "D4", instrument = 0, volume = 64, effect_name = "vibrato", param = 72),
        pattern_note(row = 32, channel = 0, note = "E4", instrument = 0, volume = 64, effect_name = "volume_slide", param = 15),
        pattern_note(row = 48, channel = 0, note = "G4", instrument = 0, volume = 64),
        pattern_note(row = 0, channel = 1, note = "C2", instrument = 1, volume = 56),
        pattern_note(row = 16, channel = 1, note = "D2", instrument = 1, volume = 56),
        pattern_note(row = 32, channel = 1, note = "E2", instrument = 1, volume = 56),
        pattern_note(row = 48, channel = 1, note = "G2", instrument = 1, volume = 56),
        pattern_note(row = 0, channel = 2, note = "C4", instrument = 2, volume = 32),
        pattern_note(row = 8, channel = 2, note = "C4", instrument = 2, volume = 28),
        pattern_note(row = 16, channel = 2, note = "C4", instrument = 2, volume = 32),
        pattern_note(row = 24, channel = 2, note = "C4", instrument = 2, volume = 28),
        pattern_note(row = 32, channel = 2, note = "C4", instrument = 2, volume = 32),
        pattern_note(row = 40, channel = 2, note = "C4", instrument = 2, volume = 28),
        pattern_note(row = 48, channel = 2, note = "C4", instrument = 2, volume = 32),
        pattern_note(row = 56, channel = 2, note = "C4", instrument = 2, volume = 28)
    ]
)

tempo_section = tracker_pattern(
    name = "tempo_section",
    rows = 32,
    data = [
        pattern_note(row = 0, channel = 0, note = "G4", instrument = 0, volume = 64),
        pattern_note(row = 8, channel = 0, note = "A4", instrument = 0, volume = 64),
        pattern_note(row = 16, channel = 0, note = "B4", instrument = 0, volume = 64),
        pattern_note(row = 24, channel = 0, note = "C5", instrument = 0, volume = 64),
        pattern_note(row = 0, channel = 1, note = "G2", instrument = 1, volume = 56)
    ]
)

outro_fade = tracker_pattern(
    name = "outro_fade",
    rows = 64,
    data = [
        pattern_note(row = 0, channel = 0, note = "C5", instrument = 0, volume = 64),
        pattern_note(row = 16, channel = 0, note = "G4", instrument = 0, volume = 48),
        pattern_note(row = 32, channel = 0, note = "E4", instrument = 0, volume = 32),
        pattern_note(row = 48, channel = 0, note = "C4", instrument = 0, volume = 16),
        pattern_note(row = 0, channel = 1, note = "C2", instrument = 1, volume = 48),
        pattern_note(row = 32, channel = 1, note = "OFF", instrument = 1, volume = 0)
    ]
)

music_spec(
    asset_id = "advanced_features",
    seed = 3020,
    output_path = "advanced_features.it",
    format = "it",
    bpm = 120,
    speed = 6,
    channels = 8,
    loop = False,
    description = "Advanced features demo - showcases automation, IT options, effects, and all keywords",
    license = "CC0-1.0",
    style_tags = ["advanced", "demo", "it-format"],
    instruments = [lead_synth, bass_pulse, percussion],
    patterns = {
        "intro_fade": intro_fade,
        "main_with_effects": main_with_effects,
        "tempo_section": tempo_section,
        "outro_fade": outro_fade
    },
    arrangement = [
        arrangement_entry("intro_fade", 1),
        arrangement_entry("main_with_effects", 2),
        arrangement_entry("tempo_section", 1),
        arrangement_entry("outro_fade", 1)
    ],
    automation = [
        {"type": "volume_fade", "pattern": "intro_fade", "channel": 0, "start_row": 0, "end_row": 32, "start_vol": 0, "end_vol": 64},
        {"type": "tempo_change", "pattern": "tempo_section", "row": 0, "bpm": 140},
        {"type": "volume_fade", "pattern": "outro_fade", "channel": 0, "start_row": 0, "end_row": 63, "start_vol": 64, "end_vol": 0}
    ],
    it_options = {"stereo": True, "global_volume": 100, "mix_volume": 64}
)
