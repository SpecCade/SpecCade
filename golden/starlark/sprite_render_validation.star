# Sprite render validation - compound asymmetric mesh with obvious front
#
# A flat wide body (box) with:
#   - A cone "nose" sticking out along +Y (the front)
#   - A small cube "fin" on top (+Z), offset toward -Y (the back)
#
# This creates a shape like a simple spaceship/arrow pointing +Y,
# with an obvious up, front, back, left, and right.
#
# Coordinate system (from code trace of entrypoint.py):
#   angle=0°:   camera at -Y looking +Y → sees the FRONT (nose pointing at you)
#   angle=90°:  camera at +X looking -X → sees the LEFT side
#   angle=180°: camera at +Y looking -Y → sees the BACK (fin visible, nose away)
#   angle=270°: camera at -X looking +X → sees the RIGHT side
#
# Three-point lighting is FIXED:
#   Key light:  (+X, -Y, +Z) → brightest on front-right
#   Fill light: (-X, -Y, +Z) → softer on front-left
#   Back light: (0, +Y, +Z)  → rim on back
#
# Expected atlas (2x2 grid, reading order):
#   ┌──────────────────────┬──────────────────────┐
#   │ 0° FRONT             │ 90° LEFT SIDE        │
#   │ nose tip toward cam, │ body profile, nose    │
#   │ bright (key+fill)    │ points right on image │
#   ├──────────────────────┼──────────────────────┤
#   │ 180° BACK            │ 270° RIGHT SIDE      │
#   │ fin visible on top,  │ body profile, nose    │
#   │ darker (backlit)     │ points left on image  │
#   └──────────────────────┴──────────────────────┘

spec(
    asset_id = "validation-sprite-compound-ship",
    asset_type = "sprite",
    description = "Compound mesh (body + nose cone + fin) for unambiguous directional validation",
    seed = 42,
    outputs = [
        output("sprites/ship_validation_atlas.png", "png"),
        output("sprites/ship_validation_atlas.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "sprite.render_from_mesh_v1",
        "params": {
            "mesh": {
                "base_primitive": "cube",
                "dimensions": [1.5, 1.0, 0.3],
                "modifiers": [],
                "attachments": [
                    {
                        "primitive": "cone",
                        "dimensions": [0.4, 0.4, 0.8],
                        "position": [0.0, 0.9, 0.0],
                        "rotation": [-90.0, 0.0, 0.0]
                    },
                    {
                        "primitive": "cube",
                        "dimensions": [0.15, 0.3, 0.4],
                        "position": [0.0, -0.3, 0.35],
                        "rotation": [0.0, 0.0, 0.0]
                    }
                ]
            },
            "camera": "orthographic",
            "lighting": "three_point",
            "frame_resolution": [256, 256],
            "rotation_angles": [0.0, 90.0, 180.0, 270.0],
            "atlas_padding": 4,
            "camera_distance": 2.5,
            "camera_elevation": 30.0
        }
    }
)
