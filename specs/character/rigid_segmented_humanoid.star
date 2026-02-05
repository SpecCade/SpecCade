# Rigid segmented humanoid - realistic proportions with rigid skinning (no smooth blending)
#
# Each body segment is assigned to exactly one bone (rigid skinning).
# Segments connect without gaps by using ABSOLUTE profile_radius values
# that are carefully matched at connection points.
#
# Skeleton preset: humanoid_connected_v1 (20 bones)
# Bone lengths from preset (critical for understanding radius calculations):
#   - hips: 0.1m (z=0.9 to 1.0)
#   - spine: 0.2m (z=1.0 to 1.2)
#   - chest: 0.2m (z=1.2 to 1.4)
#   - neck: 0.1m (z=1.4 to 1.5)
#   - head: 0.2m (z=1.5 to 1.7)
#   - shoulder: 0.1m each
#   - upper_arm: 0.25m each
#   - lower_arm: 0.25m each
#   - hand: 0.1m each
#   - upper_leg: 0.4m each
#   - lower_leg: 0.4m each
#   - foot: varies (angled)

spec(
    asset_id = "rigid_segmented_humanoid",
    asset_type = "skeletal_mesh",
    seed = 7300,
    license = "CC0-1.0",
    description = "Realistic humanoid with rigid skinning - each segment bound to one bone, no gaps between parts",
    tags = ["skeletal_mesh", "character", "humanoid", "rigid_skinning", "segmented", "armature_driven_v1"],
    outputs = [output("skeletal_mesh/rigid_segmented_humanoid.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton": [
                {"bone": "root", "head": [0.0, 0.0, 0.0], "tail": [0.0, 0.0, 0.1]},
                {"bone": "hips", "head": [0.0, 0.0, 0.1], "tail": [0.0, 0.0, 0.25], "parent": "root"},
                {"bone": "spine", "head": [0.0, 0.0, 0.25], "tail": [0.0, 0.0, 0.45], "parent": "hips"},
                {"bone": "chest", "head": [0.0, 0.0, 0.45], "tail": [0.0, 0.0, 0.7], "parent": "spine"},
                {"bone": "neck", "head": [0.0, 0.0, 0.7], "tail": [0.0, 0.0, 0.8], "parent": "chest"},
                {"bone": "head", "head": [0.0, 0.0, 0.8], "tail": [0.0, 0.0, 1.0], "parent": "neck"},

                # Left arm - HORIZONTAL from chest, starts outside body
                {"bone": "shoulder_l", "head": [0.15, 0.0, 0.6], "tail": [0.25, 0.0, 0.6], "parent": "chest"},
                {"bone": "upper_arm_l", "head": [0.25, 0.0, 0.6], "tail": [0.5, 0.0, 0.6], "parent": "shoulder_l"},
                {"bone": "lower_arm_l", "head": [0.5, 0.0, 0.6], "tail": [0.75, 0.0, 0.6], "parent": "upper_arm_l"},
                {"bone": "hand_l", "head": [0.75, 0.0, 0.6], "tail": [0.85, 0.0, 0.6], "parent": "lower_arm_l"},

                # Right arm - mirror positions
                {"bone": "shoulder_r", "head": [-0.15, 0.0, 0.6], "tail": [-0.25, 0.0, 0.6], "parent": "chest"},
                {"bone": "upper_arm_r", "head": [-0.25, 0.0, 0.6], "tail": [-0.5, 0.0, 0.6], "parent": "shoulder_r"},
                {"bone": "lower_arm_r", "head": [-0.5, 0.0, 0.6], "tail": [-0.75, 0.0, 0.6], "parent": "upper_arm_r"},
                {"bone": "hand_r", "head": [-0.75, 0.0, 0.6], "tail": [-0.85, 0.0, 0.6], "parent": "upper_arm_r"},

                # Left leg - vertical down from hips
                {"bone": "upper_leg_l", "head": [0.08, 0.0, 0.25], "tail": [0.08, 0.0, -0.15], "parent": "hips"},
                {"bone": "lower_leg_l", "head": [0.08, 0.0, -0.15], "tail": [0.08, 0.0, -0.55], "parent": "upper_leg_l"},
                {"bone": "foot_l", "head": [0.08, 0.0, -0.55], "tail": [0.08, 0.15, -0.55], "parent": "lower_leg_l"},

                # Right leg - mirror positions
                {"bone": "upper_leg_r", "head": [-0.08, 0.0, 0.25], "tail": [-0.08, 0.0, -0.15], "parent": "hips"},
                {"bone": "lower_leg_r", "head": [-0.08, 0.0, -0.15], "tail": [-0.08, 0.0, -0.55], "parent": "upper_leg_r"},
                {"bone": "foot_r", "head": [-0.08, 0.0, -0.55], "tail": [-0.08, 0.15, -0.55], "parent": "lower_leg_r"},
            ],
            "skinning_mode": "rigid",
            "material_slots": [
                {
                    "name": "skin",
                    "base_color": [0.85, 0.72, 0.62, 1.0],
                    "roughness": 0.5,
                },
                {
                    "name": "joint_ring",
                    "base_color": [0.25, 0.25, 0.28, 1.0],
                    "roughness": 0.6,
                },
            ],
            "bone_meshes": {
                # ================================================================
                # TORSO - Core body segments
                # All use ABSOLUTE radii to ensure proper connection
                # ================================================================

                # Hips/Pelvis segment - broad base of torso
                # bone length = 0.1m
                # Start: 0.14m radius, End: 0.14 * 1.05 * 0.95 = 0.14m radius
                "hips": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.14},
                    "extrusion_steps": [
                        {"extrude": 0.3, "scale": 1.0},
                        {"extrude": 0.4, "scale": 1.05},
                        {"extrude": 0.3, "scale": 0.95},
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                },

                # Spine segment - lower torso/abdomen
                # bone length = 0.2m
                # Start radius must match hips end: 0.14m
                # End: 0.14 * 1.08 * 0.95 = 0.144m
                "spine": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.14},
                    "extrusion_steps": [
                        {"extrude": 0.2, "scale": 1.0},
                        {"extrude": 0.5, "scale": 1.08},
                        {"extrude": 0.3, "scale": 0.95},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                },

                # Chest segment - upper torso/ribcage
                # bone length = 0.2m
                # Start radius must match spine end: ~0.144m
                # End: 0.144 * 1.15 * 1.10 * 0.75 = 0.137m (narrowing for neck)
                "chest": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.144},
                    "extrusion_steps": [
                        {"extrude": 0.15, "scale": 1.15},
                        {"extrude": 0.50, "scale": 1.10},
                        {"extrude": 0.35, "scale": 0.60},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                    # Shoulder connectors - bridge the gap from chest to upper arms
                    # The shoulder bones extend from x=0.1 to x=0.2 at z=1.35
                    # These cylinders connect the chest to where the upper arms begin
                    "attachments": [
                        {
                            "primitive": "cylinder",
                            "dimensions": [0.07, 0.07, 0.10],  # radius x, radius y, height
                            "offset": [0.05, 0.0, 0.15],  # offset from bone position (left shoulder)
                            "rotation": [0.0, 90.0, 0.0],  # rotate to point sideways
                            "material_index": 0,
                        },
                        {
                            "primitive": "cylinder",
                            "dimensions": [0.07, 0.07, 0.10],
                            "offset": [-0.05, 0.0, 0.15],  # right shoulder
                            "rotation": [0.0, -90.0, 0.0],
                            "material_index": 0,
                        },
                    ],
                },

                # Neck segment - connects head to torso
                # bone length = 0.1m
                # Start radius must match chest end: 0.144 * 1.15 * 1.10 * 0.60 = 0.109m
                # Using 0.055m for thinner neck appearance
                "neck": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.055},
                    "extrusion_steps": [
                        {"extrude": 0.3, "scale": 1.0},
                        {"extrude": 0.4, "scale": 0.92},
                        {"extrude": 0.3, "scale": 0.95},
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                },

                # Head segment - cranium
                # bone length = 0.2m
                # Start radius matches neck end: 0.055 * 0.92 * 0.95 = 0.048m
                "head": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.048},
                    "extrusion_steps": [
                        {"extrude": 0.08, "scale": 1.15},
                        {"extrude": 0.35, "scale": 2.50},
                        {"extrude": 0.35, "scale": 1.00},
                        {"extrude": 0.22, "scale": 0.60},
                    ],
                    "cap_start": False,
                    "cap_end": True,
                    "material_index": 0,
                },

                # ================================================================
                # LEFT ARM
                # ================================================================

                # Left shoulder - short connector from chest to upper arm
                # bone length ≈ 0.206m, connects chest to upper_arm
                # Uses overlap approach: cap_start closes the end
                "shoulder_l": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.07},
                    "extrusion_steps": [
                        {"extrude": 0.3, "scale": 1.1},
                        {"extrude": 0.4, "scale": 1.0},
                        {"extrude": 0.3, "scale": 0.95},
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                },

                # Left upper arm segment - bicep/tricep area, connects to shoulder
                # bone length = 0.25m
                # Start radius must match shoulder_l end: 0.07 * 1.1 * 1.0 * 0.95 = 0.073m
                # End: 0.073 * 1.15 * 0.90 * 0.75 = 0.057m
                "upper_arm_l": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.073},
                    "extrusion_steps": [
                        {"extrude": 0.15, "scale": 1.15},
                        {"extrude": 0.55, "scale": 0.90},
                        {"extrude": 0.30, "scale": 0.75},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                },

                # Left lower arm segment - forearm
                # bone length = 0.25m
                # Start radius matches upper_arm end: 0.057m
                # End: 0.057 * 1.05 * 0.85 * 0.75 = 0.038m
                "lower_arm_l": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.057},
                    "extrusion_steps": [
                        {"extrude": 0.20, "scale": 1.05},
                        {"extrude": 0.55, "scale": 0.85},
                        {"extrude": 0.25, "scale": 0.75},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                },

                # Left hand segment
                # bone length = 0.1m
                # Start radius matches lower_arm end: 0.038m
                "hand_l": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.038},
                    "extrusion_steps": [
                        {"extrude": 0.25, "scale": 1.3},
                        {"extrude": 0.50, "scale": 1.1},
                        {"extrude": 0.25, "scale": 0.6},
                    ],
                    "cap_start": False,
                    "cap_end": True,
                    "material_index": 0,
                },

                # ================================================================
                # RIGHT ARM - Mirror of left
                # ================================================================

                "shoulder_r": {"mirror": "shoulder_l"},
                "upper_arm_r": {"mirror": "upper_arm_l"},
                "lower_arm_r": {"mirror": "lower_arm_l"},
                "hand_r": {"mirror": "hand_l"},

                # ================================================================
                # LEFT LEG
                # ================================================================

                # Left upper leg segment - thigh
                # bone length = 0.4m
                # Start: 0.09m radius
                # End: 0.09 * 1.15 * 0.90 * 0.75 = 0.070m
                # Uses overlap approach: cap_start closes the end
                "upper_leg_l": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.09},
                    "extrusion_steps": [
                        {"extrude": 0.15, "scale": 1.15},
                        {"extrude": 0.55, "scale": 0.90},
                        {"extrude": 0.30, "scale": 0.75},
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                },

                # Left lower leg segment - calf/shin
                # bone length = 0.4m
                # Start radius matches upper_leg end: 0.070m
                # End: 0.070 * 1.05 * 0.75 * 0.70 = 0.039m
                "lower_leg_l": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.070},
                    "extrusion_steps": [
                        {"extrude": 0.15, "scale": 1.05},
                        {"extrude": 0.55, "scale": 0.75},
                        {"extrude": 0.30, "scale": 0.70},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                },

                # Left foot segment
                # bone length varies (angled bone)
                # Start radius matches lower_leg end: 0.039m
                "foot_l": {
                    "profile": "rectangle",
                    "profile_radius": [0.045, 0.039],
                    "extrusion_steps": [
                        {"extrude": 0.20, "scale": 1.3},
                        {"extrude": 0.50, "scale": 1.1},
                        {"extrude": 0.30, "scale": 0.7},
                    ],
                    "cap_start": False,
                    "cap_end": True,
                    "material_index": 0,
                },

                # ================================================================
                # RIGHT LEG - Mirror of left
                # ================================================================

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
