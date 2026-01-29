# Modular kit mesh - demonstrates wall/pipe/door kit generation
#
# This example creates a wall section with a door cutout.

spec(
    asset_id = "stdlib-modular-wall-01",
    asset_type = "static_mesh",
    seed = 42,
    outputs = [output("meshes/wall_kit.glb", "glb")],
    recipe = {
        "kind": "static_mesh.modular_kit_v1",
        "params": {
            "kit_type": {
                "type": "wall",
                "width": 3.0,
                "height": 2.5,
                "thickness": 0.15,
                "cutouts": [
                    {
                        "cutout_type": "door",
                        "x": 1.0,
                        "y": 0.0,
                        "width": 0.9,
                        "height": 2.1,
                        "has_frame": True,
                        "frame_thickness": 0.05
                    }
                ],
                "has_baseboard": True,
                "has_crown": False,
                "baseboard_height": 0.1,
                "bevel_width": 0.01
            }
        }
    }
)
