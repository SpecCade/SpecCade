# Basic idle sway animation with subtle breathing and weight shifting

spec(
    asset_id = "char-idle-basic",
    asset_type = "skeletal_animation",
    license = "CC0-1.0",
    seed = 300,
    description = "Basic idle sway animation with subtle breathing and weight shifting",
    outputs = [output("char_idle_basic.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.helpers_v1",
        "params": {
            "skeleton": "humanoid",
            "preset": "idle_sway",
            "settings": {
                "stride_length": 0.0,
                "cycle_frames": 120,
                "foot_roll": False,
                "arm_swing": 0.05
            },
            "clip_name": "idle_basic",
            "fps": 30,
            "save_blend": False
        }
    }
)
