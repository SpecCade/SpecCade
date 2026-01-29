# Sprite spec for enum coverage testing
#
# Recipe: sprite.sheet_v1 - covers asset_type::sprite with colored frames

spec(
    asset_id = "enum-coverage-sprite-01",
    asset_type = "sprite",
    license = "CC0-1.0",
    seed = 99001,
    description = "Sprite spec for enum coverage testing - covers asset_type::sprite",
    outputs = [
        output("sprite/enum_coverage_sheet.png", "png"),
        output("sprite/enum_coverage_sheet.json", "json", kind = "metadata"),
        output("sprite/enum_coverage_preview.png", "png", kind = "preview")
    ],
    recipe = {
        "kind": "sprite.sheet_v1",
        "params": {
            "resolution": [128, 128],
            "padding": 1,
            "frames": [
                {"id": "frame_0", "width": 32, "height": 32, "pivot": [0.5, 1.0], "color": [1.0, 0.0, 0.0, 1.0]},
                {"id": "frame_1", "width": 32, "height": 32, "pivot": [0.5, 1.0], "color": [0.0, 1.0, 0.0, 1.0]}
            ]
        }
    }
)
