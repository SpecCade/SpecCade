# Mirror modifier example
#
# The mirror modifier creates symmetric geometry by reflecting across axes.
# Useful for characters, vehicles, and other symmetric objects.
# Covers: mirror_modifier()

spec(
    asset_id = "stdlib-mesh-mirror-01",
    asset_type = "static_mesh",
    seed = 42,
    outputs = [output("meshes/mirror.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe(
            "cube",
            [0.5, 1.0, 0.5],
            [
                # Mirror on X and Y axes
                mirror_modifier(True, True, False),
                bevel_modifier(0.02, 2)
            ]
        )
    }
)
