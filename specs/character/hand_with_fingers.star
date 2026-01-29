# Hand with thumb and fingers - demonstrates sub-part features

spec(
    asset_id = "hand_with_fingers",
    asset_type = "skeletal_mesh",
    license = "CC0-1.0",
    seed = 7003,
    description = "Hand with thumb and fingers - demonstrates sub-part features",
    outputs = [output("hand_with_fingers.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton": [
                {"bone": "wrist", "head": [0, 0, 0], "tail": [0.1, 0, 0]},
                {"bone": "palm", "head": [0.1, 0, 0], "tail": [0.2, 0, 0], "parent": "wrist"},
                {"bone": "thumb_01", "head": [0.12, 0.04, 0], "tail": [0.15, 0.06, 0], "parent": "palm"},
                {"bone": "thumb_02", "head": [0.15, 0.06, 0], "tail": [0.18, 0.07, 0], "parent": "thumb_01"},
                {"bone": "finger_index_01", "head": [0.2, 0.02, 0], "tail": [0.24, 0.02, 0], "parent": "palm"},
                {"bone": "finger_index_02", "head": [0.24, 0.02, 0], "tail": [0.27, 0.02, 0], "parent": "finger_index_01"},
                {"bone": "finger_middle_01", "head": [0.2, 0, 0], "tail": [0.25, 0, 0], "parent": "palm"},
                {"bone": "finger_middle_02", "head": [0.25, 0, 0], "tail": [0.29, 0, 0], "parent": "finger_middle_01"},
                {"bone": "finger_ring_01", "head": [0.2, -0.02, 0], "tail": [0.24, -0.02, 0], "parent": "palm"},
                {"bone": "finger_ring_02", "head": [0.24, -0.02, 0], "tail": [0.27, -0.02, 0], "parent": "finger_ring_01"},
                {"bone": "finger_pinky_01", "head": [0.19, -0.04, 0], "tail": [0.22, -0.04, 0], "parent": "palm"},
                {"bone": "finger_pinky_02", "head": [0.22, -0.04, 0], "tail": [0.24, -0.04, 0], "parent": "finger_pinky_01"}
            ],
            "bone_meshes": {
                "wrist": {"profile": "circle(8)", "profile_radius": 0.15},
                "palm": {"profile": "circle(8)", "profile_radius": 0.15},
                "thumb_01": {"profile": "circle(8)", "profile_radius": 0.15},
                "thumb_02": {"profile": "circle(8)", "profile_radius": 0.15},
                "finger_index_01": {"profile": "circle(8)", "profile_radius": 0.15},
                "finger_index_02": {"profile": "circle(8)", "profile_radius": 0.15},
                "finger_middle_01": {"profile": "circle(8)", "profile_radius": 0.15},
                "finger_middle_02": {"profile": "circle(8)", "profile_radius": 0.15},
                "finger_ring_01": {"profile": "circle(8)", "profile_radius": 0.15},
                "finger_ring_02": {"profile": "circle(8)", "profile_radius": 0.15},
                "finger_pinky_01": {"profile": "circle(8)", "profile_radius": 0.15},
                "finger_pinky_02": {"profile": "circle(8)", "profile_radius": 0.15}
            },
            "constraints": {"max_triangles": 600}
        }
    }
)
