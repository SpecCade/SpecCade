# Mirror modifier example
#
# The mirror modifier creates symmetric geometry by reflecting across axes.
# Useful for characters, vehicles, and other symmetric objects.
# Covers: mirror_modifier()
#
# Uses a cone so mirroring on X produces a clearly recognizable
# double-cone (diamond) shape that proves the mirror worked.

spec(
    asset_id = "stdlib-mesh-mirror-01",
    asset_type = "static_mesh",
    seed = 42,
    outputs = [output("meshes/mirror.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe(
            "cone",
            [1.0, 1.0, 1.0],
            [
                # Mirror on X axis — cone becomes a diamond, proving the reflection
                mirror_modifier(True, False, False),
                bevel_modifier(0.02, 2)
            ]
        )
    }
)
