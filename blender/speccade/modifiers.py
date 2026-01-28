"""
Modifiers module for SpecCade Blender asset generation.

This module handles the application of Blender modifiers to mesh objects,
including bevel, edge split, subdivision, mirror, array, solidify,
decimate, and triangulate modifiers.
"""

import math
from typing import Dict

try:
    import bpy
except ImportError:
    bpy = None  # type: ignore


def apply_modifier(obj: 'bpy.types.Object', modifier_spec: Dict) -> None:
    """Apply a modifier to an object."""
    mod_type = modifier_spec.get("type", "").lower()

    if mod_type == "bevel":
        mod = obj.modifiers.new(name="Bevel", type='BEVEL')
        mod.width = modifier_spec.get("width", 0.02)
        mod.segments = modifier_spec.get("segments", 2)
        # Angle limit (in radians in spec, converted to radians in Blender)
        if "angle_limit" in modifier_spec:
            mod.angle_limit = modifier_spec["angle_limit"]

    elif mod_type == "edge_split":
        mod = obj.modifiers.new(name="EdgeSplit", type='EDGE_SPLIT')
        mod.split_angle = math.radians(modifier_spec.get("angle", 30))

    elif mod_type == "subdivision":
        mod = obj.modifiers.new(name="Subdivision", type='SUBSURF')
        mod.levels = modifier_spec.get("levels", 1)
        mod.render_levels = modifier_spec.get("render_levels", 2)

    elif mod_type == "mirror":
        mod = obj.modifiers.new(name="Mirror", type='MIRROR')
        mod.use_axis[0] = modifier_spec.get("axis_x", True)
        mod.use_axis[1] = modifier_spec.get("axis_y", False)
        mod.use_axis[2] = modifier_spec.get("axis_z", False)

    elif mod_type == "array":
        mod = obj.modifiers.new(name="Array", type='ARRAY')
        mod.count = modifier_spec.get("count", 2)
        offset = modifier_spec.get("offset", [1, 0, 0])
        mod.relative_offset_displace = offset

    elif mod_type == "solidify":
        mod = obj.modifiers.new(name="Solidify", type='SOLIDIFY')
        mod.thickness = modifier_spec.get("thickness", 0.1)
        mod.offset = modifier_spec.get("offset", 0)

    elif mod_type == "decimate":
        mod = obj.modifiers.new(name="Decimate", type='DECIMATE')
        mod.ratio = modifier_spec.get("ratio", 0.5)

    elif mod_type == "triangulate":
        mod = obj.modifiers.new(name="Triangulate", type='TRIANGULATE')
        # Map ngon_method to Blender's ngon_method enum
        ngon_method = modifier_spec.get("ngon_method", "beauty").upper()
        ngon_map = {
            "BEAUTY": 'BEAUTY',
            "CLIP": 'CLIP',
            "FIXED": 'FIXED'
        }
        mod.ngon_method = ngon_map.get(ngon_method, 'BEAUTY')
        # Map quad_method to Blender's quad_method enum
        quad_method = modifier_spec.get("quad_method", "shortest_diagonal").upper()
        quad_map = {
            "BEAUTY": 'BEAUTY',
            "FIXED": 'FIXED',
            "SHORTEST_DIAGONAL": 'SHORTEST_DIAGONAL',
            "LONGEST_DIAGONAL": 'LONGEST_DIAGONAL'
        }
        mod.quad_method = quad_map.get(quad_method, 'SHORTEST_DIAGONAL')

    else:
        print(f"Warning: Unknown modifier type: {mod_type}")


def apply_all_modifiers(obj: 'bpy.types.Object') -> None:
    """Apply all modifiers to an object."""
    bpy.context.view_layer.objects.active = obj
    for mod in obj.modifiers:
        try:
            bpy.ops.object.modifier_apply(modifier=mod.name)
        except RuntimeError as e:
            print(f"Warning: Could not apply modifier {mod.name}: {e}")
