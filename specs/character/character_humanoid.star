# Simple humanoid character - demonstrates character stdlib
#
# This example creates a basic humanoid skeletal mesh using the
# character stdlib functions for body parts, materials, and skinning.
#
# [VALIDATION]
# SHAPE: Humanoid figure with cylindrical torso, spherical head, cylindrical limbs
# PROPORTIONS: Head sphere ~0.11m, torso ~0.12-0.14m radius, arms ~0.04-0.05m, legs ~0.05-0.07m
# ORIENTATION: Standing upright (+Z up), facing +Y forward, T-pose with arms extended sideways
# FRONT VIEW: Symmetric humanoid shape - head on top, arms extended left/right, legs below
# BACK VIEW: Mirror of front, spine area visible
# LEFT VIEW: Profile view - head, single arm cylinder, single leg cylinder
# RIGHT VIEW: Mirror of left view
# TOP VIEW: Looking down at head sphere, torso below, arms extending sideways
# ISO VIEW: Full 3D humanoid form clearly visible in T-pose
# NOTES: Body uses skin-tone material (0.8, 0.6, 0.5), head slightly lighter (0.9, 0.7, 0.6)

spec(
    asset_id = "stdlib-character-humanoid-01",
    asset_type = "skeletal_mesh",
    seed = 42,
    description = "Stdlib humanoid character example",
    outputs = [output("characters/humanoid.glb", "glb")],
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
                    "material_index": 1,
                    "cap_start": False,
                    "cap_end": True,
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.55, 0.65, 0.55],
                            "offset": [0, 0, 0.4],
                            "material_index": 1
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
                    name = "body_material",
                    base_color = [0.8, 0.6, 0.5, 1.0]
                ),
                material_slot(
                    name = "head_material",
                    base_color = [0.9, 0.7, 0.6, 1.0]
                )
            ]
        }
    }
)
