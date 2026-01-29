# Attack swing animation - simple overhead swing

skeletal_animation_spec(
    asset_id = "attack_swing",
    seed = 8003,
    output_path = "attack_swing.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "attack_swing",
    duration_seconds = 0.8,
    fps = 30,
    loop = False,
    description = "Attack swing animation - simple overhead swing",
    license = "CC0-1.0",
    keyframes = [
        animation_keyframe(
            time = 0.0,
            bones = {
                "upper_arm_r": bone_transform(rotation = [-60, 30, 0]),
                "lower_arm_r": bone_transform(rotation = [-90, 0, 0]),
                "spine": bone_transform(rotation = [10, -20, 0])
            }
        ),
        animation_keyframe(
            time = 0.33,
            bones = {
                "upper_arm_r": bone_transform(rotation = [-120, 45, 0]),
                "lower_arm_r": bone_transform(rotation = [-45, 0, 0]),
                "spine": bone_transform(rotation = [15, -30, -5])
            }
        ),
        animation_keyframe(
            time = 0.47,
            bones = {
                "upper_arm_r": bone_transform(rotation = [30, -10, 0]),
                "lower_arm_r": bone_transform(rotation = [-15, 0, 0]),
                "spine": bone_transform(rotation = [-10, 20, 5])
            }
        ),
        animation_keyframe(
            time = 0.8,
            bones = {
                "upper_arm_r": bone_transform(rotation = [45, -20, 0]),
                "lower_arm_r": bone_transform(rotation = [-30, 0, 0]),
                "spine": bone_transform(rotation = [-5, 10, 3])
            }
        )
    ],
    interpolation = "bezier"
)
