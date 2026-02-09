# Modular part boolean coverage spec.
#
# [VALIDATION]
# SHAPE: Robot-like torso/head with visible boolean cuts and add-ons.
# PROPORTIONS: Chest is a wide cuboid, head is compact and slightly flattened.
# ORIENTATION: Upright, mirrored upper arms, cut details on chest/front.
# FRONT VIEW: Vent-like subtraction slots and symmetric shoulder bulges.
# ISO VIEW: Union, difference, and intersect operations are all visible.

spec(
    asset_id = "part_boolean",
    asset_type = "skeletal_mesh",
    seed = 4102,
    license = "CC0-1.0",
    description = "Boolean operation coverage for modular bone parts",
    tags = ["skeletal_mesh", "character", "armature_driven_v1", "modular_parts", "boolean"],
    outputs = [output("skeletal_mesh/part_boolean.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton_preset": "humanoid_basic_v1",
            "bone_meshes": {
                "chest": {
                    "part": {
                        "base": {"primitive": "cube", "dimensions": [0.28, 0.20, 1.0]},
                        "operations": [
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.20, 0.05, 0.16], "offset": [0.0, 0.12, 0.56]}},
                            {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.09, 0.09, 0.09], "offset": [0.16, 0.0, 0.82]}},
                            {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.09, 0.09, 0.09], "offset": [-0.16, 0.0, 0.82]}},
                        ],
                    },
                },
                "head": {
                    "part": {
                        "base": {"primitive": "sphere", "dimensions": [0.22, 0.20, 0.24]},
                        "operations": [
                            {"op": "intersect", "target": {"primitive": "cube", "dimensions": [0.24, 0.22, 0.22], "offset": [0.0, 0.0, 0.52]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.16, 0.05, 0.12], "offset": [0.0, 0.11, 0.52]}},
                        ],
                    },
                },
                "upper_arm_l": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.09, 0.09, 1.0]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.11, 0.11, 0.11], "offset": [0.0, 0.0, 0.0]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.05, 0.20, 0.30], "offset": [0.0, 0.0, 0.40]}},
                        ],
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                },
                "upper_arm_r": {"mirror": "upper_arm_l"},
            },
        },
    },
)
