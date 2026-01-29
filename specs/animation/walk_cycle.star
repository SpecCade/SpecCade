# Walk cycle animation - basic bipedal locomotion

skeletal_animation_spec(
    asset_id = "walk_cycle",
    seed = 8002,
    output_path = "walk_cycle.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "walk_cycle",
    duration_seconds = 1.0,
    fps = 30,
    loop = True,
    description = "Walk cycle animation - basic bipedal locomotion",
    license = "CC0-1.0",
    keyframes = [
        animation_keyframe(
            time = 0.0,
            bones = {
                "upper_leg_l": bone_transform(rotation = [-20, 0, 0]),
                "lower_leg_l": bone_transform(rotation = [10, 0, 0]),
                "upper_leg_r": bone_transform(rotation = [20, 0, 0]),
                "lower_leg_r": bone_transform(rotation = [-30, 0, 0]),
                "upper_arm_l": bone_transform(rotation = [15, 0, 0]),
                "upper_arm_r": bone_transform(rotation = [-15, 0, 0]),
                "spine": bone_transform(rotation = [0, 0, -3])
            }
        ),
        animation_keyframe(
            time = 0.25,
            bones = {
                "upper_leg_l": bone_transform(rotation = [0, 0, 0]),
                "lower_leg_l": bone_transform(rotation = [-40, 0, 0]),
                "upper_leg_r": bone_transform(rotation = [0, 0, 0]),
                "lower_leg_r": bone_transform(rotation = [0, 0, 0]),
                "upper_arm_l": bone_transform(rotation = [0, 0, 0]),
                "upper_arm_r": bone_transform(rotation = [0, 0, 0]),
                "spine": bone_transform(rotation = [0, 0, 0])
            }
        ),
        animation_keyframe(
            time = 0.5,
            bones = {
                "upper_leg_l": bone_transform(rotation = [20, 0, 0]),
                "lower_leg_l": bone_transform(rotation = [-30, 0, 0]),
                "upper_leg_r": bone_transform(rotation = [-20, 0, 0]),
                "lower_leg_r": bone_transform(rotation = [10, 0, 0]),
                "upper_arm_l": bone_transform(rotation = [-15, 0, 0]),
                "upper_arm_r": bone_transform(rotation = [15, 0, 0]),
                "spine": bone_transform(rotation = [0, 0, 3])
            }
        ),
        animation_keyframe(
            time = 0.75,
            bones = {
                "upper_leg_l": bone_transform(rotation = [0, 0, 0]),
                "lower_leg_l": bone_transform(rotation = [0, 0, 0]),
                "upper_leg_r": bone_transform(rotation = [0, 0, 0]),
                "lower_leg_r": bone_transform(rotation = [-40, 0, 0]),
                "upper_arm_l": bone_transform(rotation = [0, 0, 0]),
                "upper_arm_r": bone_transform(rotation = [0, 0, 0]),
                "spine": bone_transform(rotation = [0, 0, 0])
            }
        ),
        animation_keyframe(
            time = 1.0,
            bones = {
                "upper_leg_l": bone_transform(rotation = [-20, 0, 0]),
                "lower_leg_l": bone_transform(rotation = [10, 0, 0]),
                "upper_leg_r": bone_transform(rotation = [20, 0, 0]),
                "lower_leg_r": bone_transform(rotation = [-30, 0, 0]),
                "upper_arm_l": bone_transform(rotation = [15, 0, 0]),
                "upper_arm_r": bone_transform(rotation = [-15, 0, 0]),
                "spine": bone_transform(rotation = [0, 0, -3])
            }
        )
    ],
    interpolation = "bezier"
)
