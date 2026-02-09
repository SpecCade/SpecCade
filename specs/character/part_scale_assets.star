# Asset-backed modular part scale composition.
#
# Prerequisite for generation:
#   Generate `specs/mesh/mesh_cube.star` into the same --out-root first,
#   so `meshes/cube.glb` exists for both `asset` and `asset_ref` lookups.
#
# [VALIDATION]
# SHAPE: Two block parts sourced from imported mesh assets (file path + asset_ref).
# PROPORTIONS: Chest asset is larger; head asset is smaller and offset upward.
# ORIENTATION: Upright placement on torso/head bones, facing +Y.
# FRONT VIEW: Imported cube-based armor plates centered on chest and head.
# ISO VIEW: Demonstrates asset.scale multiplied with part.scale factors.

spec(
    asset_id = "part_scale_assets",
    asset_type = "skeletal_mesh",
    seed = 4106,
    license = "CC0-1.0",
    description = "Asset and asset_ref part scaling composition",
    tags = ["skeletal_mesh", "armature_driven_v1", "modular_parts", "asset_ref", "scale_rules"],
    outputs = [output("skeletal_mesh/part_scale_assets.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton_preset": "humanoid_basic_v1",
            "bone_meshes": {
                "chest": {
                    "part": {
                        "base": {
                            "asset": "./meshes/cube.glb",
                            "offset": [0.0, 0.0, 0.50],
                            "rotation": [0.0, 0.0, 0.0],
                            "scale": 0.28,
                        },
                        "scale": {
                            "axes": ["x", "y", "z"],
                            "amount_from_z": {"x": 0.30, "y": 0.30, "z": 1.0},
                        },
                    },
                },
                "head": {
                    "part": {
                        "base": {
                            "asset_ref": "cube",
                            "offset": [0.0, 0.0, 0.50],
                            "rotation": [0.0, 0.0, 0.0],
                            "scale": 0.18,
                        },
                        "scale": {"axes": [], "amount_from_z": {"x": 0.0, "y": 0.0, "z": 0.0}},
                    },
                },
            },
        },
    },
)
