#!/usr/bin/env python3
"""
Animation Helpers Operator for SpecCade.

This module provides preset-based animation generation for common locomotion
patterns including walk cycles, run cycles, and idle sway animations.

Usage:
    This module is designed to be imported and used by the main entrypoint.py
    for handling 'animation.helpers_v1' recipe types.
"""

import math
from typing import Any, Dict, List, Optional, Tuple

try:
    import bpy
    from mathutils import Euler, Vector
    BLENDER_AVAILABLE = True
except ImportError:
    BLENDER_AVAILABLE = False


# =============================================================================
# Preset Configurations
# =============================================================================

# Default preset configurations for each animation type
PRESET_CONFIGS = {
    "walk_cycle": {
        "stride_length": 0.8,
        "cycle_frames": 60,
        "foot_roll": True,
        "arm_swing": 0.3,
        "hip_sway": 3.0,
        "spine_twist": 5.0,
        "foot_lift": 0.15,
    },
    "run_cycle": {
        "stride_length": 1.2,
        "cycle_frames": 30,
        "foot_roll": True,
        "arm_swing": 0.5,
        "hip_sway": 5.0,
        "spine_twist": 8.0,
        "foot_lift": 0.25,
    },
    "idle_sway": {
        "stride_length": 0.0,
        "cycle_frames": 120,
        "foot_roll": False,
        "arm_swing": 0.05,
        "hip_sway": 2.0,
        "spine_twist": 1.0,
        "foot_lift": 0.0,
    },
}

# Default IK configurations per limb
DEFAULT_IK_SETTINGS = {
    "foot_l": {"pole_angle": 90.0, "chain_length": 2},
    "foot_r": {"pole_angle": 90.0, "chain_length": 2},
    "hand_l": {"pole_angle": -90.0, "chain_length": 2},
    "hand_r": {"pole_angle": -90.0, "chain_length": 2},
}


# =============================================================================
# Foot Roll System
# =============================================================================

def create_foot_roll_bones(
    armature: 'bpy.types.Object',
    side: str,
    foot_bone_name: str,
    roll_limits: Tuple[float, float] = (-30.0, 60.0)
) -> Dict[str, str]:
    """
    Create foot roll pivot bones for a heel-toe roll system.

    Args:
        armature: The armature object.
        side: 'l' or 'r' for left/right.
        foot_bone_name: Name of the foot bone to attach roll system to.
        roll_limits: (min, max) roll angle limits in degrees.

    Returns:
        Dictionary mapping role names to created bone names.
    """
    if not BLENDER_AVAILABLE:
        return {}

    result = {}

    # Ensure we're in edit mode
    bpy.context.view_layer.objects.active = armature
    original_mode = bpy.context.mode
    if original_mode != 'EDIT_ARMATURE':
        bpy.ops.object.mode_set(mode='EDIT')

    edit_bones = armature.data.edit_bones

    # Find the foot bone
    foot_bone = edit_bones.get(foot_bone_name)
    if not foot_bone:
        print(f"Warning: Foot bone '{foot_bone_name}' not found")
        if original_mode != 'EDIT_ARMATURE':
            bpy.ops.object.mode_set(mode='OBJECT')
        return result

    # Calculate pivot positions based on foot geometry
    foot_head = foot_bone.head.copy()
    foot_tail = foot_bone.tail.copy()
    foot_length = (foot_tail - foot_head).length

    # Heel pivot - behind the foot
    heel_name = f"heel_roll_{side}"
    heel_bone = edit_bones.new(heel_name)
    heel_offset = Vector((0, -0.1, 0))  # Behind foot
    heel_bone.head = foot_head + heel_offset
    heel_bone.tail = heel_bone.head + Vector((0, 0.05, 0))
    heel_bone.roll = 0
    result["heel"] = heel_name

    # Ball pivot - mid-foot
    ball_name = f"ball_roll_{side}"
    ball_bone = edit_bones.new(ball_name)
    ball_pos = foot_head + (foot_tail - foot_head) * 0.5
    ball_bone.head = ball_pos
    ball_bone.tail = ball_pos + Vector((0, 0.05, 0))
    ball_bone.roll = 0
    ball_bone.parent = heel_bone
    result["ball"] = ball_name

    # Toe pivot - at the toe tip
    toe_name = f"toe_roll_{side}"
    toe_bone = edit_bones.new(toe_name)
    toe_bone.head = foot_tail
    toe_bone.tail = foot_tail + Vector((0, 0.05, 0))
    toe_bone.roll = 0
    toe_bone.parent = ball_bone
    result["toe"] = toe_name

    if original_mode != 'EDIT_ARMATURE':
        bpy.ops.object.mode_set(mode='OBJECT')

    return result


def setup_foot_roll_drivers(
    armature: 'bpy.types.Object',
    side: str,
    roll_bones: Dict[str, str],
    ik_target_name: str
) -> None:
    """
    Set up drivers for automatic foot roll based on IK target rotation.

    Args:
        armature: The armature object.
        side: 'l' or 'r' for left/right.
        roll_bones: Dictionary of roll bone names from create_foot_roll_bones.
        ik_target_name: Name of the IK target controlling this foot.
    """
    if not BLENDER_AVAILABLE or not roll_bones:
        return

    # Get the IK target
    ik_target = bpy.data.objects.get(ik_target_name)
    if not ik_target:
        print(f"Warning: IK target '{ik_target_name}' not found for foot roll")
        return

    bpy.context.view_layer.objects.active = armature
    bpy.ops.object.mode_set(mode='POSE')

    # Add a custom property to the IK target for roll control
    if "foot_roll" not in ik_target:
        ik_target["foot_roll"] = 0.0
        # Add min/max for the property
        id_props = ik_target.id_properties_ui("foot_roll")
        id_props.update(min=-1.0, max=1.0, soft_min=-1.0, soft_max=1.0)

    # Set up drivers for heel/toe roll based on the foot_roll property
    heel_bone = armature.pose.bones.get(roll_bones.get("heel", ""))
    ball_bone = armature.pose.bones.get(roll_bones.get("ball", ""))
    toe_bone = armature.pose.bones.get(roll_bones.get("toe", ""))

    if heel_bone and ball_bone:
        # Heel rotates for negative roll (heel down)
        heel_driver = heel_bone.driver_add("rotation_euler", 0)
        if heel_driver:
            heel_driver.driver.type = 'SCRIPTED'
            var = heel_driver.driver.variables.new()
            var.name = "roll"
            var.type = 'SINGLE_PROP'
            var.targets[0].id = ik_target
            var.targets[0].data_path = '["foot_roll"]'
            # Heel rotates up when roll < 0
            heel_driver.driver.expression = "min(0, roll * 0.52)"  # ~30 degrees max

    if toe_bone:
        # Toe rotates for positive roll (toe up)
        toe_driver = toe_bone.driver_add("rotation_euler", 0)
        if toe_driver:
            toe_driver.driver.type = 'SCRIPTED'
            var = toe_driver.driver.variables.new()
            var.name = "roll"
            var.type = 'SINGLE_PROP'
            var.targets[0].id = ik_target
            var.targets[0].data_path = '["foot_roll"]'
            # Toe rotates up when roll > 0
            toe_driver.driver.expression = "max(0, roll * 1.05)"  # ~60 degrees max

    bpy.ops.object.mode_set(mode='OBJECT')


# =============================================================================
# IK Setup for Animation Helpers
# =============================================================================

def setup_ik_for_locomotion(
    armature: 'bpy.types.Object',
    skeleton_type: str,
    ik_targets: Dict[str, Dict]
) -> Dict[str, Any]:
    """
    Set up IK chains for locomotion animation.

    Args:
        armature: The armature object.
        skeleton_type: 'humanoid' or 'quadruped'.
        ik_targets: Per-limb IK target configurations.

    Returns:
        Dictionary mapping IK chain names to their control objects.
    """
    if not BLENDER_AVAILABLE:
        return {}

    result = {}

    if skeleton_type == "humanoid":
        # Set up leg IK
        for side in ['l', 'r']:
            limb_name = f"foot_{side}"
            settings = ik_targets.get(limb_name, DEFAULT_IK_SETTINGS.get(limb_name, {}))

            try:
                chain_result = setup_limb_ik(
                    armature,
                    tip_bone=f"lower_leg_{side}",
                    chain_length=settings.get("chain_length", 2),
                    target_name=f"ik_foot_{side}",
                    pole_name=f"pole_knee_{side}",
                    pole_angle=settings.get("pole_angle", 90.0),
                    pole_offset=Vector((0.1 if side == 'l' else -0.1, 0.3, 0.5))
                )
                result[f"ik_leg_{side}"] = chain_result
            except Exception as e:
                print(f"Warning: Failed to set up leg IK for {side}: {e}")

        # Set up arm IK
        for side in ['l', 'r']:
            limb_name = f"hand_{side}"
            settings = ik_targets.get(limb_name, DEFAULT_IK_SETTINGS.get(limb_name, {}))

            try:
                chain_result = setup_limb_ik(
                    armature,
                    tip_bone=f"lower_arm_{side}",
                    chain_length=settings.get("chain_length", 2),
                    target_name=f"ik_hand_{side}",
                    pole_name=f"pole_elbow_{side}",
                    pole_angle=settings.get("pole_angle", -90.0),
                    pole_offset=Vector((0.3 if side == 'l' else -0.3, -0.3, 1.35))
                )
                result[f"ik_arm_{side}"] = chain_result
            except Exception as e:
                print(f"Warning: Failed to set up arm IK for {side}: {e}")

    elif skeleton_type == "quadruped":
        # Set up quadruped IK (forelegs and hindlegs)
        for prefix, position in [("front", "foreleg"), ("back", "hindleg")]:
            for side in ['l', 'r']:
                try:
                    chain_result = setup_limb_ik(
                        armature,
                        tip_bone=f"{position}_lower_{side}",
                        chain_length=2,
                        target_name=f"ik_{prefix}_paw_{side}",
                        pole_name=f"pole_{prefix}_knee_{side}",
                        pole_angle=90.0 if prefix == "front" else -90.0,
                        pole_offset=Vector((0.15 if side == 'l' else -0.15, 0.2, 0))
                    )
                    result[f"ik_{prefix}_{side}"] = chain_result
                except Exception as e:
                    print(f"Warning: Failed to set up {prefix} leg IK for {side}: {e}")

    return result


def setup_limb_ik(
    armature: 'bpy.types.Object',
    tip_bone: str,
    chain_length: int,
    target_name: str,
    pole_name: Optional[str] = None,
    pole_angle: float = 0.0,
    pole_offset: Optional[Vector] = None
) -> Dict[str, Any]:
    """
    Set up IK for a single limb.

    Args:
        armature: The armature object.
        tip_bone: Name of the bone at the end of the IK chain.
        chain_length: Number of bones in the chain.
        target_name: Name for the IK target empty.
        pole_name: Name for the pole target empty (optional).
        pole_angle: Pole angle in degrees.
        pole_offset: Offset for pole target position.

    Returns:
        Dictionary with 'target' and optional 'pole' objects.
    """
    result = {}

    # Get the tip bone
    bone = armature.data.bones.get(tip_bone)
    if not bone:
        raise ValueError(f"Bone '{tip_bone}' not found in armature")

    # Ensure we're in object mode
    if bpy.context.mode != 'OBJECT':
        bpy.ops.object.mode_set(mode='OBJECT')

    # Create IK target empty
    bpy.ops.object.empty_add(type='PLAIN_AXES', radius=0.1)
    target = bpy.context.active_object
    target.name = target_name
    target.location = armature.matrix_world @ bone.tail_local
    result['target'] = target

    # Create pole target if specified
    pole_obj = None
    if pole_name:
        bpy.ops.object.empty_add(type='PLAIN_AXES', radius=0.08)
        pole_obj = bpy.context.active_object
        pole_obj.name = pole_name

        if pole_offset:
            pole_obj.location = pole_offset
        else:
            # Default position: in front of the middle bone
            middle_bone = bone.parent if bone.parent else bone
            mid_pos = armature.matrix_world @ middle_bone.head_local
            pole_obj.location = mid_pos + Vector((0, 0.3, 0))

        result['pole'] = pole_obj

    # Add IK constraint
    bpy.context.view_layer.objects.active = armature
    bpy.ops.object.mode_set(mode='POSE')

    pose_bone = armature.pose.bones.get(tip_bone)
    if pose_bone:
        ik_constraint = pose_bone.constraints.new('IK')
        ik_constraint.name = f"IK_{target_name}"
        ik_constraint.target = target
        ik_constraint.chain_count = chain_length
        ik_constraint.influence = 1.0

        if pole_obj:
            ik_constraint.pole_target = pole_obj
            ik_constraint.pole_angle = math.radians(pole_angle)

    bpy.ops.object.mode_set(mode='OBJECT')

    return result


# =============================================================================
# Locomotion Cycle Generation
# =============================================================================

def generate_walk_cycle_keyframes(
    armature: 'bpy.types.Object',
    ik_controls: Dict[str, Any],
    settings: Dict[str, Any],
    fps: int = 30
) -> None:
    """
    Generate walk cycle animation keyframes.

    Args:
        armature: The armature object.
        ik_controls: Dictionary of IK control objects.
        settings: Cycle settings from spec.
        fps: Frames per second.
    """
    if not BLENDER_AVAILABLE:
        return

    cycle_frames = settings.get("cycle_frames", 60)
    stride_length = settings.get("stride_length", 0.8)
    foot_lift = settings.get("foot_lift", 0.15)
    arm_swing = settings.get("arm_swing", 0.3)
    hip_sway = settings.get("hip_sway", 3.0)
    spine_twist = settings.get("spine_twist", 5.0)

    half_cycle = cycle_frames // 2
    quarter_cycle = cycle_frames // 4

    # Set scene frame range
    bpy.context.scene.frame_start = 1
    bpy.context.scene.frame_end = cycle_frames
    bpy.context.scene.render.fps = fps

    # Get IK targets
    foot_l = ik_controls.get("ik_leg_l", {}).get("target")
    foot_r = ik_controls.get("ik_leg_r", {}).get("target")
    hand_l = ik_controls.get("ik_arm_l", {}).get("target")
    hand_r = ik_controls.get("ik_arm_r", {}).get("target")

    # Generate foot positions through the cycle
    for frame in range(1, cycle_frames + 1):
        bpy.context.scene.frame_set(frame)

        # Calculate phase (0-1) for left leg (right leg is offset by 0.5)
        phase_l = (frame - 1) / cycle_frames
        phase_r = (phase_l + 0.5) % 1.0

        if foot_l:
            pos = calculate_foot_position(phase_l, stride_length, foot_lift)
            foot_l.location = Vector(pos)
            foot_l.keyframe_insert(data_path="location", frame=frame)

            # Set foot roll if present
            if "foot_roll" in foot_l:
                foot_l["foot_roll"] = calculate_foot_roll(phase_l)
                foot_l.keyframe_insert(data_path='["foot_roll"]', frame=frame)

        if foot_r:
            pos = calculate_foot_position(phase_r, stride_length, foot_lift)
            pos[0] = -pos[0]  # Mirror X for right side
            foot_r.location = Vector(pos)
            foot_r.keyframe_insert(data_path="location", frame=frame)

            if "foot_roll" in foot_r:
                foot_r["foot_roll"] = calculate_foot_roll(phase_r)
                foot_r.keyframe_insert(data_path='["foot_roll"]', frame=frame)

        # Arm swing (opposite to legs)
        if hand_l and arm_swing > 0:
            swing_angle = math.sin(phase_r * math.pi * 2) * arm_swing * 0.5
            hand_l.location.y += swing_angle
            hand_l.keyframe_insert(data_path="location", frame=frame)

        if hand_r and arm_swing > 0:
            swing_angle = math.sin(phase_l * math.pi * 2) * arm_swing * 0.5
            hand_r.location.y += swing_angle
            hand_r.keyframe_insert(data_path="location", frame=frame)

    # Apply hip sway and spine twist to armature bones
    bpy.context.view_layer.objects.active = armature
    bpy.ops.object.mode_set(mode='POSE')

    for frame in range(1, cycle_frames + 1):
        bpy.context.scene.frame_set(frame)
        phase = (frame - 1) / cycle_frames

        # Hip sway
        hips_bone = armature.pose.bones.get("hips")
        if hips_bone and hip_sway > 0:
            sway = math.sin(phase * math.pi * 2) * math.radians(hip_sway)
            hips_bone.rotation_euler[2] = sway
            hips_bone.keyframe_insert(data_path="rotation_euler", frame=frame)

        # Spine twist
        spine_bone = armature.pose.bones.get("spine")
        if spine_bone and spine_twist > 0:
            twist = math.sin(phase * math.pi * 2) * math.radians(spine_twist)
            spine_bone.rotation_euler[1] = twist
            spine_bone.keyframe_insert(data_path="rotation_euler", frame=frame)

    bpy.ops.object.mode_set(mode='OBJECT')


def generate_run_cycle_keyframes(
    armature: 'bpy.types.Object',
    ik_controls: Dict[str, Any],
    settings: Dict[str, Any],
    fps: int = 30
) -> None:
    """
    Generate run cycle animation keyframes.
    Similar to walk but with more dynamic motion and faster timing.
    """
    # Run cycle uses similar logic to walk but with:
    # - Higher foot lift
    # - More arm swing
    # - Flight phase (both feet off ground)

    settings = settings.copy()
    settings["foot_lift"] = settings.get("foot_lift", 0.25)
    settings["arm_swing"] = settings.get("arm_swing", 0.5)

    generate_walk_cycle_keyframes(armature, ik_controls, settings, fps)


def generate_idle_sway_keyframes(
    armature: 'bpy.types.Object',
    ik_controls: Dict[str, Any],
    settings: Dict[str, Any],
    fps: int = 30
) -> None:
    """
    Generate idle sway animation keyframes.
    Subtle breathing and weight shifting.
    """
    if not BLENDER_AVAILABLE:
        return

    cycle_frames = settings.get("cycle_frames", 120)
    hip_sway = settings.get("hip_sway", 2.0)

    bpy.context.scene.frame_start = 1
    bpy.context.scene.frame_end = cycle_frames
    bpy.context.scene.render.fps = fps

    bpy.context.view_layer.objects.active = armature
    bpy.ops.object.mode_set(mode='POSE')

    for frame in range(1, cycle_frames + 1):
        bpy.context.scene.frame_set(frame)
        phase = (frame - 1) / cycle_frames

        # Subtle hip sway
        hips_bone = armature.pose.bones.get("hips")
        if hips_bone:
            sway = math.sin(phase * math.pi * 2) * math.radians(hip_sway)
            hips_bone.rotation_euler[2] = sway
            # Also add slight up/down for breathing
            hips_bone.location.z = math.sin(phase * math.pi * 4) * 0.005
            hips_bone.keyframe_insert(data_path="rotation_euler", frame=frame)
            hips_bone.keyframe_insert(data_path="location", frame=frame)

        # Subtle spine movement
        spine_bone = armature.pose.bones.get("spine")
        if spine_bone:
            # Breathing expansion
            breath = math.sin(phase * math.pi * 4) * math.radians(1.0)
            spine_bone.rotation_euler[0] = breath
            spine_bone.keyframe_insert(data_path="rotation_euler", frame=frame)

        # Subtle head movement
        head_bone = armature.pose.bones.get("head")
        if head_bone:
            look = math.sin(phase * math.pi * 2 + 0.5) * math.radians(2.0)
            head_bone.rotation_euler[1] = look
            head_bone.keyframe_insert(data_path="rotation_euler", frame=frame)

    bpy.ops.object.mode_set(mode='OBJECT')


def calculate_foot_position(phase: float, stride: float, lift: float) -> List[float]:
    """
    Calculate foot position at a given phase of the walk cycle.

    Args:
        phase: 0-1 phase through the cycle.
        stride: Total stride length.
        lift: Maximum foot lift height.

    Returns:
        [x, y, z] position relative to rest pose.
    """
    # X: lateral position (slight)
    x = 0.1  # Base offset from center

    # Y: forward/back position
    # Contact at phase 0, passing at 0.25, toe-off at 0.5, swing through to 1.0
    if phase < 0.5:
        # Contact to toe-off: foot moves backward relative to body
        y = stride * (0.5 - phase * 2)
    else:
        # Swing phase: foot moves forward
        y = stride * ((phase - 0.5) * 2 - 0.5)

    # Z: vertical position
    if phase < 0.15:
        # Contact phase: foot on ground
        z = 0
    elif phase < 0.35:
        # Lift off: foot rises
        t = (phase - 0.15) / 0.2
        z = lift * math.sin(t * math.pi)
    elif phase < 0.5:
        # Mid-swing: foot at max height
        z = lift
    elif phase < 0.65:
        # Descending
        t = (phase - 0.5) / 0.15
        z = lift * (1 - t)
    else:
        # Heel strike: approaching ground
        z = 0

    return [x, y, z]


def calculate_foot_roll(phase: float) -> float:
    """
    Calculate foot roll value at a given phase.

    Returns:
        Roll value from -1 (heel down) to 1 (toe up).
    """
    if phase < 0.15:
        # Heel strike to flat: heel down
        return -1.0 + (phase / 0.15)
    elif phase < 0.35:
        # Flat to toe-off
        t = (phase - 0.15) / 0.2
        return t
    elif phase < 0.5:
        # Toe-off: toe up
        return 1.0
    else:
        # Swing phase: neutral
        return 0.0


# =============================================================================
# Main Handler
# =============================================================================

def handle_animation_helpers(
    spec: Dict,
    armature: 'bpy.types.Object',
    out_root: 'Path'
) -> Dict[str, Any]:
    """
    Handle animation.helpers_v1 recipe type.

    Args:
        spec: The full spec dictionary.
        armature: The armature object to animate.
        out_root: Output root path.

    Returns:
        Dictionary containing metrics and generated objects.
    """
    recipe = spec.get("recipe", {})
    params = recipe.get("params", {})

    skeleton_type = params.get("skeleton", "humanoid")
    preset = params.get("preset", "walk_cycle")
    settings = params.get("settings", {})
    ik_targets = params.get("ik_targets", {})
    clip_name = params.get("clip_name", preset)
    fps = params.get("fps", 30)

    # Merge preset defaults with custom settings
    preset_config = PRESET_CONFIGS.get(preset, PRESET_CONFIGS["walk_cycle"]).copy()
    preset_config.update(settings)
    settings = preset_config

    result = {
        "preset": preset,
        "skeleton_type": skeleton_type,
        "cycle_frames": settings.get("cycle_frames", 60),
        "fps": fps,
    }

    # Set up IK chains
    ik_controls = setup_ik_for_locomotion(armature, skeleton_type, ik_targets)
    result["ik_chains_created"] = len(ik_controls)

    # Set up foot roll if enabled
    if settings.get("foot_roll", True) and skeleton_type == "humanoid":
        for side in ['l', 'r']:
            foot_bone = f"foot_{side}"
            ik_target_name = f"ik_foot_{side}"
            roll_bones = create_foot_roll_bones(armature, side, foot_bone)
            if roll_bones:
                setup_foot_roll_drivers(armature, side, roll_bones, ik_target_name)
        result["foot_roll_enabled"] = True
    else:
        result["foot_roll_enabled"] = False

    # Create animation action
    action = bpy.data.actions.new(name=clip_name)
    armature.animation_data_create()
    armature.animation_data.action = action

    # Generate keyframes based on preset
    if preset == "walk_cycle":
        generate_walk_cycle_keyframes(armature, ik_controls, settings, fps)
    elif preset == "run_cycle":
        generate_run_cycle_keyframes(armature, ik_controls, settings, fps)
    elif preset == "idle_sway":
        generate_idle_sway_keyframes(armature, ik_controls, settings, fps)
    else:
        print(f"Warning: Unknown preset '{preset}', using walk_cycle")
        generate_walk_cycle_keyframes(armature, ik_controls, settings, fps)

    result["keyframes_generated"] = True
    result["clip_name"] = clip_name
    result["action_name"] = action.name

    return result
