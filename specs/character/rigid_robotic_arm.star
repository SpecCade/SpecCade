# Rigid robotic arm - industrial robot arm with segmented joints
#
# A mechanical arm suitable for factory/industrial settings.
# Segments: base mount -> shoulder -> upper arm -> elbow -> lower arm -> wrist -> gripper

spec(
    asset_id = "rigid_robotic_arm",
    asset_type = "skeletal_mesh",
    seed = 7400,
    license = "CC0-1.0",
    description = "Industrial robotic arm with segmented joints - base, shoulder, elbow, wrist, gripper",
    tags = ["skeletal_mesh", "robot", "mechanical", "arm", "industrial", "armature_driven_v1"],
    outputs = [output("skeletal_mesh/rigid_robotic_arm.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton": [
                # Base mount - sits on ground
                {"bone": "base", "head": [0.0, 0.0, 0.0], "tail": [0.0, 0.0, 0.15]},
                # Shoulder rotator - horizontal rotation
                {"bone": "shoulder_rotator", "head": [0.0, 0.0, 0.15], "tail": [0.0, 0.0, 0.25], "parent": "base"},
                # Upper arm - extends upward and outward
                {"bone": "upper_arm", "head": [0.0, 0.0, 0.25], "tail": [0.0, 0.0, 0.55], "parent": "shoulder_rotator"},
                # Elbow joint
                {"bone": "elbow", "head": [0.0, 0.0, 0.55], "tail": [0.0, 0.0, 0.62], "parent": "upper_arm"},
                # Lower arm - extends further
                {"bone": "lower_arm", "head": [0.0, 0.0, 0.62], "tail": [0.0, 0.0, 0.90], "parent": "elbow"},
                # Wrist rotator
                {"bone": "wrist", "head": [0.0, 0.0, 0.90], "tail": [0.0, 0.0, 0.97], "parent": "lower_arm"},
                # Gripper/end effector
                {"bone": "gripper", "head": [0.0, 0.0, 0.97], "tail": [0.0, 0.0, 1.10], "parent": "wrist"},
            ],
            "skinning_mode": "rigid",
            "material_slots": [
                {
                    "name": "metal_body",
                    "base_color": [0.85, 0.45, 0.08, 1.0],
                    "metallic": 0.9,
                    "roughness": 0.35,
                },
                {
                    "name": "joint_dark",
                    "base_color": [0.15, 0.15, 0.18, 1.0],
                    "metallic": 0.8,
                    "roughness": 0.4,
                },
                {
                    "name": "accent_yellow",
                    "base_color": [0.95, 0.75, 0.05, 1.0],
                    "roughness": 0.5,
                },
            ],
            "bone_meshes": {
                # BASE - Heavy foundation
                "base": {
                    "profile": "hexagon(8)",
                    "profile_radius": {"absolute": 0.18},
                    "extrusion_steps": [
                        {"extrude": 0.3, "scale": 1.0},
                        {"extrude": 0.5, "scale": 0.85},
                        {"extrude": 0.2, "scale": 0.70},
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 1,
                },

                # SHOULDER ROTATOR - Joint mechanism
                "shoulder_rotator": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.12},
                    "extrusion_steps": [
                        {"extrude": 0.2, "scale": 1.15},
                        {"extrude": 0.6, "scale": 1.0},
                        {"extrude": 0.2, "scale": 0.90},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                    "attachments": [
                        {
                            "primitive": "cylinder",
                            "dimensions": [0.08, 0.08, 0.04],
                            "offset": [0.0, 0.0, 0.5],
                            "rotation": [0.0, 0.0, 0.0],
                            "material_index": 2,
                        },
                    ],
                },

                # UPPER ARM - Main structural segment
                "upper_arm": {
                    "profile": "hexagon(6)",
                    "profile_radius": {"absolute": 0.10},
                    "extrusion_steps": [
                        {"extrude": 0.15, "scale": 1.10},
                        {"extrude": 0.60, "scale": 0.95},
                        {"extrude": 0.25, "scale": 0.85},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                },

                # ELBOW - Compact joint
                "elbow": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.085},
                    "extrusion_steps": [
                        {"extrude": 0.3, "scale": 1.12},
                        {"extrude": 0.4, "scale": 1.0},
                        {"extrude": 0.3, "scale": 0.88},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 1,
                    "attachments": [
                        {
                            "primitive": "torus",
                            "dimensions": [0.10, 0.10, 0.02],
                            "offset": [0.0, 0.0, 0.5],
                            "rotation": [0.0, 0.0, 0.0],
                            "material_index": 2,
                        },
                    ],
                },

                # LOWER ARM - Slimmer segment
                "lower_arm": {
                    "profile": "hexagon(6)",
                    "profile_radius": {"absolute": 0.075},
                    "extrusion_steps": [
                        {"extrude": 0.20, "scale": 1.05},
                        {"extrude": 0.55, "scale": 0.92},
                        {"extrude": 0.25, "scale": 0.80},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                },

                # WRIST - Small rotator joint
                "wrist": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.055},
                    "extrusion_steps": [
                        {"extrude": 0.3, "scale": 1.10},
                        {"extrude": 0.4, "scale": 1.0},
                        {"extrude": 0.3, "scale": 0.90},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 1,
                },

                # GRIPPER - End effector
                "gripper": {
                    "profile": "rectangle",
                    "profile_radius": [0.05, 0.03],
                    "extrusion_steps": [
                        {"extrude": 0.25, "scale": 1.20},
                        {"extrude": 0.50, "scale": 0.85},
                        {"extrude": 0.25, "scale": 0.60},
                    ],
                    "cap_start": False,
                    "cap_end": True,
                    "material_index": 0,
                    "attachments": [
                        # Gripper jaws
                        {
                            "primitive": "cube",
                            "dimensions": [0.025, 0.04, 0.08],
                            "offset": [0.04, 0.0, 0.85],
                            "rotation": [0.0, 0.0, 15.0],
                            "material_index": 1,
                        },
                        {
                            "primitive": "cube",
                            "dimensions": [0.025, 0.04, 0.08],
                            "offset": [-0.04, 0.0, 0.85],
                            "rotation": [0.0, 0.0, -15.0],
                            "material_index": 1,
                        },
                    ],
                },
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
                "max_triangles": 8000,
                "max_bones": 16,
                "max_materials": 4,
            },
        },
    },
)
