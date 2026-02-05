# Rigid segmented stylized female - feminine proportions with rigid skinning
#
# Each body segment is assigned to exactly one bone (rigid skinning).
# Feminine proportions: wider hips, narrower waist, smaller shoulders/hands.

spec(
    asset_id = "rigid_segmented_stylized_female",
    asset_type = "skeletal_mesh",
    seed = 7301,
    license = "CC0-1.0",
    description = "Stylized female humanoid with rigid skinning - feminine proportions",
    tags = ["skeletal_mesh", "character", "humanoid", "female", "rigid_skinning", "segmented", "armature_driven_v1"],
    outputs = [output("skeletal_mesh/rigid_segmented_stylized_female.glb", "glb")],
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

                # Left arm - narrower shoulders than male
                {"bone": "shoulder_l", "head": [0.12, 0.0, 0.58], "tail": [0.20, 0.0, 0.58], "parent": "chest"},
                {"bone": "upper_arm_l", "head": [0.20, 0.0, 0.58], "tail": [0.42, 0.0, 0.58], "parent": "shoulder_l"},
                {"bone": "lower_arm_l", "head": [0.42, 0.0, 0.58], "tail": [0.62, 0.0, 0.58], "parent": "upper_arm_l"},
                {"bone": "hand_l", "head": [0.62, 0.0, 0.58], "tail": [0.70, 0.0, 0.58], "parent": "lower_arm_l"},

                # Right arm - mirror
                {"bone": "shoulder_r", "head": [-0.12, 0.0, 0.58], "tail": [-0.20, 0.0, 0.58], "parent": "chest"},
                {"bone": "upper_arm_r", "head": [-0.20, 0.0, 0.58], "tail": [-0.42, 0.0, 0.58], "parent": "shoulder_r"},
                {"bone": "lower_arm_r", "head": [-0.42, 0.0, 0.58], "tail": [-0.62, 0.0, 0.58], "parent": "upper_arm_r"},
                {"bone": "hand_r", "head": [-0.62, 0.0, 0.58], "tail": [-0.70, 0.0, 0.58], "parent": "upper_arm_r"},

                # Left leg - slightly wider stance
                {"bone": "upper_leg_l", "head": [0.09, 0.0, 0.25], "tail": [0.09, 0.0, -0.12], "parent": "hips"},
                {"bone": "lower_leg_l", "head": [0.09, 0.0, -0.12], "tail": [0.09, 0.0, -0.48], "parent": "upper_leg_l"},
                {"bone": "foot_l", "head": [0.09, 0.0, -0.48], "tail": [0.09, 0.12, -0.48], "parent": "lower_leg_l"},

                # Right leg - mirror
                {"bone": "upper_leg_r", "head": [-0.09, 0.0, 0.25], "tail": [-0.09, 0.0, -0.12], "parent": "hips"},
                {"bone": "lower_leg_r", "head": [-0.09, 0.0, -0.12], "tail": [-0.09, 0.0, -0.48], "parent": "upper_leg_r"},
                {"bone": "foot_r", "head": [-0.09, 0.0, -0.48], "tail": [-0.09, 0.12, -0.48], "parent": "lower_leg_r"},
            ],
            "skinning_mode": "rigid",
            "material_slots": [
                {
                    "name": "skin",
                    "base_color": [0.88, 0.75, 0.65, 1.0],
                    "roughness": 0.45,
                },
                {
                    "name": "joint_ring",
                    "base_color": [0.22, 0.20, 0.25, 1.0],
                    "roughness": 0.55,
                },
            ],
            "bone_meshes": {
                # TORSO - Feminine proportions (wider hips, narrow waist, moderate chest)

                "hips": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.15},
                    "extrusion_steps": [
                        {"extrude": 0.25, "scale": 1.0},
                        {"extrude": 0.50, "scale": 1.12},
                        {"extrude": 0.25, "scale": 0.85},
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                },

                "spine": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.128},
                    "extrusion_steps": [
                        {"extrude": 0.3, "scale": 0.92},
                        {"extrude": 0.4, "scale": 0.88},
                        {"extrude": 0.3, "scale": 1.05},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                },

                "chest": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.12},
                    "extrusion_steps": [
                        {"extrude": 0.20, "scale": 1.15},
                        {"extrude": 0.50, "scale": 1.08},
                        {"extrude": 0.30, "scale": 0.55},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                    "attachments": [
                        {
                            "primitive": "cylinder",
                            "dimensions": [0.055, 0.055, 0.08],
                            "offset": [0.04, 0.0, 0.12],
                            "rotation": [0.0, 90.0, 0.0],
                            "material_index": 0,
                        },
                        {
                            "primitive": "cylinder",
                            "dimensions": [0.055, 0.055, 0.08],
                            "offset": [-0.04, 0.0, 0.12],
                            "rotation": [0.0, -90.0, 0.0],
                            "material_index": 0,
                        },
                    ],
                },

                "neck": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.045},
                    "extrusion_steps": [
                        {"extrude": 0.4, "scale": 1.0},
                        {"extrude": 0.6, "scale": 0.92},
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                },

                "head": {
                    "profile": "circle(14)",
                    "profile_radius": {"absolute": 0.042},
                    "extrusion_steps": [
                        {"extrude": 0.08, "scale": 1.20},
                        {"extrude": 0.35, "scale": 2.40},
                        {"extrude": 0.35, "scale": 1.05},
                        {"extrude": 0.22, "scale": 0.55},
                    ],
                    "cap_start": False,
                    "cap_end": True,
                    "material_index": 0,
                },

                # ARMS - Slimmer than male

                "shoulder_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.055},
                    "extrusion_steps": [
                        {"extrude": 0.4, "scale": 1.08},
                        {"extrude": 0.6, "scale": 0.92},
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                },

                "upper_arm_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.055},
                    "extrusion_steps": [
                        {"extrude": 0.20, "scale": 1.10},
                        {"extrude": 0.55, "scale": 0.88},
                        {"extrude": 0.25, "scale": 0.78},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                },

                "lower_arm_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.043},
                    "extrusion_steps": [
                        {"extrude": 0.25, "scale": 1.05},
                        {"extrude": 0.50, "scale": 0.82},
                        {"extrude": 0.25, "scale": 0.75},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                },

                "hand_l": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.030},
                    "extrusion_steps": [
                        {"extrude": 0.30, "scale": 1.25},
                        {"extrude": 0.45, "scale": 1.08},
                        {"extrude": 0.25, "scale": 0.55},
                    ],
                    "cap_start": False,
                    "cap_end": True,
                    "material_index": 0,
                },

                "shoulder_r": {"mirror": "shoulder_l"},
                "upper_arm_r": {"mirror": "upper_arm_l"},
                "lower_arm_r": {"mirror": "lower_arm_l"},
                "hand_r": {"mirror": "hand_l"},

                # LEGS - Feminine proportions

                "upper_leg_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.085},
                    "extrusion_steps": [
                        {"extrude": 0.18, "scale": 1.12},
                        {"extrude": 0.52, "scale": 0.88},
                        {"extrude": 0.30, "scale": 0.72},
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                },

                "lower_leg_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.058},
                    "extrusion_steps": [
                        {"extrude": 0.20, "scale": 1.05},
                        {"extrude": 0.50, "scale": 0.78},
                        {"extrude": 0.30, "scale": 0.72},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                },

                "foot_l": {
                    "profile": "rectangle",
                    "profile_radius": [0.035, 0.032],
                    "extrusion_steps": [
                        {"extrude": 0.25, "scale": 1.25},
                        {"extrude": 0.50, "scale": 1.08},
                        {"extrude": 0.25, "scale": 0.65},
                    ],
                    "cap_start": False,
                    "cap_end": True,
                    "material_index": 0,
                },

                "upper_leg_r": {"mirror": "upper_leg_l"},
                "lower_leg_r": {"mirror": "lower_leg_l"},
                "foot_r": {"mirror": "foot_l"},
            },
            "export": {
                "include_armature": True,
                "include_normals": True,
                "include_uvs": True,
                "triangulate": True,
                "include_skin_weights": True,
                "save_blend": False,
            },
            "constraints": {
                "max_triangles": 15000,
                "max_bones": 64,
                "max_materials": 4,
            },
        },
    },
)
