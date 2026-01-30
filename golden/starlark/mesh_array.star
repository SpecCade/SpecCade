# Array modifier example
#
# The array modifier creates repeated copies with offsets.
# Great for fences, stairs, chains, and other repeating structures.
# Covers: array_modifier()

spec(
    asset_id = "stdlib-mesh-array-01",
    asset_type = "static_mesh",
    seed = 42,
    outputs = [output("meshes/array.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe(
            "cube",
            [0.2, 0.2, 0.2],
            [
                bevel_modifier(0.01, 1),
                # Create 5 copies along X axis
                array_modifier(5, [0.3, 0.0, 0.0])
            ]
        )
    }
)
