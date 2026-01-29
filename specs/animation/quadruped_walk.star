# Quadruped walk cycle - four-legged locomotion

skeletal_animation_spec(
    asset_id = "quadruped_walk",
    seed = 8008,
    output_path = "quadruped_walk.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "quadruped_walk",
    duration_seconds = 1.33,
    fps = 30,
    loop = True,
    description = "Quadruped walk cycle - four-legged locomotion",
    license = "CC0-1.0",
    keyframes = [
        animation_keyframe(
            time = 0.0,
            bones = {
                "spine": bone_transform(rotation = [0, 0, -2]),
                "upper_leg_l": bone_transform(rotation = [-15, 0, 0]),
                "lower_leg_l": bone_transform(rotation = [10, 0, 0]),
                "upper_leg_r": bone_transform(rotation = [20, 0, 0]),
                "lower_leg_r": bone_transform(rotation = [-25, 0, 0])
            }
        ),
        animation_keyframe(
            time = 0.67,
            bones = {
                "spine": bone_transform(rotation = [0, 0, 2]),
                "upper_leg_l": bone_transform(rotation = [20, 0, 0]),
                "lower_leg_l": bone_transform(rotation = [-25, 0, 0]),
                "upper_leg_r": bone_transform(rotation = [-15, 0, 0]),
                "lower_leg_r": bone_transform(rotation = [10, 0, 0])
            }
        ),
        animation_keyframe(
            time = 1.33,
            bones = {
                "spine": bone_transform(rotation = [0, 0, -2]),
                "upper_leg_l": bone_transform(rotation = [-15, 0, 0]),
                "lower_leg_l": bone_transform(rotation = [10, 0, 0]),
                "upper_leg_r": bone_transform(rotation = [20, 0, 0]),
                "lower_leg_r": bone_transform(rotation = [-25, 0, 0])
            }
        )
    ],
    interpolation = "bezier"
)
