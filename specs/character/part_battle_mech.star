# Modular battle mech validation spec.
#
# [VALIDATION]
# SHAPE: Heavy biped mech with broad torso core, shoulder missile pods, and cannon forearms.
# PROPORTIONS: Wide upper body, narrow waist joint, thick armored legs, oversized forward feet.
# ORIENTATION: Upright humanoid; feet point forward (+Y); left/right sides are mirrored.
# FRONT VIEW: Symmetric mech with large shoulder pods and visible forearm laser emitters.
# TOP VIEW: Shoulder launchers read as multi-tube pods; chest core paneling is visible.
# ISO VIEW: Chest cuts, launcher bores, cannon muzzles, and foot armor should all read clearly.

spec(
    asset_id = "part_battle_mech",
    asset_type = "skeletal_mesh",
    seed = 4511,
    license = "CC0-1.0",
    description = "Biped battle mech using modular bone parts, booleans, attachments, and mirrors",
    tags = ["skeletal_mesh", "character", "mech", "robot", "armature_driven_v1", "modular_parts"],
    outputs = [output("skeletal_mesh/part_battle_mech.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton": [
                {"bone": "root", "head": [0.0, 0.0, 0.0], "tail": [0.0, 0.0, 0.08]},
                {"bone": "hips", "head": [0.0, 0.0, 0.08], "tail": [0.0, 0.0, 0.23], "parent": "root"},
                {"bone": "spine", "head": [0.0, 0.0, 0.23], "tail": [0.0, 0.0, 0.46], "parent": "hips"},
                {"bone": "chest", "head": [0.0, 0.0, 0.46], "tail": [0.0, 0.0, 0.73], "parent": "spine"},
                {"bone": "neck", "head": [0.0, 0.0, 0.73], "tail": [0.0, 0.0, 0.84], "parent": "chest"},
                {"bone": "head", "head": [0.0, 0.0, 0.84], "tail": [0.0, 0.0, 1.08], "parent": "neck"},

                {"bone": "shoulder_l", "head": [0.18, 0.00, 0.68], "tail": [0.34, 0.02, 0.67], "parent": "chest"},
                {"bone": "upper_arm_l", "head": [0.34, 0.02, 0.67], "tail": [0.60, 0.05, 0.63], "parent": "shoulder_l"},
                {"bone": "lower_arm_l", "head": [0.60, 0.05, 0.63], "tail": [0.84, 0.08, 0.59], "parent": "upper_arm_l"},
                {"bone": "hand_l", "head": [0.84, 0.08, 0.59], "tail": [0.98, 0.10, 0.58], "parent": "lower_arm_l"},

                {"bone": "shoulder_r", "head": [-0.18, 0.00, 0.68], "tail": [-0.34, 0.02, 0.67], "parent": "chest"},
                {"bone": "upper_arm_r", "head": [-0.34, 0.02, 0.67], "tail": [-0.60, 0.05, 0.63], "parent": "shoulder_r"},
                {"bone": "lower_arm_r", "head": [-0.60, 0.05, 0.63], "tail": [-0.84, 0.08, 0.59], "parent": "upper_arm_r"},
                {"bone": "hand_r", "head": [-0.84, 0.08, 0.59], "tail": [-0.98, 0.10, 0.58], "parent": "lower_arm_r"},

                {"bone": "upper_leg_l", "head": [0.12, 0.0, 0.23], "tail": [0.12, 0.0, -0.21], "parent": "hips"},
                {"bone": "lower_leg_l", "head": [0.12, 0.0, -0.21], "tail": [0.12, 0.0, -0.64], "parent": "upper_leg_l"},
                {"bone": "foot_l", "head": [0.12, 0.0, -0.64], "tail": [0.12, 0.26, -0.68], "parent": "lower_leg_l"},

                {"bone": "upper_leg_r", "head": [-0.12, 0.0, 0.23], "tail": [-0.12, 0.0, -0.21], "parent": "hips"},
                {"bone": "lower_leg_r", "head": [-0.12, 0.0, -0.21], "tail": [-0.12, 0.0, -0.64], "parent": "upper_leg_r"},
                {"bone": "foot_r", "head": [-0.12, 0.0, -0.64], "tail": [-0.12, 0.26, -0.68], "parent": "lower_leg_r"},
            ],
            "skinning_mode": "rigid",
            "material_slots": [
                material_slot(name = "armor_primary", base_color = [0.24, 0.43, 0.80, 1.0], metallic = 0.56, roughness = 0.30),
                material_slot(name = "armor_dark", base_color = [0.14, 0.16, 0.20, 1.0], metallic = 0.84, roughness = 0.24),
                material_slot(name = "signal_red", base_color = [0.90, 0.15, 0.18, 1.0], metallic = 0.24, roughness = 0.34),
            ],
            "bone_meshes": {
                "hips": {
                    "part": {
                        "base": {"primitive": "cube", "dimensions": [0.32, 0.22, 1.0]},
                        "operations": [
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.20, 0.08, 0.14], "offset": [0.0, 0.10, 0.46]}},
                        ],
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                    "modifiers": [{"bevel": {"width": 0.010, "segments": 2}}],
                    "material_index": 1,
                },
                "spine": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.145, 0.145, 1.0]},
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                    "material_index": 1,
                },
                "chest": {
                    "part": {
                        "base": {"primitive": "cube", "dimensions": [0.48, 0.32, 1.0]},
                        "operations": [
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.22, 0.06, 0.18], "offset": [0.0, 0.17, 0.58]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.12, 0.05, 0.14], "offset": [0.14, 0.17, 0.54]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.12, 0.05, 0.14], "offset": [-0.14, 0.17, 0.54]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.24, 0.11, 0.28], "offset": [0.0, 0.07, 0.42]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.30, 0.10, 0.12], "offset": [0.0, -0.09, 0.74]}},
                        ],
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                    "attachments": [
                        {"primitive": "sphere", "dimensions": [0.055, 0.032, 0.055], "offset": [0.0, 0.19, 0.58], "material_index": 2},
                    ],
                    "modifiers": [{"bevel": {"width": 0.014, "segments": 2}}],
                    "material_index": 0,
                },
                "neck": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.082, 0.082, 1.0]},
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                    "material_index": 1,
                },
                "head": {
                    "part": {
                        "base": {"primitive": "cube", "dimensions": [0.22, 0.20, 0.24]},
                        "operations": [
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.16, 0.05, 0.09], "offset": [0.0, 0.11, 0.55]}},
                            {"op": "union", "target": {"primitive": "cylinder", "dimensions": [0.014, 0.014, 0.24], "offset": [0.07, 0.0, 0.94]}},
                            {"op": "union", "target": {"primitive": "cylinder", "dimensions": [0.014, 0.014, 0.24], "offset": [-0.07, 0.0, 0.94]}},
                        ],
                        "scale": {"axes": []},
                    },
                    "modifiers": [{"bevel": {"width": 0.008, "segments": 1}}],
                    "material_index": 0,
                },

                "shoulder_l": {
                    "part": {
                        "base": {"primitive": "cube", "dimensions": [0.32, 0.24, 0.22], "offset": [0.0, 0.00, 0.52]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.34, 0.18, 0.10], "offset": [0.0, -0.02, 0.66]}},

                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.038, 0.038, 0.28], "offset": [0.08, 0.14, 0.44], "rotation": [-90.0, 0.0, 0.0]}},
                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.038, 0.038, 0.28], "offset": [0.00, 0.14, 0.44], "rotation": [-90.0, 0.0, 0.0]}},
                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.038, 0.038, 0.28], "offset": [-0.08, 0.14, 0.44], "rotation": [-90.0, 0.0, 0.0]}},
                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.038, 0.038, 0.28], "offset": [0.08, 0.14, 0.58], "rotation": [-90.0, 0.0, 0.0]}},
                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.038, 0.038, 0.28], "offset": [0.00, 0.14, 0.58], "rotation": [-90.0, 0.0, 0.0]}},
                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.038, 0.038, 0.28], "offset": [-0.08, 0.14, 0.58], "rotation": [-90.0, 0.0, 0.0]}},
                        ],
                        "scale": {"axes": []},
                    },
                    "attachments": [
                        {"primitive": "cone", "dimensions": [0.020, 0.020, 0.06], "offset": [0.08, 0.18, 0.44], "rotation": [-90.0, 0.0, 0.0], "material_index": 2},
                        {"primitive": "cone", "dimensions": [0.020, 0.020, 0.06], "offset": [0.00, 0.18, 0.44], "rotation": [-90.0, 0.0, 0.0], "material_index": 2},
                        {"primitive": "cone", "dimensions": [0.020, 0.020, 0.06], "offset": [-0.08, 0.18, 0.44], "rotation": [-90.0, 0.0, 0.0], "material_index": 2},
                        {"primitive": "cone", "dimensions": [0.020, 0.020, 0.06], "offset": [0.08, 0.18, 0.58], "rotation": [-90.0, 0.0, 0.0], "material_index": 2},
                        {"primitive": "cone", "dimensions": [0.020, 0.020, 0.06], "offset": [0.00, 0.18, 0.58], "rotation": [-90.0, 0.0, 0.0], "material_index": 2},
                        {"primitive": "cone", "dimensions": [0.020, 0.020, 0.06], "offset": [-0.08, 0.18, 0.58], "rotation": [-90.0, 0.0, 0.0], "material_index": 2},
                    ],
                    "modifiers": [{"bevel": {"width": 0.008, "segments": 1}}],
                    "material_index": 0,
                },
                "shoulder_r": {"mirror": "shoulder_l"},

                "upper_arm_l": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.12, 0.12, 1.0]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.14, 0.16, 0.36], "offset": [0.0, 0.05, 0.52]}},
                        ],
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                    "material_index": 0,
                },
                "upper_arm_r": {"mirror": "upper_arm_l"},
                "lower_arm_l": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.10, 0.10, 1.0]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.13, 0.18, 0.34], "offset": [0.0, 0.06, 0.58]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.07, 0.05, 0.12], "offset": [0.0, 0.14, 0.66]}},
                        ],
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                    "material_index": 1,
                },
                "lower_arm_r": {"mirror": "lower_arm_l"},
                "hand_l": {
                    "part": {
                        "base": {"primitive": "cube", "dimensions": [0.12, 0.10, 0.12]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "cylinder", "dimensions": [0.050, 0.050, 0.28], "offset": [0.0, 0.16, 0.50], "rotation": [-90.0, 0.0, 0.0]}},
                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.021, 0.021, 0.30], "offset": [0.0, 0.16, 0.50], "rotation": [-90.0, 0.0, 0.0]}},
                        ],
                        "scale": {"axes": []},
                    },
                    "attachments": [
                        {"primitive": "cone", "dimensions": [0.024, 0.024, 0.08], "offset": [0.0, 0.24, 0.50], "rotation": [-90.0, 0.0, 0.0], "material_index": 2},
                        {"primitive": "sphere", "dimensions": [0.024, 0.024, 0.024], "offset": [0.0, 0.27, 0.50], "material_index": 2},
                    ],
                    "material_index": 1,
                },
                "hand_r": {"mirror": "hand_l"},

                "upper_leg_l": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.15, 0.15, 1.0]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.14, 0.10, 0.34], "offset": [0.0, 0.06, 0.54]}},
                        ],
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                    "material_index": 0,
                },
                "upper_leg_r": {"mirror": "upper_leg_l"},
                "lower_leg_l": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.13, 0.13, 1.0]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.16, 0.16, 0.16], "offset": [0.0, 0.0, 1.0]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.16, 0.12, 0.32], "offset": [0.0, 0.08, 0.54]}},
                        ],
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                    "material_index": 1,
                },
                "lower_leg_r": {"mirror": "lower_leg_l"},
                "foot_l": {
                    "part": {
                        # For forward-pointing foot bones, local +z is the toe direction.
                        "base": {"primitive": "cube", "dimensions": [0.20, 0.14, 0.48], "offset": [0.0, 0.02, 0.46]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.12, 0.12, 0.12], "offset": [0.0, -0.08, 0.16]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.14, 0.12, 0.20], "offset": [0.0, 0.02, 0.78]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.15, 0.16, 0.10], "offset": [0.0, -0.01, 0.26]}},
                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.030, 0.030, 0.14], "offset": [0.05, 0.14, 0.70], "rotation": [-90.0, 0.0, 0.0]}},
                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.030, 0.030, 0.14], "offset": [-0.05, 0.14, 0.70], "rotation": [-90.0, 0.0, 0.0]}},
                        ],
                        "scale": {"axes": []},
                    },
                    "modifiers": [{"bevel": {"width": 0.007, "segments": 1}}],
                    "material_index": 1,
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
            "constraints": {"max_triangles": 34000, "max_bones": 64, "max_materials": 3},
        },
    },
)
