# UI Item Card template example
# Demonstrates generating item card templates with multiple rarity variants.

{
    "spec_version": 1,
    "asset_id": "starlark-ui-item-card-01",
    "asset_type": "ui",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [
        {
            "kind": "primary",
            "format": "png",
            "path": "ui/item_cards.png"
        },
        {
            "kind": "metadata",
            "format": "json",
            "path": "ui/item_cards.json"
        }
    ],
    "recipe": {
        "kind": "ui.item_card_v1",
        "params": {
            "resolution": [128, 192],
            "padding": 2,
            "border_width": 3,
            "corner_radius": 8,
            "slots": {
                "icon_region": [16, 16, 96, 96],
                "rarity_indicator_region": [16, 120, 96, 16],
                "background_region": [0, 0, 128, 192]
            },
            "rarity_presets": [
                {
                    "tier": "common",
                    "border_color": [0.5, 0.5, 0.5, 1.0],
                    "background_color": [0.15, 0.15, 0.15, 1.0]
                },
                {
                    "tier": "uncommon",
                    "border_color": [0.2, 0.8, 0.2, 1.0],
                    "background_color": [0.1, 0.2, 0.1, 1.0]
                },
                {
                    "tier": "rare",
                    "border_color": [0.2, 0.5, 1.0, 1.0],
                    "background_color": [0.1, 0.15, 0.25, 1.0]
                },
                {
                    "tier": "epic",
                    "border_color": [0.7, 0.3, 0.9, 1.0],
                    "background_color": [0.2, 0.1, 0.25, 1.0]
                },
                {
                    "tier": "legendary",
                    "border_color": [1.0, 0.8, 0.2, 1.0],
                    "background_color": [0.25, 0.2, 0.1, 1.0],
                    "glow_color": [1.0, 0.9, 0.4, 0.4]
                }
            ]
        }
    }
}
