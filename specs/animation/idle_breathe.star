# Idle breathing animation - subtle chest expansion

skeletal_animation_spec(
    asset_id = "idle_breathe",
    seed = 8001,
    output_path = "idle_breathe.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "idle_breathe",
    duration_seconds = 2.0,
    fps = 30,
    loop = True,
    description = "Idle breathing animation - subtle chest expansion",
    license = "CC0-1.0",
    keyframes = [
        animation_keyframe(
            time = 0.0,
            bones = {
                "spine": bone_transform(rotation = [0, 0, 0]),
                "chest": bone_transform(rotation = [0, 0, 0]),
                "head": bone_transform(rotation = [0, 0, 0])
            }
        ),
        animation_keyframe(
            time = 1.0,
            bones = {
                "spine": bone_transform(rotation = [1, 0, 0]),
                "chest": bone_transform(rotation = [2, 0, 0]),
                "head": bone_transform(rotation = [-1, 0, 0])
            }
        ),
        animation_keyframe(
            time = 2.0,
            bones = {
                "spine": bone_transform(rotation = [0, 0, 0]),
                "chest": bone_transform(rotation = [0, 0, 0]),
                "head": bone_transform(rotation = [0, 0, 0])
            }
        )
    ],
    interpolation = "bezier"
)
