# Floating bob animation - gentle vertical oscillation

skeletal_animation_spec(
    asset_id = "floating_bob",
    seed = 8007,
    output_path = "floating_bob.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "floating_bob",
    duration_seconds = 3.0,
    fps = 30,
    loop = True,
    description = "Floating bob animation - gentle vertical oscillation",
    license = "CC0-1.0",
    keyframes = [
        animation_keyframe(
            time = 0.0,
            bones = {
                "spine": bone_transform(rotation = [5, 0, 0]),
                "upper_arm_l": bone_transform(rotation = [-30, 0, 15]),
                "upper_arm_r": bone_transform(rotation = [-30, 0, -15])
            }
        ),
        animation_keyframe(
            time = 1.5,
            bones = {
                "spine": bone_transform(rotation = [0, 0, 0]),
                "upper_arm_l": bone_transform(rotation = [-35, 0, 20]),
                "upper_arm_r": bone_transform(rotation = [-35, 0, -20])
            }
        ),
        animation_keyframe(
            time = 3.0,
            bones = {
                "spine": bone_transform(rotation = [5, 0, 0]),
                "upper_arm_l": bone_transform(rotation = [-30, 0, 15]),
                "upper_arm_r": bone_transform(rotation = [-30, 0, -15])
            }
        )
    ],
    interpolation = "bezier"
)
