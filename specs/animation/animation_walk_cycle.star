# Simple walk cycle animation - demonstrates animation stdlib
#
# This example creates a basic walk cycle for a humanoid skeleton
# using the standard keyframe animation format.
#
# [VALIDATION]
# SHAPE: Humanoid skeleton performing walk cycle
# MOTION: Legs swing forward/back alternately, arms counter-swing, spine subtle tilt
# FRAME 0: Contact pose - left leg forward (25 deg), right leg back (-25 deg), left arm back, right arm forward
# FRAME 12 (0.5s): Passing pose - legs swapped, right leg forward, left leg back
# FRAME 24 (1.0s): Return to contact pose (loop point)
# ORIENTATION: Character facing +Y, walking in place (no root motion)
# FRONT VIEW (frame 0): Left leg in front, right arm in front
# FRONT VIEW (frame 12): Right leg in front, left arm in front
# ISO VIEW: Full body visible, leg swing angle clearly visible (~25 degrees)
# NOTES: Linear interpolation, smooth transitions, loop=true so frame 24 matches frame 0

spec(
    asset_id = "stdlib-animation-walk-cycle-01",
    asset_type = "skeletal_animation",
    seed = 42,
    license = "CC0-1.0",
    description = "Stdlib walk cycle animation example",
    outputs = [output("animations/walk_cycle.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.keyframe_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "walk",
            "duration_seconds": 1.0,
            "fps": 24,
            "loop": True,
            "interpolation": "linear",
            "keyframes": [
                {
                    "time": 0.0,
                    "bones": {
                        "upper_leg_l": {"rotation": [25.0, 0.0, 0.0]},
                        "upper_leg_r": {"rotation": [-25.0, 0.0, 0.0]},
                        "lower_leg_l": {"rotation": [-15.0, 0.0, 0.0]},
                        "lower_leg_r": {"rotation": [10.0, 0.0, 0.0]},
                        "spine": {"rotation": [2.0, 0.0, 0.0]},
                        "upper_arm_l": {"rotation": [-15.0, 0.0, -5.0]},
                        "upper_arm_r": {"rotation": [15.0, 0.0, 5.0]}
                    }
                },
                {
                    "time": 0.5,
                    "bones": {
                        "upper_leg_l": {"rotation": [-25.0, 0.0, 0.0]},
                        "upper_leg_r": {"rotation": [25.0, 0.0, 0.0]},
                        "lower_leg_l": {"rotation": [10.0, 0.0, 0.0]},
                        "lower_leg_r": {"rotation": [-15.0, 0.0, 0.0]},
                        "spine": {"rotation": [-2.0, 0.0, 0.0]},
                        "upper_arm_l": {"rotation": [15.0, 0.0, 5.0]},
                        "upper_arm_r": {"rotation": [-15.0, 0.0, -5.0]}
                    }
                },
                {
                    "time": 1.0,
                    "bones": {
                        "upper_leg_l": {"rotation": [25.0, 0.0, 0.0]},
                        "upper_leg_r": {"rotation": [-25.0, 0.0, 0.0]},
                        "lower_leg_l": {"rotation": [-15.0, 0.0, 0.0]},
                        "lower_leg_r": {"rotation": [10.0, 0.0, 0.0]},
                        "spine": {"rotation": [2.0, 0.0, 0.0]},
                        "upper_arm_l": {"rotation": [-15.0, 0.0, -5.0]},
                        "upper_arm_r": {"rotation": [15.0, 0.0, 5.0]}
                    }
                }
            ]
        }
    }
)
