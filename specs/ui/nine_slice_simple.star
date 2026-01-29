# Simple nine-slice UI panel with basic colors
#
# Recipe: ui.nine_slice_v1 - generates a 9-slice panel texture for scalable UI elements

spec(
    asset_id = "ui-panel-simple",
    asset_type = "ui",
    license = "CC0-1.0",
    seed = 42,
    description = "Simple nine-slice UI panel with basic colors",
    outputs = [
        output("ui/panel_simple.png", "png"),
        output("ui/panel_simple_meta.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "ui.nine_slice_v1",
        "params": {
            "resolution": [256, 256],
            "padding": 2,
            "regions": {
                "corner_size": [16, 16],
                "top_left": [0.2, 0.2, 0.2, 1.0],
                "top_right": [0.2, 0.2, 0.2, 1.0],
                "bottom_left": [0.2, 0.2, 0.2, 1.0],
                "bottom_right": [0.2, 0.2, 0.2, 1.0],
                "top_edge": [0.3, 0.3, 0.3, 1.0],
                "bottom_edge": [0.3, 0.3, 0.3, 1.0],
                "left_edge": [0.3, 0.3, 0.3, 1.0],
                "right_edge": [0.3, 0.3, 0.3, 1.0],
                "center": [0.9, 0.9, 0.9, 1.0]
            }
        }
    }
)
