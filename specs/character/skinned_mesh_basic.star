# Skinned mesh basic (golden fixture)
#
# Note: This spec references an external mesh file path. Validation does not
# require the file to exist, but generation with the Blender backend will.

spec(
    asset_id = "skinned_mesh_basic",
    asset_type = "skeletal_mesh",
    seed = 7120,
    license = "CC0-1.0",
    description = "Golden fixture: basic skinned mesh binding (placeholder mesh_file)",
    tags = ["golden", "skeletal_mesh", "character", "skinned_mesh_v1"],
    outputs = [output("skeletal_mesh/skinned_mesh_basic.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.skinned_mesh_v1",
        "params": {
            "mesh_file": "assets/placeholder/skinned_mesh_basic.glb",
            "skeleton_preset": "humanoid_basic_v1",
            "binding": {
                "mode": "auto_weights",
                "max_bone_influences": 4,
                "vertex_group_map": {},
            },
            "material_slots": [
                {"name": "body", "base_color": [0.70, 0.74, 0.80, 1.0], "roughness": 0.7},
            ],
            "export": {
                "include_armature": True,
                "include_normals": True,
                "include_uvs": True,
                "triangulate": True,
                "include_skin_weights": True,
                "save_blend": False,
            },
            "constraints": {
                "max_triangles": 15000,
                "max_bones": 64,
                "max_materials": 8,
            },
        },
    },
)
