# Simple cube mesh - demonstrates mesh stdlib
#
# This example creates a basic cube with bevel and subdivision modifiers.
#
# [VALIDATION]
# SHAPE: A beveled cube with smooth subdivision
# PROPORTIONS: Equal 1.0 unit dimensions on all axes
# ORIENTATION: Cube centered at origin, aligned to world axes (XYZ)
# FRONT VIEW: Square face with beveled edges visible, centered
# BACK VIEW: Identical to front (cube symmetry)
# LEFT VIEW: Square face with beveled edges, centered
# RIGHT VIEW: Identical to left (cube symmetry)
# TOP VIEW: Square face looking down, centered
# ISO VIEW: Three faces visible (front, top, right corner)
# NOTES: Bevel modifier (0.02, 2 segments) should create smooth edge transitions, subdivision (level 2) adds smoothness

spec(
    asset_id = "stdlib-mesh-cube-01",
    asset_type = "static_mesh",
    seed = 42,
    outputs = [output("meshes/cube.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe(
            "cube",
            [1.0, 1.0, 1.0],
            [
                bevel_modifier(0.02, 2),
                subdivision_modifier(2)
            ]
        )
    }
)
