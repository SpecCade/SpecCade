# Relaxed idle animation with more pronounced hip sway and slower breathing

spec(
    asset_id = "char-idle-relaxed",
    asset_type = "skeletal_animation",
    license = "CC0-1.0",
    seed = 400,
    description = "Relaxed idle animation with more pronounced hip sway and slower breathing",
    outputs = [output("char_idle_relaxed.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.helpers_v1",
        "params": {
            "skeleton": "humanoid",
            "preset": "idle_sway",
            "settings": {
                "stride_length": 0.0,
                "cycle_frames": 180,
                "foot_roll": False,
                "arm_swing": 0.08,
                "hip_sway": 3.5,
                "spine_twist": 1.5
            },
            "clip_name": "idle_relaxed",
            "fps": 24,
            "save_blend": False
        }
    }
)
