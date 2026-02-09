# Blank-material humanoid character - placeholder for texturing
#
# All body parts use a single white material (material_index=0).
# Use this as a starting template when the final look will come
# from textures or runtime shading rather than baked vertex colours.

spec(
    asset_id = "stdlib-character-humanoid-blank-01",
    asset_type = "skeletal_mesh",
    seed = 42,
    description = "Blank-material humanoid placeholder",
    outputs = [output("characters/humanoid_blank.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "bone_meshes": {
                "hips": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.12},
                    "material_index": 0,
                    "cap_start": True,
                    "cap_end": False
                },
                "spine": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.13},
                    "material_index": 0,
                    "cap_start": False,
                    "cap_end": False
                },
                "chest": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.14},
                    "material_index": 0,
                    "cap_start": False,
                    "cap_end": True
                },
                "neck": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.05},
                    "material_index": 0,
                    "cap_start": True,
                    "cap_end": False
                },
                "head": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.05},
                    "material_index": 0,
                    "cap_start": False,
                    "cap_end": True,
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.55, 0.65, 0.55],
                            "offset": [0, 0, 0.4],
                            "material_index": 0
                        }
                    ]
                },
                "shoulder_l": {"profile": "circle(6)", "profile_radius": {"absolute": 0.06}, "cap_start": True, "cap_end": False, "material_index": 0},
                "shoulder_r": {"mirror": "shoulder_l"},
                "upper_arm_l": {
                    "profile": "circle(6)",
                    "profile_radius": {"absolute": 0.05},
                    "material_index": 0
                },
                "upper_arm_r": {"mirror": "upper_arm_l"},
                "lower_arm_l": {
                    "profile": "circle(6)",
                    "profile_radius": {"absolute": 0.04},
                    "material_index": 0,
                    "cap_end": True
                },
                "lower_arm_r": {"mirror": "lower_arm_l"},
                "hand_l": {
                    "profile": "circle(6)",
                    "profile_radius": {"absolute": 0.03},
                    "material_index": 0,
                    "cap_end": True
                },
                "hand_r": {"mirror": "hand_l"},
                "upper_leg_l": {
                    "profile": "circle(6)",
                    "profile_radius": {"absolute": 0.07},
                    "material_index": 0,
                    "cap_start": True
                },
                "upper_leg_r": {"mirror": "upper_leg_l"},
                "lower_leg_l": {
                    "profile": "circle(6)",
                    "profile_radius": {"absolute": 0.05},
                    "material_index": 0,
                    "cap_end": True
                },
                "lower_leg_r": {"mirror": "lower_leg_l"},
                "foot_l": {
                    "profile": "circle(6)",
                    "profile_radius": {"absolute": 0.04},
                    "material_index": 0,
                    "cap_end": True
                },
                "foot_r": {"mirror": "foot_l"}
            },
            "material_slots": [
                material_slot(
                    name = "blank_white",
                    base_color = [1.0, 1.0, 1.0, 1.0]
                )
            ]
        }
    }
)
