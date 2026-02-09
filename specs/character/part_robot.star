# Full modular part robot character.
#
# [VALIDATION]
# SHAPE: Fully segmented robot humanoid assembled from part booleans.
# PROPORTIONS: Boxy chest, compact head, cylindrical limbs with joint bulbs.
# ORIENTATION: Upright humanoid with mirrored left/right limbs.
# FRONT VIEW: Symmetric mechanical silhouette with shoulder and knee joints.
# ISO VIEW: Hard-surface boolean details on chest/head are clearly visible.

spec(
    asset_id = "part_robot",
    asset_type = "skeletal_mesh",
    seed = 4104,
    license = "CC0-1.0",
    description = "Full robot built with modular bone parts only",
    tags = ["skeletal_mesh", "character", "armature_driven_v1", "robot", "modular_parts"],
    outputs = [output("skeletal_mesh/part_robot.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton_preset": "humanoid_basic_v1",
            "material_slots": [
                material_slot(name = "paint", base_color = [0.78, 0.22, 0.10, 1.0], metallic = 0.45, roughness = 0.36),
                material_slot(name = "metal_dark", base_color = [0.16, 0.17, 0.19, 1.0], metallic = 0.82, roughness = 0.30),
            ],
            "bone_meshes": {
                "hips": {"part": {"base": {"primitive": "cube", "dimensions": [0.24, 0.16, 1.0]}}, "material_index": 1},
                "spine": {"part": {"base": {"primitive": "cube", "dimensions": [0.20, 0.15, 1.0]}}, "material_index": 0},
                "chest": {
                    "part": {
                        "base": {"primitive": "cube", "dimensions": [0.32, 0.22, 1.0]},
                        "operations": [
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.20, 0.05, 0.14], "offset": [0.0, 0.12, 0.52]}},
                            {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.09, 0.09, 0.09], "offset": [0.17, 0.0, 0.82]}},
                            {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.09, 0.09, 0.09], "offset": [-0.17, 0.0, 0.82]}},
                        ],
                    },
                    "modifiers": [{"bevel": {"width": 0.012, "segments": 2}}],
                    "material_index": 0,
                },
                "neck": {"part": {"base": {"primitive": "cylinder", "dimensions": [0.06, 0.06, 1.0]}}, "material_index": 1},
                "head": {
                    "part": {
                        "base": {"primitive": "cube", "dimensions": [0.20, 0.20, 0.95]},
                        "operations": [
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.16, 0.06, 0.22], "offset": [0.0, 0.10, 0.52]}},
                            {"op": "union", "target": {"primitive": "cylinder", "dimensions": [0.015, 0.015, 0.30], "offset": [0.07, 0.0, 0.90]}},
                        ],
                    },
                    "modifiers": [{"bevel": {"width": 0.010, "segments": 1}}],
                    "material_index": 0,
                },

                "upper_arm_l": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.10, 0.10, 1.0]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.12, 0.12, 0.12], "offset": [0.0, 0.0, 0.0]}},
                        ],
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                    "material_index": 0,
                },
                "upper_arm_r": {"mirror": "upper_arm_l"},
                "lower_arm_l": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.08, 0.08, 1.0]},
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                    "material_index": 1,
                },
                "lower_arm_r": {"mirror": "lower_arm_l"},
                "hand_l": {"part": {"base": {"primitive": "cube", "dimensions": [0.08, 0.06, 0.70]}}, "material_index": 1},
                "hand_r": {"mirror": "hand_l"},

                "upper_leg_l": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.11, 0.11, 1.0]},
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                    "material_index": 0,
                },
                "upper_leg_r": {"mirror": "upper_leg_l"},
                "lower_leg_l": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.09, 0.09, 1.0]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.10, 0.10, 0.10], "offset": [0.0, 0.0, 1.0]}},
                        ],
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                    "material_index": 1,
                },
                "lower_leg_r": {"mirror": "lower_leg_l"},
                "foot_l": {"part": {"base": {"primitive": "cube", "dimensions": [0.10, 0.15, 0.50]}}, "material_index": 1},
                "foot_r": {"mirror": "foot_l"},
            },
        },
    },
)
