# VFX flipbook explosion effect with 16 frames
#
# Recipe: vfx.flipbook_v1 - generates an animated flipbook texture for explosion effects

spec(
    asset_id = "explosion-flipbook-01",
    asset_type = "vfx",
    license = "CC0-1.0",
    seed = 12345,
    description = "VFX flipbook explosion effect with 16 frames",
    outputs = [
        output("vfx/explosion-flipbook-01.png", "png"),
        output("vfx/explosion-flipbook-01.metadata.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "vfx.flipbook_v1",
        "params": {
            "resolution": [512, 512],
            "padding": 2,
            "effect": "explosion",
            "frame_count": 16,
            "frame_size": [64, 64],
            "fps": 24,
            "loop_mode": "once"
        }
    }
)
