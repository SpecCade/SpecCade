# Animation stdlib coverage spec
#
# Covers all animation stdlib functions:
# - skeletal_animation_spec()
# - animation_keyframe()
# - bone_transform()
# - animation_export_settings()
# - ik_keyframe()
# - ik_target_transform()

# Example usage of IK functions (for coverage)
# Note: These are called to satisfy coverage requirements
ik_test_keyframe = ik_keyframe(
    time = 0.0,
    targets = {
        "ik_hand_l": ik_target_transform(
            position = [0.5, 0.0, 0.5],
            rotation = [0.0, 0.0, 0.0],
            ik_fk_blend = 1.0
        ),
        "ik_hand_r": ik_target_transform(
            position = [-0.5, 0.0, 0.5]
        )
    }
)

# Example usage of skeletal_animation_spec() with all helper functions
skeletal_animation_spec(
    asset_id = "stdlib-animation-coverage-01",
    seed = 42,
    output_path = "animations/coverage.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "coverage_test",
    duration_seconds = 1.0,
    fps = 24,
    loop = True,
    keyframes = [
        animation_keyframe(
            time = 0.0,
            bones = {
                "spine": bone_transform(rotation = [0.0, 0.0, 0.0]),
                "upper_leg_l": bone_transform(
                    position = [0.0, 0.0, 0.0],
                    rotation = [15.0, 0.0, 0.0],
                    scale = [1.0, 1.0, 1.0]
                )
            }
        ),
        animation_keyframe(
            time = 0.5,
            bones = {
                "spine": bone_transform(rotation = [5.0, 0.0, 0.0]),
                "upper_leg_l": bone_transform(rotation = [-15.0, 0.0, 0.0])
            }
        )
    ],
    interpolation = "linear",
    export = animation_export_settings(
        bake_transforms = True,
        optimize_keyframes = True,
        separate_file = False,
        save_blend = False
    ),
    description = "Coverage test for animation stdlib functions"
)
