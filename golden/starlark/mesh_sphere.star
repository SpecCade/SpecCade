# Decimated sphere mesh - demonstrates mesh_recipe with decimate
#
# This example creates a sphere and reduces polygon count with decimation.

spec(
    asset_id = "stdlib-mesh-sphere-01",
    asset_type = "static_mesh",
    seed = 42,
    outputs = [output("meshes/sphere.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe(
            "sphere",
            [2.0, 2.0, 2.0],
            [decimate_modifier(0.5)]
        )
    }
)
