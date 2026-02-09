# Stylized adventure-heroine validation spec.
#
# [VALIDATION]
# SHAPE: Stylized feminine heroine with a large head, readable hair mass, tapered limbs, and chunky boots.
# PROPORTIONS: Narrow waist, wider hips, moderate chest, long legs, and compact forearms/hands.
# ORIENTATION: Upright; feet point forward (+Y); mirrored left/right limbs.
# FRONT VIEW: Clear feminine silhouette (head/hair wider than neck, torso taper, hip flare).
# TOP VIEW: Hair reads as distinct back mass + side locks, not just a plain sphere.
# ISO VIEW: Mixed workflow is visible: organic body from extrusion, structured head/boots from part composition.

spec(
    asset_id = "part_stylized_megalegends_female",
    asset_type = "skeletal_mesh",
    seed = 4512,
    license = "CC0-1.0",
    description = "Stylized heroine inspired by adventure-anime proportions using mixed extrusion and modular parts",
    tags = ["skeletal_mesh", "character", "female", "stylized", "armature_driven_v1", "modular_parts", "mixed_mode"],
    outputs = [output("skeletal_mesh/part_stylized_megalegends_female.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton": [
                {"bone": "root", "head": [0.0, 0.0, 0.0], "tail": [0.0, 0.0, 0.08]},
                {"bone": "hips", "head": [0.0, 0.0, 0.08], "tail": [0.0, 0.0, 0.25], "parent": "root"},
                {"bone": "spine", "head": [0.0, 0.0, 0.25], "tail": [0.0, 0.0, 0.44], "parent": "hips"},
                {"bone": "chest", "head": [0.0, 0.0, 0.44], "tail": [0.0, 0.0, 0.65], "parent": "spine"},
                {"bone": "neck", "head": [0.0, 0.0, 0.65], "tail": [0.0, 0.0, 0.74], "parent": "chest"},
                {"bone": "head", "head": [0.0, 0.0, 0.74], "tail": [0.0, 0.0, 1.02], "parent": "neck"},

                {"bone": "shoulder_l", "head": [0.10, 0.01, 0.60], "tail": [0.16, 0.03, 0.56], "parent": "chest"},
                {"bone": "upper_arm_l", "head": [0.16, 0.03, 0.56], "tail": [0.33, 0.07, 0.46], "parent": "shoulder_l"},
                {"bone": "lower_arm_l", "head": [0.33, 0.07, 0.46], "tail": [0.47, 0.11, 0.38], "parent": "upper_arm_l"},
                {"bone": "hand_l", "head": [0.47, 0.11, 0.38], "tail": [0.57, 0.14, 0.36], "parent": "lower_arm_l"},

                {"bone": "shoulder_r", "head": [-0.10, 0.01, 0.60], "tail": [-0.16, 0.03, 0.56], "parent": "chest"},
                {"bone": "upper_arm_r", "head": [-0.16, 0.03, 0.56], "tail": [-0.33, 0.07, 0.46], "parent": "shoulder_r"},
                {"bone": "lower_arm_r", "head": [-0.33, 0.07, 0.46], "tail": [-0.47, 0.11, 0.38], "parent": "upper_arm_r"},
                {"bone": "hand_r", "head": [-0.47, 0.11, 0.38], "tail": [-0.57, 0.14, 0.36], "parent": "lower_arm_r"},

                {"bone": "upper_leg_l", "head": [0.11, 0.0, 0.25], "tail": [0.11, 0.0, -0.13], "parent": "hips"},
                {"bone": "lower_leg_l", "head": [0.11, 0.0, -0.13], "tail": [0.11, 0.0, -0.50], "parent": "upper_leg_l"},
                {"bone": "foot_l", "head": [0.11, 0.0, -0.50], "tail": [0.11, 0.17, -0.54], "parent": "lower_leg_l"},

                {"bone": "upper_leg_r", "head": [-0.11, 0.0, 0.25], "tail": [-0.11, 0.0, -0.13], "parent": "hips"},
                {"bone": "lower_leg_r", "head": [-0.11, 0.0, -0.13], "tail": [-0.11, 0.0, -0.50], "parent": "upper_leg_r"},
                {"bone": "foot_r", "head": [-0.11, 0.0, -0.50], "tail": [-0.11, 0.17, -0.54], "parent": "lower_leg_r"},
            ],
            "skinning_mode": "rigid",
            "material_slots": [
                material_slot(name = "skin", base_color = [0.93, 0.79, 0.70, 1.0], roughness = 0.42),
                material_slot(name = "hair", base_color = [0.30, 0.17, 0.12, 1.0], roughness = 0.48),
                material_slot(name = "outfit", base_color = [0.20, 0.45, 0.84, 1.0], roughness = 0.38),
                material_slot(name = "boots", base_color = [0.92, 0.93, 0.95, 1.0], roughness = 0.30),
                material_slot(name = "accent", base_color = [0.97, 0.30, 0.38, 1.0], roughness = 0.36),
            ],
            "bone_meshes": {
                "hips": {
                    "profile": "circle(14)",
                    "profile_radius": {"absolute": 0.122},
                    "extrusion_steps": [
                        {"extrude": 0.18, "scale": 1.16},
                        {"extrude": 0.44, "scale": 1.28},
                        {"extrude": 0.38, "scale": 0.72},
                    ],
                    "attachments": [
                        {"primitive": "cube", "dimensions": [0.08, 0.06, 0.12], "offset": [0.11, 0.01, 0.10], "rotation": [0.0, 0.0, -10.0], "material_index": 2},
                        {"primitive": "cube", "dimensions": [0.08, 0.06, 0.12], "offset": [-0.11, 0.01, 0.10], "rotation": [0.0, 0.0, 10.0], "material_index": 2},
                    ],
                    "material_index": 2,
                },
                "spine": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.086},
                    "extrusion_steps": [
                        {"extrude": 0.26, "scale": 1.02},
                        {"extrude": 0.48, "scale": 0.86},
                        {"extrude": 0.26, "scale": 1.10},
                    ],
                    "material_index": 2,
                },
                "chest": {
                    "profile": "circle(14)",
                    "profile_radius": {"absolute": 0.106},
                    "extrusion_steps": [
                        {"extrude": 0.20, "scale": 1.16},
                        {"extrude": 0.56, "scale": 1.03},
                        {"extrude": 0.24, "scale": 0.62},
                    ],
                    "attachments": [
                        {"primitive": "sphere", "dimensions": [0.052, 0.061, 0.058], "offset": [0.046, 0.045, 0.10], "material_index": 2},
                        {"primitive": "sphere", "dimensions": [0.052, 0.061, 0.058], "offset": [-0.046, 0.045, 0.10], "material_index": 2},
                    ],
                    "material_index": 2,
                },
                "neck": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.038},
                    "extrusion_steps": [
                        {"extrude": 0.36, "scale": 1.03},
                        {"extrude": 0.64, "scale": 0.86},
                    ],
                    "material_index": 0,
                },
                "head": {
                    "part": {
                        "base": {"primitive": "sphere", "dimensions": [0.24, 0.21, 0.27]},
                        "operations": [
                            {"op": "intersect", "target": {"primitive": "cube", "dimensions": [0.28, 0.24, 0.26], "offset": [0.0, 0.0, 0.54]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.16, 0.04, 0.06], "offset": [0.0, 0.10, 0.58]}},
                        ],
                        "scale": {"axes": []},
                    },
                    "attachments": [
                        {"primitive": "sphere", "dimensions": [0.52, 0.44, 0.42], "offset": [0.0, -0.08, 0.62], "material_index": 1},
                        {"primitive": "cube", "dimensions": [0.22, 0.12, 0.30], "offset": [0.22, -0.03, 0.48], "rotation": [0.0, 0.0, -14.0], "material_index": 1},
                        {"primitive": "cube", "dimensions": [0.22, 0.12, 0.30], "offset": [-0.22, -0.03, 0.48], "rotation": [0.0, 0.0, 14.0], "material_index": 1},
                        {"primitive": "cylinder", "dimensions": [0.10, 0.10, 0.46], "offset": [0.0, -0.26, 0.44], "rotation": [90.0, 0.0, 0.0], "material_index": 1},
                        {"primitive": "sphere", "dimensions": [0.14, 0.14, 0.14], "offset": [0.0, -0.44, 0.44], "material_index": 1},
                        {"primitive": "sphere", "dimensions": [0.030, 0.024, 0.028], "offset": [0.05, 0.12, 0.57], "material_index": 4},
                        {"primitive": "sphere", "dimensions": [0.030, 0.024, 0.028], "offset": [-0.05, 0.12, 0.57], "material_index": 4},
                    ],
                    "modifiers": [{"bevel": {"width": 0.006, "segments": 1}}],
                    "material_index": 0,
                },

                "shoulder_l": {
                    "part": {
                        "base": {"primitive": "sphere", "dimensions": [0.058, 0.058, 0.058]},
                        "scale": {"axes": []},
                    },
                    "material_index": 4,
                },
                "shoulder_r": {"mirror": "shoulder_l"},

                "upper_arm_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.042},
                    "extrusion_steps": [
                        {"extrude": 0.24, "scale": 1.08},
                        {"extrude": 0.54, "scale": 0.82},
                        {"extrude": 0.22, "scale": 0.70},
                    ],
                    "material_index": 0,
                },
                "upper_arm_r": {"mirror": "upper_arm_l"},
                "lower_arm_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.034},
                    "extrusion_steps": [
                        {"extrude": 0.24, "scale": 1.08},
                        {"extrude": 0.52, "scale": 0.78},
                        {"extrude": 0.24, "scale": 0.66},
                    ],
                    "material_index": 0,
                },
                "lower_arm_r": {"mirror": "lower_arm_l"},
                "hand_l": {
                    "part": {
                        "base": {"primitive": "cube", "dimensions": [0.08, 0.05, 0.10]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.06, 0.04, 0.06], "offset": [0.0, 0.05, 0.52]}},
                        ],
                        "scale": {"axes": []},
                    },
                    "material_index": 4,
                },
                "hand_r": {"mirror": "hand_l"},

                "upper_leg_l": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.073},
                    "extrusion_steps": [
                        {"extrude": 0.22, "scale": 1.14},
                        {"extrude": 0.54, "scale": 0.84},
                        {"extrude": 0.24, "scale": 0.70},
                    ],
                    "material_index": 0,
                },
                "upper_leg_r": {"mirror": "upper_leg_l"},
                "lower_leg_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.052},
                    "extrusion_steps": [
                        {"extrude": 0.22, "scale": 1.12},
                        {"extrude": 0.50, "scale": 0.72},
                        {"extrude": 0.28, "scale": 0.62},
                    ],
                    "attachments": [
                        {"primitive": "sphere", "dimensions": [0.032, 0.045, 0.10], "offset": [0.0, -0.03, -0.10], "material_index": 0},
                    ],
                    "material_index": 0,
                },
                "lower_leg_r": {"mirror": "lower_leg_l"},
                "foot_l": {
                    "part": {
                        # Foot bone points forward; local +z follows toe direction.
                        "base": {"primitive": "cube", "dimensions": [0.12, 0.10, 0.36], "offset": [0.0, 0.01, 0.44]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.10, 0.10, 0.10], "offset": [0.0, -0.04, 0.16]}},
                            {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.12, 0.11, 0.12], "offset": [0.0, 0.05, 0.70]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.08, 0.10, 0.06], "offset": [0.0, 0.02, 0.24]}},
                        ],
                        "scale": {"axes": []},
                    },
                    "modifiers": [{"bevel": {"width": 0.008, "segments": 2}}],
                    "material_index": 3,
                },
                "foot_r": {"mirror": "foot_l"},
            },
            "export": {
                "include_armature": True,
                "include_normals": True,
                "include_uvs": True,
                "triangulate": True,
                "include_skin_weights": True,
            },
            "constraints": {"max_triangles": 30000, "max_bones": 64, "max_materials": 5},
        },
    },
)
