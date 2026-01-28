"""
SpecCade Skeleton Module

This module handles armature (skeleton) creation and manipulation for rigged characters.
It provides functions to create armatures from presets, apply skeleton overrides,
create custom skeletons, and perform bone mirroring operations.

Key functions:
- create_armature(): Create an armature from a named preset
- apply_skeleton_overrides(): Modify an existing armature with new bone specifications
- create_custom_skeleton(): Create a fully custom skeleton from bone definitions
- get_bone_position(): Get the world-space position of a bone
- mirror_bone_name(): Convert bone names between left and right sides
"""

import math
from typing import Any, Dict, List, Optional

# Blender modules - only available when running inside Blender
try:
    import bpy
    from mathutils import Vector
    BLENDER_AVAILABLE = True
except ImportError:
    BLENDER_AVAILABLE = False

from .skeleton_presets import SKELETON_PRESETS


def create_armature(preset_name: str) -> 'bpy.types.Object':
    """Create an armature from a preset.

    Args:
        preset_name: Name of the skeleton preset to use (e.g., 'humanoid_basic_v1',
                     'humanoid_detailed_v1', 'humanoid_game_v1').

    Returns:
        The created armature object.

    Raises:
        ValueError: If the preset name is not recognized.
    """
    preset = SKELETON_PRESETS.get(preset_name)
    if not preset:
        raise ValueError(f"Unknown skeleton preset: {preset_name}")

    # Create armature data
    armature_data = bpy.data.armatures.new("Armature")
    armature_obj = bpy.data.objects.new("Armature", armature_data)
    bpy.context.collection.objects.link(armature_obj)
    bpy.context.view_layer.objects.active = armature_obj

    # Enter edit mode to add bones
    bpy.ops.object.mode_set(mode='EDIT')

    # Create bones
    edit_bones = armature_data.edit_bones
    created_bones = {}

    for bone_name, bone_spec in preset.items():
        bone = edit_bones.new(bone_name)
        bone.head = Vector(bone_spec["head"])
        bone.tail = Vector(bone_spec["tail"])
        # Apply bone roll if specified (in degrees, convert to radians)
        if "roll" in bone_spec:
            bone.roll = math.radians(bone_spec["roll"])
        created_bones[bone_name] = bone

    # Set up bone hierarchy
    for bone_name, bone_spec in preset.items():
        parent_name = bone_spec.get("parent")
        if parent_name and parent_name in created_bones:
            created_bones[bone_name].parent = created_bones[parent_name]

    bpy.ops.object.mode_set(mode='OBJECT')

    return armature_obj


def apply_skeleton_overrides(
    armature_obj: 'bpy.types.Object',
    skeleton_spec: List[Dict[str, Any]],
) -> None:
    """
    Apply skeleton overrides/additions to an existing armature.

    This supports specs that provide a `skeleton_preset` plus a `skeleton` list
    to tweak bone positions/hierarchy or add extra bones.

    Args:
        armature_obj: The armature object to modify.
        skeleton_spec: List of bone specifications. Each spec is a dict with:
                       - 'bone': Required bone name
                       - 'head': Optional [x, y, z] head position
                       - 'tail': Optional [x, y, z] tail position
                       - 'roll': Optional roll angle in degrees
                       - 'parent': Optional parent bone name
    """
    armature_data = armature_obj.data

    bpy.ops.object.select_all(action='DESELECT')
    armature_obj.select_set(True)
    bpy.context.view_layer.objects.active = armature_obj
    bpy.ops.object.mode_set(mode='EDIT')

    edit_bones = armature_data.edit_bones
    created_or_updated = {}

    # Create or update bones
    for bone_spec in skeleton_spec:
        bone_name = bone_spec.get("bone")
        if not bone_name:
            continue

        bone = edit_bones.get(bone_name)
        if bone is None:
            bone = edit_bones.new(bone_name)

        if "head" in bone_spec:
            bone.head = Vector(tuple(bone_spec["head"]))
        if "tail" in bone_spec:
            bone.tail = Vector(tuple(bone_spec["tail"]))
        # Apply bone roll if specified (in degrees, convert to radians)
        if "roll" in bone_spec:
            bone.roll = math.radians(bone_spec["roll"])

        created_or_updated[bone_name] = bone

    # Apply parent overrides
    for bone_spec in skeleton_spec:
        bone_name = bone_spec.get("bone")
        if not bone_name:
            continue
        parent_name = bone_spec.get("parent")
        if not parent_name:
            continue

        bone = edit_bones.get(bone_name)
        parent = edit_bones.get(parent_name)
        if bone is not None and parent is not None:
            bone.parent = parent

    bpy.ops.object.mode_set(mode='OBJECT')


def get_bone_position(armature: 'bpy.types.Object', bone_name: str) -> 'Vector':
    """Get the world position of a bone.

    Args:
        armature: The armature object containing the bone.
        bone_name: Name of the bone to get the position of.

    Returns:
        The world-space position of the bone's head as a Vector.
        Returns (0, 0, 0) if the bone is not found.
    """
    bone = armature.data.bones.get(bone_name)
    if bone:
        return armature.matrix_world @ bone.head_local
    return Vector((0, 0, 0))


def create_custom_skeleton(skeleton_spec: List[Dict]) -> 'bpy.types.Object':
    """
    Create a custom skeleton from a list of bone specifications.

    This function creates a fully custom armature, supporting automatic
    bone mirroring for symmetric characters.

    Args:
        skeleton_spec: List of bone definitions. Each definition is a dict with:
                       - 'bone': Required bone name
                       - 'head': [x, y, z] head position (default: [0, 0, 0])
                       - 'tail': [x, y, z] tail position (default: [0, 0, 0.1])
                       - 'parent': Optional parent bone name
                       - 'mirror': Optional bone name to mirror from (flips X coords)

    Returns:
        The created armature object.
    """
    # First pass: create all bones without mirroring
    bones_to_create = {}
    mirror_bones = {}

    for bone_spec in skeleton_spec:
        bone_name = bone_spec.get('bone')
        if not bone_name:
            continue

        mirror_from = bone_spec.get('mirror')
        if mirror_from:
            mirror_bones[bone_name] = bone_spec
        else:
            bones_to_create[bone_name] = bone_spec

    # Create armature
    armature_data = bpy.data.armatures.new("Armature")
    armature_obj = bpy.data.objects.new("Armature", armature_data)
    bpy.context.collection.objects.link(armature_obj)
    bpy.context.view_layer.objects.active = armature_obj

    # Enter edit mode to add bones
    bpy.ops.object.mode_set(mode='EDIT')
    edit_bones = armature_data.edit_bones
    created_bones = {}

    # Create non-mirrored bones
    for bone_name, bone_spec in bones_to_create.items():
        head = bone_spec.get('head', [0, 0, 0])
        tail = bone_spec.get('tail', [0, 0, 0.1])

        bone = edit_bones.new(bone_name)
        bone.head = Vector(head)
        bone.tail = Vector(tail)
        created_bones[bone_name] = bone

    # Set up parent relationships for non-mirrored bones
    for bone_name, bone_spec in bones_to_create.items():
        parent_name = bone_spec.get('parent')
        if parent_name and parent_name in created_bones:
            created_bones[bone_name].parent = created_bones[parent_name]

    # Create mirrored bones
    for bone_name, bone_spec in mirror_bones.items():
        mirror_from = bone_spec.get('mirror')
        source_bone = created_bones.get(mirror_from)

        if not source_bone:
            print(f"Warning: Mirror source bone '{mirror_from}' not found for '{bone_name}'")
            continue

        # Create mirrored bone by flipping X coordinates
        bone = edit_bones.new(bone_name)
        bone.head = Vector((-source_bone.head.x, source_bone.head.y, source_bone.head.z))
        bone.tail = Vector((-source_bone.tail.x, source_bone.tail.y, source_bone.tail.z))
        created_bones[bone_name] = bone

        # Mirror parent relationship
        source_spec = bones_to_create.get(mirror_from, {})
        source_parent = source_spec.get('parent')
        if source_parent:
            # Try to find the mirrored parent
            mirrored_parent = mirror_bone_name(source_parent)
            if mirrored_parent in created_bones:
                bone.parent = created_bones[mirrored_parent]
            elif source_parent in created_bones:
                bone.parent = created_bones[source_parent]

    bpy.ops.object.mode_set(mode='OBJECT')
    return armature_obj


def mirror_bone_name(name: str) -> str:
    """
    Convert a bone name from left to right or vice versa.

    Supports common naming conventions:
    - Suffix: '_l' <-> '_r', '_L' <-> '_R'
    - Infix: '_l_' <-> '_r_', '_L_' <-> '_R_'

    Args:
        name: Original bone name.

    Returns:
        Mirrored bone name. Returns the original name if no mirroring
        pattern is detected.
    """
    if name.endswith('_l'):
        return name[:-2] + '_r'
    elif name.endswith('_L'):
        return name[:-2] + '_R'
    elif name.endswith('_r'):
        return name[:-2] + '_l'
    elif name.endswith('_R'):
        return name[:-2] + '_L'
    elif '_l_' in name:
        return name.replace('_l_', '_r_')
    elif '_L_' in name:
        return name.replace('_L_', '_R_')
    elif '_r_' in name:
        return name.replace('_r_', '_l_')
    elif '_R_' in name:
        return name.replace('_R_', '_L_')
    return name
