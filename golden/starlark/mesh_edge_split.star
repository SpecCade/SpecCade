# Golden test: Edge split modifier
#
# This example demonstrates the edge_split_modifier() function
# which splits edges sharper than a given angle.

spec(
    asset_id = "stdlib-mesh-edge-split-01",
    asset_type = "static_mesh",
    seed = 42,
    outputs = [output("meshes/edge_split.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe(
            "cube",
            [1.0, 1.0, 1.0],
            [
                edge_split_modifier(angle = 30.0),
                bevel_modifier(width = 0.05, segments = 2),
            ]
        )
    }
)
