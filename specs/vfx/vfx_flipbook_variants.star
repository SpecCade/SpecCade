# VFX flipbook variants golden spec
#
# Exercises vfx.flipbook_v1 with different effect types, frame counts,
# resolutions, and loop modes.

# Smoke effect: large frames, many frames, ping-pong loop
spec(
    asset_id = "golden-vfx-flipbook-smoke-01",
    asset_type = "vfx",
    seed = 5001,
    description = "Smoke flipbook: 24 frames at 128x128 with ping-pong loop",
    tags = ["golden", "vfx", "flipbook", "smoke"],
    outputs = [
        output("vfx/flipbook_smoke.png", "png"),
        output("vfx/flipbook_smoke.metadata.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "vfx.flipbook_v1",
        "params": {
            "resolution": [1024, 512],
            "padding": 4,
            "effect": "smoke",
            "frame_count": 24,
            "frame_size": [128, 128],
            "fps": 30,
            "loop_mode": "ping_pong"
        }
    }
)

# Energy effect: small frames, looping
spec(
    asset_id = "golden-vfx-flipbook-energy-01",
    asset_type = "vfx",
    seed = 5002,
    description = "Energy flipbook: 8 frames at 64x64 with loop mode",
    tags = ["golden", "vfx", "flipbook", "energy"],
    outputs = [
        output("vfx/flipbook_energy.png", "png"),
        output("vfx/flipbook_energy.metadata.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "vfx.flipbook_v1",
        "params": {
            "resolution": [256, 256],
            "padding": 2,
            "effect": "energy",
            "frame_count": 8,
            "frame_size": [64, 64],
            "fps": 12,
            "loop_mode": "loop"
        }
    }
)

# Dissolve effect: non-square frames, once playback
spec(
    asset_id = "golden-vfx-flipbook-dissolve-01",
    asset_type = "vfx",
    seed = 5003,
    description = "Dissolve flipbook: 12 frames at 96x64, once playback",
    tags = ["golden", "vfx", "flipbook", "dissolve"],
    outputs = [
        output("vfx/flipbook_dissolve.png", "png"),
        output("vfx/flipbook_dissolve.metadata.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "vfx.flipbook_v1",
        "params": {
            "resolution": [512, 256],
            "padding": 2,
            "effect": "dissolve",
            "frame_count": 12,
            "frame_size": [96, 64],
            "fps": 18,
            "loop_mode": "once"
        }
    }
)
