"""
SpecCade Normals Settings Module

This module handles normals automation settings for mesh objects.
Supports various presets for controlling mesh shading appearance.
"""

import math
from typing import Dict

# Blender modules - only available when running inside Blender
try:
    import bpy
    BLENDER_AVAILABLE = True
except ImportError:
    BLENDER_AVAILABLE = False


def apply_normals_settings(obj: 'bpy.types.Object', normals: Dict) -> None:
    """
    Apply normals automation settings to a mesh object.

    Supported presets:
    - flat: Flat shading (faceted appearance)
    - smooth: Smooth shading (interpolated normals)
    - auto_smooth: Auto-smooth based on angle threshold
    - weighted_normals: Weighted normals based on face area
    - hard_edge_by_angle: Mark hard edges at angles above threshold
    """
    if not normals:
        return

    preset = normals.get("preset", "smooth").lower()
    angle = normals.get("angle", 30.0)  # Default angle in degrees
    keep_sharp = normals.get("keep_sharp", True)

    # Convert angle from degrees to radians
    angle_rad = math.radians(angle)

    bpy.context.view_layer.objects.active = obj

    if preset == "flat":
        # Flat shading - each face has its own normal direction
        bpy.ops.object.shade_flat()

    elif preset == "smooth":
        # Smooth shading - normals are interpolated across faces
        bpy.ops.object.shade_smooth()

    elif preset == "auto_smooth":
        # Auto-smooth: smooth shading with angle-based sharp edges
        bpy.ops.object.shade_smooth()

        # Blender 4.1+ uses mesh attributes instead of auto_smooth
        if hasattr(obj.data, 'use_auto_smooth'):
            # Blender 3.x / 4.0
            obj.data.use_auto_smooth = True
            obj.data.auto_smooth_angle = angle_rad
        else:
            # Blender 4.1+: Use Smooth by Angle modifier or geometry nodes
            # For now, use the shade_smooth_edges operator with angle
            bpy.ops.object.mode_set(mode='EDIT')
            bpy.ops.mesh.select_all(action='SELECT')
            # Mark sharp edges based on angle
            bpy.ops.mesh.edges_select_sharp(sharpness=angle_rad)
            bpy.ops.mesh.mark_sharp()
            bpy.ops.mesh.select_all(action='DESELECT')
            bpy.ops.object.mode_set(mode='OBJECT')

    elif preset == "weighted_normals":
        # Weighted normals: use face area to weight normal calculation
        bpy.ops.object.shade_smooth()

        # Add weighted normal modifier
        mod = obj.modifiers.new(name="WeightedNormal", type='WEIGHTED_NORMAL')
        mod.mode = 'FACE_AREA'
        mod.weight = 50  # Default weight
        mod.keep_sharp = keep_sharp

        # Apply the modifier
        bpy.ops.object.modifier_apply(modifier=mod.name)

    elif preset == "hard_edge_by_angle":
        # Mark edges as sharp if angle exceeds threshold
        bpy.ops.object.shade_smooth()

        bpy.ops.object.mode_set(mode='EDIT')
        bpy.ops.mesh.select_all(action='SELECT')

        # Select sharp edges based on angle threshold
        bpy.ops.mesh.edges_select_sharp(sharpness=angle_rad)

        # Mark selected edges as sharp
        bpy.ops.mesh.mark_sharp()

        bpy.ops.mesh.select_all(action='DESELECT')
        bpy.ops.object.mode_set(mode='OBJECT')

        # Enable auto-smooth to respect sharp edges (Blender 3.x/4.0)
        if hasattr(obj.data, 'use_auto_smooth'):
            obj.data.use_auto_smooth = True
            obj.data.auto_smooth_angle = math.pi  # 180 degrees - only use marked edges

    else:
        # Default to smooth shading for unknown presets
        bpy.ops.object.shade_smooth()
