# Adversarial: Music with 255 channels (exceeds XM max of 32)
# Expected: validation rejects channels > 32 for XM format

lead_inst = tracker_instrument(
    name = "lead",
    synthesis = instrument_synthesis("sine"),
    envelope = envelope(0.01, 0.05, 0.5, 0.1)
)

intro = tracker_pattern(64, notes = {
    "0": [pattern_note(0, "C4", 0, vol = 64)]
})

spec(
    asset_id = "adv-max-channels-music",
    asset_type = "music",
    license = "CC0-1.0",
    seed = 99902,
    outputs = [output("music/max_channels.xm", "xm")],
    recipe = {
        "kind": "music.tracker_song_v1",
        "params": {
            "format": "xm",
            "bpm": 120,
            "speed": 6,
            "channels": 255,
            "instruments": [lead_inst],
            "patterns": {"intro": intro},
            "arrangement": [arrangement_entry("intro", 1)]
        }
    }
)
