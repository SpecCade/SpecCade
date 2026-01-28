"""
UV Mapping module for SpecCade Blender asset generation.

This module handles UV projection, texel density computation and scaling,
UV island packing, and lightmap UV generation for mesh objects.
"""

import math
from typing import Tuple, Union, Dict

try:
    import bpy
except ImportError:
    bpy = None  # type: ignore


def triangle_area_2d(p1: Tuple[float, float], p2: Tuple[float, float], p3: Tuple[float, float]) -> float:
    """Compute the signed area of a 2D triangle."""
    return 0.5 * ((p2[0] - p1[0]) * (p3[1] - p1[1]) - (p3[0] - p1[0]) * (p2[1] - p1[1]))


def apply_uv_projection(obj: 'bpy.types.Object', projection: Union[str, Dict]) -> None:
    """
    Apply UV projection to an object with optional texel density scaling and lightmap UVs.

    Args:
        obj: The mesh object.
        projection: Either a string (method name) or a dict with:
            - method: UV projection method (box, cylinder, sphere, smart, lightmap)
            - angle_limit: Angle limit in degrees for smart projection
            - cube_size: Cube size for box projection
            - texel_density: Target texel density in pixels per unit
            - uv_margin: UV island margin (0.0 to 1.0, default 0.001)
            - lightmap_uv: Generate secondary UV channel for lightmaps
    """
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')

    # Handle both simple string and dict formats
    if isinstance(projection, str):
        projection_method = projection.lower()
        angle_limit = None
        cube_size = None
        texel_density = None
        uv_margin = None
        lightmap_uv = False
    elif isinstance(projection, dict):
        projection_method = projection.get("method", "smart").lower()
        angle_limit = projection.get("angle_limit")
        cube_size = projection.get("cube_size")
        texel_density = projection.get("texel_density")
        uv_margin = projection.get("uv_margin")
        lightmap_uv = projection.get("lightmap_uv", False)
    else:
        projection_method = "smart"
        angle_limit = None
        cube_size = None
        texel_density = None
        uv_margin = None
        lightmap_uv = False

    # Default UV margin
    if uv_margin is None:
        uv_margin = 0.001

    # Apply UV projection method
    if projection_method == "box":
        if cube_size is not None:
            bpy.ops.uv.cube_project(cube_size=cube_size)
        else:
            bpy.ops.uv.cube_project()
    elif projection_method == "cylinder":
        bpy.ops.uv.cylinder_project()
    elif projection_method == "sphere":
        bpy.ops.uv.sphere_project()
    elif projection_method == "smart":
        if angle_limit is not None:
            bpy.ops.uv.smart_project(angle_limit=math.radians(angle_limit), island_margin=uv_margin)
        else:
            bpy.ops.uv.smart_project(island_margin=uv_margin)
    elif projection_method == "lightmap":
        bpy.ops.uv.lightmap_pack(PREF_MARGIN_DIV=uv_margin)
    else:
        # Default to smart project
        bpy.ops.uv.smart_project(island_margin=uv_margin)

    bpy.ops.object.mode_set(mode='OBJECT')

    # Scale UVs to meet texel density target if specified
    if texel_density is not None and texel_density > 0:
        scale_uvs_for_texel_density(obj, texel_density)

    # Pack UV islands with margin
    pack_uv_islands(obj, uv_margin)

    # Generate lightmap UVs on UV1 if requested
    if lightmap_uv:
        generate_lightmap_uvs(obj, uv_margin)


def compute_current_texel_density(obj: 'bpy.types.Object', texture_size: int = 1024) -> float:
    """
    Compute the current average texel density (pixels per world unit).

    Args:
        obj: The mesh object.
        texture_size: Reference texture size in pixels (default 1024).

    Returns:
        Average texel density in pixels per world unit.
    """
    mesh = obj.data
    if not mesh.uv_layers:
        return 0.0

    uv_layer = mesh.uv_layers.active.data

    total_uv_area = 0.0
    total_world_area = 0.0

    for poly in mesh.polygons:
        # Get UV coordinates for this face
        uvs = [uv_layer[loop_idx].uv for loop_idx in poly.loop_indices]

        # Compute UV space area (triangulate the polygon)
        uv_area = 0.0
        for i in range(1, len(uvs) - 1):
            uv_area += abs(triangle_area_2d(
                (uvs[0].x, uvs[0].y),
                (uvs[i].x, uvs[i].y),
                (uvs[i + 1].x, uvs[i + 1].y)
            ))

        # Compute world space area
        world_area = poly.area

        total_uv_area += uv_area
        total_world_area += world_area

    if total_world_area < 1e-8:
        return 0.0

    # UV area is in 0-1 space, so multiply by texture_size^2 to get pixel area
    # Then divide by world area to get pixels per world unit squared
    # Take sqrt to get linear texel density
    pixel_area = total_uv_area * (texture_size ** 2)
    if total_world_area > 0:
        density_squared = pixel_area / total_world_area
        return math.sqrt(density_squared)
    return 0.0


def scale_uvs_for_texel_density(obj: 'bpy.types.Object', target_density: float, texture_size: int = 1024) -> None:
    """
    Scale UVs to achieve the target texel density.

    Args:
        obj: The mesh object.
        target_density: Target texel density in pixels per world unit.
        texture_size: Reference texture size in pixels (default 1024).
    """
    current_density = compute_current_texel_density(obj, texture_size)
    if current_density < 1e-8:
        return

    # Compute scale factor needed
    scale_factor = target_density / current_density

    # Scale all UVs
    mesh = obj.data
    if not mesh.uv_layers:
        return

    uv_layer = mesh.uv_layers.active.data

    # Compute UV centroid for scaling around center
    centroid_u = 0.0
    centroid_v = 0.0
    count = 0
    for uv_data in uv_layer:
        centroid_u += uv_data.uv.x
        centroid_v += uv_data.uv.y
        count += 1

    if count > 0:
        centroid_u /= count
        centroid_v /= count

    # Scale UVs around centroid
    for uv_data in uv_layer:
        uv_data.uv.x = centroid_u + (uv_data.uv.x - centroid_u) * scale_factor
        uv_data.uv.y = centroid_v + (uv_data.uv.y - centroid_v) * scale_factor


def pack_uv_islands(obj: 'bpy.types.Object', margin: float = 0.001) -> None:
    """
    Pack UV islands with specified margin.

    Args:
        obj: The mesh object.
        margin: Island margin (0.0 to 1.0).
    """
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')

    # Pack islands
    bpy.ops.uv.pack_islands(margin=margin)

    bpy.ops.object.mode_set(mode='OBJECT')


def generate_lightmap_uvs(obj: 'bpy.types.Object', margin: float = 0.001) -> None:
    """
    Generate a secondary UV channel (UV1) for lightmaps.

    Args:
        obj: The mesh object.
        margin: Island margin for lightmap packing.
    """
    mesh = obj.data

    # Create a new UV layer for lightmaps if it doesn't exist
    lightmap_layer_name = "UVMap_Lightmap"
    if lightmap_layer_name not in mesh.uv_layers:
        mesh.uv_layers.new(name=lightmap_layer_name)

    # Set the lightmap layer as active
    mesh.uv_layers.active = mesh.uv_layers[lightmap_layer_name]

    # Enter edit mode and select all
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')

    # Use lightmap pack for the secondary UVs
    # PREF_MARGIN_DIV controls the margin between islands
    try:
        bpy.ops.uv.lightmap_pack(PREF_MARGIN_DIV=margin)
    except TypeError:
        # Fallback for different Blender versions
        bpy.ops.uv.lightmap_pack()

    bpy.ops.object.mode_set(mode='OBJECT')

    # Restore the original UV layer as active (UV0)
    if len(mesh.uv_layers) > 1:
        # Find the first UV layer that isn't the lightmap layer
        for layer in mesh.uv_layers:
            if layer.name != lightmap_layer_name:
                mesh.uv_layers.active = layer
                break
