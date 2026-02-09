# Golden test: Edge split modifier
#
# This example demonstrates the edge_split_modifier() function
# which splits edges sharper than a given angle.
# Uses a cylinder so some edges are smooth (<30°) and some are split (>30°),
# creating a visible difference vs a plain smooth-shaded cylinder.

spec(
    asset_id = "stdlib-mesh-edge-split-01",
    asset_type = "static_mesh",
    seed = 42,
    outputs = [output("meshes/edge_split.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe(
            "cylinder",
            [1.0, 1.0, 1.0],
            [
                edge_split_modifier(angle = 30.0),
            ]
        )
    }
)
