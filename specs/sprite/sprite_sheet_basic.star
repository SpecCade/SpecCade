# Sprite sheet basic - demonstrates spritesheet atlas packing
#
# This example packs 4 solid-color frames into a 256x256 atlas with
# pivot points at the bottom-center (typical for character sprites).
#
# Recipe: sprite.sheet_v1 (Tier 1, Rust-only, byte-identical)

spec(
    asset_id = "sprite-sheet-basic",
    asset_type = "sprite",
    seed = 42,
    outputs = [
        output("sprites/sheet.png", "png"),
        output("sprites/sheet.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "sprite.sheet_v1",
        "params": {
            "resolution": [256, 256],
            "padding": 2,
            "frames": [
                {"id": "frame_0", "width": 32, "height": 32, "pivot": [0.5, 1.0], "color": [1.0, 0.0, 0.0, 1.0]},
                {"id": "frame_1", "width": 32, "height": 32, "pivot": [0.5, 1.0], "color": [0.0, 1.0, 0.0, 1.0]},
                {"id": "frame_2", "width": 32, "height": 32, "pivot": [0.5, 1.0], "color": [0.0, 0.0, 1.0, 1.0]},
                {"id": "frame_3", "width": 32, "height": 32, "pivot": [0.5, 1.0], "color": [1.0, 1.0, 0.0, 1.0]}
            ]
        }
    }
)
