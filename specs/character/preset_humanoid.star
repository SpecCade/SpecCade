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
            "skeleton_preset": "humanoid_connected_v1",
            "bone_meshes": {
                "hips": {"profile": "circle(8)", "profile_radius": {"absolute": 0.12}, "cap_start": True},
                "spine": {"profile": "circle(8)", "profile_radius": {"absolute": 0.13}},
                "chest": {"profile": "circle(8)", "profile_radius": {"absolute": 0.14}, "cap_end": True},
                "neck": {"profile": "circle(8)", "profile_radius": {"absolute": 0.05}, "cap_start": True},
                "head": {"profile": "circle(12)", "profile_radius": {"absolute": 0.1}, "cap_end": True},
                # Shoulders bridge chest to upper arms
                "shoulder_l": {"profile": "circle(6)", "profile_radius": {"absolute": 0.06}, "cap_start": True},
                "shoulder_r": {"mirror": "shoulder_l"},
                "upper_arm_l": {"profile": "circle(6)", "profile_radius": {"absolute": 0.055}},
                "upper_arm_r": {"mirror": "upper_arm_l"},
                "lower_arm_l": {"profile": "circle(6)", "profile_radius": {"absolute": 0.045}},
                "lower_arm_r": {"mirror": "lower_arm_l"},
                "hand_l": {"profile": "circle(6)", "profile_radius": {"absolute": 0.035}, "cap_end": True},
                "hand_r": {"mirror": "hand_l"},
                # Legs
                "upper_leg_l": {"profile": "circle(6)", "profile_radius": {"absolute": 0.07}, "cap_start": True, "cap_end": True},
                "upper_leg_r": {"mirror": "upper_leg_l"},
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
