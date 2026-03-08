# Mesh source for skinned mesh contract coverage
#
# A simple capsule-like source mesh that can be rebound to a skeleton by the
# `skeletal_mesh.skinned_mesh_v1` recipe.

spec(
    asset_id = "skinned-mesh-source",
    asset_type = "static_mesh",
    seed = 7060,
    license = "CC0-1.0",
    description = "Source mesh for skinned_mesh_v1 contract validation",
    outputs = [output("meshes/skinned_mesh_source.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": {
            "base_primitive": "cylinder",
            "dimensions": [0.7, 0.7, 1.8],
            "modifiers": [
                {"type": "bevel", "width": 0.04, "segments": 2},
                {"type": "subdivision", "levels": 1, "render_levels": 1}
            ],
            "uv_projection": "smart",
            "material_slots": [
                {
                    "name": "body",
                    "base_color": [0.72, 0.76, 0.82, 1.0],
                    "roughness": 0.65
                }
            ],
            "export": {
                "apply_modifiers": True,
                "triangulate": True,
                "include_normals": True,
                "include_uvs": True
            }
        }
    }
)
