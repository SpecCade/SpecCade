# Sprite animation basic - demonstrates animation clip definition
#
# This example defines a simple idle animation referencing 4 frames
# with explicit per-frame durations and loop playback.
#
# Recipe: sprite.animation_v1 (Tier 1, metadata-only JSON output)

spec(
    asset_id = "sprite-anim-basic",
    asset_type = "sprite",
    seed = 42,
    outputs = [
        output("anims/idle.json", "json")
    ],
    recipe = {
        "kind": "sprite.animation_v1",
        "params": {
            "name": "idle",
            "fps": 12,
            "loop_mode": "loop",
            "frames": [
                {"frame_id": "frame_0", "duration_ms": 100},
                {"frame_id": "frame_1", "duration_ms": 100},
                {"frame_id": "frame_2", "duration_ms": 100},
                {"frame_id": "frame_3", "duration_ms": 200}
            ]
        }
    }
)
