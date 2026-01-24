# UI Damage Number Sprites example
# Demonstrates generating damage number digit sprites with multiple style variants.

{
    "spec_version": 1,
    "asset_id": "starlark-ui-damage-number-01",
    "asset_type": "ui",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [
        {
            "kind": "primary",
            "format": "png",
            "path": "ui/damage_numbers.png"
        },
        {
            "kind": "metadata",
            "format": "json",
            "path": "ui/damage_numbers.json"
        }
    ],
    "recipe": {
        "kind": "ui.damage_number_v1",
        "params": {
            "glyph_size": [16, 24],
            "outline_width": 2,
            "padding": 2,
            "styles": [
                {
                    "style_type": "normal",
                    "text_color": [1.0, 1.0, 1.0, 1.0],
                    "outline_color": [0.0, 0.0, 0.0, 1.0]
                },
                {
                    "style_type": "critical",
                    "text_color": [1.0, 0.9, 0.0, 1.0],
                    "outline_color": [1.0, 0.0, 0.0, 1.0],
                    "glow_color": [1.0, 0.5, 0.0, 0.5],
                    "scale": 1.25
                },
                {
                    "style_type": "healing",
                    "text_color": [0.0, 1.0, 0.0, 1.0],
                    "outline_color": [0.0, 0.3, 0.0, 1.0]
                }
            ]
        }
    }
}
