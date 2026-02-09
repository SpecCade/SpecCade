# Modular part scale rule coverage.
#
# [VALIDATION]
# SHAPE: Six vertical test columns demonstrating different part scale policies.
# PROPORTIONS: Uniform/default columns widen with length; fixed column keeps constant width.
# ORIENTATION: Columns stand upright along +Z and are spatially separated on X/Y.
# FRONT VIEW: Distinct width differences between fixed, z-only, and hybrid examples.
# ISO VIEW: Easy side-by-side comparison of default, empty-scale, fixed, z-only, and hybrid.

spec(
    asset_id = "part_scale_rules",
    asset_type = "skeletal_mesh",
    seed = 4105,
    license = "CC0-1.0",
    description = "Scale rule validation for modular bone parts",
    tags = ["skeletal_mesh", "armature_driven_v1", "modular_parts", "scale_rules"],
    outputs = [output("skeletal_mesh/part_scale_rules.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton": [
                {"bone": "root", "head": [0.0, 0.0, 0.0], "tail": [0.0, 0.0, 0.1]},
                {"bone": "uniform_short", "head": [-0.90, 0.0, 0.1], "tail": [-0.90, 0.0, 0.60], "parent": "root"},
                {"bone": "uniform_long", "head": [-0.45, 0.0, 0.1], "tail": [-0.45, 0.0, 1.60], "parent": "root"},
                {"bone": "empty_scale", "head": [0.00, 0.0, 0.1], "tail": [0.00, 0.0, 1.40], "parent": "root"},
                {"bone": "fixed_piece", "head": [0.45, 0.0, 0.1], "tail": [0.45, 0.0, 1.40], "parent": "root"},
                {"bone": "z_only_piece", "head": [0.90, 0.0, 0.1], "tail": [0.90, 0.0, 1.40], "parent": "root"},
                {"bone": "hybrid_piece", "head": [1.35, 0.0, 0.1], "tail": [1.35, 0.0, 1.40], "parent": "root"},
            ],
            "bone_meshes": {
                "uniform_short": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.10, 0.10, 1.0]},
                    },
                },
                "uniform_long": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.10, 0.10, 1.0]},
                    },
                },
                "empty_scale": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.10, 0.10, 1.0]},
                        "scale": {},
                    },
                },
                "fixed_piece": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.10, 0.10, 1.0]},
                        "scale": {"axes": []},
                    },
                },
                "z_only_piece": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.10, 0.10, 1.0]},
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                },
                "hybrid_piece": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.10, 0.10, 1.0]},
                        "scale": {
                            "axes": ["x", "y", "z"],
                            "amount_from_z": {"x": 0.35, "y": 0.50, "z": 1.0},
                        },
                    },
                },
            },
        },
    },
)
