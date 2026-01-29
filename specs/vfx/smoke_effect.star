# VFX flipbook smoke effect with 12 frames
#
# Recipe: vfx.flipbook_v1 - generates an animated flipbook texture for smoke effects

spec(
    asset_id = "smoke-effect-01",
    asset_type = "vfx",
    license = "CC0-1.0",
    seed = 67890,
    description = "VFX flipbook smoke effect with 12 frames",
    outputs = [
        output("vfx/smoke-effect-01.png", "png"),
        output("vfx/smoke-effect-01.metadata.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "vfx.flipbook_v1",
        "params": {
            "resolution": [256, 256],
            "padding": 2,
            "effect": "smoke",
            "frame_count": 12,
            "frame_size": [48, 48],
            "fps": 30,
            "loop_mode": "loop"
        }
    }
)
