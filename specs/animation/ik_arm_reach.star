# Arm reach animation - reaching forward motion

skeletal_animation_spec(
    asset_id = "ik_arm_reach",
    seed = 8006,
    output_path = "ik_arm_reach.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "arm_reach",
    duration_seconds = 2.0,
    fps = 30,
    loop = False,
    description = "Arm reach animation - reaching forward motion",
    license = "CC0-1.0",
    keyframes = [
        animation_keyframe(
            time = 0.0,
            bones = {
                "upper_arm_r": bone_transform(rotation = [0, 0, 0]),
                "lower_arm_r": bone_transform(rotation = [0, 0, 0]),
                "spine": bone_transform(rotation = [0, 0, 0]),
                "head": bone_transform(rotation = [0, 0, 0])
            }
        ),
        animation_keyframe(
            time = 0.33,
            bones = {
                "upper_arm_r": bone_transform(rotation = [-90, 20, 0]),
                "lower_arm_r": bone_transform(rotation = [-15, 0, 0]),
                "spine": bone_transform(rotation = [-10, 0, 0]),
                "head": bone_transform(rotation = [-20, 15, 0])
            }
        ),
        animation_keyframe(
            time = 1.0,
            bones = {
                "upper_arm_r": bone_transform(rotation = [-120, 30, 0]),
                "lower_arm_r": bone_transform(rotation = [0, 0, 0]),
                "spine": bone_transform(rotation = [-15, 0, 5]),
                "head": bone_transform(rotation = [-30, 20, 0])
            }
        ),
        animation_keyframe(
            time = 2.0,
            bones = {
                "upper_arm_r": bone_transform(rotation = [0, 0, 0]),
                "lower_arm_r": bone_transform(rotation = [0, 0, 0]),
                "spine": bone_transform(rotation = [0, 0, 0]),
                "head": bone_transform(rotation = [0, 0, 0])
            }
        )
    ],
    interpolation = "bezier",
    export = {
        "save_blend": True
    }
)
