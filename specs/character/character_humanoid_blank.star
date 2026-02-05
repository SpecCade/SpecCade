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
                "spine": {
                    "profile": "circle(8)",
                    "profile_radius": 0.15,
                    "material_index": 0,
                    "cap_start": True,
                    "cap_end": False
                },
                "chest": {
                    "profile": "circle(8)",
                    "profile_radius": 0.18,
                    "material_index": 0,
                    "cap_start": False,
                    "cap_end": True
                },
                "head": {
                    "profile": "circle(8)",
                    "profile_radius": 0.08,
                    "material_index": 0,
                    "cap_start": False,
                    "cap_end": True,
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.15, 0.18, 0.15],
                            "offset": [0, 0, 0.6],
                            "material_index": 0
                        }
                    ]
                },
                "shoulder_l": {"profile": "circle(6)", "profile_radius": 0.07, "cap_start": True, "cap_end": False, "material_index": 0},
                "shoulder_r": {"mirror": "shoulder_l"},
                "upper_arm_l": {
                    "profile": "circle(6)",
                    "profile_radius": 0.08,
                    "material_index": 0,
                    "rotate": [0, 0, 90]
                },
                "upper_arm_r": {
                    "profile": "circle(6)",
                    "profile_radius": 0.08,
                    "material_index": 0,
                    "rotate": [0, 0, -90]
                },
                "upper_leg_l": {
                    "profile": "circle(6)",
                    "profile_radius": 0.1,
                    "material_index": 0,
                    "rotate": [180, 0, 0]
                },
                "upper_leg_r": {
                    "profile": "circle(6)",
                    "profile_radius": 0.1,
                    "material_index": 0,
                    "rotate": [180, 0, 0]
                }
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
