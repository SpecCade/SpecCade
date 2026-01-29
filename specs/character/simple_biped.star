# Simple biped character - minimal humanoid skeleton

spec(
    asset_id = "simple_biped",
    asset_type = "skeletal_mesh",
    license = "CC0-1.0",
    seed = 7001,
    description = "Simple biped character - minimal humanoid skeleton",
    outputs = [output("simple_biped.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton": [
                {"bone": "root", "head": [0, 0, 0], "tail": [0, 0, 0.1]},
                {"bone": "spine", "head": [0, 0, 0.1], "tail": [0, 0, 0.5], "parent": "root"},
                {"bone": "chest", "head": [0, 0, 0.5], "tail": [0, 0, 0.8], "parent": "spine"},
                {"bone": "neck", "head": [0, 0, 0.8], "tail": [0, 0, 0.9], "parent": "chest"},
                {"bone": "head", "head": [0, 0, 0.9], "tail": [0, 0, 1.1], "parent": "neck"},
                {"bone": "arm_upper_L", "head": [0.15, 0, 0.75], "tail": [0.35, 0, 0.75], "parent": "chest"},
                {"bone": "arm_lower_L", "head": [0.35, 0, 0.75], "tail": [0.55, 0, 0.75], "parent": "arm_upper_L"},
                {"bone": "hand_L", "head": [0.55, 0, 0.75], "tail": [0.65, 0, 0.75], "parent": "arm_lower_L"},
                {"bone": "arm_upper_R", "mirror": "arm_upper_L"},
                {"bone": "arm_lower_R", "mirror": "arm_lower_L"},
                {"bone": "hand_R", "mirror": "hand_L"},
                {"bone": "leg_upper_L", "head": [0.1, 0, 0.1], "tail": [0.1, 0, -0.35], "parent": "root"},
                {"bone": "leg_lower_L", "head": [0.1, 0, -0.35], "tail": [0.1, 0, -0.75], "parent": "leg_upper_L"},
                {"bone": "foot_L", "head": [0.1, 0, -0.75], "tail": [0.1, 0.15, -0.75], "parent": "leg_lower_L"},
                {"bone": "leg_upper_R", "mirror": "leg_upper_L"},
                {"bone": "leg_lower_R", "mirror": "leg_lower_L"},
                {"bone": "foot_R", "mirror": "foot_L"}
            ],
            "bone_meshes": {
                "root": {"profile": "circle(8)", "profile_radius": 0.15},
                "spine": {"profile": "circle(8)", "profile_radius": 0.15},
                "chest": {"profile": "circle(8)", "profile_radius": 0.15},
                "neck": {"profile": "circle(8)", "profile_radius": 0.15},
                "head": {"profile": "circle(8)", "profile_radius": 0.15},
                "arm_upper_L": {"profile": "circle(8)", "profile_radius": 0.15},
                "arm_lower_L": {"profile": "circle(8)", "profile_radius": 0.15},
                "hand_L": {"profile": "circle(8)", "profile_radius": 0.15},
                "arm_upper_R": {"mirror": "arm_upper_L"},
                "arm_lower_R": {"mirror": "arm_lower_L"},
                "hand_R": {"mirror": "hand_L"},
                "leg_upper_L": {"profile": "circle(8)", "profile_radius": 0.15},
                "leg_lower_L": {"profile": "circle(8)", "profile_radius": 0.15},
                "foot_L": {"profile": "circle(8)", "profile_radius": 0.15},
                "leg_upper_R": {"mirror": "leg_upper_L"},
                "leg_lower_R": {"mirror": "leg_lower_L"},
                "foot_R": {"mirror": "foot_L"}
            },
            "constraints": {"max_triangles": 800}
        }
    }
)
