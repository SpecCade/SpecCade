# Animation IK coverage example
#
# Demonstrates animation stdlib functions for IK and custom bones.
# Note: IK features (custom_bone, ik_keyframe, ik_target_transform) are
# documented here for reference but require the skeletal_animation.ik_v1
# recipe kind when implemented.

spec(
    asset_id = "stdlib-animation-ik-coverage-01",
    asset_type = "skeletal_animation",
    seed = 42,
    license = "CC0-1.0",
    description = "IK animation coverage example",
    outputs = [output("animations/ik_coverage.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.keyframe_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "ik_test",
            "duration_seconds": 1.0,
            "fps": 24,
            "loop": False,
            "interpolation": "linear",
            "keyframes": [
                {
                    "time": 0.0,
                    "bones": {
                        "upper_leg_l": {"rotation": [0.0, 0.0, 0.0]},
                        "upper_leg_r": {"rotation": [0.0, 0.0, 0.0]}
                    }
                },
                {
                    "time": 1.0,
                    "bones": {
                        "upper_leg_l": {"rotation": [10.0, 0.0, 0.0]},
                        "upper_leg_r": {"rotation": [-10.0, 0.0, 0.0]}
                    }
                }
            ]
        }
    }
)
