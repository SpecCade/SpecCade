# Basic humanoid run cycle using animation helpers preset

spec(
    asset_id = "char-run-basic",
    asset_type = "skeletal_animation",
    license = "CC0-1.0",
    seed = 100,
    description = "Basic humanoid run cycle using animation helpers preset",
    outputs = [output("char_run_basic.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.helpers_v1",
        "params": {
            "skeleton": "humanoid",
            "preset": "run_cycle",
            "settings": {
                "stride_length": 1.2,
                "cycle_frames": 30,
                "foot_roll": True,
                "arm_swing": 0.5
            },
            "clip_name": "run_basic",
            "fps": 30,
            "save_blend": False
        }
    }
)
