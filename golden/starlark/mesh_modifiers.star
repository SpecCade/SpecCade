# Mesh modifiers example
#
# Demonstrates all available mesh modifiers for 3D geometry.
# Covers: bevel_modifier(), subdivision_modifier(), decimate_modifier(),
#         edge_split_modifier(), mirror_modifier(), array_modifier(), solidify_modifier()

spec(
    asset_id = "stdlib-mesh-modifiers-01",
    asset_type = "static_mesh",
    seed = 42,
    outputs = [output("meshes/modifiers.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe(
            "cube",
            [1.0, 1.0, 1.0],
            [
                # Bevel edges for smoothing
                bevel_modifier(0.03, 3),

                # Subdivision for smoothness
                subdivision_modifier(2, 2),

                # Edge split for sharp normals
                edge_split_modifier(30.0)
            ]
        )
    }
)
