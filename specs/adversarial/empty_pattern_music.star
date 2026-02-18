# Adversarial: Music with 0-row pattern
# Expected: validation rejects patterns with 0 rows

lead_inst = tracker_instrument(
    name = "lead",
    synthesis = instrument_synthesis("sine"),
    envelope = envelope(0.01, 0.05, 0.5, 0.1)
)

# Pattern with 0 rows is invalid
empty_pat = tracker_pattern(0, notes = {})

spec(
    asset_id = "adv-empty-pattern-music",
    asset_type = "music",
    license = "CC0-1.0",
    seed = 99903,
    outputs = [output("music/empty_pattern.xm", "xm")],
    recipe = {
        "kind": "music.tracker_song_v1",
        "params": {
            "format": "xm",
            "bpm": 120,
            "speed": 6,
            "channels": 4,
            "instruments": [lead_inst],
            "patterns": {"empty": empty_pat},
            "arrangement": [arrangement_entry("empty", 1)]
        }
    }
)
