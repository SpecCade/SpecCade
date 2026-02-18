# Extended bitmap font golden spec
#
# Exercises font.bitmap_v1 with different glyph sizes, proportional style,
# colored glyphs, and different charset ranges.

# Large 8x8 proportional font with colored glyphs
spec(
    asset_id = "golden-font-bitmap-proportional-01",
    asset_type = "font",
    seed = 9001,
    description = "8x8 proportional bitmap font with amber-on-black retro terminal style",
    tags = ["golden", "font", "bitmap", "proportional", "retro"],
    outputs = [
        output("fonts/bitmap_proportional_atlas.png", "png"),
        output("fonts/bitmap_proportional_metrics.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "font.bitmap_v1",
        "params": {
            "charset": [32, 126],
            "glyph_size": [8, 8],
            "padding": 1,
            "font_style": "proportional",
            "color": [1.0, 0.75, 0.0, 1.0]
        }
    }
)

# Small 5x7 uppercase-only font for HUD labels
spec(
    asset_id = "golden-font-bitmap-uppercase-01",
    asset_type = "font",
    seed = 9002,
    description = "5x7 monospace uppercase-only font for HUD labels",
    tags = ["golden", "font", "bitmap", "monospace", "hud"],
    outputs = [
        output("fonts/bitmap_uppercase_atlas.png", "png"),
        output("fonts/bitmap_uppercase_metrics.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "font.bitmap_v1",
        "params": {
            "charset": [32, 90],
            "glyph_size": [5, 7],
            "padding": 2,
            "font_style": "monospace",
            "color": [0.9, 0.95, 1.0, 1.0]
        }
    }
)

# 6x9 digits-only font for score display
spec(
    asset_id = "golden-font-bitmap-digits-01",
    asset_type = "font",
    seed = 9003,
    description = "6x9 monospace digits-only font for score counters",
    tags = ["golden", "font", "bitmap", "monospace", "digits"],
    outputs = [
        output("fonts/bitmap_digits_atlas.png", "png"),
        output("fonts/bitmap_digits_metrics.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "font.bitmap_v1",
        "params": {
            "charset": [48, 57],
            "glyph_size": [6, 9],
            "padding": 2,
            "font_style": "monospace",
            "color": [1.0, 1.0, 1.0, 1.0]
        }
    }
)
