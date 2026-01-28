# Simple walk cycle animation - demonstrates animation stdlib
#
# This example creates a basic walk cycle for a humanoid skeleton
# using the animation stdlib functions for keyframes and transforms.
#
# [VALIDATION]
# SHAPE: Humanoid skeleton performing walk cycle
# MOTION: Legs swing forward/back alternately, arms counter-swing, spine subtle tilt
# FRAME 0: Contact pose - left leg forward (25 deg), right leg back (-25 deg), left arm back, right arm forward
# FRAME 12 (0.5s): Passing pose - legs swapped, right leg forward, left leg back
# FRAME 24 (1.0s): Return to contact pose (loop point)
# ORIENTATION: Character facing +Y, walking in place (no root motion)
# FRONT VIEW (frame 0): Left leg in front, right arm in front
# FRONT VIEW (frame 12): Right leg in front, left arm in front
# ISO VIEW: Full body visible, leg swing angle clearly visible (~25 degrees)
# NOTES: Linear interpolation, smooth transitions, loop=true so frame 24 matches frame 0

skeletal_animation_spec(
    asset_id = "stdlib-animation-walk-cycle-01",
    seed = 42,
    output_path = "animations/walk_cycle.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "walk",
    duration_seconds = 1.0,
    fps = 24,
    loop = True,
    description = "Stdlib walk cycle animation example",
    keyframes = [
        # Frame 0: Contact pose (left foot forward)
        animation_keyframe(
            time = 0.0,
            bones = {
                "upper_leg_l": bone_transform(rotation = [25.0, 0.0, 0.0]),
                "upper_leg_r": bone_transform(rotation = [-25.0, 0.0, 0.0]),
                "lower_leg_l": bone_transform(rotation = [-15.0, 0.0, 0.0]),
                "lower_leg_r": bone_transform(rotation = [10.0, 0.0, 0.0]),
                "spine": bone_transform(rotation = [2.0, 0.0, 0.0]),
                "upper_arm_l": bone_transform(rotation = [-15.0, 0.0, -5.0]),
                "upper_arm_r": bone_transform(rotation = [15.0, 0.0, 5.0])
            }
        ),
        # Frame 12: Passing pose
        animation_keyframe(
            time = 0.5,
            bones = {
                "upper_leg_l": bone_transform(rotation = [-25.0, 0.0, 0.0]),
                "upper_leg_r": bone_transform(rotation = [25.0, 0.0, 0.0]),
                "lower_leg_l": bone_transform(rotation = [10.0, 0.0, 0.0]),
                "lower_leg_r": bone_transform(rotation = [-15.0, 0.0, 0.0]),
                "spine": bone_transform(rotation = [-2.0, 0.0, 0.0]),
                "upper_arm_l": bone_transform(rotation = [15.0, 0.0, 5.0]),
                "upper_arm_r": bone_transform(rotation = [-15.0, 0.0, -5.0])
            }
        ),
        # Frame 24: Return to contact pose (cycle complete)
        animation_keyframe(
            time = 1.0,
            bones = {
                "upper_leg_l": bone_transform(rotation = [25.0, 0.0, 0.0]),
                "upper_leg_r": bone_transform(rotation = [-25.0, 0.0, 0.0]),
                "lower_leg_l": bone_transform(rotation = [-15.0, 0.0, 0.0]),
                "lower_leg_r": bone_transform(rotation = [10.0, 0.0, 0.0]),
                "spine": bone_transform(rotation = [2.0, 0.0, 0.0]),
                "upper_arm_l": bone_transform(rotation = [-15.0, 0.0, -5.0]),
                "upper_arm_r": bone_transform(rotation = [15.0, 0.0, 5.0])
            }
        )
    ],
    interpolation = "linear",
    export = animation_export_settings(
        bake_transforms = True,
        optimize_keyframes = False
    )
)
