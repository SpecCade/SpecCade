"""
Controls module for SpecCade Blender rigging.

This module handles control setup for armatures, including:
- Space switching (dynamic parent changes)
- Finger controls (curl and spread)
- Stretch settings for IK chains
"""

import math
from typing import Dict, List, Optional

try:
    import bpy
except ImportError:
    bpy = None  # type: ignore


def setup_space_switch(
    armature: 'bpy.types.Object',
    switch_config: Dict
) -> None:
    """
    Set up space switching for a bone.

    Creates constraints and custom properties to allow dynamic parent changes.

    Args:
        armature: The armature object.
        switch_config: Dictionary with space switch configuration.
    """
    name = switch_config.get('name', 'space_switch')
    bone_name = switch_config.get('bone')
    spaces = switch_config.get('spaces', [])
    default_space = switch_config.get('default_space', 0)

    if not bone_name:
        raise ValueError(f"Space switch '{name}' requires 'bone' field")

    if bone_name not in armature.pose.bones:
        raise ValueError(f"Bone '{bone_name}' not found in armature")

    if not spaces:
        raise ValueError(f"Space switch '{name}' requires at least one space")

    pose_bone = armature.pose.bones[bone_name]

    # Create custom property for space selection
    prop_name = f"space_{name}"
    armature[prop_name] = default_space

    # Set up property UI
    if '_RNA_UI' not in armature:
        armature['_RNA_UI'] = {}

    space_names = [s.get('name', f'Space {i}') for i, s in enumerate(spaces)]
    armature['_RNA_UI'][prop_name] = {
        "min": 0,
        "max": len(spaces) - 1,
        "soft_min": 0,
        "soft_max": len(spaces) - 1,
        "description": f"Parent space for {bone_name}: " + ", ".join(
            f"{i}={n}" for i, n in enumerate(space_names)
        )
    }

    # Create Copy Transform constraints for each space with driven influence
    for i, space in enumerate(spaces):
        space_name = space.get('name', f'Space_{i}')
        space_type = space.get('type', 'world')

        # Create constraint
        constraint = pose_bone.constraints.new('COPY_TRANSFORMS')
        constraint.name = f"Space_{space_name}"
        constraint.mix_mode = 'REPLACE'
        constraint.target_space = 'WORLD'
        constraint.owner_space = 'WORLD'

        # Set target based on space type
        if space_type == 'world':
            # No target - world space (disable constraint)
            constraint.mute = True
        elif space_type == 'root':
            constraint.target = armature
            constraint.subtarget = 'root'
        elif space_type == 'bone':
            target_bone = space.get('bone')
            if target_bone:
                constraint.target = armature
                constraint.subtarget = target_bone
        elif space_type == 'object':
            target_obj = space.get('object')
            if target_obj and target_obj in bpy.data.objects:
                constraint.target = bpy.data.objects[target_obj]

        # Add driver for influence based on space selection
        if space_type != 'world':
            driver = constraint.driver_add('influence').driver
            driver.type = 'SCRIPTED'
            driver.expression = f'1.0 if space == {i} else 0.0'

            var = driver.variables.new()
            var.name = 'space'
            var.type = 'SINGLE_PROP'
            var.targets[0].id = armature
            var.targets[0].data_path = f'["{prop_name}"]'

    print(f"Set up space switch '{name}' on {bone_name} with {len(spaces)} spaces")


def setup_finger_controls(
    armature: 'bpy.types.Object',
    controls_config: Dict
) -> None:
    """
    Set up simplified finger controls with curl and spread.

    Creates custom properties that drive finger bone rotations.

    Args:
        armature: The armature object.
        controls_config: Dictionary with finger controls configuration.
    """
    name = controls_config.get('name', 'finger_controls')
    side = controls_config.get('side', 'left')
    bone_prefix = controls_config.get('bone_prefix', 'finger')
    fingers = controls_config.get('fingers', ['thumb', 'index', 'middle', 'ring', 'pinky'])
    bones_per_finger = controls_config.get('bones_per_finger', 3)
    max_curl = controls_config.get('max_curl_degrees', 90.0)
    max_spread = controls_config.get('max_spread_degrees', 15.0)

    side_suffix = '_l' if side == 'left' else '_r'

    # Create custom properties for global curl and spread
    curl_prop = f"curl_{name}"
    spread_prop = f"spread_{name}"

    armature[curl_prop] = 0.0
    armature[spread_prop] = 0.0

    # Set up property UI
    if '_RNA_UI' not in armature:
        armature['_RNA_UI'] = {}

    armature['_RNA_UI'][curl_prop] = {
        "min": 0.0,
        "max": 1.0,
        "soft_min": 0.0,
        "soft_max": 1.0,
        "description": f"Global finger curl (0=flat, 1=fist)"
    }

    armature['_RNA_UI'][spread_prop] = {
        "min": 0.0,
        "max": 1.0,
        "soft_min": 0.0,
        "soft_max": 1.0,
        "description": f"Global finger spread"
    }

    # Create per-finger curl properties
    for finger in fingers:
        finger_curl_prop = f"curl_{name}_{finger}"
        armature[finger_curl_prop] = -1.0  # -1 means "use global"

        armature['_RNA_UI'][finger_curl_prop] = {
            "min": -1.0,
            "max": 1.0,
            "soft_min": -1.0,
            "soft_max": 1.0,
            "description": f"{finger.title()} curl override (-1=global)"
        }

    # Add drivers to finger bones
    for finger in fingers:
        finger_curl_prop = f"curl_{name}_{finger}"
        num_bones = 2 if finger == 'thumb' else bones_per_finger

        # Spread angle varies by finger position (index spreads less, pinky more)
        finger_spread_mult = {
            'thumb': 2.0,
            'index': 0.5,
            'middle': 0.0,
            'ring': -0.5,
            'pinky': -1.0
        }.get(finger, 0.0)

        for bone_idx in range(1, num_bones + 1):
            bone_name = f"{bone_prefix}_{finger}{bone_idx:02d}{side_suffix}"

            if bone_name not in armature.pose.bones:
                # Try alternate naming convention with underscore before index
                bone_name = f"{bone_prefix}_{finger}_{bone_idx:02d}{side_suffix}"

            if bone_name not in armature.pose.bones:
                # Try skeleton preset convention: {finger}_{bone_idx:02d}{side_suffix}
                # e.g., "index_01_l", "thumb_02_r"
                bone_name = f"{finger}_{bone_idx:02d}{side_suffix}"

            if bone_name not in armature.pose.bones:
                continue

            pose_bone = armature.pose.bones[bone_name]
            pose_bone.rotation_mode = 'XYZ'

            # Add curl driver to X rotation (flexion axis)
            try:
                driver = pose_bone.driver_add('rotation_euler', 0).driver
                driver.type = 'SCRIPTED'

                # Expression: use per-finger curl if >= 0, else use global
                # Curl amount scaled by max_curl and converted to radians
                curl_rad = math.radians(max_curl)
                driver.expression = f'(curl_finger if curl_finger >= 0 else curl_global) * {curl_rad}'

                var1 = driver.variables.new()
                var1.name = 'curl_global'
                var1.type = 'SINGLE_PROP'
                var1.targets[0].id = armature
                var1.targets[0].data_path = f'["{curl_prop}"]'

                var2 = driver.variables.new()
                var2.name = 'curl_finger'
                var2.type = 'SINGLE_PROP'
                var2.targets[0].id = armature
                var2.targets[0].data_path = f'["{finger_curl_prop}"]'
            except Exception as e:
                print(f"Warning: Could not add curl driver to {bone_name}: {e}")

            # Add spread driver to Z rotation (only for first bone of each finger)
            if bone_idx == 1 and finger_spread_mult != 0.0:
                try:
                    driver = pose_bone.driver_add('rotation_euler', 2).driver
                    driver.type = 'SCRIPTED'

                    spread_rad = math.radians(max_spread * finger_spread_mult)
                    driver.expression = f'spread * {spread_rad}'

                    var = driver.variables.new()
                    var.name = 'spread'
                    var.type = 'SINGLE_PROP'
                    var.targets[0].id = armature
                    var.targets[0].data_path = f'["{spread_prop}"]'
                except Exception as e:
                    print(f"Warning: Could not add spread driver to {bone_name}: {e}")

    print(f"Set up finger controls '{name}' for {len(fingers)} fingers")


def apply_stretch_settings(armature: 'bpy.types.Object', stretch_settings: Dict) -> None:
    """
    Apply stretch settings to IK chains.

    Enables bones to stretch beyond their rest length when IK targets
    are out of reach.

    Args:
        armature: The armature object.
        stretch_settings: Dictionary with stretch configuration:
            - enabled: Whether stretch is enabled
            - max_stretch: Maximum stretch factor (1.0 = no stretch, 2.0 = double)
            - min_stretch: Minimum stretch factor (0.5 = half length)
            - volume_preservation: 'none', 'uniform', 'x', or 'z'
    """
    max_stretch = stretch_settings.get('max_stretch', 1.5)
    min_stretch = stretch_settings.get('min_stretch', 0.5)
    volume_mode = stretch_settings.get('volume_preservation', 'none')

    # Ensure we're in object mode first
    if bpy.context.mode != 'OBJECT':
        bpy.ops.object.mode_set(mode='OBJECT')

    bpy.context.view_layer.objects.active = armature
    bpy.ops.object.mode_set(mode='POSE')

    # Find all IK constraints and enable stretch
    for pose_bone in armature.pose.bones:
        for constraint in pose_bone.constraints:
            if constraint.type == 'IK':
                constraint.use_stretch = True
                # Note: Blender's IK stretch doesn't have min/max controls directly
                # We can simulate this with additional constraints if needed

    # Apply volume preservation via scale constraints if needed
    if volume_mode != 'none':
        for pose_bone in armature.pose.bones:
            # Check if this bone is part of an IK chain
            has_ik = any(c.type == 'IK' for c in pose_bone.constraints)
            if has_ik or (pose_bone.parent and any(c.type == 'IK' for c in pose_bone.parent.constraints)):
                constraint = pose_bone.constraints.new('MAINTAIN_VOLUME')
                constraint.name = f"Stretch_Volume_{pose_bone.name}"
                constraint.mode = 'STRICT'
                constraint.owner_space = 'LOCAL'

                if volume_mode == 'uniform':
                    constraint.free_axis = 'SAMEVOL_Y'  # Volume along bone axis
                elif volume_mode == 'x':
                    constraint.free_axis = 'SAMEVOL_X'
                elif volume_mode == 'z':
                    constraint.free_axis = 'SAMEVOL_Z'

    bpy.ops.object.mode_set(mode='OBJECT')
    print(f"Applied stretch settings: max={max_stretch}, min={min_stretch}, volume={volume_mode}")
