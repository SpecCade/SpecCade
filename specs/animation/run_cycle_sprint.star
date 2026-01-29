# Fast sprint run cycle with higher foot lift and arm swing

spec(
    asset_id = "char-run-sprint",
    asset_type = "skeletal_animation",
    license = "CC0-1.0",
    seed = 200,
    description = "Fast sprint run cycle with higher foot lift and arm swing",
    outputs = [output("char_run_sprint.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.helpers_v1",
        "params": {
            "skeleton": "humanoid",
            "preset": "run_cycle",
            "settings": {
                "stride_length": 1.5,
                "cycle_frames": 24,
                "foot_roll": True,
                "arm_swing": 0.7,
                "hip_sway": 6.0,
                "spine_twist": 10.0,
                "foot_lift": 0.35
            },
            "ik_targets": {
                "foot_l": {"pole_angle": 90, "chain_length": 2},
                "foot_r": {"pole_angle": 90, "chain_length": 2}
            },
            "clip_name": "run_sprint",
            "fps": 30,
            "save_blend": False
        }
    }
)
