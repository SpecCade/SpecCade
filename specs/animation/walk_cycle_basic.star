# Basic humanoid walk cycle using animation helpers preset

spec(
    asset_id = "char-walk-basic",
    asset_type = "skeletal_animation",
    license = "CC0-1.0",
    seed = 42,
    description = "Basic humanoid walk cycle using animation helpers preset",
    outputs = [output("char_walk_basic.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.helpers_v1",
        "params": {
            "skeleton": "humanoid",
            "preset": "walk_cycle",
            "settings": {
                "stride_length": 0.8,
                "cycle_frames": 60,
                "foot_roll": True,
                "arm_swing": 0.3
            },
            "clip_name": "walk_basic",
            "fps": 30,
            "save_blend": False
        }
    }
)
