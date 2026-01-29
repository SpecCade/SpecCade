# Basic UI icon set with common icons
#
# Recipe: ui.icon_set_v1 - generates an icon atlas with multiple categorized icons

spec(
    asset_id = "ui-icons-basic",
    asset_type = "ui",
    license = "CC0-1.0",
    seed = 123,
    description = "Basic UI icon set with common icons",
    outputs = [
        output("ui/icons_basic.png", "png"),
        output("ui/icons_basic_meta.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "ui.icon_set_v1",
        "params": {
            "resolution": [512, 512],
            "padding": 2,
            "icons": [
                {
                    "id": "close",
                    "width": 32,
                    "height": 32,
                    "color": [1.0, 0.0, 0.0, 1.0],
                    "category": "action"
                },
                {
                    "id": "settings",
                    "width": 48,
                    "height": 48,
                    "color": [0.5, 0.5, 0.5, 1.0],
                    "category": "system"
                },
                {
                    "id": "heart",
                    "width": 24,
                    "height": 24,
                    "color": [1.0, 0.0, 0.5, 1.0],
                    "category": "social"
                },
                {
                    "id": "info",
                    "width": 32,
                    "height": 32,
                    "color": [0.0, 0.5, 1.0, 1.0],
                    "category": "status"
                }
            ]
        }
    }
)
