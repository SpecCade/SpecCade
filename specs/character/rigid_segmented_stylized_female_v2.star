# Rigid segmented stylized female v2 - improved anatomical shapes
#
# Fixes from validation feedback:
# - Feet have proper heel/arch/toe shape with clear forward direction
# - Chest has distinct forward projection
# - Legs have calf muscle bulge (back) vs flat shin (front)
# - Feminine proportions: wider hips, narrow waist, moderate bust

spec(
    asset_id = "rigid_segmented_stylized_female_v2",
    asset_type = "skeletal_mesh",
    seed = 7302,
    license = "CC0-1.0",
    description = "Stylized female humanoid v2 - improved anatomical shapes with clear front/back",
    tags = ["skeletal_mesh", "character", "humanoid", "female", "rigid_skinning", "armature_driven_v1"],
    outputs = [output("skeletal_mesh/rigid_segmented_stylized_female_v2.glb", "glb")],
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

                # Arms - narrower shoulders
                {"bone": "shoulder_l", "head": [0.12, 0.0, 0.58], "tail": [0.20, 0.0, 0.58], "parent": "chest"},
                {"bone": "upper_arm_l", "head": [0.20, 0.0, 0.58], "tail": [0.42, 0.0, 0.58], "parent": "shoulder_l"},
                {"bone": "lower_arm_l", "head": [0.42, 0.0, 0.58], "tail": [0.62, 0.0, 0.58], "parent": "upper_arm_l"},
                {"bone": "hand_l", "head": [0.62, 0.0, 0.58], "tail": [0.70, 0.0, 0.58], "parent": "lower_arm_l"},

                {"bone": "shoulder_r", "head": [-0.12, 0.0, 0.58], "tail": [-0.20, 0.0, 0.58], "parent": "chest"},
                {"bone": "upper_arm_r", "head": [-0.20, 0.0, 0.58], "tail": [-0.42, 0.0, 0.58], "parent": "shoulder_r"},
                {"bone": "lower_arm_r", "head": [-0.42, 0.0, 0.58], "tail": [-0.62, 0.0, 0.58], "parent": "upper_arm_r"},
                {"bone": "hand_r", "head": [-0.62, 0.0, 0.58], "tail": [-0.70, 0.0, 0.58], "parent": "upper_arm_r"},

                # Legs - go DOWN from hips
                {"bone": "upper_leg_l", "head": [0.09, 0.0, 0.25], "tail": [0.09, 0.0, -0.12], "parent": "hips"},
                {"bone": "lower_leg_l", "head": [0.09, 0.0, -0.12], "tail": [0.09, 0.0, -0.48], "parent": "upper_leg_l"},
                # Foot goes FORWARD (+Y) then down to toes
                {"bone": "foot_l", "head": [0.09, 0.0, -0.48], "tail": [0.09, 0.15, -0.52], "parent": "lower_leg_l"},

                {"bone": "upper_leg_r", "head": [-0.09, 0.0, 0.25], "tail": [-0.09, 0.0, -0.12], "parent": "hips"},
                {"bone": "lower_leg_r", "head": [-0.09, 0.0, -0.12], "tail": [-0.09, 0.0, -0.48], "parent": "upper_leg_r"},
                {"bone": "foot_r", "head": [-0.09, 0.0, -0.48], "tail": [-0.09, 0.15, -0.52], "parent": "lower_leg_r"},
            ],
            "skinning_mode": "rigid",
            "material_slots": [
                {"name": "skin", "base_color": [0.92, 0.78, 0.68, 1.0], "roughness": 0.42},
                {"name": "accent", "base_color": [0.75, 0.55, 0.45, 1.0], "roughness": 0.50},
            ],
            "bone_meshes": {
                # === TORSO - Feminine hourglass shape ===

                "hips": {
                    "profile": "circle(14)",
                    "profile_radius": {"absolute": 0.13},
                    "extrusion_steps": [
                        {"extrude": 0.15, "scale": 1.15},   # Widen at hip bone
                        {"extrude": 0.50, "scale": 1.25},   # Maximum hip width
                        {"extrude": 0.35, "scale": 0.72},   # Narrow to waist
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                    # Buttocks attachments for clear back side
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.08, 0.10, 0.08],
                            "offset": [0.05, -0.06, 0.08],  # Back (-Y), left side
                            "material_index": 0,
                        },
                        {
                            "primitive": "sphere",
                            "dimensions": [0.08, 0.10, 0.08],
                            "offset": [-0.05, -0.06, 0.08],  # Back (-Y), right side
                            "material_index": 0,
                        },
                    ],
                },

                "spine": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.09},
                    "extrusion_steps": [
                        {"extrude": 0.25, "scale": 1.02},
                        {"extrude": 0.50, "scale": 0.95},   # Narrow waist
                        {"extrude": 0.25, "scale": 1.15},   # Expand to ribcage
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                },

                "chest": {
                    "profile": "circle(14)",
                    "profile_radius": {"absolute": 0.10},
                    "extrusion_steps": [
                        {"extrude": 0.18, "scale": 1.22},   # Ribcage expansion
                        {"extrude": 0.55, "scale": 1.05},   # Main chest
                        {"extrude": 0.27, "scale": 0.52},   # Taper to neck
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                    # Bust - clear forward direction indicator
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.065, 0.07, 0.065],
                            "offset": [0.045, 0.05, 0.10],  # Forward (+Y), left
                            "material_index": 0,
                        },
                        {
                            "primitive": "sphere",
                            "dimensions": [0.065, 0.07, 0.065],
                            "offset": [-0.045, 0.05, 0.10],  # Forward (+Y), right
                            "material_index": 0,
                        },
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
                    "profile": "circle(16)",
                    "profile_radius": {"absolute": 0.038},
                    "extrusion_steps": [
                        {"extrude": 0.06, "scale": 1.25},   # Chin/jaw
                        {"extrude": 0.18, "scale": 2.20},   # Face widens
                        {"extrude": 0.45, "scale": 1.18},   # Cranium
                        {"extrude": 0.31, "scale": 0.45},   # Top of head
                    ],
                    "cap_start": False,
                    "cap_end": True,
                    "material_index": 0,
                    # Nose for clear forward direction
                    "attachments": [
                        {
                            "primitive": "cone",
                            "dimensions": [0.018, 0.018, 0.035],
                            "offset": [0.0, 0.085, 0.06],  # Forward from face
                            "rotation": [-90.0, 0.0, 0.0],
                            "material_index": 0,
                        },
                    ],
                },

                # === ARMS ===

                "shoulder_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.048},
                    "extrusion_steps": [
                        {"extrude": 0.35, "scale": 1.15},
                        {"extrude": 0.65, "scale": 0.88},
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                },

                "upper_arm_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.048},
                    "extrusion_steps": [
                        {"extrude": 0.18, "scale": 1.18},   # Deltoid bulge
                        {"extrude": 0.55, "scale": 0.82},   # Taper
                        {"extrude": 0.27, "scale": 0.72},   # Elbow
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                },

                "lower_arm_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.038},
                    "extrusion_steps": [
                        {"extrude": 0.22, "scale": 1.12},   # Forearm muscle
                        {"extrude": 0.52, "scale": 0.78},   # Taper
                        {"extrude": 0.26, "scale": 0.68},   # Wrist
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                },

                "hand_l": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.025},
                    "extrusion_steps": [
                        {"extrude": 0.25, "scale": 1.45},   # Palm widens
                        {"extrude": 0.50, "scale": 1.05},   # Palm body
                        {"extrude": 0.25, "scale": 0.48},   # Fingers taper
                    ],
                    "cap_start": False,
                    "cap_end": True,
                    "material_index": 0,
                },

                "shoulder_r": {"mirror": "shoulder_l"},
                "upper_arm_r": {"mirror": "upper_arm_l"},
                "lower_arm_r": {"mirror": "lower_arm_l"},
                "hand_r": {"mirror": "hand_l"},

                # === LEGS - with muscle definition ===

                "upper_leg_l": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.078},
                    "extrusion_steps": [
                        {"extrude": 0.15, "scale": 1.18},   # Upper thigh
                        {"extrude": 0.50, "scale": 0.92},   # Mid thigh
                        {"extrude": 0.35, "scale": 0.68},   # Above knee
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                    # Thigh muscle bulge (front/outer)
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.045, 0.06, 0.12],
                            "offset": [0.02, 0.04, -0.15],  # Front-outer thigh
                            "material_index": 0,
                        },
                    ],
                },

                "lower_leg_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.052},
                    "extrusion_steps": [
                        {"extrude": 0.18, "scale": 1.12},   # Below knee
                        {"extrude": 0.52, "scale": 0.75},   # Shin/calf
                        {"extrude": 0.30, "scale": 0.62},   # Ankle
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                    # Calf muscle bulge (BACK side only - key for front/back)
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.038, 0.055, 0.10],
                            "offset": [0.0, -0.035, -0.12],  # Back (-Y) = calf
                            "material_index": 0,
                        },
                    ],
                },

                # Foot - proper anatomical shape with depth
                "foot_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.042},  # Larger base
                    "extrusion_steps": [
                        {"extrude": 0.08, "scale": 1.35},   # Ankle transition
                        {"extrude": 0.15, "scale": 1.60},   # Heel area
                        {"extrude": 0.45, "scale": 1.25},   # Ball of foot
                        {"extrude": 0.32, "scale": 0.48},   # Toes taper
                    ],
                    "cap_start": False,
                    "cap_end": True,
                    "material_index": 0,
                    # Heel bump - clear back indicator
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.032, 0.045, 0.035],
                            "offset": [0.0, -0.038, 0.02],  # Back of foot = heel
                            "material_index": 0,
                        },
                        # Top of foot (instep) - adds depth
                        {
                            "primitive": "sphere",
                            "dimensions": [0.028, 0.05, 0.022],
                            "offset": [0.0, 0.02, 0.025],  # Top of foot
                            "material_index": 0,
                        },
                    ],
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
            },
            "constraints": {"max_triangles": 18000, "max_bones": 64, "max_materials": 4},
        },
    },
)
