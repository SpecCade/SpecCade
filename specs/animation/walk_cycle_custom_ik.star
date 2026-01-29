# Walk cycle with custom IK pole angles and chain lengths

spec(
    asset_id = "char-walk-custom-ik",
    asset_type = "skeletal_animation",
    license = "CC0-1.0",
    seed = 1234,
    description = "Walk cycle with custom IK pole angles and chain lengths",
    outputs = [output("char_walk_custom_ik.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.helpers_v1",
        "params": {
            "skeleton": "humanoid",
            "preset": "walk_cycle",
            "settings": {
                "stride_length": 0.7,
                "cycle_frames": 48,
                "foot_roll": True,
                "arm_swing": 0.4,
                "hip_sway": 4.0,
                "spine_twist": 6.0,
                "foot_lift": 0.12
            },
            "ik_targets": {
                "foot_l": {"pole_angle": 85, "chain_length": 2},
                "foot_r": {"pole_angle": 95, "chain_length": 2},
                "hand_l": {"pole_angle": -85, "chain_length": 2},
                "hand_r": {"pole_angle": -95, "chain_length": 2}
            },
            "clip_name": "walk_custom",
            "fps": 24,
            "save_blend": False
        }
    }
)
