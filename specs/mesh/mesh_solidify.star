# Solidify modifier example
#
# The solidify modifier adds thickness to thin geometry.
# Converts a flat surface into a solid shell.
# Covers: solidify_modifier()

spec(
    asset_id = "stdlib-mesh-solidify-01",
    asset_type = "static_mesh",
    seed = 42,
    outputs = [output("meshes/solidify.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe(
            "plane",
            [2.0, 2.0, 0.0],
            [
                # Add thickness with outward offset
                solidify_modifier(0.1, 1.0),
                bevel_modifier(0.01, 2)
            ]
        )
    }
)
