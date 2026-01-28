"""
Constraints module for SpecCade Blender rigging.

This module handles bone constraint setup for armatures, including:
- Hinge constraints (single-axis rotation limits for elbows/knees)
- Ball constraints (cone-like rotation with twist for shoulders/hips)
- Planar constraints (rotation in a plane)
- Soft constraints (spring-like resistance)
- Foot roll systems
- Aim/look-at constraints
- Twist bone distribution
"""

import math
from typing import Dict, Optional

try:
    import bpy
except ImportError:
    bpy = None  # type: ignore


def setup_constraint(
    armature: 'bpy.types.Object',
    constraint_spec: Dict
) -> None:
    """
    Set up a bone constraint on an armature.

    Args:
        armature: The armature object.
        constraint_spec: Dictionary with constraint configuration.
            Required keys:
                - type: One of 'hinge', 'ball', 'planar', 'soft'
                - bone: Target bone name
            Type-specific keys:
                Hinge:
                    - axis: 'X', 'Y', or 'Z' (default 'X')
                    - min_angle: Minimum angle in degrees (default 0)
                    - max_angle: Maximum angle in degrees (default 160)
                Ball:
                    - cone_angle: Cone angle in degrees (default 45)
                    - twist_min: Minimum twist in degrees (default -45)
                    - twist_max: Maximum twist in degrees (default 45)
                Planar:
                    - plane_normal: Locked axis 'X', 'Y', or 'Z' (default 'X')
                    - min_angle: Minimum angle in degrees (default -30)
                    - max_angle: Maximum angle in degrees (default 30)
                Soft:
                    - stiffness: Stiffness factor 0-1 (default 0.5)
                    - damping: Damping factor 0-1 (default 0.5)
    """
    constraint_type = constraint_spec.get('type', '').lower()
    bone_name = constraint_spec.get('bone', '')

    if not bone_name:
        raise ValueError("Constraint requires a 'bone' name")

    if not constraint_type:
        raise ValueError("Constraint requires a 'type'")

    # Ensure we're in object mode first
    if bpy.context.mode != 'OBJECT':
        bpy.ops.object.mode_set(mode='OBJECT')

    bpy.context.view_layer.objects.active = armature

    # Verify bone exists
    if bone_name not in armature.data.bones:
        raise ValueError(f"Bone '{bone_name}' not found in armature")

    # Enter pose mode to add constraint
    bpy.ops.object.mode_set(mode='POSE')

    pose_bone = armature.pose.bones.get(bone_name)
    if not pose_bone:
        raise ValueError(f"Pose bone '{bone_name}' not found")

    if constraint_type == 'hinge':
        _setup_hinge_constraint(pose_bone, constraint_spec)
    elif constraint_type == 'ball':
        _setup_ball_constraint(pose_bone, constraint_spec)
    elif constraint_type == 'planar':
        _setup_planar_constraint(pose_bone, constraint_spec)
    elif constraint_type == 'soft':
        _setup_soft_constraint(pose_bone, constraint_spec)
    else:
        raise ValueError(f"Unknown constraint type: {constraint_type}")

    bpy.ops.object.mode_set(mode='OBJECT')


def _setup_hinge_constraint(
    pose_bone: 'bpy.types.PoseBone',
    constraint_spec: Dict
) -> None:
    """
    Set up a hinge constraint (single-axis rotation limit).

    Maps to LIMIT_ROTATION with only one axis enabled.
    Ideal for elbows and knees.
    """
    axis = constraint_spec.get('axis', 'X').upper()
    min_angle = constraint_spec.get('min_angle', 0.0)
    max_angle = constraint_spec.get('max_angle', 160.0)

    # Create limit rotation constraint
    constraint = pose_bone.constraints.new('LIMIT_ROTATION')
    constraint.name = f"Hinge_{pose_bone.name}"
    constraint.owner_space = 'LOCAL'

    # Enable only the specified axis, lock others to zero
    if axis == 'X':
        constraint.use_limit_x = True
        constraint.min_x = math.radians(min_angle)
        constraint.max_x = math.radians(max_angle)
        # Lock Y and Z
        constraint.use_limit_y = True
        constraint.min_y = 0
        constraint.max_y = 0
        constraint.use_limit_z = True
        constraint.min_z = 0
        constraint.max_z = 0
    elif axis == 'Y':
        constraint.use_limit_y = True
        constraint.min_y = math.radians(min_angle)
        constraint.max_y = math.radians(max_angle)
        # Lock X and Z
        constraint.use_limit_x = True
        constraint.min_x = 0
        constraint.max_x = 0
        constraint.use_limit_z = True
        constraint.min_z = 0
        constraint.max_z = 0
    elif axis == 'Z':
        constraint.use_limit_z = True
        constraint.min_z = math.radians(min_angle)
        constraint.max_z = math.radians(max_angle)
        # Lock X and Y
        constraint.use_limit_x = True
        constraint.min_x = 0
        constraint.max_x = 0
        constraint.use_limit_y = True
        constraint.min_y = 0
        constraint.max_y = 0


def _setup_ball_constraint(
    pose_bone: 'bpy.types.PoseBone',
    constraint_spec: Dict
) -> None:
    """
    Set up a ball constraint (cone-like rotation limit with twist).

    Maps to LIMIT_ROTATION with cone-like limits on two axes and twist on the third.
    Ideal for shoulders and hips.
    """
    cone_angle = constraint_spec.get('cone_angle', 45.0)
    twist_min = constraint_spec.get('twist_min', -45.0)
    twist_max = constraint_spec.get('twist_max', 45.0)

    # Create limit rotation constraint
    constraint = pose_bone.constraints.new('LIMIT_ROTATION')
    constraint.name = f"Ball_{pose_bone.name}"
    constraint.owner_space = 'LOCAL'

    # Y axis is the twist axis (along the bone)
    # X and Z axes form the cone
    constraint.use_limit_x = True
    constraint.min_x = math.radians(-cone_angle)
    constraint.max_x = math.radians(cone_angle)

    constraint.use_limit_y = True
    constraint.min_y = math.radians(twist_min)
    constraint.max_y = math.radians(twist_max)

    constraint.use_limit_z = True
    constraint.min_z = math.radians(-cone_angle)
    constraint.max_z = math.radians(cone_angle)


def _setup_planar_constraint(
    pose_bone: 'bpy.types.PoseBone',
    constraint_spec: Dict
) -> None:
    """
    Set up a planar constraint (rotation in a plane, one axis locked).

    Maps to LIMIT_ROTATION with one axis locked to zero.
    Ideal for wrists and ankles with limited lateral movement.
    """
    plane_normal = constraint_spec.get('plane_normal', 'X').upper()
    min_angle = constraint_spec.get('min_angle', -30.0)
    max_angle = constraint_spec.get('max_angle', 30.0)

    # Create limit rotation constraint
    constraint = pose_bone.constraints.new('LIMIT_ROTATION')
    constraint.name = f"Planar_{pose_bone.name}"
    constraint.owner_space = 'LOCAL'

    # Lock the plane normal axis, allow rotation on the other two
    if plane_normal == 'X':
        # Lock X, allow Y and Z
        constraint.use_limit_x = True
        constraint.min_x = 0
        constraint.max_x = 0
        constraint.use_limit_y = True
        constraint.min_y = math.radians(min_angle)
        constraint.max_y = math.radians(max_angle)
        constraint.use_limit_z = True
        constraint.min_z = math.radians(min_angle)
        constraint.max_z = math.radians(max_angle)
    elif plane_normal == 'Y':
        # Lock Y, allow X and Z
        constraint.use_limit_x = True
        constraint.min_x = math.radians(min_angle)
        constraint.max_x = math.radians(max_angle)
        constraint.use_limit_y = True
        constraint.min_y = 0
        constraint.max_y = 0
        constraint.use_limit_z = True
        constraint.min_z = math.radians(min_angle)
        constraint.max_z = math.radians(max_angle)
    elif plane_normal == 'Z':
        # Lock Z, allow X and Y
        constraint.use_limit_x = True
        constraint.min_x = math.radians(min_angle)
        constraint.max_x = math.radians(max_angle)
        constraint.use_limit_y = True
        constraint.min_y = math.radians(min_angle)
        constraint.max_y = math.radians(max_angle)
        constraint.use_limit_z = True
        constraint.min_z = 0
        constraint.max_z = 0


def _setup_soft_constraint(
    pose_bone: 'bpy.types.PoseBone',
    constraint_spec: Dict
) -> None:
    """
    Set up a soft constraint (spring-like resistance with damping).

    Uses COPY_ROTATION with limited influence and a DAMPED_TRACK for damping effect.
    Ideal for secondary motion like tails and hair.

    Note: True spring dynamics require baking or the use of rigid body physics.
    This implementation provides a simplified approximation using constraint influence.
    """
    stiffness = constraint_spec.get('stiffness', 0.5)
    damping = constraint_spec.get('damping', 0.5)

    # Clamp values to valid range
    stiffness = max(0.0, min(1.0, stiffness))
    damping = max(0.0, min(1.0, damping))

    # For soft constraints, we use a combination approach:
    # 1. Low stiffness = more freedom (less constraint influence)
    # 2. High damping = more smoothing (we approximate with limit constraints)

    # If stiffness is 0, we don't add any constraint (fully free)
    if stiffness <= 0.0:
        return

    # Create a limit rotation constraint with very wide limits
    # The stiffness affects how quickly the bone returns to rest
    constraint = pose_bone.constraints.new('LIMIT_ROTATION')
    constraint.name = f"Soft_{pose_bone.name}"
    constraint.owner_space = 'LOCAL'

    # Wide limits scaled by inverse of stiffness (lower stiffness = wider limits)
    max_angle = 90.0 * (1.0 - stiffness * 0.5)  # 45-90 degrees based on stiffness

    constraint.use_limit_x = True
    constraint.min_x = math.radians(-max_angle)
    constraint.max_x = math.radians(max_angle)

    constraint.use_limit_y = True
    constraint.min_y = math.radians(-max_angle)
    constraint.max_y = math.radians(max_angle)

    constraint.use_limit_z = True
    constraint.min_z = math.radians(-max_angle)
    constraint.max_z = math.radians(max_angle)

    # The influence is inversely related to stiffness
    # Higher stiffness = more constraint influence
    constraint.influence = stiffness

    # Add a damped track if damping is significant
    # This helps smooth out rapid movements
    if damping > 0.3:
        # We add a slight copy rotation from parent to simulate damping lag
        # This is a simplification - true damping requires simulation
        if pose_bone.parent:
            damp_constraint = pose_bone.constraints.new('COPY_ROTATION')
            damp_constraint.name = f"SoftDamp_{pose_bone.name}"
            damp_constraint.target = pose_bone.id_data  # The armature
            damp_constraint.subtarget = pose_bone.parent.name
            damp_constraint.influence = damping * 0.3  # Subtle effect
            damp_constraint.mix_mode = 'ADD'
            damp_constraint.owner_space = 'LOCAL'
            damp_constraint.target_space = 'LOCAL'


def setup_foot_system(armature: 'bpy.types.Object', foot_system: Dict) -> None:
    """
    Set up an IK foot roll system.

    Creates a reverse foot rig with heel, ball, and toe pivots for natural
    foot movement during IK animation.

    Args:
        armature: The armature object.
        foot_system: Dictionary with foot system configuration:
            - name: Name of the foot system
            - ik_target: IK target bone name
            - heel_bone: Heel pivot bone name
            - toe_bone: Toe pivot bone name
            - ball_bone: Optional ball (mid-foot) pivot bone name
            - roll_limits: [min, max] roll angle limits in degrees
    """
    name = foot_system.get('name', 'foot')
    ik_target = foot_system.get('ik_target', '')
    heel_bone = foot_system.get('heel_bone', '')
    toe_bone = foot_system.get('toe_bone', '')
    ball_bone = foot_system.get('ball_bone')
    roll_limits = foot_system.get('roll_limits', [-30.0, 60.0])

    if not ik_target or not heel_bone or not toe_bone:
        raise ValueError(f"Foot system '{name}' requires ik_target, heel_bone, and toe_bone")

    # Ensure we're in object mode first
    if bpy.context.mode != 'OBJECT':
        bpy.ops.object.mode_set(mode='OBJECT')

    bpy.context.view_layer.objects.active = armature

    # Verify bones exist
    for bone_name in [heel_bone, toe_bone] + ([ball_bone] if ball_bone else []):
        if bone_name not in armature.data.bones:
            raise ValueError(f"Bone '{bone_name}' not found in armature for foot system '{name}'")

    # Enter pose mode to add constraints
    bpy.ops.object.mode_set(mode='POSE')

    # Add roll limit constraint to heel
    heel_pose = armature.pose.bones.get(heel_bone)
    if heel_pose:
        constraint = heel_pose.constraints.new('LIMIT_ROTATION')
        constraint.name = f"FootRoll_{name}_Heel"
        constraint.owner_space = 'LOCAL'
        constraint.use_limit_x = True
        constraint.min_x = math.radians(roll_limits[0])
        constraint.max_x = math.radians(roll_limits[1])

    # Add roll limit constraint to toe
    toe_pose = armature.pose.bones.get(toe_bone)
    if toe_pose:
        constraint = toe_pose.constraints.new('LIMIT_ROTATION')
        constraint.name = f"FootRoll_{name}_Toe"
        constraint.owner_space = 'LOCAL'
        constraint.use_limit_x = True
        constraint.min_x = math.radians(0)  # Toe only lifts up
        constraint.max_x = math.radians(roll_limits[1])

    # Add ball bone constraints if present
    if ball_bone:
        ball_pose = armature.pose.bones.get(ball_bone)
        if ball_pose:
            constraint = ball_pose.constraints.new('LIMIT_ROTATION')
            constraint.name = f"FootRoll_{name}_Ball"
            constraint.owner_space = 'LOCAL'
            constraint.use_limit_x = True
            constraint.min_x = math.radians(roll_limits[0] * 0.5)
            constraint.max_x = math.radians(roll_limits[1] * 0.5)

    bpy.ops.object.mode_set(mode='OBJECT')
    print(f"Set up foot system: {name}")


def setup_aim_constraint(armature: 'bpy.types.Object', aim_spec: Dict) -> None:
    """
    Set up an aim (look-at) constraint.

    Makes a bone always point toward a target, useful for eyes,
    weapons, or tracking systems.

    Args:
        armature: The armature object.
        aim_spec: Dictionary with aim constraint configuration:
            - name: Constraint name
            - bone: Bone to apply the constraint to
            - target: Target to aim at (bone name or empty object name)
            - track_axis: Axis to point at target ('X', '-X', 'Y', '-Y', 'Z', '-Z')
            - up_axis: Up reference axis ('X', 'Y', 'Z')
            - influence: Constraint influence (0.0-1.0)
    """
    name = aim_spec.get('name', 'aim')
    bone_name = aim_spec.get('bone', '')
    target = aim_spec.get('target', '')
    track_axis = aim_spec.get('track_axis', 'X')
    up_axis = aim_spec.get('up_axis', 'Z')
    influence = aim_spec.get('influence', 1.0)

    if not bone_name or not target:
        raise ValueError(f"Aim constraint '{name}' requires bone and target")

    # Ensure we're in object mode first
    if bpy.context.mode != 'OBJECT':
        bpy.ops.object.mode_set(mode='OBJECT')

    bpy.context.view_layer.objects.active = armature

    if bone_name not in armature.data.bones:
        raise ValueError(f"Bone '{bone_name}' not found in armature")

    bpy.ops.object.mode_set(mode='POSE')

    pose_bone = armature.pose.bones.get(bone_name)
    if not pose_bone:
        raise ValueError(f"Pose bone '{bone_name}' not found")

    # Create track to constraint
    constraint = pose_bone.constraints.new('DAMPED_TRACK')
    constraint.name = name
    constraint.influence = max(0.0, min(1.0, influence))

    # Set target (either a bone in this armature or external object)
    if target in armature.data.bones:
        constraint.target = armature
        constraint.subtarget = target
    else:
        target_obj = bpy.data.objects.get(target)
        if target_obj:
            constraint.target = target_obj
        else:
            # Create an empty as the target
            bpy.ops.object.mode_set(mode='OBJECT')
            bpy.ops.object.empty_add(type='PLAIN_AXES')
            target_empty = bpy.context.active_object
            target_empty.name = target
            bpy.ops.object.mode_set(mode='POSE')
            constraint.target = target_empty

    # Map track axis
    track_axis_map = {
        'X': 'TRACK_X',
        '-X': 'TRACK_NEGATIVE_X',
        'Y': 'TRACK_Y',
        '-Y': 'TRACK_NEGATIVE_Y',
        'Z': 'TRACK_Z',
        '-Z': 'TRACK_NEGATIVE_Z',
    }
    constraint.track_axis = track_axis_map.get(track_axis, 'TRACK_X')

    bpy.ops.object.mode_set(mode='OBJECT')
    print(f"Set up aim constraint: {name}")


def setup_twist_bone(armature: 'bpy.types.Object', twist_spec: Dict) -> None:
    """
    Set up twist bone distribution.

    Distributes rotation from a source bone to a twist bone,
    useful for preventing candy-wrapper deformation in forearms and thighs.

    Args:
        armature: The armature object.
        twist_spec: Dictionary with twist bone configuration:
            - name: Optional name for this twist setup
            - source: Source bone to copy rotation from
            - target: Target twist bone
            - axis: Axis to copy rotation on ('X', 'Y', 'Z')
            - influence: Influence factor (0.0-1.0)
    """
    name = twist_spec.get('name', 'twist')
    source = twist_spec.get('source', '')
    target = twist_spec.get('target', '')
    axis = twist_spec.get('axis', 'Y').upper()
    influence = twist_spec.get('influence', 0.5)

    if not source or not target:
        raise ValueError(f"Twist setup '{name}' requires source and target bones")

    # Ensure we're in object mode first
    if bpy.context.mode != 'OBJECT':
        bpy.ops.object.mode_set(mode='OBJECT')

    bpy.context.view_layer.objects.active = armature

    for bone_name in [source, target]:
        if bone_name not in armature.data.bones:
            raise ValueError(f"Bone '{bone_name}' not found in armature")

    bpy.ops.object.mode_set(mode='POSE')

    target_pose = armature.pose.bones.get(target)
    if not target_pose:
        raise ValueError(f"Pose bone '{target}' not found")

    # Create copy rotation constraint
    constraint = target_pose.constraints.new('COPY_ROTATION')
    constraint.name = f"Twist_{name}"
    constraint.target = armature
    constraint.subtarget = source
    constraint.influence = max(0.0, min(1.0, influence))
    constraint.mix_mode = 'ADD'
    constraint.owner_space = 'LOCAL'
    constraint.target_space = 'LOCAL'

    # Only copy the specified axis
    constraint.use_x = (axis == 'X')
    constraint.use_y = (axis == 'Y')
    constraint.use_z = (axis == 'Z')

    bpy.ops.object.mode_set(mode='OBJECT')
    print(f"Set up twist bone: {target} from {source}")
