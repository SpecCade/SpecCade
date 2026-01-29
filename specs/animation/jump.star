# Jump animation - anticipation, launch, apex, landing

skeletal_animation_spec(
    asset_id = "jump",
    seed = 8004,
    output_path = "jump.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "jump",
    duration_seconds = 1.2,
    fps = 30,
    loop = False,
    description = "Jump animation - anticipation, launch, apex, landing",
    license = "CC0-1.0",
    keyframes = [
        animation_keyframe(
            time = 0.0,
            bones = {
                "spine": bone_transform(rotation = [20, 0, 0]),
                "upper_leg_l": bone_transform(rotation = [45, 0, 0]),
                "lower_leg_l": bone_transform(rotation = [-90, 0, 0]),
                "upper_leg_r": bone_transform(rotation = [45, 0, 0]),
                "lower_leg_r": bone_transform(rotation = [-90, 0, 0])
            }
        ),
        animation_keyframe(
            time = 0.27,
            bones = {
                "spine": bone_transform(rotation = [-10, 0, 0]),
                "upper_leg_l": bone_transform(rotation = [-15, 0, 0]),
                "lower_leg_l": bone_transform(rotation = [0, 0, 0]),
                "upper_leg_r": bone_transform(rotation = [-15, 0, 0]),
                "lower_leg_r": bone_transform(rotation = [0, 0, 0])
            }
        ),
        animation_keyframe(
            time = 0.47,
            bones = {
                "spine": bone_transform(rotation = [0, 0, 0]),
                "upper_leg_l": bone_transform(rotation = [10, 0, 0]),
                "lower_leg_l": bone_transform(rotation = [-20, 0, 0]),
                "upper_leg_r": bone_transform(rotation = [10, 0, 0]),
                "lower_leg_r": bone_transform(rotation = [-20, 0, 0])
            }
        ),
        animation_keyframe(
            time = 1.2,
            bones = {
                "spine": bone_transform(rotation = [15, 0, 0]),
                "upper_leg_l": bone_transform(rotation = [30, 0, 0]),
                "lower_leg_l": bone_transform(rotation = [-60, 0, 0]),
                "upper_leg_r": bone_transform(rotation = [30, 0, 0]),
                "lower_leg_r": bone_transform(rotation = [-60, 0, 0])
            }
        )
    ],
    interpolation = "bezier"
)
