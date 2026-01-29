# Death fall animation - ragdoll-like collapse

skeletal_animation_spec(
    asset_id = "death_fall",
    seed = 8005,
    output_path = "death_fall.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "death_fall",
    duration_seconds = 1.5,
    fps = 30,
    loop = False,
    description = "Death fall animation - ragdoll-like collapse",
    license = "CC0-1.0",
    keyframes = [
        animation_keyframe(
            time = 0.0,
            bones = {
                "spine": bone_transform(rotation = [0, 0, 0]),
                "head": bone_transform(rotation = [0, 0, 0])
            }
        ),
        animation_keyframe(
            time = 0.2,
            bones = {
                "spine": bone_transform(rotation = [-15, 0, 10]),
                "head": bone_transform(rotation = [20, -30, 0])
            }
        ),
        animation_keyframe(
            time = 0.8,
            bones = {
                "spine": bone_transform(rotation = [-45, 0, 20]),
                "head": bone_transform(rotation = [40, -45, 20])
            }
        ),
        animation_keyframe(
            time = 1.5,
            bones = {
                "spine": bone_transform(rotation = [-80, 0, 30]),
                "head": bone_transform(rotation = [60, -60, 30])
            }
        )
    ],
    interpolation = "bezier"
)
