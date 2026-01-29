# Basic 5x7 monospace bitmap font with ASCII printable characters

spec(
    asset_id = "bitmap-basic-01",
    asset_type = "font",
    license = "CC0-1.0",
    seed = 42,
    description = "Basic 5x7 monospace bitmap font with ASCII printable characters",
    style_tags = ["retro", "pixel", "monospace"],
    outputs = [
        output("fonts/bitmap_basic_atlas.png", "png"),
        output("fonts/bitmap_basic_metrics.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "font.bitmap_v1",
        "params": {
            "charset": [32, 126],
            "glyph_size": [5, 7],
            "padding": 2,
            "font_style": "monospace",
            "color": [1.0, 1.0, 1.0, 1.0]
        }
    }
)
