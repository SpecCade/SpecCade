# Icon set variants golden spec
#
# Exercises ui.icon_set_v1 with many icons, different sizes, and categories.

spec(
    asset_id = "golden-ui-icon-set-01",
    asset_type = "ui",
    seed = 8001,
    description = "Game HUD icon set with varied sizes and categories",
    tags = ["golden", "ui", "icon_set", "hud"],
    outputs = [
        output("ui/icons_hud.png", "png"),
        output("ui/icons_hud_meta.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "ui.icon_set_v1",
        "params": {
            "resolution": [512, 512],
            "padding": 2,
            "icons": [
                {"id": "health",     "width": 32, "height": 32, "color": [0.9, 0.1, 0.1, 1.0], "category": "status"},
                {"id": "mana",       "width": 32, "height": 32, "color": [0.1, 0.3, 0.9, 1.0], "category": "status"},
                {"id": "stamina",    "width": 32, "height": 32, "color": [0.2, 0.8, 0.2, 1.0], "category": "status"},
                {"id": "shield",     "width": 32, "height": 32, "color": [0.7, 0.7, 0.8, 1.0], "category": "status"},
                {"id": "sword",      "width": 48, "height": 48, "color": [0.8, 0.8, 0.85, 1.0], "category": "equipment"},
                {"id": "bow",        "width": 48, "height": 48, "color": [0.6, 0.4, 0.2, 1.0],  "category": "equipment"},
                {"id": "helmet",     "width": 48, "height": 48, "color": [0.5, 0.5, 0.55, 1.0], "category": "equipment"},
                {"id": "potion",     "width": 24, "height": 32, "color": [0.8, 0.2, 0.4, 1.0],  "category": "consumable"},
                {"id": "scroll",     "width": 24, "height": 32, "color": [0.9, 0.85, 0.7, 1.0], "category": "consumable"},
                {"id": "coin",       "width": 24, "height": 24, "color": [1.0, 0.85, 0.0, 1.0], "category": "currency"},
                {"id": "gem",        "width": 24, "height": 24, "color": [0.3, 0.9, 0.9, 1.0],  "category": "currency"},
                {"id": "minimap",    "width": 64, "height": 64, "color": [0.2, 0.3, 0.2, 0.8],  "category": "navigation"}
            ]
        }
    }
)
