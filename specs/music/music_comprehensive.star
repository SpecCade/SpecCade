# Comprehensive Tier-1 tracker song fixture
# Exercises multiple instruments, patterns, automation, and IT options.

lead = tracker_instrument(
    name = "lead",
    synthesis = instrument_synthesis("pulse", duty_cycle = 0.5),
    envelope = envelope(0.005, 0.08, 0.6, 0.15),
    default_volume = 56,
)

bass = tracker_instrument(
    name = "bass",
    synthesis = instrument_synthesis("sawtooth"),
    envelope = envelope(0.003, 0.12, 0.5, 0.12),
    default_volume = 48,
)

drums = tracker_instrument(
    name = "drums",
    synthesis = instrument_synthesis("noise"),
    envelope = envelope(0.001, 0.03, 0.0, 0.02),
    default_volume = 48,
)

intro = tracker_pattern(64, notes = {
    "0": [
        pattern_note(0, "C4", 0, vol = 48),
        pattern_note(8, "E4", 0, vol = 48),
        pattern_note(16, "G4", 0, vol = 52),
        pattern_note(24, "C5", 0, vol = 56),
        pattern_note(32, "G4", 0, vol = 52),
        pattern_note(40, "E4", 0, vol = 48),
        pattern_note(48, "D4", 0, vol = 48),
        pattern_note(56, "C4", 0, vol = 48),
    ],
    "1": [
        pattern_note(0, "C2", 1, vol = 48),
        pattern_note(16, "C2", 1, vol = 48),
        pattern_note(32, "G1", 1, vol = 48),
        pattern_note(48, "C2", 1, vol = 48),
    ],
    "2": [
        pattern_note(0, "C5", 2, vol = 40),
        pattern_note(8, "C5", 2, vol = 32),
        pattern_note(16, "C5", 2, vol = 40),
        pattern_note(24, "C5", 2, vol = 32),
        pattern_note(32, "C5", 2, vol = 40),
        pattern_note(40, "C5", 2, vol = 32),
        pattern_note(48, "C5", 2, vol = 40),
        pattern_note(56, "C5", 2, vol = 32),
    ],
})

main = tracker_pattern(64, notes = {
    "0": [
        pattern_note(0, "A4", 0, vol = 56),
        pattern_note(16, "C5", 0, vol = 56),
        pattern_note(32, "E5", 0, vol = 56),
        pattern_note(48, "A5", 0, vol = 56),
    ],
    "1": [
        pattern_note(0, "A2", 1, vol = 48),
        pattern_note(16, "C3", 1, vol = 48),
        pattern_note(32, "E3", 1, vol = 48),
        pattern_note(48, "A2", 1, vol = 48),
    ],
    "2": [
        pattern_note(0, "C5", 2, vol = 36),
        pattern_note(16, "C5", 2, vol = 36),
        pattern_note(32, "C5", 2, vol = 36),
        pattern_note(48, "C5", 2, vol = 36),
    ],
})

breakdown = tracker_pattern(32, notes = {
    "0": [
        pattern_note(0, "F4", 0, vol = 40),
        pattern_note(8, "A4", 0, vol = 40),
        pattern_note(16, "C5", 0, vol = 44),
        pattern_note(24, "A4", 0, vol = 40),
    ],
    "1": [
        pattern_note(0, "F2", 1, vol = 44),
        pattern_note(16, "C2", 1, vol = 44),
    ],
})

song = tracker_song(
    format = "it",
    bpm = 120,
    speed = 6,
    channels = 6,
    loop = True,
    instruments = [lead, bass, drums],
    patterns = {
        "intro": intro,
        "main": main,
        "breakdown": breakdown,
    },
    arrangement = [
        arrangement_entry("intro", 1),
        arrangement_entry("main", 2),
        arrangement_entry("breakdown", 1),
        arrangement_entry("main", 1),
    ],
    automation = [
        volume_fade("main", 0, 0, 48, 56, 32),
        tempo_change("main", 32, 140),
    ],
    it_options = it_options(stereo = True, global_volume = 128, mix_volume = 80),
)

spec(
    asset_id = "golden-music-comprehensive-01",
    asset_type = "music",
    seed = 4244,
    license = "CC0-1.0",
    description = "Comprehensive tracker song fixture (IT) for golden tests",
    outputs = [output("music/music_comprehensive.it", "it")],
    recipe = song,
)
