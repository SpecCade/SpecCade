# Simple modular part character.
#
# [VALIDATION]
# SHAPE: Blocky humanoid built from one primitive part per bone.
# PROPORTIONS: Torso wider than limbs, spherical head, rectangular feet.
# ORIENTATION: Upright, facing +Y, left/right mirrored limbs.
# FRONT VIEW: Symmetric silhouette with broad chest and straight legs.
# ISO VIEW: Clear segmented kitbash look (no smooth extrusion blending).

spec(
    asset_id = "part_simple",
    asset_type = "skeletal_mesh",
    seed = 4101,
    license = "CC0-1.0",
    description = "Simple part-based humanoid using one primitive per bone",
    tags = ["skeletal_mesh", "character", "armature_driven_v1", "modular_parts"],
    outputs = [output("skeletal_mesh/part_simple.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton_preset": "humanoid_basic_v1",
            "material_slots": [
                material_slot(name = "body", base_color = [0.70, 0.73, 0.78, 1.0], roughness = 0.55),
            ],
            "bone_meshes": {
                "hips": {"part": {"base": {"primitive": "cube", "dimensions": [0.24, 0.16, 1.0]}}},
                "spine": {"part": {"base": {"primitive": "cube", "dimensions": [0.22, 0.15, 1.0]}}},
                "chest": {"part": {"base": {"primitive": "cube", "dimensions": [0.30, 0.20, 1.0]}}},
                "neck": {"part": {"base": {"primitive": "cylinder", "dimensions": [0.07, 0.07, 1.0]}}},
                "head": {"part": {"base": {"primitive": "sphere", "dimensions": [0.22, 0.22, 0.26]}}},

                "shoulder_l": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.07, 0.07, 1.0]},
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                },
                "shoulder_r": {"mirror": "shoulder_l"},
                "upper_arm_l": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.08, 0.08, 1.0]},
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                },
                "upper_arm_r": {"mirror": "upper_arm_l"},
                "lower_arm_l": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.065, 0.065, 1.0]},
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                },
                "lower_arm_r": {"mirror": "lower_arm_l"},
                "hand_l": {"part": {"base": {"primitive": "cube", "dimensions": [0.08, 0.05, 0.7]}}},
                "hand_r": {"mirror": "hand_l"},

                "upper_leg_l": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.10, 0.10, 1.0]},
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                },
                "upper_leg_r": {"mirror": "upper_leg_l"},
                "lower_leg_l": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.085, 0.085, 1.0]},
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                },
                "lower_leg_r": {"mirror": "lower_leg_l"},
                "foot_l": {"part": {"base": {"primitive": "cube", "dimensions": [0.09, 0.14, 0.50]}}},
                "foot_r": {"mirror": "foot_l"},
            },
        },
    },
)
