# Animation IK coverage example
#
# Demonstrates animation stdlib functions for IK and custom bones.
# Covers: custom_bone, ik_keyframe, ik_target_transform

# custom_bone creates a custom bone definition for custom skeletons
# (roll is not a parameter in custom_bone - use the skeleton definition for bone roll)
custom = custom_bone(
    bone = "tail_01",
    parent = "spine_02",
    head = [0.0, 0.0, 0.5],
    tail = [0.0, 0.0, 1.0]
)

custom_2 = custom_bone(
    bone = "wing_l_01",
    parent = "spine_01",
    head = [0.5, 0.0, 0.3],
    tail = [1.0, 0.0, 0.3]
)

# ik_target_transform creates IK target position/rotation
foot_target = ik_target_transform(
    position = [0.0, 0.0, 0.0],
    rotation = [0.0, 0.0, 0.0, 1.0]
)

hand_target = ik_target_transform(
    position = [0.5, 0.5, 1.0],
    rotation = [0.0, 0.707, 0.0, 0.707]
)

# ik_keyframe creates an IK keyframe at a specific time
ik_frame_0 = ik_keyframe(
    time = 0.0,
    targets = {
        "foot_ik_l": ik_target_transform([0.2, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0]),
        "foot_ik_r": ik_target_transform([-0.2, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0]),
        "hand_ik_l": ik_target_transform([0.3, 0.3, 0.8], [0.0, 0.0, 0.0, 1.0]),
        "hand_ik_r": ik_target_transform([-0.3, 0.3, 0.8], [0.0, 0.0, 0.0, 1.0])
    }
)

ik_frame_1 = ik_keyframe(
    time = 0.5,
    targets = {
        "foot_ik_l": ik_target_transform([0.2, 0.3, 0.1], [0.0, 0.0, 0.0, 1.0]),
        "foot_ik_r": ik_target_transform([-0.2, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0]),
        "hand_ik_l": ik_target_transform([0.4, 0.4, 0.9], [0.0, 0.0, 0.0, 1.0]),
        "hand_ik_r": ik_target_transform([-0.4, 0.2, 0.7], [0.0, 0.0, 0.0, 1.0])
    }
)

# Create a skeletal animation spec that would use IK data
skeletal_animation_spec(
    asset_id = "stdlib-animation-ik-coverage-01",
    seed = 42,
    output_path = "animations/ik_coverage.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "ik_test",
    duration_seconds = 1.0,
    fps = 24,
    loop = False,
    description = "IK animation coverage example",
    keyframes = [
        animation_keyframe(
            time = 0.0,
            bones = {
                "upper_leg_l": bone_transform(rotation = [0.0, 0.0, 0.0]),
                "upper_leg_r": bone_transform(rotation = [0.0, 0.0, 0.0])
            }
        ),
        animation_keyframe(
            time = 1.0,
            bones = {
                "upper_leg_l": bone_transform(rotation = [10.0, 0.0, 0.0]),
                "upper_leg_r": bone_transform(rotation = [-10.0, 0.0, 0.0])
            }
        )
    ],
    interpolation = "linear"
)
