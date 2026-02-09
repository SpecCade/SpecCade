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
                {"bone": "upper_arm_l", "head": [0.34, 0.02, 0.67], "tail": [0.60, 0.05, 0.66], "parent": "shoulder_l"},
                {"bone": "lower_arm_l", "head": [0.60, 0.05, 0.66], "tail": [0.84, 0.08, 0.65], "parent": "upper_arm_l"},
                {"bone": "hand_l", "head": [0.84, 0.08, 0.65], "tail": [0.98, 0.10, 0.64], "parent": "lower_arm_l"},

                {"bone": "shoulder_r", "head": [-0.18, 0.00, 0.68], "tail": [-0.34, 0.02, 0.67], "parent": "chest"},
                {"bone": "upper_arm_r", "head": [-0.34, 0.02, 0.67], "tail": [-0.60, 0.05, 0.66], "parent": "shoulder_r"},
                {"bone": "lower_arm_r", "head": [-0.60, 0.05, 0.66], "tail": [-0.84, 0.08, 0.65], "parent": "upper_arm_r"},
                {"bone": "hand_r", "head": [-0.84, 0.08, 0.65], "tail": [-0.98, 0.10, 0.64], "parent": "lower_arm_r"},

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
                        "base": {"primitive": "cube", "dimensions": [0.54, 0.36, 1.0]},
                        "operations": [
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.24, 0.07, 0.18], "offset": [0.0, 0.18, 0.58]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.13, 0.05, 0.14], "offset": [0.16, 0.18, 0.53]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.13, 0.05, 0.14], "offset": [-0.16, 0.18, 0.53]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.08, 0.09, 0.20], "offset": [0.0, 0.05, 0.60]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.26, 0.14, 0.32], "offset": [0.0, 0.06, 0.40]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.34, 0.11, 0.13], "offset": [0.0, -0.11, 0.76]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.24, 0.14, 0.18], "offset": [0.0, 0.12, 0.74]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.26, 0.24, 0.30], "offset": [0.0, 0.34, 0.54]}},
                            {"op": "union", "target": {"primitive": "cone", "dimensions": [0.09, 0.09, 0.20], "offset": [0.0, 0.50, 0.56], "rotation": [-90.0, 0.0, 0.0]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.18, 0.04, 0.08], "offset": [0.0, 0.50, 0.56]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.16, 0.18, 0.20], "offset": [0.0, 0.48, 0.40]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.34, 0.24, 0.34], "offset": [0.0, -0.34, 0.58]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.24, 0.14, 0.26], "offset": [0.0, -0.52, 0.58]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.14, 0.04, 0.10], "offset": [0.0, -0.58, 0.58]}},
                        ],
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                    "attachments": [
                        {"primitive": "sphere", "dimensions": [0.10, 0.06, 0.10], "offset": [0.0, 0.44, 0.58], "material_index": 2},
                        {"primitive": "cube", "dimensions": [0.24, 0.04, 0.04], "offset": [0.0, 0.42, 0.58], "material_index": 2},
                        {"primitive": "cube", "dimensions": [0.14, 0.03, 0.03], "offset": [0.0, 0.52, 0.58], "material_index": 2},
                        {"primitive": "cylinder", "dimensions": [0.065, 0.065, 0.24], "offset": [0.12, -0.48, 0.54], "rotation": [90.0, 0.0, 0.0], "material_index": 1},
                        {"primitive": "cylinder", "dimensions": [0.065, 0.065, 0.24], "offset": [-0.12, -0.48, 0.54], "rotation": [90.0, 0.0, 0.0], "material_index": 1},
                        {"primitive": "cone", "dimensions": [0.05, 0.05, 0.14], "offset": [0.12, -0.56, 0.54], "rotation": [90.0, 0.0, 0.0], "material_index": 1},
                        {"primitive": "cone", "dimensions": [0.05, 0.05, 0.14], "offset": [-0.12, -0.56, 0.54], "rotation": [90.0, 0.0, 0.0], "material_index": 1},
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
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.12, 0.07, 0.08], "offset": [0.0, 0.16, 0.52]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.09, 0.03, 0.04], "offset": [0.0, 0.18, 0.52]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.12, 0.06, 0.08], "offset": [0.0, -0.14, 0.52]}},
                            {"op": "union", "target": {"primitive": "cylinder", "dimensions": [0.014, 0.014, 0.24], "offset": [0.07, 0.0, 0.94]}},
                            {"op": "union", "target": {"primitive": "cylinder", "dimensions": [0.014, 0.014, 0.24], "offset": [-0.07, 0.0, 0.94]}},
                        ],
                        "scale": {"axes": []},
                    },
                    "attachments": [
                        {"primitive": "cube", "dimensions": [0.08, 0.02, 0.03], "offset": [0.0, 0.20, 0.52], "material_index": 2},
                        {"primitive": "cube", "dimensions": [0.10, 0.03, 0.05], "offset": [0.0, -0.18, 0.50], "material_index": 1},
                    ],
                    "modifiers": [{"bevel": {"width": 0.008, "segments": 1}}],
                    "material_index": 0,
                },

                "shoulder_l": {
                    "part": {
                        "base": {"primitive": "cube", "dimensions": [0.38, 0.28, 0.24], "offset": [0.0, 0.00, 0.52]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.40, 0.22, 0.12], "offset": [0.0, -0.03, 0.70]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.30, 0.16, 0.15], "offset": [0.0, -0.12, 0.52]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.30, 0.10, 0.10], "offset": [0.0, 0.03, 0.84]}},

                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.044, 0.044, 0.34], "offset": [0.10, 0.16, 0.44], "rotation": [-90.0, 0.0, 0.0]}},
                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.044, 0.044, 0.34], "offset": [0.00, 0.16, 0.44], "rotation": [-90.0, 0.0, 0.0]}},
                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.044, 0.044, 0.34], "offset": [-0.10, 0.16, 0.44], "rotation": [-90.0, 0.0, 0.0]}},
                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.044, 0.044, 0.34], "offset": [0.10, 0.16, 0.60], "rotation": [-90.0, 0.0, 0.0]}},
                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.044, 0.044, 0.34], "offset": [0.00, 0.16, 0.60], "rotation": [-90.0, 0.0, 0.0]}},
                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.044, 0.044, 0.34], "offset": [-0.10, 0.16, 0.60], "rotation": [-90.0, 0.0, 0.0]}},
                        ],
                        "scale": {"axes": []},
                    },
                    "attachments": [
                        {"primitive": "cone", "dimensions": [0.022, 0.022, 0.10], "offset": [0.10, 0.22, 0.44], "rotation": [-90.0, 0.0, 0.0], "material_index": 2},
                        {"primitive": "cone", "dimensions": [0.022, 0.022, 0.10], "offset": [0.00, 0.22, 0.44], "rotation": [-90.0, 0.0, 0.0], "material_index": 2},
                        {"primitive": "cone", "dimensions": [0.022, 0.022, 0.10], "offset": [-0.10, 0.22, 0.44], "rotation": [-90.0, 0.0, 0.0], "material_index": 2},
                        {"primitive": "cone", "dimensions": [0.022, 0.022, 0.10], "offset": [0.10, 0.22, 0.60], "rotation": [-90.0, 0.0, 0.0], "material_index": 2},
                        {"primitive": "cone", "dimensions": [0.022, 0.022, 0.10], "offset": [0.00, 0.22, 0.60], "rotation": [-90.0, 0.0, 0.0], "material_index": 2},
                        {"primitive": "cone", "dimensions": [0.022, 0.022, 0.10], "offset": [-0.10, 0.22, 0.60], "rotation": [-90.0, 0.0, 0.0], "material_index": 2},
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
                        "base": {"primitive": "cylinder", "dimensions": [0.11, 0.11, 1.0]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.15, 0.20, 0.38], "offset": [0.0, 0.06, 0.58]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.10, 0.16, 0.24], "offset": [0.0, -0.03, 0.42]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.08, 0.05, 0.14], "offset": [0.0, 0.15, 0.66]}},
                        ],
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                    "material_index": 1,
                },
                "lower_arm_r": {"mirror": "lower_arm_l"},
                "hand_l": {
                    "part": {
                        "base": {"primitive": "cube", "dimensions": [0.14, 0.11, 0.14]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "cylinder", "dimensions": [0.060, 0.060, 0.48], "offset": [0.0, 0.00, 0.62]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.08, 0.10, 0.22], "offset": [0.0, 0.00, 0.30]}},
                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.023, 0.023, 0.50], "offset": [0.0, 0.00, 0.64]}},
                        ],
                        "scale": {"axes": []},
                    },
                    "attachments": [
                        {"primitive": "cone", "dimensions": [0.030, 0.030, 0.14], "offset": [0.0, 0.00, 1.02], "material_index": 2},
                        {"primitive": "sphere", "dimensions": [0.030, 0.030, 0.030], "offset": [0.0, 0.00, 1.08], "material_index": 2},
                    ],
                    "material_index": 1,
                },
                "hand_r": {"mirror": "hand_l"},

                "upper_leg_l": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.16, 0.16, 1.0]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.16, 0.10, 0.36], "offset": [0.0, 0.07, 0.56]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.14, 0.12, 0.20], "offset": [0.0, -0.03, 0.34]}},
                        ],
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                    "material_index": 0,
                },
                "upper_leg_r": {"mirror": "upper_leg_l"},
                "lower_leg_l": {
                    "part": {
                        "base": {"primitive": "cylinder", "dimensions": [0.14, 0.14, 1.0]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.18, 0.18, 0.18], "offset": [0.0, 0.0, 1.0]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.18, 0.14, 0.34], "offset": [0.0, 0.09, 0.56]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.11, 0.06, 0.20], "offset": [0.0, 0.16, 0.54]}},
                        ],
                        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                    },
                    "material_index": 1,
                },
                "lower_leg_r": {"mirror": "lower_leg_l"},
                "foot_l": {
                    "part": {
                        # For forward-pointing foot bones, local +z is the toe direction.
                        "base": {"primitive": "cube", "dimensions": [0.23, 0.16, 0.56], "offset": [0.0, 0.02, 0.48]},
                        "operations": [
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.14, 0.13, 0.14], "offset": [0.0, -0.09, 0.16]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.16, 0.14, 0.22], "offset": [0.0, 0.02, 0.86]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.24, 0.05, 0.52], "offset": [0.0, -0.10, 0.46]}},
                            {"op": "union", "target": {"primitive": "cone", "dimensions": [0.07, 0.07, 0.12], "offset": [0.0, 0.02, 1.02], "rotation": [0.0, 90.0, 0.0]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.18, 0.08, 0.16], "offset": [0.0, 0.02, 1.12]}},
                            {"op": "union", "target": {"primitive": "cube", "dimensions": [0.15, 0.10, 0.12], "offset": [0.0, -0.12, 0.06]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.17, 0.18, 0.12], "offset": [0.0, -0.01, 0.26]}},
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.12, 0.12, 0.08], "offset": [0.0, 0.02, 1.00]}},
                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.034, 0.034, 0.16], "offset": [0.06, 0.15, 0.76], "rotation": [-90.0, 0.0, 0.0]}},
                            {"op": "difference", "target": {"primitive": "cylinder", "dimensions": [0.034, 0.034, 0.16], "offset": [-0.06, 0.15, 0.76], "rotation": [-90.0, 0.0, 0.0]}},
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
