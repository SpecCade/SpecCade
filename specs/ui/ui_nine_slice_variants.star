# Nine-slice variants golden spec
#
# Exercises ui.nine_slice_v1 with different border sizes, aspect ratios,
# edge widths, and background color.

# Wide panel with non-square corners and custom edge sizes
spec(
    asset_id = "golden-ui-nine-slice-wide-01",
    asset_type = "ui",
    seed = 7001,
    description = "Wide nine-slice panel with non-square corners and custom edge sizes",
    tags = ["golden", "ui", "nine_slice", "panel"],
    outputs = [
        output("ui/panel_wide.png", "png"),
        output("ui/panel_wide_meta.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "ui.nine_slice_v1",
        "params": {
            "resolution": [512, 256],
            "padding": 4,
            "background_color": [0.0, 0.0, 0.0, 0.0],
            "regions": {
                "corner_size": [24, 16],
                "edge_width": 12,
                "edge_height": 8,
                "top_left": [0.15, 0.25, 0.45, 1.0],
                "top_right": [0.15, 0.25, 0.45, 1.0],
                "bottom_left": [0.10, 0.18, 0.35, 1.0],
                "bottom_right": [0.10, 0.18, 0.35, 1.0],
                "top_edge": [0.12, 0.22, 0.42, 1.0],
                "bottom_edge": [0.08, 0.15, 0.30, 1.0],
                "left_edge": [0.12, 0.20, 0.38, 1.0],
                "right_edge": [0.12, 0.20, 0.38, 1.0],
                "center": [0.05, 0.10, 0.25, 0.9]
            }
        }
    }
)

# Small tooltip panel with large corners relative to resolution
spec(
    asset_id = "golden-ui-nine-slice-tooltip-01",
    asset_type = "ui",
    seed = 7002,
    description = "Small tooltip nine-slice panel with large rounded corners",
    tags = ["golden", "ui", "nine_slice", "tooltip"],
    outputs = [
        output("ui/panel_tooltip.png", "png"),
        output("ui/panel_tooltip_meta.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "ui.nine_slice_v1",
        "params": {
            "resolution": [128, 128],
            "padding": 1,
            "regions": {
                "corner_size": [32, 32],
                "top_left": [0.95, 0.92, 0.85, 1.0],
                "top_right": [0.95, 0.92, 0.85, 1.0],
                "bottom_left": [0.90, 0.87, 0.80, 1.0],
                "bottom_right": [0.90, 0.87, 0.80, 1.0],
                "top_edge": [0.93, 0.90, 0.83, 1.0],
                "bottom_edge": [0.88, 0.85, 0.78, 1.0],
                "left_edge": [0.91, 0.88, 0.81, 1.0],
                "right_edge": [0.91, 0.88, 0.81, 1.0],
                "center": [0.98, 0.97, 0.94, 0.95]
            }
        }
    }
)
