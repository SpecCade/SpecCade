# Skinned mesh basic (golden fixture)
#
# Basic skinned humanoid character using armature-driven procedural mesh.

spec(
    asset_id = "skinned_mesh_basic",
    asset_type = "skeletal_mesh",
    seed = 7120,
    license = "CC0-1.0",
    description = "Golden fixture: basic skinned humanoid character",
    tags = ["golden", "skeletal_mesh", "character", "armature_driven_v1"],
    outputs = [output("skeletal_mesh/skinned_mesh_basic.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "bone_meshes": {
                "spine": {
                    "profile": "circle(8)",
                    "profile_radius": 0.12,
                    "material_index": 0,
                    "cap_start": True,
                    "cap_end": False,
                },
                "chest": {
                    "profile": "circle(8)",
                    "profile_radius": 0.14,
                    "material_index": 0,
                    "cap_start": False,
                    "cap_end": True,
                },
                "head": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.1},
                    "material_index": 0,
                    "cap_start": False,
                    "cap_end": True,
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.12, 0.14, 0.12],
                            "offset": [0, 0, 0.5],
                            "material_index": 0,
                        },
                    ],
                },
                # Shoulder bones connect chest center to upper arms
                # Note: Can't bridge because shoulder is diagonal, upper_arm is horizontal
                "shoulder_l": {
                    "profile": "circle(6)",
                    "profile_radius": 0.07,
                    "material_index": 0,
                    "cap_start": True,
                    "cap_end": False,
                },
                "shoulder_r": {"mirror": "shoulder_l"},
                "upper_arm_l": {
                    "profile": "circle(6)",
                    "profile_radius": 0.06,
                    "material_index": 0,
                    "cap_start": False,  # Shoulder end overlaps this start
                    "cap_end": True,
                },
                "upper_arm_r": {"mirror": "upper_arm_l"},
                "upper_leg_l": {
                    "profile": "circle(6)",
                    "profile_radius": 0.08,
                    "material_index": 0,
                },
                "upper_leg_r": {"mirror": "upper_leg_l"},
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
