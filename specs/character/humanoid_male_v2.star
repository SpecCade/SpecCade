# Humanoid male v2 - improved anatomical shapes with clear masculine proportions
#
# Fixes from validation feedback:
# - Feet have proper heel/arch/toe shape with clear forward direction
# - Chest has defined pectorals for forward direction
# - Legs have calf muscle bulge (back) vs flat shin (front)
# - Masculine proportions: broad shoulders, narrow hips, thick neck, larger hands/feet

spec(
    asset_id = "humanoid_male_v2",
    asset_type = "skeletal_mesh",
    seed = 7103,
    license = "CC0-1.0",
    description = "Humanoid male v2 - improved anatomical shapes, broad shoulders, muscular build",
    tags = ["skeletal_mesh", "character", "humanoid", "male", "rigid_skinning", "armature_driven_v1"],
    outputs = [output("skeletal_mesh/humanoid_male_v2.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton": [
                {"bone": "root", "head": [0.0, 0.0, 0.0], "tail": [0.0, 0.0, 0.1]},
                {"bone": "hips", "head": [0.0, 0.0, 0.1], "tail": [0.0, 0.0, 0.28], "parent": "root"},
                {"bone": "spine", "head": [0.0, 0.0, 0.28], "tail": [0.0, 0.0, 0.48], "parent": "hips"},
                {"bone": "chest", "head": [0.0, 0.0, 0.48], "tail": [0.0, 0.0, 0.72], "parent": "spine"},
                {"bone": "neck", "head": [0.0, 0.0, 0.72], "tail": [0.0, 0.0, 0.82], "parent": "chest"},
                {"bone": "head", "head": [0.0, 0.0, 0.82], "tail": [0.0, 0.0, 1.02], "parent": "neck"},

                # Arms - BROADER shoulders than female
                {"bone": "shoulder_l", "head": [0.15, 0.0, 0.65], "tail": [0.26, 0.0, 0.65], "parent": "chest"},
                {"bone": "upper_arm_l", "head": [0.26, 0.0, 0.65], "tail": [0.52, 0.0, 0.65], "parent": "shoulder_l"},
                {"bone": "lower_arm_l", "head": [0.52, 0.0, 0.65], "tail": [0.76, 0.0, 0.65], "parent": "upper_arm_l"},
                {"bone": "hand_l", "head": [0.76, 0.0, 0.65], "tail": [0.88, 0.0, 0.65], "parent": "lower_arm_l"},

                {"bone": "shoulder_r", "head": [-0.15, 0.0, 0.65], "tail": [-0.26, 0.0, 0.65], "parent": "chest"},
                {"bone": "upper_arm_r", "head": [-0.26, 0.0, 0.65], "tail": [-0.52, 0.0, 0.65], "parent": "shoulder_r"},
                {"bone": "lower_arm_r", "head": [-0.52, 0.0, 0.65], "tail": [-0.76, 0.0, 0.65], "parent": "upper_arm_r"},
                {"bone": "hand_r", "head": [-0.76, 0.0, 0.65], "tail": [-0.88, 0.0, 0.65], "parent": "upper_arm_r"},

                # Legs - narrower stance than female (V-shape torso)
                {"bone": "upper_leg_l", "head": [0.085, 0.0, 0.28], "tail": [0.085, 0.0, -0.16], "parent": "hips"},
                {"bone": "lower_leg_l", "head": [0.085, 0.0, -0.16], "tail": [0.085, 0.0, -0.56], "parent": "upper_leg_l"},
                # Foot goes FORWARD (+Y) then angles down
                {"bone": "foot_l", "head": [0.085, 0.0, -0.56], "tail": [0.085, 0.18, -0.60], "parent": "lower_leg_l"},

                {"bone": "upper_leg_r", "head": [-0.085, 0.0, 0.28], "tail": [-0.085, 0.0, -0.16], "parent": "hips"},
                {"bone": "lower_leg_r", "head": [-0.085, 0.0, -0.16], "tail": [-0.085, 0.0, -0.56], "parent": "upper_leg_r"},
                {"bone": "foot_r", "head": [-0.085, 0.0, -0.56], "tail": [-0.085, 0.18, -0.60], "parent": "lower_leg_r"},
            ],
            "skinning_mode": "rigid",
            "material_slots": [
                {"name": "skin", "base_color": [0.78, 0.62, 0.52, 1.0], "roughness": 0.58},
                {"name": "muscle", "base_color": [0.72, 0.55, 0.45, 1.0], "roughness": 0.52},
            ],
            "bone_meshes": {
                # === TORSO - Masculine V-shape ===

                "hips": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.12},
                    "extrusion_steps": [
                        {"extrude": 0.20, "scale": 1.08},   # Slight hip
                        {"extrude": 0.55, "scale": 1.02},   # Straight sides (not curvy)
                        {"extrude": 0.25, "scale": 0.95},   # Slight taper up
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                    # Glutes - back indicator
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.065, 0.085, 0.065],
                            "offset": [0.045, -0.055, 0.09],  # Back left
                            "material_index": 0,
                        },
                        {
                            "primitive": "sphere",
                            "dimensions": [0.065, 0.085, 0.065],
                            "offset": [-0.045, -0.055, 0.09],  # Back right
                            "material_index": 0,
                        },
                    ],
                },

                "spine": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.11},
                    "extrusion_steps": [
                        {"extrude": 0.30, "scale": 1.05},   # Lower back
                        {"extrude": 0.45, "scale": 1.12},   # Expand to lats
                        {"extrude": 0.25, "scale": 1.18},   # Wide upper back
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                    # Lats - back muscle definition
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.06, 0.08, 0.10],
                            "offset": [0.08, -0.04, 0.10],  # Back left
                            "material_index": 1,
                        },
                        {
                            "primitive": "sphere",
                            "dimensions": [0.06, 0.08, 0.10],
                            "offset": [-0.08, -0.04, 0.10],  # Back right
                            "material_index": 1,
                        },
                    ],
                },

                "chest": {
                    "profile": "hexagon(8)",
                    "profile_radius": {"absolute": 0.13},
                    "extrusion_steps": [
                        {"extrude": 0.15, "scale": 1.28},   # Wide chest
                        {"extrude": 0.55, "scale": 1.15},   # Main chest
                        {"extrude": 0.30, "scale": 0.55},   # Taper to neck
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                    # PECTORALS - key forward direction indicator
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.08, 0.065, 0.055],
                            "offset": [0.055, 0.08, 0.08],  # Forward (+Y), left pec
                            "material_index": 1,
                        },
                        {
                            "primitive": "sphere",
                            "dimensions": [0.08, 0.065, 0.055],
                            "offset": [-0.055, 0.08, 0.08],  # Forward (+Y), right pec
                            "material_index": 1,
                        },
                        # Sternum groove (subtle indent between pecs)
                        {
                            "primitive": "cylinder",
                            "dimensions": [0.015, 0.015, 0.08],
                            "offset": [0.0, 0.05, 0.08],
                            "rotation": [90.0, 0.0, 0.0],
                            "material_index": 0,
                        },
                    ],
                },

                "neck": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.055},  # Thicker neck than female
                    "extrusion_steps": [
                        {"extrude": 0.30, "scale": 1.15},
                        {"extrude": 0.70, "scale": 0.88},
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                    # Adam's apple - forward indicator
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.018, 0.022, 0.018],
                            "offset": [0.0, 0.04, 0.03],  # Front of neck
                            "material_index": 0,
                        },
                    ],
                },

                "head": {
                    "profile": "circle(14)",
                    "profile_radius": {"absolute": 0.045},  # Larger head than female
                    "extrusion_steps": [
                        {"extrude": 0.08, "scale": 1.20},   # Jaw
                        {"extrude": 0.18, "scale": 2.05},   # Face widens
                        {"extrude": 0.48, "scale": 1.12},   # Cranium
                        {"extrude": 0.26, "scale": 0.50},   # Top
                    ],
                    "cap_start": False,
                    "cap_end": True,
                    "material_index": 0,
                    # Nose and brow for clear forward
                    "attachments": [
                        {
                            "primitive": "cone",
                            "dimensions": [0.022, 0.022, 0.045],
                            "offset": [0.0, 0.095, 0.07],  # Forward - nose
                            "rotation": [-90.0, 0.0, 0.0],
                            "material_index": 0,
                        },
                        {
                            "primitive": "cube",
                            "dimensions": [0.12, 0.025, 0.025],
                            "offset": [0.0, 0.085, 0.12],  # Brow ridge
                            "material_index": 0,
                        },
                    ],
                },

                # === ARMS - Larger/more muscular than female ===

                "shoulder_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.068},  # Bigger shoulders
                    "extrusion_steps": [
                        {"extrude": 0.30, "scale": 1.22},   # Deltoid cap
                        {"extrude": 0.70, "scale": 0.82},
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                },

                "upper_arm_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.065},
                    "extrusion_steps": [
                        {"extrude": 0.15, "scale": 1.25},   # Deltoid insertion
                        {"extrude": 0.25, "scale": 1.18},   # Bicep bulge
                        {"extrude": 0.40, "scale": 0.85},   # Taper
                        {"extrude": 0.20, "scale": 0.72},   # Elbow
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                    # Bicep bulge - front
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.04, 0.05, 0.08],
                            "offset": [0.0, 0.04, -0.08],  # Front of arm
                            "material_index": 1,
                        },
                    ],
                },

                "lower_arm_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.048},
                    "extrusion_steps": [
                        {"extrude": 0.18, "scale": 1.22},   # Forearm muscle
                        {"extrude": 0.55, "scale": 0.78},   # Taper
                        {"extrude": 0.27, "scale": 0.65},   # Wrist
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                },

                "hand_l": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.035},  # Larger hands
                    "extrusion_steps": [
                        {"extrude": 0.22, "scale": 1.55},   # Palm widens
                        {"extrude": 0.52, "scale": 1.08},   # Palm body
                        {"extrude": 0.26, "scale": 0.45},   # Fingers
                    ],
                    "cap_start": False,
                    "cap_end": True,
                    "material_index": 0,
                },

                "shoulder_r": {"mirror": "shoulder_l"},
                "upper_arm_r": {"mirror": "upper_arm_l"},
                "lower_arm_r": {"mirror": "lower_arm_l"},
                "hand_r": {"mirror": "hand_l"},

                # === LEGS - Larger with muscle definition ===

                "upper_leg_l": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.095},  # Bigger thighs
                    "extrusion_steps": [
                        {"extrude": 0.12, "scale": 1.15},   # Upper thigh
                        {"extrude": 0.55, "scale": 0.88},   # Mid thigh
                        {"extrude": 0.33, "scale": 0.68},   # Above knee
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                    # Quad muscle - front
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.055, 0.07, 0.14],
                            "offset": [0.02, 0.05, -0.18],  # Front-outer quad
                            "material_index": 1,
                        },
                    ],
                },

                "lower_leg_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.062},
                    "extrusion_steps": [
                        {"extrude": 0.15, "scale": 1.15},   # Below knee
                        {"extrude": 0.55, "scale": 0.72},   # Shin/calf taper
                        {"extrude": 0.30, "scale": 0.58},   # Ankle
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                    # Calf muscle - BACK (key direction indicator)
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.048, 0.07, 0.12],
                            "offset": [0.0, -0.045, -0.14],  # Back (-Y) = calf
                            "material_index": 1,
                        },
                    ],
                },

                # Foot - proper shape with depth and clear front/back
                "foot_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.050},  # Bigger base for men
                    "extrusion_steps": [
                        {"extrude": 0.08, "scale": 1.30},   # Ankle
                        {"extrude": 0.15, "scale": 1.65},   # Heel area
                        {"extrude": 0.48, "scale": 1.28},   # Ball of foot
                        {"extrude": 0.29, "scale": 0.45},   # Toes
                    ],
                    "cap_start": False,
                    "cap_end": True,
                    "material_index": 0,
                    # Heel bump - back indicator
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.042, 0.055, 0.040],
                            "offset": [0.0, -0.045, 0.025],  # Back = heel
                            "material_index": 0,
                        },
                        # Instep - adds foot depth
                        {
                            "primitive": "sphere",
                            "dimensions": [0.035, 0.06, 0.028],
                            "offset": [0.0, 0.025, 0.030],  # Top of foot
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
            "constraints": {"max_triangles": 20000, "max_bones": 64, "max_materials": 4},
        },
    },
)
