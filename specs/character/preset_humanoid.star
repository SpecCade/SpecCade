# Humanoid using skeleton_preset - demonstrates modern body_parts system

spec(
    asset_id = "preset_humanoid",
    asset_type = "skeletal_mesh",
    license = "CC0-1.0",
    seed = 7005,
    description = "Humanoid using skeleton_preset - demonstrates modern body_parts system",
    outputs = [output("preset_humanoid.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton_preset": "humanoid_basic_v1",
            "bone_meshes": {
                "spine": {"profile": "circle(8)", "profile_radius": 0.15}
            },
            "material_slots": [
                {"name": "body_material", "base_color": [0.8, 0.6, 0.5, 1.0]},
                {"name": "head_material", "base_color": [0.9, 0.7, 0.6, 1.0]}
            ],
            "export": {
                "include_armature": True,
                "include_normals": True,
                "include_uvs": True,
                "triangulate": True,
                "include_skin_weights": True,
                "save_blend": False
            },
            "constraints": {
                "max_triangles": 5000,
                "max_bones": 64,
                "max_materials": 4
            }
        }
    }
)
