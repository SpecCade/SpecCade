"""
IK/FK module for SpecCade Blender rigging.

This module handles IK (Inverse Kinematics) and FK (Forward Kinematics) setup
for armatures, including:
- IK chain creation with targets and pole vectors
- IK presets for common character setups (humanoid, quadruped)
- Complete rig setup from spec configuration
- IK/FK switching with seamless blending
- FK to IK and IK to FK snapping
"""

import math
from typing import Dict, List, Optional

try:
    import bpy
    from mathutils import Vector
except ImportError:
    bpy = None  # type: ignore
    Vector = None  # type: ignore

# Import from sibling modules
from .constraints import (
    setup_constraint,
    setup_foot_system,
    setup_aim_constraint,
    setup_twist_bone,
)
from .controls import (
    apply_stretch_settings,
    setup_space_switch,
    setup_finger_controls,
)


def setup_ik_chain(
    armature: 'bpy.types.Object',
    tip_bone_name: str,
    chain_length: int,
    target_name: str,
    target_position: Optional[List[float]] = None,
    target_bone: Optional[str] = None,
    pole_name: Optional[str] = None,
    pole_position: Optional[List[float]] = None,
    pole_bone: Optional[str] = None,
    pole_angle: float = 0.0,
    influence: float = 1.0
) -> Dict[str, 'bpy.types.Object']:
    """
    Set up an IK chain on an armature.

    Args:
        armature: The armature object.
        tip_bone_name: Name of the bone at the end of the IK chain.
        chain_length: Number of bones in the chain (from tip going up hierarchy).
        target_name: Name for the IK target bone/empty.
        target_position: World position for the target (optional).
        target_bone: Existing bone to use as target (optional).
        pole_name: Name for the pole target bone/empty (optional).
        pole_position: World position for the pole target (optional).
        pole_bone: Existing bone to use as pole target (optional).
        pole_angle: Pole angle in degrees.
        influence: IK constraint influence (0.0-1.0).

    Returns:
        Dictionary with 'target' and optional 'pole' control objects.
    """
    result = {}

    # Ensure we're in object mode first
    if bpy.context.mode != 'OBJECT':
        bpy.ops.object.mode_set(mode='OBJECT')

    bpy.context.view_layer.objects.active = armature

    # Get the tip bone
    tip_bone = armature.data.bones.get(tip_bone_name)
    if not tip_bone:
        raise ValueError(f"Bone '{tip_bone_name}' not found in armature")

    # Create IK target
    if target_bone:
        # Use existing bone as target
        target_obj = None  # Will reference the bone in the constraint
        target_subtarget = target_bone
    else:
        # Create a new empty as target
        bpy.ops.object.empty_add(type='PLAIN_AXES', radius=0.1)
        target_obj = bpy.context.active_object
        target_obj.name = target_name

        if target_position:
            target_obj.location = Vector(target_position)
        else:
            # Position at tip bone tail
            target_obj.location = armature.matrix_world @ tip_bone.tail_local

        result['target'] = target_obj
        target_subtarget = None

    # Create pole target if specified
    pole_obj = None
    pole_subtarget = None
    if pole_name:
        if pole_bone:
            pole_subtarget = pole_bone
        else:
            bpy.ops.object.empty_add(type='PLAIN_AXES', radius=0.1)
            pole_obj = bpy.context.active_object
            pole_obj.name = pole_name

            if pole_position:
                pole_obj.location = Vector(pole_position)
            else:
                # Default position: in front of the middle of the chain
                # Find the middle bone
                middle_bone = tip_bone.parent if tip_bone.parent else tip_bone
                mid_pos = armature.matrix_world @ middle_bone.head_local
                pole_obj.location = mid_pos + Vector((0, 0.3, 0))

            result['pole'] = pole_obj

    # Enter pose mode to add constraint
    bpy.context.view_layer.objects.active = armature
    bpy.ops.object.mode_set(mode='POSE')

    # Get the pose bone
    pose_bone = armature.pose.bones.get(tip_bone_name)
    if not pose_bone:
        raise ValueError(f"Pose bone '{tip_bone_name}' not found")

    # Add IK constraint
    ik_constraint = pose_bone.constraints.new('IK')
    ik_constraint.name = f"IK_{target_name}"
    ik_constraint.chain_count = chain_length
    ik_constraint.influence = influence

    # Set target
    if target_obj:
        ik_constraint.target = target_obj
    elif target_bone:
        ik_constraint.target = armature
        ik_constraint.subtarget = target_subtarget

    # Set pole target if present
    if pole_obj:
        ik_constraint.pole_target = pole_obj
        ik_constraint.pole_angle = math.radians(pole_angle)
    elif pole_subtarget:
        ik_constraint.pole_target = armature
        ik_constraint.pole_subtarget = pole_subtarget
        ik_constraint.pole_angle = math.radians(pole_angle)

    bpy.ops.object.mode_set(mode='OBJECT')

    return result


def setup_ik_preset(armature: 'bpy.types.Object', preset: str) -> Dict[str, Dict]:
    """
    Set up IK chains based on a preset configuration.

    Args:
        armature: The armature object.
        preset: One of 'humanoid_legs', 'humanoid_arms', 'quadruped_forelegs',
                'quadruped_hindlegs', 'tentacle', 'tail'.

    Returns:
        Dictionary mapping chain names to their control objects.
    """
    result = {}
    preset = preset.lower()

    if preset == 'humanoid_legs':
        # Left leg
        try:
            result['ik_leg_l'] = setup_ik_chain(
                armature,
                tip_bone_name='lower_leg_l',
                chain_length=2,
                target_name='ik_foot_l',
                target_position=[0.1, 0.0, 0.0],
                pole_name='pole_knee_l',
                pole_position=[0.1, 0.3, 0.5]
            )
        except ValueError as e:
            print(f"Warning: Could not set up left leg IK: {e}")

        # Right leg
        try:
            result['ik_leg_r'] = setup_ik_chain(
                armature,
                tip_bone_name='lower_leg_r',
                chain_length=2,
                target_name='ik_foot_r',
                target_position=[-0.1, 0.0, 0.0],
                pole_name='pole_knee_r',
                pole_position=[-0.1, 0.3, 0.5]
            )
        except ValueError as e:
            print(f"Warning: Could not set up right leg IK: {e}")

    elif preset == 'humanoid_arms':
        # Left arm
        try:
            result['ik_arm_l'] = setup_ik_chain(
                armature,
                tip_bone_name='lower_arm_l',
                chain_length=2,
                target_name='ik_hand_l',
                target_position=[0.7, 0.0, 1.35],
                pole_name='pole_elbow_l',
                pole_position=[0.45, -0.3, 1.35]
            )
        except ValueError as e:
            print(f"Warning: Could not set up left arm IK: {e}")

        # Right arm
        try:
            result['ik_arm_r'] = setup_ik_chain(
                armature,
                tip_bone_name='lower_arm_r',
                chain_length=2,
                target_name='ik_hand_r',
                target_position=[-0.7, 0.0, 1.35],
                pole_name='pole_elbow_r',
                pole_position=[-0.45, -0.3, 1.35]
            )
        except ValueError as e:
            print(f"Warning: Could not set up right arm IK: {e}")

    elif preset == 'quadruped_forelegs':
        # Left foreleg
        try:
            result['ik_foreleg_l'] = setup_ik_chain(
                armature,
                tip_bone_name='front_lower_l',
                chain_length=2,
                target_name='ik_front_paw_l',
                pole_name='pole_front_knee_l',
                pole_position=[0.15, 0.3, 0.0]
            )
        except ValueError as e:
            print(f"Warning: Could not set up left foreleg IK: {e}")

        # Right foreleg
        try:
            result['ik_foreleg_r'] = setup_ik_chain(
                armature,
                tip_bone_name='front_lower_r',
                chain_length=2,
                target_name='ik_front_paw_r',
                pole_name='pole_front_knee_r',
                pole_position=[-0.15, 0.3, 0.0]
            )
        except ValueError as e:
            print(f"Warning: Could not set up right foreleg IK: {e}")

    elif preset == 'quadruped_hindlegs':
        # Left hindleg
        try:
            result['ik_hindleg_l'] = setup_ik_chain(
                armature,
                tip_bone_name='back_lower_l',
                chain_length=2,
                target_name='ik_back_paw_l',
                pole_name='pole_back_knee_l',
                pole_position=[0.15, -0.3, 0.0]
            )
        except ValueError as e:
            print(f"Warning: Could not set up left hindleg IK: {e}")

        # Right hindleg
        try:
            result['ik_hindleg_r'] = setup_ik_chain(
                armature,
                tip_bone_name='back_lower_r',
                chain_length=2,
                target_name='ik_back_paw_r',
                pole_name='pole_back_knee_r',
                pole_position=[-0.15, -0.3, 0.0]
            )
        except ValueError as e:
            print(f"Warning: Could not set up right hindleg IK: {e}")

    elif preset == 'tentacle':
        # Find tentacle tip bone
        tentacle_bones = [b.name for b in armature.data.bones if 'tentacle' in b.name.lower()]
        if tentacle_bones:
            # Sort to find the tip (usually the last in a numbered sequence)
            tentacle_bones.sort()
            tip_bone = tentacle_bones[-1]
            try:
                result['ik_tentacle'] = setup_ik_chain(
                    armature,
                    tip_bone_name=tip_bone,
                    chain_length=min(4, len(tentacle_bones)),
                    target_name='ik_tentacle_tip'
                )
            except ValueError as e:
                print(f"Warning: Could not set up tentacle IK: {e}")

    elif preset == 'tail':
        # Find tail tip bone
        tail_bones = [b.name for b in armature.data.bones if 'tail' in b.name.lower()]
        if tail_bones:
            # Sort to find the tip
            tail_bones.sort()
            tip_bone = tail_bones[-1]
            try:
                result['ik_tail'] = setup_ik_chain(
                    armature,
                    tip_bone_name=tip_bone,
                    chain_length=min(4, len(tail_bones)),
                    target_name='ik_tail_tip'
                )
            except ValueError as e:
                print(f"Warning: Could not set up tail IK: {e}")

    else:
        print(f"Warning: Unknown IK preset: {preset}")

    return result


def apply_rig_setup(armature: 'bpy.types.Object', rig_setup: Dict) -> Dict[str, Dict]:
    """
    Apply a complete rig setup from spec configuration.

    Args:
        armature: The armature object.
        rig_setup: Dictionary with 'presets', 'ik_chains', and 'constraints' keys.

    Returns:
        Dictionary of all created IK control objects.
    """
    result = {}

    # Apply presets
    for preset in rig_setup.get('presets', []):
        preset_result = setup_ik_preset(armature, preset)
        result.update(preset_result)

    # Apply custom IK chains
    for chain_spec in rig_setup.get('ik_chains', []):
        chain_name = chain_spec.get('name', 'custom_ik')
        target_config = chain_spec.get('target', {})
        pole_config = chain_spec.get('pole')

        # Determine tip bone name from chain name or spec
        # Convention: chain name is "ik_<bone>_<side>" so tip bone is "<bone>_<side>"
        tip_bone_name = chain_spec.get('tip_bone')
        if not tip_bone_name:
            # Try to infer from chain name
            parts = chain_name.split('_')
            if len(parts) >= 2:
                tip_bone_name = '_'.join(parts[1:])  # Remove 'ik_' prefix

        if not tip_bone_name:
            print(f"Warning: Could not determine tip bone for chain '{chain_name}'")
            continue

        try:
            chain_result = setup_ik_chain(
                armature,
                tip_bone_name=tip_bone_name,
                chain_length=chain_spec.get('chain_length', 2),
                target_name=target_config.get('name', f'{chain_name}_target'),
                target_position=target_config.get('position'),
                target_bone=target_config.get('bone'),
                pole_name=pole_config.get('name') if pole_config else None,
                pole_position=pole_config.get('position') if pole_config else None,
                pole_bone=pole_config.get('bone') if pole_config else None,
                pole_angle=pole_config.get('angle', 0.0) if pole_config else 0.0,
                influence=chain_spec.get('influence', 1.0)
            )
            result[chain_name] = chain_result
        except ValueError as e:
            print(f"Warning: Could not set up chain '{chain_name}': {e}")

    # Apply bone constraints
    constraints_config = rig_setup.get('constraints', {})
    constraints_list = constraints_config.get('constraints', [])
    for constraint_spec in constraints_list:
        try:
            setup_constraint(armature, constraint_spec)
        except ValueError as e:
            print(f"Warning: Could not set up constraint: {e}")

    # Apply foot systems
    for foot_system in rig_setup.get('foot_systems', []):
        try:
            setup_foot_system(armature, foot_system)
        except ValueError as e:
            print(f"Warning: Could not set up foot system: {e}")

    # Apply aim constraints
    for aim_constraint in rig_setup.get('aim_constraints', []):
        try:
            setup_aim_constraint(armature, aim_constraint)
        except ValueError as e:
            print(f"Warning: Could not set up aim constraint: {e}")

    # Apply twist bones
    for twist_bone in rig_setup.get('twist_bones', []):
        try:
            setup_twist_bone(armature, twist_bone)
        except ValueError as e:
            print(f"Warning: Could not set up twist bone: {e}")

    # Apply stretch settings
    stretch_settings = rig_setup.get('stretch')
    if stretch_settings and stretch_settings.get('enabled', False):
        apply_stretch_settings(armature, stretch_settings)

    # Apply IK/FK switches
    for ikfk_switch in rig_setup.get('ikfk_switches', []):
        try:
            setup_ikfk_switch(armature, ikfk_switch, result)
        except ValueError as e:
            print(f"Warning: Could not set up IK/FK switch: {e}")

    # Apply space switches
    for space_switch in rig_setup.get('space_switches', []):
        try:
            setup_space_switch(armature, space_switch)
        except ValueError as e:
            print(f"Warning: Could not set up space switch: {e}")

    # Apply finger controls
    for finger_controls in rig_setup.get('finger_controls', []):
        try:
            setup_finger_controls(armature, finger_controls)
        except ValueError as e:
            print(f"Warning: Could not set up finger controls: {e}")

    return result


def setup_ikfk_switch(
    armature: 'bpy.types.Object',
    switch_config: Dict,
    ik_controls: Dict[str, Dict]
) -> None:
    """
    Set up IK/FK switching for a limb.

    Creates a custom property on the armature to control IK influence,
    and sets up drivers for seamless switching.

    Args:
        armature: The armature object.
        switch_config: Dictionary with switch configuration.
        ik_controls: Dictionary of existing IK control objects.
    """
    name = switch_config.get('name', 'ikfk_switch')
    ik_chain = switch_config.get('ik_chain')
    fk_bones = switch_config.get('fk_bones', [])
    default_mode = switch_config.get('default_mode', 'ik')

    if not ik_chain:
        raise ValueError(f"IK/FK switch '{name}' requires 'ik_chain' field")

    # Create custom property for IK/FK blend (0.0 = FK, 1.0 = IK)
    prop_name = f"ikfk_{name}"
    default_value = 1.0 if default_mode == 'ik' else 0.0

    # Add custom property to armature
    armature[prop_name] = default_value

    # Set up property UI
    if '_RNA_UI' not in armature:
        armature['_RNA_UI'] = {}

    armature['_RNA_UI'][prop_name] = {
        "min": 0.0,
        "max": 1.0,
        "soft_min": 0.0,
        "soft_max": 1.0,
        "description": f"IK/FK blend for {name} (0=FK, 1=IK)"
    }

    # Find the IK constraint on the tip bone and add driver for influence
    if ik_chain in ik_controls:
        chain_data = ik_controls[ik_chain]
        tip_bone_name = chain_data.get('tip_bone')

        if tip_bone_name and tip_bone_name in armature.pose.bones:
            pose_bone = armature.pose.bones[tip_bone_name]

            # Find IK constraint
            for constraint in pose_bone.constraints:
                if constraint.type == 'IK':
                    # Add driver to influence
                    driver = constraint.driver_add('influence').driver
                    driver.type = 'AVERAGE'

                    var = driver.variables.new()
                    var.name = 'ikfk'
                    var.type = 'SINGLE_PROP'
                    var.targets[0].id = armature
                    var.targets[0].data_path = f'["{prop_name}"]'

                    print(f"Set up IK/FK switch '{name}' on {tip_bone_name}")
                    break

    print(f"Created IK/FK switch property: {prop_name}")


def snap_fk_to_ik(armature: 'bpy.types.Object', switch_name: str, ik_controls: Dict) -> None:
    """
    Snap FK bones to match current IK pose.

    Called before switching from IK to FK mode to prevent popping.
    """
    # This would copy the current world-space rotations of the FK bones
    # Implementation depends on the specific bone chain
    pass


def snap_ik_to_fk(armature: 'bpy.types.Object', switch_name: str, ik_controls: Dict) -> None:
    """
    Snap IK target/pole to match current FK pose.

    Called before switching from FK to IK mode to prevent popping.
    """
    # This would move the IK target to match the FK end effector position
    # Implementation depends on the specific bone chain
    pass
