# Simple cube mesh - demonstrates mesh stdlib
#
# This example creates a basic cube with bevel and subdivision modifiers.

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
