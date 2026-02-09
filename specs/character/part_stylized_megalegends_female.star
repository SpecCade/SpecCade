# Stylized adventure-heroine validation spec.
#
# [VALIDATION]
# SHAPE: Stylized feminine heroine with clear anatomy, short bob hair, and chunky boots.
# PROPORTIONS: Narrow shoulders, moderate chest, narrow waist, wider hips, long legs.
# ORIENTATION: Upright; feet point forward (+Y); mirrored left/right limbs.
# FRONT VIEW: Feminine silhouette with readable chest/waist/hip contrast and clean head shape.
# TOP VIEW: Hair reads as a separate cap volume around the head.
# ISO VIEW: Mixed workflow present (extrusion body + part overlays on head/feet).

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
                {"bone": "root", "head": [0.0, 0.0, 0.0], "tail": [0.0, 0.0, 0.1]},
                {"bone": "hips", "head": [0.0, 0.0, 0.1], "tail": [0.0, 0.0, 0.25], "parent": "root"},
                {"bone": "spine", "head": [0.0, 0.0, 0.25], "tail": [0.0, 0.0, 0.42], "parent": "hips"},
                {"bone": "chest", "head": [0.0, 0.0, 0.42], "tail": [0.0, 0.0, 0.65], "parent": "spine"},
                {"bone": "neck", "head": [0.0, 0.0, 0.65], "tail": [0.0, 0.0, 0.73], "parent": "chest"},
                {"bone": "head", "head": [0.0, 0.0, 0.73], "tail": [0.0, 0.0, 0.92], "parent": "neck"},

                {"bone": "shoulder_l", "head": [0.12, 0.0, 0.58], "tail": [0.20, 0.0, 0.58], "parent": "chest"},
                {"bone": "upper_arm_l", "head": [0.20, 0.0, 0.58], "tail": [0.42, 0.0, 0.58], "parent": "shoulder_l"},
                {"bone": "lower_arm_l", "head": [0.42, 0.0, 0.58], "tail": [0.62, 0.0, 0.58], "parent": "upper_arm_l"},
                {"bone": "hand_l", "head": [0.62, 0.0, 0.58], "tail": [0.70, 0.0, 0.58], "parent": "lower_arm_l"},

                {"bone": "shoulder_r", "head": [-0.12, 0.0, 0.58], "tail": [-0.20, 0.0, 0.58], "parent": "chest"},
                {"bone": "upper_arm_r", "head": [-0.20, 0.0, 0.58], "tail": [-0.42, 0.0, 0.58], "parent": "shoulder_r"},
                {"bone": "lower_arm_r", "head": [-0.42, 0.0, 0.58], "tail": [-0.62, 0.0, 0.58], "parent": "upper_arm_r"},
                {"bone": "hand_r", "head": [-0.62, 0.0, 0.58], "tail": [-0.70, 0.0, 0.58], "parent": "lower_arm_r"},

                {"bone": "upper_leg_l", "head": [0.09, 0.0, 0.25], "tail": [0.09, 0.0, -0.12], "parent": "hips"},
                {"bone": "lower_leg_l", "head": [0.09, 0.0, -0.12], "tail": [0.09, 0.0, -0.48], "parent": "upper_leg_l"},
                {"bone": "foot_l", "head": [0.09, 0.0, -0.48], "tail": [0.09, 0.15, -0.52], "parent": "lower_leg_l"},

                {"bone": "upper_leg_r", "head": [-0.09, 0.0, 0.25], "tail": [-0.09, 0.0, -0.12], "parent": "hips"},
                {"bone": "lower_leg_r", "head": [-0.09, 0.0, -0.12], "tail": [-0.09, 0.0, -0.48], "parent": "upper_leg_r"},
                {"bone": "foot_r", "head": [-0.09, 0.0, -0.48], "tail": [-0.09, 0.15, -0.52], "parent": "lower_leg_r"},
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
                    "profile_radius": {"absolute": 0.13},
                    "extrusion_steps": [
                        {"extrude": 0.15, "scale": 1.15},
                        {"extrude": 0.50, "scale": 1.25},
                        {"extrude": 0.35, "scale": 0.72},
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 2,
                    "attachments": [
                        {"primitive": "cube", "dimensions": [0.09, 0.07, 0.08], "offset": [0.0, 0.10, 0.10], "material_index": 4},
                        {"primitive": "sphere", "dimensions": [0.08, 0.10, 0.08], "offset": [0.05, -0.06, 0.08], "material_index": 2},
                        {"primitive": "sphere", "dimensions": [0.08, 0.10, 0.08], "offset": [-0.05, -0.06, 0.08], "material_index": 2},
                        {"primitive": "cube", "dimensions": [0.11, 0.05, 0.07], "offset": [0.0, -0.12, 0.12], "material_index": 2},
                    ],
                },
                "spine": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.09},
                    "extrusion_steps": [
                        {"extrude": 0.25, "scale": 1.02},
                        {"extrude": 0.50, "scale": 0.95},
                        {"extrude": 0.25, "scale": 1.15},
                    ],
                    "material_index": 2,
                },
                "chest": {
                    "profile": "circle(14)",
                    "profile_radius": {"absolute": 0.10},
                    "extrusion_steps": [
                        {"extrude": 0.18, "scale": 1.22},
                        {"extrude": 0.55, "scale": 1.05},
                        {"extrude": 0.27, "scale": 0.52},
                    ],
                    "material_index": 2,
                    "attachments": [
                        {"primitive": "sphere", "dimensions": [0.11, 0.16, 0.11], "offset": [0.064, 0.16, 0.11], "material_index": 2},
                        {"primitive": "sphere", "dimensions": [0.11, 0.16, 0.11], "offset": [-0.064, 0.16, 0.11], "material_index": 2},
                        {"primitive": "cube", "dimensions": [0.12, 0.06, 0.10], "offset": [0.0, 0.20, 0.12], "material_index": 4},
                        {"primitive": "cube", "dimensions": [0.12, 0.04, 0.10], "offset": [0.0, 0.07, 0.12], "material_index": 4},
                        {"primitive": "cube", "dimensions": [0.10, 0.05, 0.10], "offset": [0.0, -0.14, 0.12], "material_index": 2},
                        {"primitive": "sphere", "dimensions": [0.08, 0.05, 0.06], "offset": [0.0, -0.17, 0.16], "material_index": 2},
                        {"primitive": "cube", "dimensions": [0.05, 0.05, 0.08], "offset": [0.06, -0.18, 0.16], "material_index": 2},
                        {"primitive": "cube", "dimensions": [0.05, 0.05, 0.08], "offset": [-0.06, -0.18, 0.16], "material_index": 2},
                    ],
                },
                "neck": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.038},
                    "extrusion_steps": [
                        {"extrude": 0.35, "scale": 1.08},
                        {"extrude": 0.65, "scale": 0.90},
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                },
                "head": {
                    "part": {
                        "base": {"primitive": "sphere", "dimensions": [0.24, 0.20, 0.25], "offset": [0.0, 0.0, 0.53]},
                        "operations": [
                            {"op": "intersect", "target": {"primitive": "cube", "dimensions": [0.27, 0.23, 0.24], "offset": [0.0, 0.01, 0.53]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.12, 0.024, 0.03], "offset": [0.0, 0.11, 0.57]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.11, 0.05, 0.022], "offset": [0.0, 0.00, 0.40]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.24, 0.09, 0.15], "offset": [0.0, -0.17, 0.57]}},
                            {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.086, 0.062, 0.072], "offset": [0.0, 0.19, 0.50]}},
                            {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.14, 0.05, 0.06], "offset": [0.0, 0.20, 0.60]}},
                            {"op": "difference", "target": {"primitive": "sphere", "dimensions": [0.060, 0.036, 0.036], "offset": [0.052, 0.16, 0.56]}},
                            {"op": "difference", "target": {"primitive": "sphere", "dimensions": [0.060, 0.036, 0.036], "offset": [-0.052, 0.16, 0.56]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.14, 0.06, 0.04], "offset": [0.0, -0.03, 0.62]}},
                        ],
                        "scale": {"axes": []},
                    },
                    "attachments": [
                        {"primitive": "cube", "dimensions": [0.26, 0.18, 0.15], "offset": [0.0, -0.12, 0.60], "material_index": 1},
                        {"primitive": "sphere", "dimensions": [0.21, 0.11, 0.10], "offset": [0.0, 0.07, 0.68], "material_index": 1},
                        {"primitive": "cube", "dimensions": [0.22, 0.08, 0.11], "offset": [0.0, 0.18, 0.57], "material_index": 1},
                        {"primitive": "sphere", "dimensions": [0.13, 0.08, 0.16], "offset": [0.16, 0.00, 0.51], "material_index": 1},
                        {"primitive": "sphere", "dimensions": [0.13, 0.08, 0.16], "offset": [-0.16, 0.00, 0.51], "material_index": 1},
                        {"primitive": "cube", "dimensions": [0.12, 0.05, 0.11], "offset": [0.0, 0.20, 0.54], "material_index": 1},
                        {"primitive": "cone", "dimensions": [0.04, 0.04, 0.14], "offset": [0.06, 0.20, 0.45], "rotation": [0.0, 0.0, -10.0], "material_index": 1},
                        {"primitive": "cone", "dimensions": [0.04, 0.04, 0.14], "offset": [-0.06, 0.20, 0.45], "rotation": [0.0, 0.0, 10.0], "material_index": 1},
                        {"primitive": "cone", "dimensions": [0.05, 0.05, 0.24], "offset": [0.12, 0.12, 0.38], "rotation": [0.0, 0.0, -22.0], "material_index": 1},
                        {"primitive": "cone", "dimensions": [0.05, 0.05, 0.24], "offset": [-0.12, 0.12, 0.38], "rotation": [0.0, 0.0, 22.0], "material_index": 1},
                        {"primitive": "sphere", "dimensions": [0.22, 0.11, 0.13], "offset": [0.0, -0.22, 0.47], "material_index": 1},
                        {"primitive": "cone", "dimensions": [0.10, 0.10, 0.46], "offset": [0.0, -0.48, 0.50], "rotation": [-90.0, 0.0, 0.0], "material_index": 1},
                        {"primitive": "sphere", "dimensions": [0.09, 0.06, 0.07], "offset": [0.0, -0.54, 0.56], "material_index": 1},
                        {"primitive": "sphere", "dimensions": [0.050, 0.030, 0.032], "offset": [0.055, 0.20, 0.56], "material_index": 4},
                        {"primitive": "sphere", "dimensions": [0.050, 0.030, 0.032], "offset": [-0.055, 0.20, 0.56], "material_index": 4},
                        {"primitive": "cone", "dimensions": [0.032, 0.032, 0.10], "offset": [0.0, 0.20, 0.52], "rotation": [-90.0, 0.0, 0.0], "material_index": 0},
                        {"primitive": "cube", "dimensions": [0.06, 0.03, 0.02], "offset": [0.0, -0.08, 0.64], "material_index": 1},
                    ],
                    "modifiers": [{"bevel": {"width": 0.005, "segments": 1}}],
                    "material_index": 0,
                },

                "shoulder_l": {
                    "part": {
                        "base": {"primitive": "sphere", "dimensions": [0.055, 0.055, 0.055]},
                        "scale": {"axes": []},
                    },
                    "material_index": 4,
                },
                "shoulder_r": {"mirror": "shoulder_l"},
                "upper_arm_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.055},
                    "extrusion_steps": [
                        {"extrude": 0.20, "scale": 1.10},
                        {"extrude": 0.55, "scale": 0.88},
                        {"extrude": 0.25, "scale": 0.78},
                    ],
                    "material_index": 0,
                },
                "upper_arm_r": {"mirror": "upper_arm_l"},
                "lower_arm_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.043},
                    "extrusion_steps": [
                        {"extrude": 0.25, "scale": 1.05},
                        {"extrude": 0.50, "scale": 0.82},
                        {"extrude": 0.25, "scale": 0.75},
                    ],
                    "material_index": 0,
                },
                "lower_arm_r": {"mirror": "lower_arm_l"},
                "hand_l": {
                    "part": {
                        "base": {"primitive": "cube", "dimensions": [0.06, 0.045, 0.08]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.05, 0.035, 0.05], "offset": [0.0, 0.04, 0.55]}},
                        ],
                        "scale": {"axes": []},
                    },
                    "material_index": 4,
                },
                "hand_r": {"mirror": "hand_l"},

                "upper_leg_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.085},
                    "extrusion_steps": [
                        {"extrude": 0.18, "scale": 1.12},
                        {"extrude": 0.52, "scale": 0.88},
                        {"extrude": 0.30, "scale": 0.72},
                    ],
                    "material_index": 0,
                    "attachments": [
                        {"primitive": "sphere", "dimensions": [0.045, 0.06, 0.12], "offset": [0.02, 0.05, -0.15], "material_index": 0},
                        {"primitive": "sphere", "dimensions": [0.04, 0.05, 0.09], "offset": [0.0, -0.05, -0.12], "material_index": 0},
                        {"primitive": "sphere", "dimensions": [0.045, 0.07, 0.08], "offset": [0.0, 0.07, -0.20], "material_index": 0},
                    ],
                },
                "upper_leg_r": {"mirror": "upper_leg_l"},
                "lower_leg_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.052},
                    "extrusion_steps": [
                        {"extrude": 0.18, "scale": 1.12},
                        {"extrude": 0.52, "scale": 0.75},
                        {"extrude": 0.30, "scale": 0.62},
                    ],
                    "material_index": 0,
                    "attachments": [
                        {"primitive": "sphere", "dimensions": [0.042, 0.056, 0.11], "offset": [0.0, 0.05, 0.46], "material_index": 0},
                        {"primitive": "sphere", "dimensions": [0.040, 0.055, 0.11], "offset": [0.0, -0.05, 0.30], "material_index": 0},
                        {"primitive": "sphere", "dimensions": [0.046, 0.06, 0.09], "offset": [0.0, 0.06, 0.14], "material_index": 0},
                        {"primitive": "sphere", "dimensions": [0.038, 0.055, 0.10], "offset": [0.0, -0.04, -0.12], "material_index": 0},
                    ],
                },
                "lower_leg_r": {"mirror": "lower_leg_l"},
                "foot_l": {
                    "part": {
                        # Foot bone points forward; local +z follows toe direction.
                        "base": {"primitive": "cube", "dimensions": [0.13, 0.09, 0.26], "offset": [0.0, 0.00, 0.34]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.17, 0.05, 0.38], "offset": [0.0, -0.08, 0.42]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.16, 0.08, 0.18], "offset": [0.0, 0.03, 0.74]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.10, 0.05, 0.14], "offset": [0.0, -0.10, 0.14]}},
                            {"op": "union", "target": {"primitive": "cone", "dimensions": [0.09, 0.05, 0.25], "offset": [0.0, 0.02, 1.06], "rotation": [0.0, 90.0, 0.0]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.12, 0.05, 0.12], "offset": [0.0, -0.11, 0.04]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.10, 0.11, 0.08], "offset": [0.0, 0.01, 0.14]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.12, 0.10, 0.09], "offset": [0.0, 0.02, 0.34]}},
                        ],
                        "scale": {"axes": []},
                    },
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
