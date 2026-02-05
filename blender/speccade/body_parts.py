"""
SpecCade Body Parts Module

This module handles the creation and manipulation of body part meshes for rigged characters.
It supports both the new simplified mesh format and the extrusion-based format
for procedural body part generation with extrusion steps.

Key functions:
- create_body_part(): Create a body part mesh attached to a bone
- create_extrusion_part(): Create a part using the extrusion-based format with extrusion steps
- create_mirrored_part(): Create a mirrored copy of a part (L<->R reflection)
- apply_texturing(): Apply UV mapping and texture regions to a mesh
- skin_mesh_to_armature(): Parent a mesh to an armature with automatic weights
"""

import math
import re
from typing import Any, Dict, List, Optional, Tuple

# Blender modules - only available when running inside Blender
try:
    import bpy
    import bmesh
    from mathutils import Euler, Vector
    BLENDER_AVAILABLE = True
except ImportError:
    BLENDER_AVAILABLE = False

from .skeleton import get_bone_position
from .scene import create_primitive


def create_body_part(armature: 'bpy.types.Object', part_spec: Dict) -> 'bpy.types.Object':
    """Create a body part mesh attached to a bone.

    Args:
        armature: The armature object that the part will be attached to.
        part_spec: Part specification with:
                   - 'bone': Name of the bone to attach to
                   - 'mesh': Mesh specification with 'primitive', 'dimensions',
                             'offset', and 'rotation'

    Returns:
        The created mesh object.
    """
    bone_name = part_spec["bone"]
    mesh_spec = part_spec["mesh"]

    # Get bone position
    bone_pos = get_bone_position(armature, bone_name)

    # Create primitive
    primitive = mesh_spec.get("primitive", "cube")
    dimensions = mesh_spec.get("dimensions", [0.1, 0.1, 0.1])
    obj = create_primitive(primitive, dimensions)

    # Apply offset and rotation if specified
    offset = mesh_spec.get("offset", [0, 0, 0])
    obj.location = bone_pos + Vector(offset)

    if "rotation" in mesh_spec:
        rot = mesh_spec["rotation"]
        obj.rotation_euler = Euler((math.radians(rot[0]), math.radians(rot[1]), math.radians(rot[2])))

    bpy.ops.object.transform_apply(rotation=True)

    obj.name = f"mesh_{bone_name}"

    return obj


# =============================================================================
# Extrusion Part System (SPEC dict format)
# =============================================================================

def parse_base_shape(base: str) -> Tuple[str, int]:
    """
    Parse a base shape specification like 'hexagon(6)' or 'circle(8)'.

    Args:
        base: Shape specification string (e.g., 'circle(8)', 'hexagon(6)').

    Returns:
        Tuple of (shape_type, vertex_count)
    """
    match = re.match(r'(\w+)\((\d+)\)', base)
    if match:
        return match.group(1), int(match.group(2))
    # Default to circle with 8 vertices
    return base if base else 'circle', 8


def create_base_mesh_profile(shape_type: str, vertex_count: int, radius: float) -> 'bpy.types.Object':
    """
    Create a base mesh profile (2D shape) for extrusion.

    Args:
        shape_type: One of 'circle', 'hexagon', 'square', 'triangle', etc.
        vertex_count: Number of vertices in the profile.
        radius: Radius of the profile.

    Returns:
        A mesh object with the profile as a face.
    """
    bpy.ops.mesh.primitive_circle_add(
        vertices=vertex_count,
        radius=radius,
        fill_type='NGON',
        enter_editmode=False
    )
    obj = bpy.context.active_object
    return obj


def get_base_radius_values(base_radius) -> Tuple[float, float]:
    """
    Extract bottom and top radius values from a base_radius specification.

    Args:
        base_radius: Either a float (uniform) or a list [bottom, top] (tapered).

    Returns:
        Tuple of (bottom_radius, top_radius)
    """
    if base_radius is None:
        return 0.1, 0.1
    if isinstance(base_radius, (int, float)):
        return float(base_radius), float(base_radius)
    if isinstance(base_radius, list) and len(base_radius) >= 2:
        return float(base_radius[0]), float(base_radius[1])
    return 0.1, 0.1


def get_scale_factors(scale) -> Tuple[float, float]:
    """
    Extract X and Y scale factors from a scale specification.

    Args:
        scale: Either a float (uniform) or a list [X, Y] (per-axis).

    Returns:
        Tuple of (scale_x, scale_y)
    """
    if scale is None:
        return 1.0, 1.0
    if isinstance(scale, (int, float)):
        return float(scale), float(scale)
    if isinstance(scale, list) and len(scale) >= 2:
        return float(scale[0]), float(scale[1])
    return 1.0, 1.0


def get_bulge_factors(bulge) -> Tuple[float, float]:
    """
    Extract side and forward_back bulge factors.

    Args:
        bulge: Either a float (uniform) or a list [side, forward_back].

    Returns:
        Tuple of (side_bulge, forward_back_bulge)
    """
    if bulge is None:
        return 1.0, 1.0
    if isinstance(bulge, (int, float)):
        return float(bulge), float(bulge)
    if isinstance(bulge, list) and len(bulge) >= 2:
        return float(bulge[0]), float(bulge[1])
    return 1.0, 1.0


def get_tilt_angles(tilt) -> Tuple[float, float]:
    """
    Extract X and Y tilt angles in degrees.

    Args:
        tilt: Either a float (X only) or a list [X, Y].

    Returns:
        Tuple of (tilt_x, tilt_y) in degrees
    """
    if tilt is None:
        return 0.0, 0.0
    if isinstance(tilt, (int, float)):
        return float(tilt), 0.0
    if isinstance(tilt, list) and len(tilt) >= 2:
        return float(tilt[0]), float(tilt[1])
    return 0.0, 0.0


def parse_step(step) -> Dict:
    """
    Parse a step specification into a standardized dictionary.

    Args:
        step: Either a string (shorthand extrude distance) or a dict.

    Returns:
        Dictionary with step parameters.
    """
    if isinstance(step, str):
        # Shorthand: just an extrusion distance
        try:
            return {'extrude': float(step)}
        except ValueError:
            return {'extrude': 0.1}
    if isinstance(step, dict):
        return step
    return {'extrude': 0.1}


def apply_step_to_mesh(obj: 'bpy.types.Object', step: Dict, step_index: int) -> None:
    """
    Apply a single extrusion step to a mesh object.

    Args:
        obj: The mesh object to modify.
        step: Step specification dictionary with keys:
              - 'extrude': Distance to extrude (default: 0.1)
              - 'scale': Scale factor(s) [X, Y] or uniform float
              - 'translate': Translation offset [X, Y, Z]
              - 'rotate': Rotation around Z axis in degrees
              - 'bulge': Bulge factor(s) [side, forward_back]
              - 'tilt': Tilt angles [X, Y] in degrees
        step_index: Index of this step (for debugging).
    """
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')

    # Get step parameters
    extrude = step.get('extrude', 0.1)
    scale_x, scale_y = get_scale_factors(step.get('scale'))
    translate = step.get('translate', [0, 0, 0])
    rotate = step.get('rotate', 0)
    bulge_side, bulge_fb = get_bulge_factors(step.get('bulge'))
    tilt_x, tilt_y = get_tilt_angles(step.get('tilt'))

    # Select only top faces for extrusion
    bm = bmesh.from_edit_mesh(obj.data)
    bm.faces.ensure_lookup_table()

    # Find the topmost faces (highest Z average)
    if bm.faces:
        max_z = max(sum(v.co.z for v in f.verts) / len(f.verts) for f in bm.faces)
        for f in bm.faces:
            avg_z = sum(v.co.z for v in f.verts) / len(f.verts)
            f.select = abs(avg_z - max_z) < 0.001

    bmesh.update_edit_mesh(obj.data)

    # Extrude
    if extrude != 0:
        bpy.ops.mesh.extrude_region_move(
            TRANSFORM_OT_translate={'value': (0, 0, extrude)}
        )

    # Apply scale
    if scale_x != 1.0 or scale_y != 1.0:
        # Also apply bulge as additional scale modifiers
        final_scale_x = scale_x * bulge_side
        final_scale_y = scale_y * bulge_fb
        bpy.ops.transform.resize(value=(final_scale_x, final_scale_y, 1.0))

    # Apply translation
    if translate and any(t != 0 for t in translate):
        bpy.ops.transform.translate(value=tuple(translate))

    # Apply rotation around Z axis
    if rotate != 0:
        bpy.ops.transform.rotate(value=math.radians(rotate), orient_axis='Z')

    # Apply tilt (rotation around X and Y axes)
    if tilt_x != 0:
        bpy.ops.transform.rotate(value=math.radians(tilt_x), orient_axis='X')
    if tilt_y != 0:
        bpy.ops.transform.rotate(value=math.radians(tilt_y), orient_axis='Y')

    bpy.ops.object.mode_set(mode='OBJECT')


def create_extrusion_part(
    armature: 'bpy.types.Object',
    part_name: str,
    part_spec: Dict,
    all_parts: Dict[str, Dict]
) -> Optional['bpy.types.Object']:
    """
    Create a mesh part using the extrusion-based SPEC format.

    Args:
        armature: The armature object (can be None for standalone parts).
        part_name: Name of this part.
        part_spec: Part specification dictionary with:
                   - 'bone': Bone name to attach to
                   - 'base': Base shape like 'circle(8)'
                   - 'base_radius': Radius or [bottom, top] radii
                   - 'offset': World position offset
                   - 'rotation': [X, Y, Z] rotation in degrees
                   - 'steps': List of extrusion steps
                   - 'cap_start': Whether to cap the bottom (default: True)
                   - 'cap_end': Whether to cap the top (default: True)
                   - 'mirror': Name of part to mirror from
                   - 'thumb': Thumb sub-part specification
                   - 'fingers': List of finger sub-part specifications
        all_parts: Dictionary of all parts (for mirror lookups).

    Returns:
        The created mesh object, or None if creation failed.
    """
    # Check for mirror
    mirror_from = part_spec.get('mirror')
    if mirror_from:
        # Get the source part spec and mirror it
        source_spec = all_parts.get(mirror_from, {})
        if not source_spec:
            print(f"Warning: Mirror source '{mirror_from}' not found for part '{part_name}'")
            return None
        # Create the source part first if needed, then mirror
        # For now, we'll create a mirrored version by flipping X coordinates
        return create_mirrored_part(armature, part_name, part_spec, source_spec, all_parts)

    bone_name = part_spec.get('bone', part_name)

    # Parse base shape
    base = part_spec.get('base', 'circle(8)')
    shape_type, vertex_count = parse_base_shape(base)

    # Get base radius
    bottom_radius, top_radius = get_base_radius_values(part_spec.get('base_radius'))

    # Create base profile mesh
    obj = create_base_mesh_profile(shape_type, vertex_count, bottom_radius)
    obj.name = f"part_{part_name}"

    # Position the part:
    # - If explicit 'offset' is provided, use it as world position
    # - Otherwise, position at bone's head location
    if 'offset' in part_spec:
        # Explicit offset is treated as world position
        obj.location = Vector(part_spec['offset'])
    elif armature:
        # No offset specified - position at bone's head
        obj.location = get_bone_position(armature, bone_name)
    else:
        obj.location = Vector((0, 0, 0))

    # Apply initial rotation
    if 'rotation' in part_spec:
        rot = part_spec['rotation']
        obj.rotation_euler = Euler((
            math.radians(rot[0]),
            math.radians(rot[1]),
            math.radians(rot[2])
        ))
        bpy.ops.object.transform_apply(rotation=True)

    # Apply extrusion steps
    steps = part_spec.get('steps', [])
    for i, step in enumerate(steps):
        parsed_step = parse_step(step)
        apply_step_to_mesh(obj, parsed_step, i)

    # Handle caps
    cap_start = part_spec.get('cap_start', True)
    cap_end = part_spec.get('cap_end', True)

    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode='EDIT')
    bm = bmesh.from_edit_mesh(obj.data)

    if not cap_start:
        # Remove bottom face
        bm.faces.ensure_lookup_table()
        if bm.faces:
            min_z = min(sum(v.co.z for v in f.verts) / len(f.verts) for f in bm.faces)
            for f in list(bm.faces):
                avg_z = sum(v.co.z for v in f.verts) / len(f.verts)
                if abs(avg_z - min_z) < 0.001:
                    bm.faces.remove(f)

    if not cap_end:
        # Remove top face
        bm.faces.ensure_lookup_table()
        if bm.faces:
            max_z = max(sum(v.co.z for v in f.verts) / len(f.verts) for f in bm.faces)
            for f in list(bm.faces):
                avg_z = sum(v.co.z for v in f.verts) / len(f.verts)
                if abs(avg_z - max_z) < 0.001:
                    bm.faces.remove(f)

    bmesh.update_edit_mesh(obj.data)
    bpy.ops.object.mode_set(mode='OBJECT')

    # Handle thumb sub-part
    thumb_spec = part_spec.get('thumb')
    if thumb_spec:
        thumb_objects = create_sub_parts(armature, part_name, thumb_spec, 'thumb', obj)
        # Join thumb objects with main part
        for thumb_obj in thumb_objects:
            join_objects(obj, thumb_obj)

    # Handle finger sub-parts
    fingers = part_spec.get('fingers', [])
    for i, finger_spec in enumerate(fingers):
        finger_objects = create_sub_parts(armature, part_name, finger_spec, f'finger_{i}', obj)
        for finger_obj in finger_objects:
            join_objects(obj, finger_obj)

    return obj


def create_mirrored_part(
    armature: 'bpy.types.Object',
    part_name: str,
    part_spec: Dict,
    source_spec: Dict,
    all_parts: Dict[str, Dict]
) -> Optional['bpy.types.Object']:
    """
    Create a mirrored copy of a part (L->R reflection across X=0).

    Args:
        armature: The armature object.
        part_name: Name of the new mirrored part.
        part_spec: Specification for the mirrored part (may override some values).
        source_spec: Specification of the source part to mirror from.
        all_parts: Dictionary of all parts.

    Returns:
        The created mirrored mesh object.
    """
    # Create the source part first (without mirror flag)
    source_copy = source_spec.copy()
    source_copy.pop('mirror', None)  # Remove mirror to avoid recursion

    # Use the bone from the mirrored part spec if provided
    bone_name = part_spec.get('bone', part_name)
    source_copy['bone'] = bone_name

    # Create the part
    obj = create_extrusion_part(armature, f"{part_name}_temp", source_copy, all_parts)
    if not obj:
        return None

    obj.name = f"part_{part_name}"

    # Apply mirror transformation across X axis
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.transform.mirror(orient_type='GLOBAL', constraint_axis=(True, False, False))
    bpy.ops.object.mode_set(mode='OBJECT')

    # Flip normals to maintain correct facing
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.mesh.flip_normals()
    bpy.ops.object.mode_set(mode='OBJECT')

    return obj


def create_sub_parts(
    armature: 'bpy.types.Object',
    parent_name: str,
    sub_spec,
    sub_type: str,
    parent_obj: 'bpy.types.Object'
) -> List['bpy.types.Object']:
    """
    Create sub-parts (thumbs, fingers) for a parent part.

    Args:
        armature: The armature object.
        parent_name: Name of the parent part.
        sub_spec: Sub-part specification (dict or list of dicts).
        sub_type: Type of sub-part ('thumb', 'finger_0', etc.).
        parent_obj: The parent mesh object.

    Returns:
        List of created sub-part mesh objects.
    """
    results = []

    # Normalize to list
    if isinstance(sub_spec, dict):
        sub_specs = [sub_spec]
    elif isinstance(sub_spec, list):
        sub_specs = sub_spec
    else:
        return results

    for i, spec in enumerate(sub_specs):
        sub_name = f"{parent_name}_{sub_type}_{i}"
        bone_name = spec.get('bone', sub_name)

        # Parse base shape
        base = spec.get('base', 'circle(4)')
        shape_type, vertex_count = parse_base_shape(base)

        # Get base radius
        bottom_radius, top_radius = get_base_radius_values(spec.get('base_radius', 0.02))

        # Create base profile
        obj = create_base_mesh_profile(shape_type, vertex_count, bottom_radius)
        obj.name = f"subpart_{sub_name}"

        # Position relative to parent
        offset = spec.get('offset', [0, 0, 0])
        if armature:
            bone_pos = get_bone_position(armature, bone_name)
            obj.location = bone_pos + Vector(offset)
        else:
            # Position relative to parent object
            obj.location = parent_obj.location + Vector(offset)

        # Apply rotation
        if 'rotation' in spec:
            rot = spec['rotation']
            obj.rotation_euler = Euler((
                math.radians(rot[0]),
                math.radians(rot[1]),
                math.radians(rot[2])
            ))
            bpy.ops.object.transform_apply(rotation=True)

        # Apply steps
        steps = spec.get('steps', [])
        for j, step in enumerate(steps):
            parsed_step = parse_step(step)
            apply_step_to_mesh(obj, parsed_step, j)

        # Handle caps
        handle_part_caps(obj, spec.get('cap_start', True), spec.get('cap_end', True))

        results.append(obj)

    return results


def handle_part_caps(obj: 'bpy.types.Object', cap_start: bool, cap_end: bool) -> None:
    """Handle cap_start and cap_end for a part.

    Args:
        obj: The mesh object to modify.
        cap_start: Whether to keep the bottom cap.
        cap_end: Whether to keep the top cap.
    """
    if cap_start and cap_end:
        return  # Both capped, nothing to do

    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode='EDIT')
    bm = bmesh.from_edit_mesh(obj.data)

    if not cap_start:
        bm.faces.ensure_lookup_table()
        if bm.faces:
            min_z = min(sum(v.co.z for v in f.verts) / len(f.verts) for f in bm.faces)
            for f in list(bm.faces):
                avg_z = sum(v.co.z for v in f.verts) / len(f.verts)
                if abs(avg_z - min_z) < 0.001:
                    bm.faces.remove(f)

    if not cap_end:
        bm.faces.ensure_lookup_table()
        if bm.faces:
            max_z = max(sum(v.co.z for v in f.verts) / len(f.verts) for f in bm.faces)
            for f in list(bm.faces):
                avg_z = sum(v.co.z for v in f.verts) / len(f.verts)
                if abs(avg_z - max_z) < 0.001:
                    bm.faces.remove(f)

    bmesh.update_edit_mesh(obj.data)
    bpy.ops.object.mode_set(mode='OBJECT')


def create_part_instances(
    base_obj: 'bpy.types.Object',
    instances: List[Dict]
) -> List['bpy.types.Object']:
    """
    Create instances of a base part at specified positions and rotations.

    Args:
        base_obj: The base mesh object to duplicate.
        instances: List of instance specifications with 'position' and 'rotation'.

    Returns:
        List of instance mesh objects (does not include the base).
    """
    results = []

    for i, inst in enumerate(instances):
        # Duplicate the base object
        bpy.ops.object.select_all(action='DESELECT')
        base_obj.select_set(True)
        bpy.context.view_layer.objects.active = base_obj
        bpy.ops.object.duplicate(linked=False)
        inst_obj = bpy.context.active_object
        inst_obj.name = f"{base_obj.name}_inst_{i}"

        # Apply position
        position = inst.get('position', [0, 0, 0])
        if position:
            inst_obj.location = Vector(position)

        # Apply rotation
        rotation = inst.get('rotation', [0, 0, 0])
        if rotation:
            inst_obj.rotation_euler = Euler((
                math.radians(rotation[0]),
                math.radians(rotation[1]),
                math.radians(rotation[2])
            ))

        results.append(inst_obj)

    return results


def join_objects(target: 'bpy.types.Object', source: 'bpy.types.Object') -> None:
    """Join source object into target object.

    Args:
        target: The target object that will receive the source geometry.
        source: The source object that will be merged into target.
    """
    bpy.ops.object.select_all(action='DESELECT')
    target.select_set(True)
    source.select_set(True)
    bpy.context.view_layer.objects.active = target
    bpy.ops.object.join()


# =============================================================================
# Texturing / UV Mapping
# =============================================================================

def apply_texturing(obj: 'bpy.types.Object', texturing_spec: Dict) -> None:
    """
    Apply texturing/UV settings to a mesh object.

    Args:
        obj: The mesh object.
        texturing_spec: Texturing specification with:
                        - 'uv_mode': UV projection mode ('smart_project', 'lightmap_pack',
                                     'cube_project', 'cylinder_project', 'sphere_project')
                        - 'regions': Dict of region specifications
    """
    if not texturing_spec:
        return

    uv_mode = texturing_spec.get('uv_mode', 'smart_project')

    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')

    # Apply UV projection based on mode
    if uv_mode == 'smart_project':
        bpy.ops.uv.smart_project()
    elif uv_mode == 'region_based':
        # Region-based UV mapping is handled separately per region
        bpy.ops.uv.smart_project()  # Fallback to smart project
    elif uv_mode == 'lightmap_pack':
        bpy.ops.uv.lightmap_pack()
    elif uv_mode == 'cube_project':
        bpy.ops.uv.cube_project()
    elif uv_mode == 'cylinder_project':
        bpy.ops.uv.cylinder_project()
    elif uv_mode == 'sphere_project':
        bpy.ops.uv.sphere_project()
    else:
        bpy.ops.uv.smart_project()

    bpy.ops.object.mode_set(mode='OBJECT')

    # Handle texture regions
    regions = texturing_spec.get('regions', {})
    for region_name, region_spec in regions.items():
        apply_texture_region(obj, region_name, region_spec)


def apply_texture_region(obj: 'bpy.types.Object', region_name: str, region_spec: Dict) -> None:
    """
    Apply a texture region to specific parts of a mesh.

    Args:
        obj: The mesh object.
        region_name: Name of the region.
        region_spec: Region specification with:
                     - 'parts': List of part names in this region
                     - 'material_index': Index of the material slot
                     - 'color': Color as hex string "#RRGGBB" or RGB list [R, G, B]
    """
    material_index = region_spec.get('material_index')
    color = region_spec.get('color')

    if material_index is not None and color:
        # Ensure we have enough material slots
        while len(obj.data.materials) <= material_index:
            mat = bpy.data.materials.new(name=f"Region_{region_name}")
            mat.use_nodes = True
            obj.data.materials.append(mat)

        # Set material color
        mat = obj.data.materials[material_index]
        if mat and mat.use_nodes:
            principled = mat.node_tree.nodes.get("Principled BSDF")
            if principled:
                rgb = parse_region_color(color)
                principled.inputs["Base Color"].default_value = (*rgb, 1.0)


def parse_region_color(color) -> Tuple[float, float, float]:
    """
    Parse a region color specification to RGB values.

    Args:
        color: Color as hex string "#RRGGBB" or RGB list [R, G, B].

    Returns:
        Tuple of (R, G, B) with values 0-1.
    """
    if isinstance(color, str) and color.startswith('#'):
        # Hex color
        hex_color = color.lstrip('#')
        if len(hex_color) == 6:
            r = int(hex_color[0:2], 16) / 255.0
            g = int(hex_color[2:4], 16) / 255.0
            b = int(hex_color[4:6], 16) / 255.0
            return (r, g, b)
    elif isinstance(color, list) and len(color) >= 3:
        return (float(color[0]), float(color[1]), float(color[2]))

    return (1.0, 1.0, 1.0)


# =============================================================================
# Skinning
# =============================================================================

def skin_mesh_to_armature(mesh_obj: 'bpy.types.Object', armature: 'bpy.types.Object',
                           auto_weights: bool = True) -> None:
    """Parent mesh to armature with automatic weights.

    Args:
        mesh_obj: The mesh object to skin.
        armature: The armature object to parent to.
        auto_weights: If True, use automatic weight painting. If False, just parent.
    """
    # Select mesh and armature
    bpy.ops.object.select_all(action='DESELECT')
    mesh_obj.select_set(True)
    armature.select_set(True)
    bpy.context.view_layer.objects.active = armature

    # Parent with automatic weights
    if auto_weights:
        bpy.ops.object.parent_set(type='ARMATURE_AUTO')
    else:
        bpy.ops.object.parent_set(type='ARMATURE')


def assign_vertex_group(mesh_obj: 'bpy.types.Object', bone_name: str) -> None:
    """Assign all vertices to a vertex group for a bone.

    Args:
        mesh_obj: The mesh object to modify.
        bone_name: Name of the bone/vertex group.
    """
    # Create vertex group
    vg = mesh_obj.vertex_groups.new(name=bone_name)

    # Add all vertices with weight 1.0
    vertex_indices = [v.index for v in mesh_obj.data.vertices]
    vg.add(vertex_indices, 1.0, 'REPLACE')
