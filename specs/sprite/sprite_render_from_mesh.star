# Sprite render from mesh - demonstrates mesh-to-sprite rendering
#
# This example renders a simple cube from 8 angles to create an 8-directional
# sprite atlas for use in 2D games.
#
# Recipe: sprite.render_from_mesh_v1 (Tier 2, Blender backend)

spec(
    asset_id = "stdlib-sprite-render-cube-01",
    asset_type = "sprite",
    seed = 42,
    outputs = [
        output("sprites/cube_atlas.png", "png"),
        output("sprites/cube_atlas.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "sprite.render_from_mesh_v1",
        "params": {
            "mesh": mesh_recipe(
                "cube",
                [1.0, 1.0, 1.0],
                [
                    bevel_modifier(0.05, 2)
                ]
            ),
            "camera": "orthographic",
            "lighting": "three_point",
            "frame_resolution": [64, 64],
            "rotation_angles": [0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0],
            "atlas_padding": 2,
            "camera_distance": 2.5,
            "camera_elevation": 30.0
        }
    }
)
