# Decimate modifier example
#
# The decimate modifier reduces polygon count for LOD optimization.
# Useful for creating lower-detail versions of complex meshes.
# Covers: decimate_modifier()

spec(
    asset_id = "stdlib-mesh-decimate-01",
    asset_type = "static_mesh",
    seed = 42,
    outputs = [output("meshes/decimate.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe(
            "ico_sphere",
            [1.0, 1.0, 1.0],
            [
                # First subdivide to add detail
                subdivision_modifier(2),
                # Then decimate to reduce poly count
                decimate_modifier(0.3)
            ]
        )
    }
)
