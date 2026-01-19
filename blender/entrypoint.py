#!/usr/bin/env python3
"""
SpecCade Blender Entrypoint

This script is executed by Blender in background mode to generate assets
from canonical JSON specs. It uses only Python stdlib modules.

Usage:
    blender --background --factory-startup --python entrypoint.py -- \
        --mode <mode> --spec <path> --out-root <path> --report <path>

Modes:
    static_mesh     - Generate static mesh (blender_primitives_v1)
    skeletal_mesh   - Generate skeletal mesh (blender_rigged_mesh_v1)
    animation       - Generate animation clip (blender_clip_v1)
"""

import argparse
import json
import math
import sys
import time
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

# Blender modules - only available when running inside Blender
try:
    import bpy
    import bmesh
    from mathutils import Euler, Matrix, Vector
    BLENDER_AVAILABLE = True
except ImportError:
    BLENDER_AVAILABLE = False


# =============================================================================
# Report Generation
# =============================================================================

def write_report(report_path: Path, ok: bool, error: Optional[str] = None,
                 metrics: Optional[Dict] = None, output_path: Optional[str] = None,
                 blend_path: Optional[str] = None, duration_ms: Optional[int] = None) -> None:
    """Write the generation report JSON."""
    report = {
        "ok": ok,
    }
    if error:
        report["error"] = error
    if metrics:
        report["metrics"] = metrics
    if output_path:
        report["output_path"] = output_path
    if blend_path:
        report["blend_path"] = blend_path
    if duration_ms is not None:
        report["duration_ms"] = duration_ms
    if BLENDER_AVAILABLE:
        report["blender_version"] = bpy.app.version_string

    with open(report_path, 'w') as f:
        json.dump(report, f, indent=2)


def compute_mesh_metrics(obj: 'bpy.types.Object') -> Dict[str, Any]:
    """Compute comprehensive metrics for a mesh object."""
    # Ensure we're working with evaluated mesh data
    depsgraph = bpy.context.evaluated_depsgraph_get()
    obj_eval = obj.evaluated_get(depsgraph)
    mesh = obj_eval.to_mesh()

    # Topology metrics
    vertex_count = len(mesh.vertices)
    face_count = len(mesh.polygons)
    edge_count = len(mesh.edges)

    # Count triangles and quads
    triangle_count = 0
    quad_count = 0
    for poly in mesh.polygons:
        vert_count = len(poly.vertices)
        if vert_count == 3:
            triangle_count += 1
        elif vert_count == 4:
            quad_count += 1
            triangle_count += 2  # A quad is 2 triangles
        else:
            triangle_count += vert_count - 2  # N-gon triangulation

    quad_percentage = (quad_count / face_count * 100.0) if face_count > 0 else 0.0

    # Manifold metrics
    non_manifold_edges = count_non_manifold_edges(mesh)
    manifold = non_manifold_edges == 0
    degenerate_faces = count_degenerate_faces(mesh)
    zero_area_faces = count_zero_area_faces(mesh)

    # Bounding box
    bbox_min = [float('inf')] * 3
    bbox_max = [float('-inf')] * 3
    for v in mesh.vertices:
        co = obj.matrix_world @ v.co
        for i in range(3):
            bbox_min[i] = min(bbox_min[i], co[i])
            bbox_max[i] = max(bbox_max[i], co[i])

    # UV metrics
    uv_island_count = 0
    uv_coverage = 0.0
    uv_overlap_percentage = 0.0
    uv_layer_count = 0
    texel_density = 0.0
    has_uv_map = len(mesh.uv_layers) > 0

    if mesh.uv_layers:
        uv_layer_count = len(mesh.uv_layers)
        uv_layer = mesh.uv_layers.active
        if uv_layer:
            uv_island_count = count_uv_islands(mesh, uv_layer)
            uv_coverage, uv_overlap_percentage = compute_uv_coverage_and_overlap(mesh, uv_layer)
            # Compute average texel density (at 1024x1024 texture reference)
            texel_density = compute_texel_density_from_mesh(mesh, uv_layer)

    # Material slot count
    material_slot_count = len(obj.material_slots)

    obj_eval.to_mesh_clear()

    return {
        "vertex_count": vertex_count,
        "face_count": face_count,
        "edge_count": edge_count,
        "triangle_count": triangle_count,
        "quad_count": quad_count,
        "quad_percentage": round(quad_percentage, 2),
        "manifold": manifold,
        "non_manifold_edge_count": non_manifold_edges,
        "degenerate_face_count": degenerate_faces,
        "zero_area_face_count": zero_area_faces,
        "uv_island_count": uv_island_count,
        "uv_coverage": round(uv_coverage, 4),
        "uv_overlap_percentage": round(uv_overlap_percentage, 2),
        "has_uv_map": has_uv_map,
        "uv_layer_count": uv_layer_count,
        "texel_density": round(texel_density, 2),
        "bounding_box": {
            "min": bbox_min,
            "max": bbox_max
        },
        "bounds_min": bbox_min,
        "bounds_max": bbox_max,
        "material_slot_count": material_slot_count
    }


def count_non_manifold_edges(mesh: 'bpy.types.Mesh') -> int:
    """Count non-manifold edges (edges with != 2 adjacent faces)."""
    edge_face_count = {}
    for poly in mesh.polygons:
        for i in range(len(poly.vertices)):
            v1 = poly.vertices[i]
            v2 = poly.vertices[(i + 1) % len(poly.vertices)]
            edge_key = (min(v1, v2), max(v1, v2))
            edge_face_count[edge_key] = edge_face_count.get(edge_key, 0) + 1

    non_manifold = 0
    for count in edge_face_count.values():
        if count != 2:
            non_manifold += 1
    return non_manifold


def count_degenerate_faces(mesh: 'bpy.types.Mesh') -> int:
    """Count degenerate faces (zero area or invalid topology)."""
    degenerate = 0
    for poly in mesh.polygons:
        # Check for zero area
        if poly.area < 1e-8:
            degenerate += 1
            continue
        # Check for duplicate vertices
        verts = list(poly.vertices)
        if len(verts) != len(set(verts)):
            degenerate += 1
    return degenerate


def count_zero_area_faces(mesh: 'bpy.types.Mesh') -> int:
    """Count faces with zero or near-zero area (CHAR-003)."""
    zero_area = 0
    for poly in mesh.polygons:
        if poly.area < 1e-8:
            zero_area += 1
    return zero_area


def compute_uv_coverage_and_overlap(
    mesh: 'bpy.types.Mesh',
    uv_layer: 'bpy.types.MeshUVLoopLayer'
) -> Tuple[float, float]:
    """Compute UV coverage (0-1) and overlap percentage (0-100)."""
    # Collect all UV triangles
    uv_triangles = []
    for poly in mesh.polygons:
        loop_indices = list(poly.loop_indices)
        uvs = [tuple(uv_layer.data[li].uv) for li in loop_indices]
        # Triangulate the polygon
        for i in range(1, len(uvs) - 1):
            uv_triangles.append((uvs[0], uvs[i], uvs[i + 1]))

    if not uv_triangles:
        return 0.0, 0.0

    # Compute total UV area (may include overlaps)
    total_area = 0.0
    for tri in uv_triangles:
        area = triangle_area_2d(tri[0], tri[1], tri[2])
        total_area += abs(area)

    # Simple coverage estimate: clamp to [0, 1]
    # UV space is [0,1] x [0,1] = 1.0 area
    coverage = min(total_area, 1.0)

    # Overlap: if total area > 1.0, there's overlap
    # This is a simplified approximation
    if total_area > 1.0:
        overlap_percentage = ((total_area - 1.0) / total_area) * 100.0
    else:
        overlap_percentage = 0.0

    return coverage, overlap_percentage


def triangle_area_2d(p1: Tuple[float, float], p2: Tuple[float, float], p3: Tuple[float, float]) -> float:
    """Compute the signed area of a 2D triangle."""
    return 0.5 * ((p2[0] - p1[0]) * (p3[1] - p1[1]) - (p3[0] - p1[0]) * (p2[1] - p1[1]))


def count_uv_islands(mesh: 'bpy.types.Mesh', uv_layer: 'bpy.types.MeshUVLoopLayer') -> int:
    """Count UV islands using union-find."""
    if not mesh.loops:
        return 0

    # Build vertex to UV mapping
    uv_verts = {}  # vertex_index -> set of UV coords
    for poly in mesh.polygons:
        for loop_idx in poly.loop_indices:
            loop = mesh.loops[loop_idx]
            uv = tuple(round(c, 4) for c in uv_layer.data[loop_idx].uv)
            vert_idx = loop.vertex_index
            if vert_idx not in uv_verts:
                uv_verts[vert_idx] = set()
            uv_verts[vert_idx].add(uv)

    # Simple island counting - edges in UV space
    # For a proper count we'd need to do connected components on UV faces
    # This is a simplified approximation
    visited_uvs = set()
    island_count = 0

    for poly in mesh.polygons:
        uv_coords = []
        for loop_idx in poly.loop_indices:
            uv = tuple(round(c, 4) for c in uv_layer.data[loop_idx].uv)
            uv_coords.append(uv)

        # Check if any UV of this face was already visited
        face_uvs = frozenset(uv_coords)
        if not any(uv in visited_uvs for uv in uv_coords):
            island_count += 1

        visited_uvs.update(uv_coords)

    return max(1, island_count)


def compute_texel_density_from_mesh(
    mesh: 'bpy.types.Mesh',
    uv_layer: 'bpy.types.MeshUVLoopLayer',
    texture_size: int = 1024
) -> float:
    """
    Compute the average texel density from mesh data.

    Args:
        mesh: The mesh data.
        uv_layer: The UV layer to analyze.
        texture_size: Reference texture size in pixels (default 1024).

    Returns:
        Average texel density in pixels per world unit.
    """
    uv_data = uv_layer.data

    total_uv_area = 0.0
    total_world_area = 0.0

    for poly in mesh.polygons:
        # Get UV coordinates for this face
        uvs = [uv_data[loop_idx].uv for loop_idx in poly.loop_indices]

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


def compute_skeletal_mesh_metrics(obj: 'bpy.types.Object', armature: 'bpy.types.Object') -> Dict[str, Any]:
    """Compute metrics for a skeletal mesh."""
    mesh_metrics = compute_mesh_metrics(obj)

    # Add bone count
    bone_count = len(armature.data.bones)

    # Compute max bone influences and skin weight metrics (CHAR-003)
    max_influences = 0
    unweighted_vertex_count = 0
    normalized_vertex_count = 0
    max_weight_deviation = 0.0

    if obj.vertex_groups:
        for v in obj.data.vertices:
            # Count influences per vertex (weight > 0.001 threshold)
            influences = sum(1 for g in v.groups if g.weight > 0.001)
            max_influences = max(max_influences, influences)

            # Compute weight sum for this vertex
            weight_sum = sum(g.weight for g in v.groups)

            # Track unweighted vertices (total weight < 0.001)
            if weight_sum < 0.001:
                unweighted_vertex_count += 1
            else:
                # Track deviation from normalized (1.0)
                deviation = abs(weight_sum - 1.0)
                max_weight_deviation = max(max_weight_deviation, deviation)

                # Consider normalized if within 0.001 of 1.0
                if deviation < 0.001:
                    normalized_vertex_count += 1

    # Compute weight normalization percentage
    total_vertices = len(obj.data.vertices)
    weight_normalization_percentage = 0.0
    if total_vertices > 0:
        weight_normalization_percentage = (normalized_vertex_count / total_vertices) * 100.0

    mesh_metrics["bone_count"] = bone_count
    mesh_metrics["max_bone_influences"] = max_influences
    mesh_metrics["unweighted_vertex_count"] = unweighted_vertex_count
    mesh_metrics["weight_normalization_percentage"] = round(weight_normalization_percentage, 2)
    mesh_metrics["max_weight_deviation"] = round(max_weight_deviation, 6)

    return mesh_metrics


def compute_animation_metrics(armature: 'bpy.types.Object', action: 'bpy.types.Action') -> Dict[str, Any]:
    """Compute metrics for an animation."""
    bone_count = len(armature.data.bones)

    # Get frame range
    frame_start = int(action.frame_range[0])
    frame_end = int(action.frame_range[1])
    frame_count = frame_end - frame_start + 1

    # Get FPS from scene
    fps = bpy.context.scene.render.fps
    duration_seconds = frame_count / fps

    return {
        "bone_count": bone_count,
        "animation_frame_count": frame_count,
        "animation_duration_seconds": duration_seconds
    }


# =============================================================================
# Scene Setup
# =============================================================================

def clear_scene() -> None:
    """Clear the Blender scene."""
    bpy.ops.wm.read_factory_settings(use_empty=True)


def setup_scene() -> None:
    """Set up the scene for export."""
    # Ensure we have a scene
    if not bpy.context.scene:
        bpy.ops.scene.new(type='NEW')


# =============================================================================
# Primitive Creation
# =============================================================================

PRIMITIVE_CREATORS = {
    "cube": lambda dims: bpy.ops.mesh.primitive_cube_add(size=1),
    "sphere": lambda dims: bpy.ops.mesh.primitive_uv_sphere_add(radius=0.5, segments=32, ring_count=16),
    "cylinder": lambda dims: bpy.ops.mesh.primitive_cylinder_add(radius=0.5, depth=1, vertices=32),
    "cone": lambda dims: bpy.ops.mesh.primitive_cone_add(radius1=0.5, radius2=0, depth=1, vertices=32),
    "torus": lambda dims: bpy.ops.mesh.primitive_torus_add(major_radius=0.5, minor_radius=0.125),
    "plane": lambda dims: bpy.ops.mesh.primitive_plane_add(size=1),
    "ico_sphere": lambda dims: bpy.ops.mesh.primitive_ico_sphere_add(radius=0.5, subdivisions=2),
}


def create_primitive(primitive_type: str, dimensions: List[float]) -> 'bpy.types.Object':
    """Create a mesh primitive."""
    primitive_type = primitive_type.lower().replace("_", "")

    if primitive_type == "icosphere":
        primitive_type = "ico_sphere"

    if primitive_type not in PRIMITIVE_CREATORS:
        raise ValueError(f"Unknown primitive type: {primitive_type}")

    PRIMITIVE_CREATORS[primitive_type](dimensions)
    obj = bpy.context.active_object

    # Scale to dimensions
    obj.scale = (dimensions[0], dimensions[1], dimensions[2])
    bpy.ops.object.transform_apply(scale=True)

    return obj


# =============================================================================
# Modifier Application
# =============================================================================

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


# =============================================================================
# UV Projection
# =============================================================================

def apply_uv_projection(obj: 'bpy.types.Object', projection) -> None:
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


# =============================================================================
# Normals Settings
# =============================================================================

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


# =============================================================================
# Material Creation
# =============================================================================

def create_material(name: str, spec: Dict) -> 'bpy.types.Material':
    """Create a material from spec."""
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True

    # Get principled BSDF node
    nodes = mat.node_tree.nodes
    principled = nodes.get("Principled BSDF")
    if not principled:
        return mat

    # Base color
    if "base_color" in spec:
        color = spec["base_color"]
        if len(color) == 3:
            color = color + [1.0]
        principled.inputs["Base Color"].default_value = color

    # Metallic
    if "metallic" in spec:
        principled.inputs["Metallic"].default_value = spec["metallic"]

    # Roughness
    if "roughness" in spec:
        principled.inputs["Roughness"].default_value = spec["roughness"]

    # Emissive
    if "emissive" in spec:
        emissive = spec["emissive"]
        strength = spec.get("emissive_strength", 1.0)
        # Blender 4.0+ uses "Emission Color"
        if "Emission Color" in principled.inputs:
            principled.inputs["Emission Color"].default_value = emissive + [1.0]
            principled.inputs["Emission Strength"].default_value = strength
        elif "Emission" in principled.inputs:
            principled.inputs["Emission"].default_value = emissive + [1.0]

    return mat


def apply_materials(obj: 'bpy.types.Object', material_slots: List[Dict]) -> None:
    """Apply materials to an object."""
    for slot_spec in material_slots:
        name = slot_spec.get("name", "Material")
        mat = create_material(name, slot_spec)
        obj.data.materials.append(mat)


# =============================================================================
# Skeleton Creation
# =============================================================================

# Humanoid basic v1 skeleton definition
HUMANOID_BASIC_V1_BONES = {
    "root": {"head": (0, 0, 0), "tail": (0, 0, 0.1), "parent": None},
    "hips": {"head": (0, 0, 0.9), "tail": (0, 0, 1.0), "parent": "root"},
    "spine": {"head": (0, 0, 1.0), "tail": (0, 0, 1.2), "parent": "hips"},
    "chest": {"head": (0, 0, 1.2), "tail": (0, 0, 1.4), "parent": "spine"},
    "neck": {"head": (0, 0, 1.4), "tail": (0, 0, 1.5), "parent": "chest"},
    "head": {"head": (0, 0, 1.5), "tail": (0, 0, 1.7), "parent": "neck"},
    "shoulder_l": {"head": (0.1, 0, 1.35), "tail": (0.2, 0, 1.35), "parent": "chest"},
    "upper_arm_l": {"head": (0.2, 0, 1.35), "tail": (0.45, 0, 1.35), "parent": "shoulder_l"},
    "lower_arm_l": {"head": (0.45, 0, 1.35), "tail": (0.7, 0, 1.35), "parent": "upper_arm_l"},
    "hand_l": {"head": (0.7, 0, 1.35), "tail": (0.8, 0, 1.35), "parent": "lower_arm_l"},
    "shoulder_r": {"head": (-0.1, 0, 1.35), "tail": (-0.2, 0, 1.35), "parent": "chest"},
    "upper_arm_r": {"head": (-0.2, 0, 1.35), "tail": (-0.45, 0, 1.35), "parent": "shoulder_r"},
    "lower_arm_r": {"head": (-0.45, 0, 1.35), "tail": (-0.7, 0, 1.35), "parent": "upper_arm_r"},
    "hand_r": {"head": (-0.7, 0, 1.35), "tail": (-0.8, 0, 1.35), "parent": "lower_arm_r"},
    "upper_leg_l": {"head": (0.1, 0, 0.9), "tail": (0.1, 0, 0.5), "parent": "hips"},
    "lower_leg_l": {"head": (0.1, 0, 0.5), "tail": (0.1, 0, 0.1), "parent": "upper_leg_l"},
    "foot_l": {"head": (0.1, 0, 0.1), "tail": (0.1, 0.15, 0), "parent": "lower_leg_l"},
    "upper_leg_r": {"head": (-0.1, 0, 0.9), "tail": (-0.1, 0, 0.5), "parent": "hips"},
    "lower_leg_r": {"head": (-0.1, 0, 0.5), "tail": (-0.1, 0, 0.1), "parent": "upper_leg_r"},
    "foot_r": {"head": (-0.1, 0, 0.1), "tail": (-0.1, 0.15, 0), "parent": "lower_leg_r"},
}

SKELETON_PRESETS = {
    "humanoid_basic_v1": HUMANOID_BASIC_V1_BONES,
}


def create_armature(preset_name: str) -> 'bpy.types.Object':
    """Create an armature from a preset."""
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
        created_bones[bone_name] = bone

    # Set up bone hierarchy
    for bone_name, bone_spec in preset.items():
        parent_name = bone_spec.get("parent")
        if parent_name and parent_name in created_bones:
            created_bones[bone_name].parent = created_bones[parent_name]

    bpy.ops.object.mode_set(mode='OBJECT')

    return armature_obj


def get_bone_position(armature: 'bpy.types.Object', bone_name: str) -> Vector:
    """Get the world position of a bone."""
    bone = armature.data.bones.get(bone_name)
    if bone:
        return armature.matrix_world @ bone.head_local
    return Vector((0, 0, 0))


# =============================================================================
# Body Part Creation
# =============================================================================

def create_body_part(armature: 'bpy.types.Object', part_spec: Dict) -> 'bpy.types.Object':
    """Create a body part mesh attached to a bone."""
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
# Legacy Part System (ai-studio-core SPEC dict format)
# =============================================================================

def parse_base_shape(base: str) -> Tuple[str, int]:
    """
    Parse a base shape specification like 'hexagon(6)' or 'circle(8)'.

    Returns:
        Tuple of (shape_type, vertex_count)
    """
    import re
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
        step: Step specification dictionary.
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


def create_legacy_part(
    armature: 'bpy.types.Object',
    part_name: str,
    part_spec: Dict,
    all_parts: Dict[str, Dict]
) -> Optional['bpy.types.Object']:
    """
    Create a mesh part using the legacy ai-studio-core SPEC format.

    Args:
        armature: The armature object (can be None for standalone parts).
        part_name: Name of this part.
        part_spec: Part specification dictionary.
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

    # Position at bone location if armature exists
    if armature:
        bone_pos = get_bone_position(armature, bone_name)
        offset = part_spec.get('offset', [0, 0, 0])
        obj.location = bone_pos + Vector(offset)
    else:
        offset = part_spec.get('offset', [0, 0, 0])
        obj.location = Vector(offset)

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
    obj = create_legacy_part(armature, f"{part_name}_temp", source_copy, all_parts)
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
    """Handle cap_start and cap_end for a part."""
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
    """Join source object into target object."""
    bpy.ops.object.select_all(action='DESELECT')
    target.select_set(True)
    source.select_set(True)
    bpy.context.view_layer.objects.active = target
    bpy.ops.object.join()


# =============================================================================
# Custom Skeleton Creation
# =============================================================================

def create_custom_skeleton(skeleton_spec: List[Dict]) -> 'bpy.types.Object':
    """
    Create a custom skeleton from a list of bone specifications.

    Args:
        skeleton_spec: List of bone definitions with 'bone', 'head', 'tail', 'parent', 'mirror'.

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

    Args:
        name: Original bone name.

    Returns:
        Mirrored bone name.
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


# =============================================================================
# Texturing / UV Mapping
# =============================================================================

def apply_texturing(obj: 'bpy.types.Object', texturing_spec: Dict) -> None:
    """
    Apply texturing/UV settings to a mesh object.

    Args:
        obj: The mesh object.
        texturing_spec: Texturing specification with 'uv_mode' and 'regions'.
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
        region_spec: Region specification with 'parts', 'material_index', 'color'.
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
    """Parent mesh to armature with automatic weights."""
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
    """Assign all vertices to a vertex group for a bone."""
    # Create vertex group
    vg = mesh_obj.vertex_groups.new(name=bone_name)

    # Add all vertices with weight 1.0
    vertex_indices = [v.index for v in mesh_obj.data.vertices]
    vg.add(vertex_indices, 1.0, 'REPLACE')


# =============================================================================
# IK Chain Setup
# =============================================================================

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

    return result


# =============================================================================
# Bone Constraint Setup
# =============================================================================

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


# =============================================================================
# Foot System Setup
# =============================================================================

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


# =============================================================================
# Aim Constraint Setup
# =============================================================================

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


# =============================================================================
# Twist Bone Setup
# =============================================================================

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


# =============================================================================
# Stretch Settings
# =============================================================================

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


# =============================================================================
# Procedural Animation Layers
# =============================================================================

def apply_procedural_layers(
    armature: 'bpy.types.Object',
    layers: List[Dict],
    fps: int,
    frame_count: int
) -> None:
    """
    Apply procedural animation layers to bones.

    Generates automatic motion overlays like breathing, swaying, bobbing,
    and noise-based movement.

    Args:
        armature: The armature object.
        layers: List of procedural layer configurations.
        fps: Frames per second.
        frame_count: Total number of frames.
    """
    if not layers:
        return

    # Ensure we're in object mode first
    if bpy.context.mode != 'OBJECT':
        bpy.ops.object.mode_set(mode='OBJECT')

    bpy.context.view_layer.objects.active = armature
    bpy.ops.object.mode_set(mode='POSE')

    for layer in layers:
        layer_type = layer.get('type', 'breathing')
        target = layer.get('target', '')
        axis = layer.get('axis', 'pitch')
        period_frames = layer.get('period_frames', 60)
        amplitude = layer.get('amplitude', 0.01)
        phase_offset = layer.get('phase_offset', 0.0)
        frequency = layer.get('frequency', 0.3)

        pose_bone = armature.pose.bones.get(target)
        if not pose_bone:
            print(f"Warning: Bone '{target}' not found for procedural layer")
            continue

        pose_bone.rotation_mode = 'XYZ'

        # Map axis to index
        axis_map = {'pitch': 0, 'yaw': 1, 'roll': 2}
        axis_idx = axis_map.get(axis, 0)

        # Generate keyframes based on layer type
        if layer_type in ('breathing', 'sway', 'bob'):
            # Sine wave animation
            for frame in range(1, frame_count + 1):
                t = (frame - 1) / period_frames
                value = math.sin((t + phase_offset) * 2 * math.pi) * amplitude

                # Store current rotation and modify the target axis
                rot = list(pose_bone.rotation_euler)
                rot[axis_idx] = value
                pose_bone.rotation_euler = Euler(rot)
                pose_bone.keyframe_insert(data_path="rotation_euler", frame=frame, index=axis_idx)

        elif layer_type == 'noise':
            # Noise-based animation using simple pseudo-random
            import random
            random.seed(hash(target))
            for frame in range(1, frame_count + 1):
                # Smooth noise using interpolated random values
                noise_val = math.sin(frame * frequency) * random.uniform(-1, 1)
                value = noise_val * math.radians(amplitude)

                rot = list(pose_bone.rotation_euler)
                rot[axis_idx] = value
                pose_bone.rotation_euler = Euler(rot)
                pose_bone.keyframe_insert(data_path="rotation_euler", frame=frame, index=axis_idx)

    bpy.ops.object.mode_set(mode='OBJECT')
    print(f"Applied {len(layers)} procedural layers")


# =============================================================================
# Pose and Phase System
# =============================================================================

def apply_poses_and_phases(
    armature: 'bpy.types.Object',
    poses: Dict[str, Dict],
    phases: List[Dict],
    fps: int
) -> None:
    """
    Apply named poses and animation phases to an armature.

    Args:
        armature: The armature object.
        poses: Dictionary of named pose definitions.
        phases: List of animation phase configurations.
        fps: Frames per second.
    """
    if not phases:
        return

    # Ensure we're in object mode first
    if bpy.context.mode != 'OBJECT':
        bpy.ops.object.mode_set(mode='OBJECT')

    bpy.context.view_layer.objects.active = armature
    bpy.ops.object.mode_set(mode='POSE')

    for phase in phases:
        phase_name = phase.get('name', 'unnamed')
        start_frame = phase.get('start_frame', 1)
        end_frame = phase.get('end_frame', 30)
        curve = phase.get('curve', 'linear')
        pose_name = phase.get('pose')
        ik_targets = phase.get('ik_targets', {})

        # Apply pose at start frame if specified
        if pose_name and pose_name in poses:
            pose_def = poses[pose_name]
            bones_data = pose_def.get('bones', {})

            for bone_name, transform in bones_data.items():
                pose_bone = armature.pose.bones.get(bone_name)
                if not pose_bone:
                    continue

                pose_bone.rotation_mode = 'XYZ'
                pitch = math.radians(transform.get('pitch', 0))
                yaw = math.radians(transform.get('yaw', 0))
                roll = math.radians(transform.get('roll', 0))

                pose_bone.rotation_euler = Euler((pitch, yaw, roll))
                pose_bone.keyframe_insert(data_path="rotation_euler", frame=start_frame)

                # Apply location if present
                location = transform.get('location')
                if location:
                    pose_bone.location = Vector(location)
                    pose_bone.keyframe_insert(data_path="location", frame=start_frame)

        # Apply IK target keyframes
        for target_name, target_keyframes in ik_targets.items():
            target_obj = bpy.data.objects.get(target_name)
            if not target_obj:
                print(f"Warning: IK target '{target_name}' not found for phase '{phase_name}'")
                continue

            for kf in target_keyframes:
                frame = kf.get('frame', start_frame)
                location = kf.get('location', [0, 0, 0])
                ikfk = kf.get('ikfk')

                target_obj.location = Vector(location)
                target_obj.keyframe_insert(data_path="location", frame=frame)

        # Map curve type to Blender interpolation
        interp_map = {
            'linear': 'LINEAR',
            'ease_in': 'QUAD',
            'ease_out': 'QUAD',
            'ease_in_out': 'BEZIER',
            'exponential_in': 'EXPO',
            'exponential_out': 'EXPO',
            'constant': 'CONSTANT',
        }
        interp = interp_map.get(curve, 'LINEAR')

        # Set interpolation for all keyframes in this phase range
        action = armature.animation_data.action if armature.animation_data else None
        if action:
            for fcurve in action.fcurves:
                for kp in fcurve.keyframe_points:
                    if start_frame <= kp.co[0] <= end_frame:
                        kp.interpolation = interp

    bpy.ops.object.mode_set(mode='OBJECT')
    print(f"Applied {len(phases)} animation phases")


# =============================================================================
# Bake Settings
# =============================================================================

def bake_animation(
    armature: 'bpy.types.Object',
    bake_settings: Dict,
    frame_start: int,
    frame_end: int
) -> None:
    """
    Bake animation with specified settings.

    Args:
        armature: The armature object.
        bake_settings: Dictionary with bake configuration:
            - simplify: Simplify curves after baking
            - start_frame: Start frame (optional, uses scene default)
            - end_frame: End frame (optional, uses scene default)
            - visual_keying: Use visual transforms
            - clear_constraints: Clear constraints after baking
            - frame_step: Frame step for baking
            - tolerance: Tolerance for curve simplification
            - remove_ik_bones: Remove IK control bones after baking
    """
    simplify = bake_settings.get('simplify', True)
    bake_start = bake_settings.get('start_frame', frame_start)
    bake_end = bake_settings.get('end_frame', frame_end)
    visual_keying = bake_settings.get('visual_keying', True)
    clear_constraints = bake_settings.get('clear_constraints', True)
    frame_step = bake_settings.get('frame_step', 1)
    tolerance = bake_settings.get('tolerance', 0.001)
    remove_ik = bake_settings.get('remove_ik_bones', True)

    # Ensure we're in object mode first
    if bpy.context.mode != 'OBJECT':
        bpy.ops.object.mode_set(mode='OBJECT')

    bpy.context.view_layer.objects.active = armature
    bpy.ops.object.mode_set(mode='POSE')

    # Select all pose bones
    bpy.ops.pose.select_all(action='SELECT')

    # Bake the animation
    bpy.ops.nla.bake(
        frame_start=bake_start,
        frame_end=bake_end,
        step=frame_step,
        only_selected=False,
        visual_keying=visual_keying,
        clear_constraints=clear_constraints,
        use_current_action=True,
        bake_types={'POSE'}
    )

    # Simplify curves if requested
    if simplify and armature.animation_data and armature.animation_data.action:
        action = armature.animation_data.action
        for fcurve in action.fcurves:
            # Use Blender's keyframe reduction
            keyframe_count_before = len(fcurve.keyframe_points)
            # Note: Blender doesn't have a direct "simplify" operator for fcurves
            # We can use the decimate operator in graph editor or do it manually
            # For now, we just report the keyframe counts
            print(f"FCurve {fcurve.data_path}: {keyframe_count_before} keyframes")

    # Remove IK control objects if requested
    if remove_ik:
        bpy.ops.object.mode_set(mode='OBJECT')
        # Find and remove IK target empties
        objects_to_remove = []
        for obj in bpy.data.objects:
            if obj.type == 'EMPTY' and ('_target' in obj.name or '_pole' in obj.name):
                objects_to_remove.append(obj)

        for obj in objects_to_remove:
            bpy.data.objects.remove(obj, do_unlink=True)

        if objects_to_remove:
            print(f"Removed {len(objects_to_remove)} IK control objects")
    else:
        bpy.ops.object.mode_set(mode='OBJECT')

    print(f"Baked animation: frames {bake_start}-{bake_end}, visual_keying={visual_keying}")


# =============================================================================
# Animator Rig Configuration
# =============================================================================

# Widget collection name (hidden from viewport)
WIDGET_COLLECTION_NAME = "Widgets"

# Standard widget shapes
WIDGET_SHAPES = {
    "wire_circle": "WGT_circle",
    "wire_cube": "WGT_cube",
    "wire_sphere": "WGT_sphere",
    "wire_diamond": "WGT_diamond",
    "custom_mesh": "WGT_custom",
}

# Standard bone colors (L=blue, R=red, center=yellow)
BONE_COLORS = {
    "left": (0.2, 0.4, 1.0),      # Blue
    "right": (1.0, 0.3, 0.3),     # Red
    "center": (1.0, 0.9, 0.2),    # Yellow
}


def create_widget_shapes(armature: 'bpy.types.Object') -> Dict[str, 'bpy.types.Object']:
    """
    Create the 5 standard widget shapes for bone visualization.

    Widget shapes are created in a hidden collection and can be assigned
    to bones as custom shapes for better visual feedback during animation.

    Args:
        armature: The armature object (used to scale widgets appropriately).

    Returns:
        Dictionary mapping widget style names to their mesh objects.
    """
    widgets = {}

    # Create or get the widgets collection
    widget_collection = bpy.data.collections.get(WIDGET_COLLECTION_NAME)
    if not widget_collection:
        widget_collection = bpy.data.collections.new(WIDGET_COLLECTION_NAME)
        bpy.context.scene.collection.children.link(widget_collection)

    # Hide the widget collection from viewport
    widget_collection.hide_viewport = True
    widget_collection.hide_render = True

    # Calculate base scale from armature
    # Average bone length gives us a reasonable widget scale
    avg_bone_length = 0.1  # Default
    if armature.data.bones:
        total_length = sum(bone.length for bone in armature.data.bones)
        avg_bone_length = total_length / len(armature.data.bones)
    widget_scale = avg_bone_length * 0.5

    # Create Wire Circle widget
    if WIDGET_SHAPES["wire_circle"] not in bpy.data.objects:
        bpy.ops.mesh.primitive_circle_add(
            vertices=32,
            radius=widget_scale,
            fill_type='NOTHING',
            enter_editmode=False
        )
        circle = bpy.context.active_object
        circle.name = WIDGET_SHAPES["wire_circle"]
        circle.display_type = 'WIRE'
        # Move to widget collection
        for col in circle.users_collection:
            col.objects.unlink(circle)
        widget_collection.objects.link(circle)
        widgets["wire_circle"] = circle
    else:
        widgets["wire_circle"] = bpy.data.objects[WIDGET_SHAPES["wire_circle"]]

    # Create Wire Cube widget
    if WIDGET_SHAPES["wire_cube"] not in bpy.data.objects:
        bpy.ops.mesh.primitive_cube_add(size=widget_scale * 2)
        cube = bpy.context.active_object
        cube.name = WIDGET_SHAPES["wire_cube"]
        cube.display_type = 'WIRE'
        # Move to widget collection
        for col in cube.users_collection:
            col.objects.unlink(cube)
        widget_collection.objects.link(cube)
        widgets["wire_cube"] = cube
    else:
        widgets["wire_cube"] = bpy.data.objects[WIDGET_SHAPES["wire_cube"]]

    # Create Wire Sphere widget
    # Use lower polygon count for wireframe display (efficiency)
    if WIDGET_SHAPES["wire_sphere"] not in bpy.data.objects:
        bpy.ops.mesh.primitive_uv_sphere_add(
            radius=widget_scale,
            segments=12,
            ring_count=6
        )
        sphere = bpy.context.active_object
        sphere.name = WIDGET_SHAPES["wire_sphere"]
        sphere.display_type = 'WIRE'
        # Move to widget collection
        for col in sphere.users_collection:
            col.objects.unlink(sphere)
        widget_collection.objects.link(sphere)
        widgets["wire_sphere"] = sphere
    else:
        widgets["wire_sphere"] = bpy.data.objects[WIDGET_SHAPES["wire_sphere"]]

    # Create Wire Diamond widget (octahedron)
    if WIDGET_SHAPES["wire_diamond"] not in bpy.data.objects:
        # Create a diamond shape using bmesh
        mesh = bpy.data.meshes.new(WIDGET_SHAPES["wire_diamond"])
        diamond = bpy.data.objects.new(WIDGET_SHAPES["wire_diamond"], mesh)

        bm = bmesh.new()
        # Diamond vertices: top, bottom, and 4 around the middle
        top = bm.verts.new((0, 0, widget_scale))
        bottom = bm.verts.new((0, 0, -widget_scale))
        mid_verts = [
            bm.verts.new((widget_scale * 0.7, 0, 0)),
            bm.verts.new((0, widget_scale * 0.7, 0)),
            bm.verts.new((-widget_scale * 0.7, 0, 0)),
            bm.verts.new((0, -widget_scale * 0.7, 0)),
        ]
        bm.verts.ensure_lookup_table()

        # Create faces
        for i in range(4):
            next_i = (i + 1) % 4
            bm.faces.new([top, mid_verts[i], mid_verts[next_i]])
            bm.faces.new([bottom, mid_verts[next_i], mid_verts[i]])

        bm.to_mesh(mesh)
        bm.free()

        diamond.display_type = 'WIRE'
        widget_collection.objects.link(diamond)
        widgets["wire_diamond"] = diamond
    else:
        widgets["wire_diamond"] = bpy.data.objects[WIDGET_SHAPES["wire_diamond"]]

    # Create Custom Mesh placeholder (empty mesh that can be replaced)
    if WIDGET_SHAPES["custom_mesh"] not in bpy.data.objects:
        mesh = bpy.data.meshes.new(WIDGET_SHAPES["custom_mesh"])
        custom = bpy.data.objects.new(WIDGET_SHAPES["custom_mesh"], mesh)
        custom.display_type = 'WIRE'
        widget_collection.objects.link(custom)
        widgets["custom_mesh"] = custom
    else:
        widgets["custom_mesh"] = bpy.data.objects[WIDGET_SHAPES["custom_mesh"]]

    return widgets


def organize_bone_collections(
    armature: 'bpy.types.Object',
    collections_config: Optional[List[Dict]] = None
) -> Dict[str, List[str]]:
    """
    Organize bones into collections (IK Controls, FK Controls, Deform, Mechanism).

    Bone collections help animators by grouping related bones and allowing
    them to be shown/hidden as needed.

    Args:
        armature: The armature object.
        collections_config: Optional list of collection configurations. Each entry:
            - name: Collection name
            - bones: List of bone names
            - visible: Whether collection is visible (default True)
            - selectable: Whether bones are selectable (default True)

    Returns:
        Dictionary mapping collection names to lists of bone names.
    """
    # Default collections if none provided
    if collections_config is None:
        collections_config = [
            {
                "name": "IK Controls",
                "bones": [],  # Will be auto-populated
                "visible": True,
                "selectable": True,
            },
            {
                "name": "FK Controls",
                "bones": [],  # Will be auto-populated
                "visible": True,
                "selectable": True,
            },
            {
                "name": "Deform",
                "bones": [],  # Will be auto-populated
                "visible": False,
                "selectable": False,
            },
            {
                "name": "Mechanism",
                "bones": [],  # Will be auto-populated
                "visible": False,
                "selectable": False,
            },
        ]

    # Ensure we're in object mode
    if bpy.context.mode != 'OBJECT':
        bpy.ops.object.mode_set(mode='OBJECT')

    bpy.context.view_layer.objects.active = armature

    # Get all bone names
    all_bones = [bone.name for bone in armature.data.bones]

    # Auto-categorize bones if collections have empty bone lists
    ik_patterns = ["ik_", "pole_", "target_"]
    fk_patterns = ["fk_", "ctrl_"]
    deform_patterns = ["def_", "deform_"]
    mechanism_patterns = ["mch_", "mechanism_", "helper_"]

    # Track which bones are assigned
    assigned_bones = set()
    result = {}

    for config in collections_config:
        coll_name = config.get("name", "Collection")
        bones = config.get("bones", [])
        visible = config.get("visible", True)
        selectable = config.get("selectable", True)

        # If no bones specified, auto-categorize based on collection name
        if not bones:
            if "IK" in coll_name.upper():
                bones = [b for b in all_bones if any(b.lower().startswith(p) for p in ik_patterns) and b not in assigned_bones]
            elif "FK" in coll_name.upper():
                bones = [b for b in all_bones if any(b.lower().startswith(p) for p in fk_patterns) and b not in assigned_bones]
                # Also include main skeleton bones that aren't IK or mechanism
                for b in all_bones:
                    if b not in assigned_bones and b not in bones and not any(b.lower().startswith(p) for p in ik_patterns + mechanism_patterns + deform_patterns):
                        bones.append(b)
            elif "DEFORM" in coll_name.upper():
                bones = [b for b in all_bones if any(b.lower().startswith(p) for p in deform_patterns) and b not in assigned_bones]
            elif "MECHANISM" in coll_name.upper():
                bones = [b for b in all_bones if any(b.lower().startswith(p) for p in mechanism_patterns) and b not in assigned_bones]

        # Filter to only existing bones
        bones = [b for b in bones if b in all_bones]
        assigned_bones.update(bones)

        # Create bone collection in Blender 4.0+
        # Note: Blender 4.0 introduced bone collections, replacing bone groups
        use_bone_collections = hasattr(armature.data, 'collections')

        if use_bone_collections:
            try:
                # Blender 4.0+ bone collections API
                bone_coll = armature.data.collections.get(coll_name)
                if not bone_coll:
                    bone_coll = armature.data.collections.new(coll_name)

                # Set visibility
                bone_coll.is_visible = visible

                # Assign bones to collection
                for bone_name in bones:
                    bone = armature.data.bones.get(bone_name)
                    if bone:
                        bone_coll.assign(bone)

            except (AttributeError, RuntimeError) as e:
                print(f"Warning: Failed to create bone collection '{coll_name}': {e}")
                use_bone_collections = False

        if not use_bone_collections:
            # Fallback for older Blender versions using bone groups
            if hasattr(armature.pose, 'bone_groups'):
                bpy.ops.object.mode_set(mode='POSE')

                # Create bone group
                bone_group = armature.pose.bone_groups.get(coll_name)
                if not bone_group:
                    bone_group = armature.pose.bone_groups.new(name=coll_name)

                # Assign bones to group
                for bone_name in bones:
                    pose_bone = armature.pose.bones.get(bone_name)
                    if pose_bone:
                        pose_bone.bone_group = bone_group

                bpy.ops.object.mode_set(mode='OBJECT')

        result[coll_name] = bones

    return result


def apply_bone_colors(
    armature: 'bpy.types.Object',
    color_scheme: str = "standard",
    custom_colors: Optional[Dict] = None
) -> None:
    """
    Apply color coding to bones (L=blue, R=red, center=yellow).

    Color-coded bones help animators quickly identify left vs right side
    bones and center bones.

    Args:
        armature: The armature object.
        color_scheme: One of "standard", "custom", or "per_bone".
        custom_colors: For "custom" scheme: dict with "left", "right", "center" colors.
                      For "per_bone" scheme: dict mapping bone names to (r, g, b) tuples.
    """
    if bpy.context.mode != 'OBJECT':
        bpy.ops.object.mode_set(mode='OBJECT')

    bpy.context.view_layer.objects.active = armature

    def get_bone_color(bone_name: str) -> Tuple[float, float, float]:
        """Determine color for a bone based on its name and the color scheme."""
        if color_scheme == "per_bone" and custom_colors:
            if bone_name in custom_colors:
                c = custom_colors[bone_name]
                return (c.get("r", 1.0), c.get("g", 1.0), c.get("b", 1.0))
            # Fall back to standard for unlisted bones
            return BONE_COLORS["center"]

        if color_scheme == "custom" and custom_colors:
            if bone_name.endswith("_l") or bone_name.endswith("_L"):
                c = custom_colors.get("left", {})
                return (c.get("r", 0.2), c.get("g", 0.4), c.get("b", 1.0))
            elif bone_name.endswith("_r") or bone_name.endswith("_R"):
                c = custom_colors.get("right", {})
                return (c.get("r", 1.0), c.get("g", 0.3), c.get("b", 0.3))
            else:
                c = custom_colors.get("center", {})
                return (c.get("r", 1.0), c.get("g", 0.9), c.get("b", 0.2))

        # Standard scheme
        if bone_name.endswith("_l") or bone_name.endswith("_L"):
            return BONE_COLORS["left"]
        elif bone_name.endswith("_r") or bone_name.endswith("_R"):
            return BONE_COLORS["right"]
        else:
            return BONE_COLORS["center"]

    # Apply colors to bones
    # Blender 4.0+ uses bone.color for individual bone colors
    try:
        if hasattr(armature.data.bones[0], 'color') if armature.data.bones else False:
            # Blender 4.0+ bone color API
            for bone in armature.data.bones:
                color = get_bone_color(bone.name)
                bone.color.palette = 'CUSTOM'
                bone.color.custom.normal = (*color, 1.0)  # RGBA
                bone.color.custom.select = tuple(min(c + 0.2, 1.0) for c in color) + (1.0,)
                bone.color.custom.active = tuple(min(c + 0.4, 1.0) for c in color) + (1.0,)
    except (AttributeError, IndexError):
        # Fallback for older Blender versions using bone groups with colors
        if hasattr(armature.pose, 'bone_groups'):
            bpy.ops.object.mode_set(mode='POSE')

            # Create color groups
            color_groups = {}
            for color_name, color_value in BONE_COLORS.items():
                group = armature.pose.bone_groups.get(f"Color_{color_name}")
                if not group:
                    group = armature.pose.bone_groups.new(name=f"Color_{color_name}")
                group.color_set = 'CUSTOM'
                # Set theme colors
                group.colors.normal = color_value
                group.colors.select = tuple(min(c + 0.2, 1.0) for c in color_value)
                group.colors.active = tuple(min(c + 0.4, 1.0) for c in color_value)
                color_groups[color_name] = group

            # Assign bones to color groups
            for pose_bone in armature.pose.bones:
                color = get_bone_color(pose_bone.name)
                if color == BONE_COLORS["left"]:
                    pose_bone.bone_group = color_groups["left"]
                elif color == BONE_COLORS["right"]:
                    pose_bone.bone_group = color_groups["right"]
                else:
                    pose_bone.bone_group = color_groups["center"]

            bpy.ops.object.mode_set(mode='OBJECT')


def assign_widget_to_bone(
    armature: 'bpy.types.Object',
    bone_name: str,
    widget_style: str,
    widgets: Dict[str, 'bpy.types.Object']
) -> bool:
    """
    Assign a widget shape to a specific bone.

    Args:
        armature: The armature object.
        bone_name: Name of the bone to assign the widget to.
        widget_style: One of "wire_circle", "wire_cube", "wire_sphere",
                     "wire_diamond", or "custom_mesh".
        widgets: Dictionary of created widget objects.

    Returns:
        True if widget was assigned successfully.
    """
    if widget_style not in widgets:
        print(f"Warning: Unknown widget style '{widget_style}'")
        return False

    widget = widgets[widget_style]
    pose_bone = armature.pose.bones.get(bone_name)

    if not pose_bone:
        print(f"Warning: Bone '{bone_name}' not found in armature")
        return False

    pose_bone.custom_shape = widget
    return True


def apply_animator_rig_config(
    armature: 'bpy.types.Object',
    animator_rig_config: Dict
) -> Dict[str, Any]:
    """
    Apply animator rig configuration to an armature.

    This is the main entry point for setting up visual aids for animators.

    Args:
        armature: The armature object.
        animator_rig_config: Configuration dictionary with keys:
            - collections: bool - Whether to organize bone collections
            - shapes: bool - Whether to add custom bone shapes
            - colors: bool - Whether to color-code bones
            - display: str - Armature display type (OCTAHEDRAL, STICK, etc.)
            - widget_style: str - Default widget style for control bones
            - bone_collections: list - Custom bone collection definitions
            - bone_colors: dict - Bone color scheme configuration

    Returns:
        Dictionary with results from each operation.
    """
    result = {
        "widgets_created": False,
        "collections_created": False,
        "colors_applied": False,
    }

    # Set armature display type
    display_type = animator_rig_config.get("display", "OCTAHEDRAL")
    armature.data.display_type = display_type

    # Create widget shapes if enabled
    widgets = {}
    if animator_rig_config.get("shapes", True):
        widgets = create_widget_shapes(armature)
        result["widgets_created"] = True
        widgets_assigned = 0
        widgets_failed = 0

        # Assign default widget to control bones
        widget_style = animator_rig_config.get("widget_style", "wire_circle")
        for bone in armature.data.bones:
            # Assign widgets to IK targets (diamond) and other controls (specified style)
            success = False
            if bone.name.startswith("ik_"):
                success = assign_widget_to_bone(armature, bone.name, "wire_diamond", widgets)
            elif bone.name.startswith("pole_"):
                success = assign_widget_to_bone(armature, bone.name, "wire_sphere", widgets)
            elif not bone.name.startswith(("def_", "mch_", "deform_", "mechanism_")):
                success = assign_widget_to_bone(armature, bone.name, widget_style, widgets)
            else:
                continue  # Skip deform/mechanism bones, don't count as failure

            if success:
                widgets_assigned += 1
            else:
                widgets_failed += 1

        result["widgets_assigned"] = widgets_assigned
        if widgets_failed > 0:
            print(f"Warning: {widgets_failed} widget assignments failed")

    # Organize bone collections if enabled
    if animator_rig_config.get("collections", True):
        collections_config = animator_rig_config.get("bone_collections", None)
        organize_bone_collections(armature, collections_config)
        result["collections_created"] = True

    # Apply bone colors if enabled
    if animator_rig_config.get("colors", True):
        bone_colors = animator_rig_config.get("bone_colors", {})
        scheme = bone_colors.get("scheme", "standard")

        custom_colors = None
        if scheme == "custom":
            custom_colors = {
                "left": bone_colors.get("left", {}),
                "right": bone_colors.get("right", {}),
                "center": bone_colors.get("center", {}),
            }
        elif scheme == "per_bone":
            custom_colors = bone_colors.get("colors", {})

        apply_bone_colors(armature, scheme, custom_colors)
        result["colors_applied"] = True

    return result


# =============================================================================
# Animation Creation
# =============================================================================

def create_animation(armature: 'bpy.types.Object', params: Dict) -> 'bpy.types.Action':
    """Create an animation from params."""
    clip_name = params.get("clip_name", "animation")
    duration = params.get("duration_seconds", 1.0)
    fps = params.get("fps", 30)
    keyframes = params.get("keyframes", [])
    interpolation = params.get("interpolation", "linear").upper()

    # Set scene FPS
    bpy.context.scene.render.fps = fps

    # Calculate frame range
    frame_count = int(duration * fps)
    bpy.context.scene.frame_start = 1
    bpy.context.scene.frame_end = frame_count

    # Create action
    action = bpy.data.actions.new(name=clip_name)
    armature.animation_data_create()
    armature.animation_data.action = action

    # Map interpolation mode
    interp_map = {
        "LINEAR": "LINEAR",
        "BEZIER": "BEZIER",
        "CONSTANT": "CONSTANT",
    }
    interp_mode = interp_map.get(interpolation, "LINEAR")

    # Apply keyframes
    for kf_spec in keyframes:
        time = kf_spec.get("time", 0)
        frame = 1 + int(time * fps)
        bones_data = kf_spec.get("bones", {})

        for bone_name, transform in bones_data.items():
            pose_bone = armature.pose.bones.get(bone_name)
            if not pose_bone:
                print(f"Warning: Bone {bone_name} not found")
                continue

            # Apply rotation
            if "rotation" in transform:
                rot = transform["rotation"]
                pose_bone.rotation_mode = 'XYZ'
                pose_bone.rotation_euler = Euler((
                    math.radians(rot[0]),
                    math.radians(rot[1]),
                    math.radians(rot[2])
                ))
                pose_bone.keyframe_insert(data_path="rotation_euler", frame=frame)

                # Set interpolation
                for fcurve in action.fcurves:
                    if pose_bone.name in fcurve.data_path and "rotation" in fcurve.data_path:
                        for kp in fcurve.keyframe_points:
                            if kp.co[0] == frame:
                                kp.interpolation = interp_mode

            # Apply position
            if "position" in transform:
                pos = transform["position"]
                pose_bone.location = Vector(pos)
                pose_bone.keyframe_insert(data_path="location", frame=frame)

                # Set interpolation
                for fcurve in action.fcurves:
                    if pose_bone.name in fcurve.data_path and "location" in fcurve.data_path:
                        for kp in fcurve.keyframe_points:
                            if kp.co[0] == frame:
                                kp.interpolation = interp_mode

            # Apply scale
            if "scale" in transform:
                scale = transform["scale"]
                pose_bone.scale = Vector(scale)
                pose_bone.keyframe_insert(data_path="scale", frame=frame)

    return action


# =============================================================================
# Export
# =============================================================================

def export_glb(output_path: Path, include_armature: bool = True,
               include_animation: bool = False, export_tangents: bool = False) -> None:
    """Export scene to GLB format."""
    export_settings = {
        'filepath': str(output_path),
        'export_format': 'GLB',
        'export_apply': True,
        'export_texcoords': True,
        'export_normals': True,
        'export_colors': True,
        'export_tangents': export_tangents,
    }

    if include_animation:
        export_settings['export_animations'] = True
        export_settings['export_current_frame'] = False
    else:
        export_settings['export_animations'] = False

    bpy.ops.export_scene.gltf(**export_settings)


# =============================================================================
# LOD Generation
# =============================================================================

def compute_decimate_ratio_for_target(
    current_tris: int,
    target_tris: int
) -> float:
    """
    Compute the decimate ratio needed to achieve a target triangle count.

    Args:
        current_tris: Current triangle count.
        target_tris: Target triangle count.

    Returns:
        Decimate ratio (0.0 to 1.0).
    """
    if current_tris <= 0 or target_tris <= 0:
        return 1.0
    if target_tris >= current_tris:
        return 1.0
    return target_tris / current_tris


def create_lod_mesh(
    source_obj: 'bpy.types.Object',
    lod_level: int,
    target_tris: Optional[int],
    decimate_method: str = "collapse"
) -> Tuple['bpy.types.Object', Dict[str, Any]]:
    """
    Create an LOD mesh by decimating the source object.

    Args:
        source_obj: The source mesh object (will be duplicated).
        lod_level: LOD level index (0, 1, 2, ...).
        target_tris: Target triangle count (None = keep original).
        decimate_method: 'collapse' or 'planar'.

    Returns:
        Tuple of (lod_mesh_object, metrics_dict).
    """
    # Duplicate the source object
    bpy.ops.object.select_all(action='DESELECT')
    source_obj.select_set(True)
    bpy.context.view_layer.objects.active = source_obj
    bpy.ops.object.duplicate(linked=False)
    lod_obj = bpy.context.active_object
    lod_obj.name = f"{source_obj.name}_LOD{lod_level}"

    # Get current triangle count
    depsgraph = bpy.context.evaluated_depsgraph_get()
    obj_eval = lod_obj.evaluated_get(depsgraph)
    mesh = obj_eval.to_mesh()
    current_tris = sum(
        len(poly.vertices) - 2 if len(poly.vertices) > 3 else 1
        for poly in mesh.polygons
    )
    obj_eval.to_mesh_clear()

    # Apply decimate if target is specified and lower than current
    if target_tris is not None and target_tris < current_tris:
        ratio = compute_decimate_ratio_for_target(current_tris, target_tris)

        # Add and apply decimate modifier
        bpy.context.view_layer.objects.active = lod_obj
        mod = lod_obj.modifiers.new(name="LOD_Decimate", type='DECIMATE')

        if decimate_method == "planar":
            mod.decimate_type = 'DISSOLVE'
            # Planar uses angle limit instead of ratio
            # Estimate angle from target ratio
            mod.angle_limit = math.radians(5.0 + (1.0 - ratio) * 40.0)
        else:
            # Collapse (default)
            mod.decimate_type = 'COLLAPSE'
            mod.ratio = ratio

        bpy.ops.object.modifier_apply(modifier=mod.name)

    # Compute metrics for this LOD
    metrics = compute_mesh_metrics(lod_obj)
    metrics["lod_level"] = lod_level
    if target_tris is not None:
        metrics["target_tris"] = target_tris
        metrics["simplification_ratio"] = round(
            metrics["triangle_count"] / current_tris if current_tris > 0 else 1.0,
            4
        )
    else:
        metrics["simplification_ratio"] = 1.0

    return lod_obj, metrics


def generate_lod_chain(
    source_obj: 'bpy.types.Object',
    lod_chain_spec: Dict
) -> Tuple[List['bpy.types.Object'], List[Dict[str, Any]]]:
    """
    Generate a chain of LOD meshes from the source object.

    Args:
        source_obj: The source mesh object.
        lod_chain_spec: LOD chain specification with 'levels' and 'decimate_method'.

    Returns:
        Tuple of (list_of_lod_objects, list_of_per_lod_metrics).
    """
    levels = lod_chain_spec.get("levels", [])
    decimate_method = lod_chain_spec.get("decimate_method", "collapse")

    # Sort levels by level index
    levels = sorted(levels, key=lambda x: x.get("level", 0))

    lod_objects = []
    lod_metrics = []

    for level_spec in levels:
        lod_level = level_spec.get("level", 0)
        target_tris = level_spec.get("target_tris")

        lod_obj, metrics = create_lod_mesh(
            source_obj,
            lod_level,
            target_tris,
            decimate_method
        )

        lod_objects.append(lod_obj)
        lod_metrics.append(metrics)

    return lod_objects, lod_metrics


# =============================================================================
# Collision Mesh Generation
# =============================================================================

def generate_collision_mesh(
    source_obj: 'bpy.types.Object',
    collision_spec: Dict
) -> Tuple['bpy.types.Object', Dict[str, Any]]:
    """
    Generate a collision mesh from the source object.

    Args:
        source_obj: The source mesh object.
        collision_spec: Collision mesh specification with:
            - collision_type: 'convex_hull', 'simplified_mesh', or 'box'
            - target_faces: Target face count for simplified mesh
            - output_suffix: Suffix for collision mesh name

    Returns:
        Tuple of (collision_mesh_object, collision_metrics).
    """
    collision_type = collision_spec.get("collision_type", "convex_hull")
    target_faces = collision_spec.get("target_faces")
    output_suffix = collision_spec.get("output_suffix", "_col")

    # Create a copy of the source mesh for collision
    bpy.ops.object.select_all(action='DESELECT')
    source_obj.select_set(True)
    bpy.context.view_layer.objects.active = source_obj
    bpy.ops.object.duplicate(linked=False)
    col_obj = bpy.context.active_object
    col_obj.name = f"{source_obj.name}{output_suffix}"

    bpy.context.view_layer.objects.active = col_obj

    if collision_type == "convex_hull":
        # Generate convex hull collision
        _generate_convex_hull(col_obj)
    elif collision_type == "simplified_mesh":
        # Generate simplified mesh collision
        _generate_simplified_collision(col_obj, target_faces)
    elif collision_type == "box":
        # Generate box collision from bounding box
        _generate_box_collision(col_obj)
    else:
        print(f"Warning: Unknown collision type '{collision_type}', defaulting to convex_hull")
        _generate_convex_hull(col_obj)

    # Compute metrics for the collision mesh
    metrics = compute_collision_mesh_metrics(col_obj, collision_type)
    metrics["collision_type"] = collision_type
    metrics["output_suffix"] = output_suffix
    if target_faces is not None:
        metrics["target_faces"] = target_faces

    return col_obj, metrics


def _generate_convex_hull(obj: 'bpy.types.Object') -> None:
    """
    Generate a convex hull from the mesh.

    Args:
        obj: The mesh object to convert to convex hull.
    """
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')

    # Generate convex hull
    bpy.ops.mesh.convex_hull()

    bpy.ops.object.mode_set(mode='OBJECT')


def _generate_simplified_collision(obj: 'bpy.types.Object', target_faces: Optional[int]) -> None:
    """
    Generate a simplified collision mesh using decimation.

    Args:
        obj: The mesh object to simplify.
        target_faces: Target face count (if None, uses default ratio of 0.1).
    """
    bpy.context.view_layer.objects.active = obj

    # Get current face count
    depsgraph = bpy.context.evaluated_depsgraph_get()
    obj_eval = obj.evaluated_get(depsgraph)
    mesh = obj_eval.to_mesh()
    current_faces = len(mesh.polygons)
    obj_eval.to_mesh_clear()

    # Calculate ratio
    if target_faces is not None and current_faces > 0:
        ratio = min(1.0, target_faces / current_faces)
    else:
        ratio = 0.1  # Default: reduce to 10%

    # Add and apply decimate modifier
    mod = obj.modifiers.new(name="Collision_Decimate", type='DECIMATE')
    mod.decimate_type = 'COLLAPSE'
    mod.ratio = ratio

    bpy.ops.object.modifier_apply(modifier=mod.name)


def _generate_box_collision(obj: 'bpy.types.Object') -> None:
    """
    Generate a box collision mesh from the bounding box.

    Args:
        obj: The mesh object to replace with a box collision.
    """
    # Get world-space bounding box
    depsgraph = bpy.context.evaluated_depsgraph_get()
    obj_eval = obj.evaluated_get(depsgraph)
    mesh_eval = obj_eval.to_mesh()

    bbox_min = [float('inf')] * 3
    bbox_max = [float('-inf')] * 3
    for v in mesh_eval.vertices:
        co = obj.matrix_world @ v.co
        for i in range(3):
            bbox_min[i] = min(bbox_min[i], co[i])
            bbox_max[i] = max(bbox_max[i], co[i])

    obj_eval.to_mesh_clear()

    # Calculate box dimensions and center
    dimensions = [bbox_max[i] - bbox_min[i] for i in range(3)]
    center = [(bbox_max[i] + bbox_min[i]) / 2.0 for i in range(3)]

    # Remove all geometry from the object
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.mesh.delete(type='VERT')
    bpy.ops.object.mode_set(mode='OBJECT')

    # Create a new cube at the correct location
    # We'll use bmesh to add a box to the existing mesh
    mesh = obj.data
    bm = bmesh.new()

    # Create box vertices
    half_dims = [d / 2.0 for d in dimensions]
    verts = []
    for x_sign in [-1, 1]:
        for y_sign in [-1, 1]:
            for z_sign in [-1, 1]:
                v = bm.verts.new((
                    center[0] + x_sign * half_dims[0],
                    center[1] + y_sign * half_dims[1],
                    center[2] + z_sign * half_dims[2]
                ))
                verts.append(v)

    # Ensure vertex lookup table is updated
    bm.verts.ensure_lookup_table()

    # Create faces (in local space since we're adding directly to mesh data)
    # Vertex indices: 0=(-,-,-), 1=(-,-,+), 2=(-,+,-), 3=(-,+,+), 4=(+,-,-), 5=(+,-,+), 6=(+,+,-), 7=(+,+,+)
    # Transform to local coordinates
    inv_matrix = obj.matrix_world.inverted()
    for v in bm.verts:
        v.co = inv_matrix @ v.co

    # Define box faces
    face_indices = [
        [0, 1, 3, 2],  # -X face
        [4, 6, 7, 5],  # +X face
        [0, 4, 5, 1],  # -Y face
        [2, 3, 7, 6],  # +Y face
        [0, 2, 6, 4],  # -Z face
        [1, 5, 7, 3],  # +Z face
    ]

    for indices in face_indices:
        try:
            bm.faces.new([bm.verts[i] for i in indices])
        except ValueError:
            # Face might already exist or be degenerate
            pass

    bm.to_mesh(mesh)
    bm.free()

    # Recalculate normals
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.mesh.normals_make_consistent(inside=False)
    bpy.ops.object.mode_set(mode='OBJECT')


def compute_collision_mesh_metrics(obj: 'bpy.types.Object', collision_type: str) -> Dict[str, Any]:
    """
    Compute metrics for a collision mesh.

    Args:
        obj: The collision mesh object.
        collision_type: Type of collision mesh ('convex_hull', 'simplified_mesh', 'box').

    Returns:
        Dictionary of collision mesh metrics.
    """
    depsgraph = bpy.context.evaluated_depsgraph_get()
    obj_eval = obj.evaluated_get(depsgraph)
    mesh = obj_eval.to_mesh()

    vertex_count = len(mesh.vertices)
    face_count = len(mesh.polygons)

    # Count triangles
    triangle_count = 0
    for poly in mesh.polygons:
        vert_count = len(poly.vertices)
        if vert_count == 3:
            triangle_count += 1
        elif vert_count == 4:
            triangle_count += 2
        else:
            triangle_count += vert_count - 2

    # Bounding box
    bbox_min = [float('inf')] * 3
    bbox_max = [float('-inf')] * 3
    for v in mesh.vertices:
        co = obj.matrix_world @ v.co
        for i in range(3):
            bbox_min[i] = min(bbox_min[i], co[i])
            bbox_max[i] = max(bbox_max[i], co[i])

    obj_eval.to_mesh_clear()

    return {
        "vertex_count": vertex_count,
        "face_count": face_count,
        "triangle_count": triangle_count,
        "bounding_box": {
            "min": bbox_min,
            "max": bbox_max
        }
    }


def export_collision_mesh(
    collision_obj: 'bpy.types.Object',
    output_path: Path,
    export_tangents: bool = False
) -> None:
    """
    Export a collision mesh to a GLB file.

    Args:
        collision_obj: The collision mesh object.
        output_path: Output GLB file path.
        export_tangents: Whether to export tangents.
    """
    bpy.ops.object.select_all(action='DESELECT')
    collision_obj.select_set(True)
    bpy.context.view_layer.objects.active = collision_obj

    export_settings = {
        'filepath': str(output_path),
        'export_format': 'GLB',
        'export_apply': True,
        'export_texcoords': False,  # Collision meshes don't need UVs
        'export_normals': True,
        'export_colors': False,
        'export_tangents': export_tangents,
        'export_animations': False,
        'use_selection': True,
    }

    bpy.ops.export_scene.gltf(**export_settings)


# =============================================================================
# Navmesh Analysis
# =============================================================================

def analyze_navmesh(
    obj: 'bpy.types.Object',
    navmesh_spec: Dict
) -> Dict[str, Any]:
    """
    Analyze mesh geometry for walkability and produce navmesh metadata.

    This function classifies faces as walkable or non-walkable based on their
    slope (angle from vertical), and optionally detects potential stair geometry.

    Note: This produces classification metadata only, not actual navmesh generation.

    Args:
        obj: The mesh object to analyze.
        navmesh_spec: Navmesh settings with:
            - walkable_slope_max: Maximum slope angle in degrees for walkable surfaces
            - stair_detection: Whether to detect stairs
            - stair_step_height: Step height threshold for stair detection

    Returns:
        Dictionary of navmesh metrics:
            - walkable_face_count: Number of walkable faces
            - non_walkable_face_count: Number of non-walkable faces
            - walkable_percentage: Percentage of faces that are walkable
            - stair_candidates: Number of potential stair surfaces (if stair_detection enabled)
    """
    walkable_slope_max = navmesh_spec.get("walkable_slope_max", 45.0)
    stair_detection = navmesh_spec.get("stair_detection", False)
    stair_step_height = navmesh_spec.get("stair_step_height", 0.3)

    # Convert slope threshold to radians and compute the minimum Z component
    # of the normal for a face to be considered walkable
    # A face is walkable if its normal is within walkable_slope_max degrees of vertical (Z-up)
    slope_rad = math.radians(walkable_slope_max)
    min_normal_z = math.cos(slope_rad)

    # Get evaluated mesh data
    depsgraph = bpy.context.evaluated_depsgraph_get()
    obj_eval = obj.evaluated_get(depsgraph)
    mesh = obj_eval.to_mesh()

    walkable_count = 0
    non_walkable_count = 0
    walkable_heights = []  # Z positions of walkable surface centers (for stair detection)

    for poly in mesh.polygons:
        # Get face normal in world space
        # The polygon normal is in local space, so transform it
        normal_local = poly.normal
        # For world space, we need to transform by the object's rotation matrix (not translation)
        normal_world = obj.matrix_world.to_3x3() @ normal_local
        normal_world.normalize()

        # Check if face is walkable based on slope
        # A face is walkable if its normal Z component >= min_normal_z (pointing mostly up)
        if normal_world.z >= min_normal_z:
            walkable_count += 1
            if stair_detection:
                # Record the center Z position of this face for stair analysis
                center = poly.center
                world_center = obj.matrix_world @ center
                walkable_heights.append(world_center.z)
        else:
            non_walkable_count += 1

    obj_eval.to_mesh_clear()

    total_faces = walkable_count + non_walkable_count
    walkable_percentage = (walkable_count / total_faces * 100.0) if total_faces > 0 else 0.0

    metrics = {
        "walkable_face_count": walkable_count,
        "non_walkable_face_count": non_walkable_count,
        "walkable_percentage": round(walkable_percentage, 2),
    }

    # Stair detection: look for clusters of walkable surfaces at regular height intervals
    if stair_detection and walkable_heights:
        stair_candidates = _detect_stair_candidates(walkable_heights, stair_step_height)
        metrics["stair_candidates"] = stair_candidates

    return metrics


def _detect_stair_candidates(
    walkable_heights: List[float],
    step_height: float
) -> int:
    """
    Detect potential stair geometry by analyzing walkable surface heights.

    Stairs are identified as sequences of walkable surfaces with height
    differences approximately equal to the step height threshold.

    Args:
        walkable_heights: List of Z positions of walkable surface centers.
        step_height: Expected height difference between stair steps.

    Returns:
        Number of detected stair candidate surfaces.
    """
    if not walkable_heights or step_height <= 0:
        return 0

    # Sort heights and cluster into potential steps
    sorted_heights = sorted(walkable_heights)

    # Tolerance for step height matching (allow some variation)
    tolerance = step_height * 0.3

    stair_candidates = 0
    height_clusters = []
    current_cluster = [sorted_heights[0]]

    # Group heights into clusters (faces at similar heights)
    for h in sorted_heights[1:]:
        if abs(h - current_cluster[-1]) < tolerance * 0.5:
            # Same level (within 15% of step height)
            current_cluster.append(h)
        else:
            # New cluster
            if current_cluster:
                avg_height = sum(current_cluster) / len(current_cluster)
                height_clusters.append((avg_height, len(current_cluster)))
            current_cluster = [h]

    if current_cluster:
        avg_height = sum(current_cluster) / len(current_cluster)
        height_clusters.append((avg_height, len(current_cluster)))

    # Look for sequences of clusters with step_height spacing
    if len(height_clusters) >= 2:
        for i in range(len(height_clusters) - 1):
            height_diff = height_clusters[i + 1][0] - height_clusters[i][0]
            if abs(height_diff - step_height) < tolerance:
                # This looks like a stair step
                stair_candidates += height_clusters[i][1]
        # Include the top step if the previous one matched
        if stair_candidates > 0:
            stair_candidates += height_clusters[-1][1]

    return stair_candidates


def bake_textures(
    obj: 'bpy.types.Object',
    baking_spec: Dict,
    out_root: Path,
    base_name: str
) -> Dict[str, Any]:
    """
    Bake texture maps (normal, AO, curvature) from a mesh.

    Uses Cycles CPU renderer for deterministic output.

    Args:
        obj: The target mesh object (low-poly) to bake onto.
        baking_spec: Baking settings with:
            - bake_types: List of map types to bake (normal, ao, curvature, combined)
            - ray_distance: Ray casting distance for high-to-low baking
            - margin: Dilation in pixels for mip-safe edges
            - resolution: [width, height] of baked textures
            - high_poly_source: Optional path to high-poly mesh
        out_root: Output directory for baked textures.
        base_name: Base filename for outputs (e.g., asset_id).

    Returns:
        Dictionary of baking metrics:
            - baked_maps: List of baked map info (type, path, resolution)
            - ray_distance: Ray distance used
            - margin: Margin used
    """
    bake_types = baking_spec.get("bake_types", ["normal"])
    ray_distance = baking_spec.get("ray_distance", 0.1)
    margin = baking_spec.get("margin", 16)
    resolution = baking_spec.get("resolution", [1024, 1024])
    high_poly_source = baking_spec.get("high_poly_source")

    # Switch to Cycles for baking (CPU only for determinism)
    bpy.context.scene.render.engine = 'CYCLES'
    bpy.context.scene.cycles.device = 'CPU'
    bpy.context.scene.cycles.samples = 128  # Reasonable quality for baking

    # Ensure the mesh has UVs
    if not obj.data.uv_layers:
        # Auto-unwrap if no UVs exist
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.mode_set(mode='EDIT')
        bpy.ops.mesh.select_all(action='SELECT')
        bpy.ops.uv.smart_project(angle_limit=66.0)
        bpy.ops.object.mode_set(mode='OBJECT')

    # Load high-poly source if specified
    high_poly_obj = None
    if high_poly_source:
        # Import the high-poly mesh
        high_poly_path = out_root / high_poly_source
        if high_poly_path.exists():
            ext = high_poly_path.suffix.lower()
            if ext in ['.glb', '.gltf']:
                bpy.ops.import_scene.gltf(filepath=str(high_poly_path))
            elif ext == '.obj':
                bpy.ops.import_scene.obj(filepath=str(high_poly_path))
            elif ext == '.fbx':
                bpy.ops.import_scene.fbx(filepath=str(high_poly_path))

            # Find the imported high-poly object
            for imported_obj in bpy.context.selected_objects:
                if imported_obj.type == 'MESH' and imported_obj != obj:
                    high_poly_obj = imported_obj
                    break

    baked_maps = []

    # Create or get the bake material
    mat = _get_or_create_bake_material(obj)

    for bake_type in bake_types:
        # Create image for baking
        img_name = f"{base_name}_{bake_type}"
        img_width, img_height = resolution

        # Delete existing image if it exists
        if img_name in bpy.data.images:
            bpy.data.images.remove(bpy.data.images[img_name])

        # Create new image
        if bake_type in ["normal"]:
            # Normal maps need specific color space
            img = bpy.data.images.new(img_name, width=img_width, height=img_height, float_buffer=True)
            img.colorspace_settings.name = 'Non-Color'
        else:
            img = bpy.data.images.new(img_name, width=img_width, height=img_height)
            if bake_type != "combined":
                img.colorspace_settings.name = 'Non-Color'

        # Set up the image node in the material for baking target
        _setup_bake_target_node(mat, img)

        # Configure bake settings
        bpy.context.scene.render.bake.use_pass_direct = False
        bpy.context.scene.render.bake.use_pass_indirect = False
        bpy.context.scene.render.bake.use_pass_color = True
        bpy.context.scene.render.bake.margin = margin
        bpy.context.scene.render.bake.margin_type = 'EXTEND'

        # Select objects for baking
        bpy.ops.object.select_all(action='DESELECT')
        obj.select_set(True)
        bpy.context.view_layer.objects.active = obj

        # Set up for selected-to-active baking if high-poly source exists
        use_selected_to_active = high_poly_obj is not None

        if use_selected_to_active:
            high_poly_obj.select_set(True)
            bpy.context.scene.render.bake.use_selected_to_active = True
            bpy.context.scene.render.bake.cage_extrusion = ray_distance
        else:
            bpy.context.scene.render.bake.use_selected_to_active = False

        # Perform the bake based on type
        if bake_type == "normal":
            bpy.ops.object.bake(type='NORMAL')
        elif bake_type == "ao":
            bpy.ops.object.bake(type='AO')
        elif bake_type == "curvature":
            # Curvature is not a native Blender bake type
            # We emulate it using pointiness from geometry node or bake from a material
            _bake_curvature(obj, img, margin)
        elif bake_type == "combined":
            bpy.ops.object.bake(type='COMBINED')

        # Save the baked image
        output_filename = f"{base_name}_{bake_type}.png"
        output_path = out_root / output_filename
        output_path.parent.mkdir(parents=True, exist_ok=True)

        img.filepath_raw = str(output_path)
        img.file_format = 'PNG'
        img.save()

        baked_maps.append({
            "type": bake_type,
            "path": output_filename,
            "resolution": [img_width, img_height],
        })

    # Clean up high-poly object if we imported it
    if high_poly_obj:
        bpy.data.objects.remove(high_poly_obj, do_unlink=True)

    return {
        "baked_maps": baked_maps,
        "ray_distance": ray_distance,
        "margin": margin,
    }


def _get_or_create_bake_material(obj: 'bpy.types.Object') -> 'bpy.types.Material':
    """Get or create a material for baking on the object."""
    if obj.data.materials:
        mat = obj.data.materials[0]
    else:
        mat = bpy.data.materials.new(name="BakeMaterial")
        mat.use_nodes = True
        obj.data.materials.append(mat)
    return mat


def _setup_bake_target_node(mat: 'bpy.types.Material', img: 'bpy.types.Image') -> None:
    """Set up an image texture node as the bake target."""
    if not mat.use_nodes:
        mat.use_nodes = True

    nodes = mat.node_tree.nodes

    # Find or create image texture node
    img_node = None
    for node in nodes:
        if node.type == 'TEX_IMAGE' and node.name == 'BakeTarget':
            img_node = node
            break

    if not img_node:
        img_node = nodes.new(type='ShaderNodeTexImage')
        img_node.name = 'BakeTarget'

    img_node.image = img
    img_node.select = True
    nodes.active = img_node


def _bake_curvature(obj: 'bpy.types.Object', img: 'bpy.types.Image', margin: int) -> None:
    """
    Bake curvature map using geometry pointiness.

    Since Blender doesn't have a native curvature bake type, we use
    a shader that outputs the Geometry > Pointiness value.
    """
    mat = obj.data.materials[0] if obj.data.materials else None
    if not mat:
        return

    nodes = mat.node_tree.nodes
    links = mat.node_tree.links

    # Store original output link
    output_node = None
    original_surface_input = None
    for node in nodes:
        if node.type == 'OUTPUT_MATERIAL':
            output_node = node
            if node.inputs['Surface'].links:
                original_surface_input = node.inputs['Surface'].links[0].from_socket
            break

    # Create geometry node for pointiness
    geom_node = nodes.new(type='ShaderNodeNewGeometry')
    geom_node.name = 'CurvatureGeom'

    # Create color ramp to control curvature contrast
    ramp_node = nodes.new(type='ShaderNodeValToRGB')
    ramp_node.name = 'CurvatureRamp'
    # Set up ramp: 0.0 (concave) -> black, 0.5 (flat) -> gray, 1.0 (convex) -> white
    ramp_node.color_ramp.elements[0].position = 0.4
    ramp_node.color_ramp.elements[0].color = (0, 0, 0, 1)
    ramp_node.color_ramp.elements[1].position = 0.6
    ramp_node.color_ramp.elements[1].color = (1, 1, 1, 1)

    # Create emission shader to output the curvature value
    emit_node = nodes.new(type='ShaderNodeEmission')
    emit_node.name = 'CurvatureEmit'

    # Connect nodes
    links.new(geom_node.outputs['Pointiness'], ramp_node.inputs['Fac'])
    links.new(ramp_node.outputs['Color'], emit_node.inputs['Color'])
    links.new(emit_node.outputs['Emission'], output_node.inputs['Surface'])

    # Configure bake settings for emit
    bpy.context.scene.render.bake.margin = margin

    # Bake emit (which now outputs our curvature)
    bpy.ops.object.bake(type='EMIT')

    # Restore original material connection
    if original_surface_input:
        links.new(original_surface_input, output_node.inputs['Surface'])

    # Clean up temporary nodes
    nodes.remove(geom_node)
    nodes.remove(ramp_node)
    nodes.remove(emit_node)


def export_glb_with_lods(
    output_path: Path,
    lod_objects: List['bpy.types.Object'],
    export_tangents: bool = False
) -> None:
    """
    Export LOD meshes to a GLB file.
    Each LOD is exported as a separate mesh named "Mesh_LOD0", "Mesh_LOD1", etc.

    Args:
        output_path: Output GLB file path.
        lod_objects: List of LOD mesh objects in order (LOD0, LOD1, ...).
        export_tangents: Whether to export tangents.
    """
    # Ensure only LOD objects are selected
    bpy.ops.object.select_all(action='DESELECT')
    for lod_obj in lod_objects:
        lod_obj.select_set(True)

    if lod_objects:
        bpy.context.view_layer.objects.active = lod_objects[0]

    export_settings = {
        'filepath': str(output_path),
        'export_format': 'GLB',
        'export_apply': True,
        'export_texcoords': True,
        'export_normals': True,
        'export_colors': True,
        'export_tangents': export_tangents,
        'export_animations': False,
        'use_selection': True,
    }

    bpy.ops.export_scene.gltf(**export_settings)


# =============================================================================
# Mode Handlers
# =============================================================================

def handle_static_mesh(spec: Dict, out_root: Path, report_path: Path) -> None:
    """Handle static mesh generation."""
    start_time = time.time()

    try:
        recipe = spec.get("recipe", {})
        params = recipe.get("params", {})

        # Create primitive
        primitive = params.get("base_primitive", "cube")
        dimensions = params.get("dimensions", [1, 1, 1])
        obj = create_primitive(primitive, dimensions)

        # Apply modifiers
        modifiers = params.get("modifiers", [])
        for mod_spec in modifiers:
            apply_modifier(obj, mod_spec)

        # Apply modifiers to mesh
        export_settings = params.get("export", {})
        if export_settings.get("apply_modifiers", True):
            apply_all_modifiers(obj)

        # Triangulate if requested
        if export_settings.get("triangulate", True):
            mod = obj.modifiers.new(name="Triangulate", type='TRIANGULATE')
            bpy.ops.object.modifier_apply(modifier=mod.name)

        # Apply UV projection
        uv_projection = params.get("uv_projection")
        if uv_projection:
            apply_uv_projection(obj, uv_projection)

        # Apply normals settings
        normals = params.get("normals")
        if normals:
            apply_normals_settings(obj, normals)

        # Apply materials
        material_slots = params.get("material_slots", [])
        apply_materials(obj, material_slots)

        # Get output path from spec
        outputs = spec.get("outputs", [])
        primary_output = next((o for o in outputs if o.get("kind") == "primary"), None)
        if not primary_output:
            raise ValueError("No primary output specified in spec")

        output_rel_path = primary_output.get("path", "output.glb")
        output_path = out_root / output_rel_path
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Export with tangents if requested
        export_tangents = export_settings.get("tangents", False)

        # Check for LOD chain, collision mesh, navmesh analysis, and baking
        lod_chain_spec = params.get("lod_chain")
        collision_mesh_spec = params.get("collision_mesh")
        navmesh_spec = params.get("navmesh")
        baking_spec = params.get("baking")

        # Analyze navmesh first (before collision mesh or LOD chain modifies the object)
        navmesh_metrics = None
        if navmesh_spec:
            navmesh_metrics = analyze_navmesh(obj, navmesh_spec)

        # Bake textures (before LOD chain modifies the object)
        baking_metrics = None
        if baking_spec:
            asset_id = spec.get("asset_id", "mesh")
            baking_metrics = bake_textures(obj, baking_spec, output_path.parent, asset_id)

        # Generate collision mesh first (before LOD chain modifies the object)
        collision_obj = None
        collision_metrics = None
        collision_output_path = None
        if collision_mesh_spec:
            collision_obj, collision_metrics = generate_collision_mesh(obj, collision_mesh_spec)
            # Determine collision mesh output path
            output_suffix = collision_mesh_spec.get("output_suffix", "_col")
            collision_filename = output_path.stem + output_suffix + output_path.suffix
            collision_output_path = output_path.parent / collision_filename

        if lod_chain_spec:
            # Generate LOD chain
            lod_objects, lod_metrics = generate_lod_chain(obj, lod_chain_spec)

            # Delete the original object (LOD objects are copies)
            bpy.data.objects.remove(obj, do_unlink=True)

            # Export all LODs to GLB
            export_glb_with_lods(output_path, lod_objects, export_tangents=export_tangents)

            # Build combined metrics report
            metrics = {
                "lod_count": len(lod_objects),
                "lod_levels": lod_metrics,
            }

            # Add summary from LOD0 (original)
            if lod_metrics:
                lod0 = lod_metrics[0]
                metrics["vertex_count"] = lod0.get("vertex_count", 0)
                metrics["face_count"] = lod0.get("face_count", 0)
                metrics["triangle_count"] = lod0.get("triangle_count", 0)
                metrics["bounding_box"] = lod0.get("bounding_box", {})
                metrics["bounds_min"] = lod0.get("bounds_min", [0, 0, 0])
                metrics["bounds_max"] = lod0.get("bounds_max", [0, 0, 0])
                metrics["material_slot_count"] = lod0.get("material_slot_count", 0)
        else:
            # No LOD chain - export single mesh
            metrics = compute_mesh_metrics(obj)
            export_glb(output_path, export_tangents=export_tangents)

        # Export collision mesh if generated
        if collision_obj and collision_output_path:
            export_collision_mesh(collision_obj, collision_output_path, export_tangents=False)
            # Add collision mesh metrics to report
            metrics["collision_mesh"] = collision_metrics
            metrics["collision_mesh_path"] = str(collision_output_path.name)
            # Clean up collision object
            bpy.data.objects.remove(collision_obj, do_unlink=True)

        # Add navmesh metrics if analyzed
        if navmesh_metrics:
            metrics["navmesh"] = navmesh_metrics

        # Add baking metrics if baking was performed
        if baking_metrics:
            metrics["baking"] = baking_metrics

        duration_ms = int((time.time() - start_time) * 1000)
        write_report(report_path, ok=True, metrics=metrics,
                     output_path=output_rel_path, duration_ms=duration_ms)

    except Exception as e:
        write_report(report_path, ok=False, error=str(e))
        raise


def handle_skeletal_mesh(spec: Dict, out_root: Path, report_path: Path) -> None:
    """Handle skeletal mesh generation."""
    start_time = time.time()

    try:
        recipe = spec.get("recipe", {})
        params = recipe.get("params", {})

        # Create armature - support both preset and custom skeleton
        skeleton_spec = params.get("skeleton", [])
        skeleton_preset = params.get("skeleton_preset")

        if skeleton_spec:
            # Use custom skeleton definition
            armature = create_custom_skeleton(skeleton_spec)
        elif skeleton_preset:
            # Use preset skeleton
            armature = create_armature(skeleton_preset)
        else:
            # Default to humanoid basic
            armature = create_armature("humanoid_basic_v1")

        mesh_objects = []

        # Create body parts (new format)
        body_parts = params.get("body_parts", [])
        for part_spec in body_parts:
            mesh_obj = create_body_part(armature, part_spec)
            mesh_objects.append((mesh_obj, part_spec.get("bone")))

        # Create legacy parts (ai-studio-core format)
        legacy_parts = params.get("parts", {})
        if legacy_parts:
            for part_name, part_spec in legacy_parts.items():
                mesh_obj = create_legacy_part(armature, part_name, part_spec, legacy_parts)
                if mesh_obj:
                    bone_name = part_spec.get("bone", part_name)
                    mesh_objects.append((mesh_obj, bone_name))

                    # Handle instances for this part
                    instances = part_spec.get("instances", [])
                    if instances:
                        instance_objects = create_part_instances(mesh_obj, instances)
                        for inst_obj in instance_objects:
                            mesh_objects.append((inst_obj, bone_name))

        # Join all meshes into one
        if mesh_objects:
            bpy.ops.object.select_all(action='DESELECT')
            for mesh_obj, _ in mesh_objects:
                mesh_obj.select_set(True)
            bpy.context.view_layer.objects.active = mesh_objects[0][0]
            if len(mesh_objects) > 1:
                bpy.ops.object.join()
            combined_mesh = bpy.context.active_object
            combined_mesh.name = "Character"

            # Assign vertex groups for each original body part
            # (Simplified - in real implementation, we'd track vertex indices)
            for _, bone_name in mesh_objects:
                if bone_name not in combined_mesh.vertex_groups:
                    combined_mesh.vertex_groups.new(name=bone_name)

            # Apply materials
            material_slots = params.get("material_slots", [])
            apply_materials(combined_mesh, material_slots)

            # Apply texturing/UV settings
            texturing = params.get("texturing")
            if texturing:
                apply_texturing(combined_mesh, texturing)

            # Skin mesh to armature
            skinning = params.get("skinning", {})
            auto_weights = skinning.get("auto_weights", True)
            skin_mesh_to_armature(combined_mesh, armature, auto_weights)

            # Get output path
            outputs = spec.get("outputs", [])
            primary_output = next((o for o in outputs if o.get("kind") == "primary"), None)
            if not primary_output:
                raise ValueError("No primary output specified in spec")

            output_rel_path = primary_output.get("path", "output.glb")
            output_path = out_root / output_rel_path
            output_path.parent.mkdir(parents=True, exist_ok=True)

            # Compute metrics
            metrics = compute_skeletal_mesh_metrics(combined_mesh, armature)

            # Add tri_budget validation to metrics
            tri_budget = params.get("tri_budget")
            if tri_budget:
                metrics["tri_budget"] = tri_budget
                metrics["tri_budget_exceeded"] = metrics.get("triangle_count", 0) > tri_budget

            # Export GLB with tangents if requested
            export_settings = params.get("export", {})
            export_tangents = export_settings.get("tangents", False)
            export_glb(output_path, include_armature=True, export_tangents=export_tangents)

            # Save .blend file if requested
            blend_rel_path = None
            export_settings = params.get("export", {})
            if export_settings.get("save_blend", False):
                blend_rel_path = output_rel_path.replace(".glb", ".blend")
                blend_path = out_root / blend_rel_path
                bpy.ops.wm.save_as_mainfile(filepath=str(blend_path))

            duration_ms = int((time.time() - start_time) * 1000)
            write_report(report_path, ok=True, metrics=metrics,
                         output_path=output_rel_path, blend_path=blend_rel_path,
                         duration_ms=duration_ms)
        else:
            raise ValueError("No body parts or legacy parts specified")

    except Exception as e:
        write_report(report_path, ok=False, error=str(e))
        raise


def handle_animation(spec: Dict, out_root: Path, report_path: Path) -> None:
    """Handle animation generation (simple keyframes, no IK)."""
    start_time = time.time()

    try:
        recipe = spec.get("recipe", {})
        params = recipe.get("params", {})

        # Create armature
        skeleton_preset = params.get("skeleton_preset", "humanoid_basic_v1")
        armature = create_armature(skeleton_preset)

        # Create animation
        action = create_animation(armature, params)

        # Get output path
        outputs = spec.get("outputs", [])
        primary_output = next((o for o in outputs if o.get("kind") == "primary"), None)
        if not primary_output:
            raise ValueError("No primary output specified in spec")

        output_rel_path = primary_output.get("path", "output.glb")
        output_path = out_root / output_rel_path
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Compute metrics
        metrics = compute_animation_metrics(armature, action)

        # Export GLB with animation and tangents if requested
        export_settings = params.get("export", {})
        export_tangents = export_settings.get("tangents", False)
        export_glb(output_path, include_armature=True, include_animation=True, export_tangents=export_tangents)

        # Save .blend file if requested
        blend_rel_path = None
        export_settings = params.get("export", {})
        if export_settings.get("save_blend", False):
            blend_rel_path = output_rel_path.replace(".glb", ".blend")
            blend_path = out_root / blend_rel_path
            bpy.ops.wm.save_as_mainfile(filepath=str(blend_path))

        duration_ms = int((time.time() - start_time) * 1000)
        write_report(report_path, ok=True, metrics=metrics,
                     output_path=output_rel_path, blend_path=blend_rel_path,
                     duration_ms=duration_ms)

    except Exception as e:
        write_report(report_path, ok=False, error=str(e))
        raise


def handle_rigged_animation(spec: Dict, out_root: Path, report_path: Path) -> None:
    """Handle rigged animation generation (with IK support)."""
    start_time = time.time()

    try:
        recipe = spec.get("recipe", {})
        params = recipe.get("params", {})

        # Determine armature source: input_armature, character, or skeleton_preset
        input_armature_path = params.get("input_armature")
        character_ref = params.get("character")
        skeleton_preset = params.get("skeleton_preset", "humanoid_basic_v1")

        if input_armature_path:
            # Import existing armature from file
            armature_path = out_root / input_armature_path
            if armature_path.suffix.lower() in ('.glb', '.gltf'):
                bpy.ops.import_scene.gltf(filepath=str(armature_path))
            elif armature_path.suffix.lower() == '.blend':
                # Append from blend file
                with bpy.data.libraries.load(str(armature_path)) as (data_from, data_to):
                    data_to.objects = [name for name in data_from.objects if 'Armature' in name or 'armature' in name]
            # Find the imported armature
            armature = next((obj for obj in bpy.context.selected_objects if obj.type == 'ARMATURE'), None)
            if not armature:
                armature = next((obj for obj in bpy.data.objects if obj.type == 'ARMATURE'), None)
            if not armature:
                raise ValueError(f"No armature found in {input_armature_path}")
            print(f"Imported armature from: {input_armature_path}")
        elif character_ref:
            # Character reference - for now treat as preset, extend later for spec references
            armature = create_armature(character_ref)
            print(f"Created armature from character reference: {character_ref}")
        else:
            # Use skeleton preset
            armature = create_armature(skeleton_preset)

        # Apply ground offset if specified
        ground_offset = params.get("ground_offset", 0.0)
        if ground_offset != 0.0:
            armature.location.z += ground_offset
            print(f"Applied ground offset: {ground_offset}")

        # Apply rig setup (IK chains, presets, foot_systems, aim_constraints, etc.)
        rig_setup = params.get("rig_setup", {})
        if rig_setup:
            ik_controls = apply_rig_setup(armature, rig_setup)
            print(f"Created IK controls: {list(ik_controls.keys())}")

        # Apply animator rig configuration (widgets, collections, colors)
        animator_rig = params.get("animator_rig")
        if animator_rig:
            rig_result = apply_animator_rig_config(armature, animator_rig)
            print(f"Animator rig config applied: {rig_result}")

        # Calculate frame count from duration_frames or duration_seconds
        fps = params.get("fps", 30)
        duration_frames = params.get("duration_frames")
        duration_seconds = params.get("duration_seconds")

        if duration_frames:
            frame_count = duration_frames
        elif duration_seconds:
            frame_count = int(duration_seconds * fps)
        else:
            frame_count = 30  # Default to 1 second at 30fps

        # Set frame range
        bpy.context.scene.frame_start = 1
        bpy.context.scene.frame_end = frame_count
        bpy.context.scene.render.fps = fps

        # Create FK animation from keyframes
        if params.get("keyframes"):
            action = create_animation(armature, params)
        else:
            # Create empty action for IK-only animation
            clip_name = params.get("clip_name", "animation")
            action = bpy.data.actions.new(name=clip_name)
            armature.animation_data_create()
            armature.animation_data.action = action

        # Apply poses and phases
        poses = params.get("poses", {})
        phases = params.get("phases", [])
        if phases:
            apply_poses_and_phases(armature, poses, phases, fps)

        # Apply procedural animation layers
        procedural_layers = params.get("procedural_layers", [])
        if procedural_layers:
            apply_procedural_layers(armature, procedural_layers, fps, frame_count)

        # Apply IK keyframes
        ik_keyframes = params.get("ik_keyframes", [])
        for ik_kf in ik_keyframes:
            time_sec = ik_kf.get("time", 0)
            frame = 1 + int(time_sec * fps)
            targets = ik_kf.get("targets", {})

            for target_name, transform in targets.items():
                # Find the IK target object
                target_obj = bpy.data.objects.get(target_name)
                if not target_obj:
                    print(f"Warning: IK target '{target_name}' not found")
                    continue

                # Apply position
                if "position" in transform:
                    target_obj.location = Vector(transform["position"])
                    target_obj.keyframe_insert(data_path="location", frame=frame)

                # Apply rotation
                if "rotation" in transform:
                    rot = transform["rotation"]
                    target_obj.rotation_euler = Euler((
                        math.radians(rot[0]),
                        math.radians(rot[1]),
                        math.radians(rot[2])
                    ))
                    target_obj.keyframe_insert(data_path="rotation_euler", frame=frame)

        # Get output path
        outputs = spec.get("outputs", [])
        primary_output = next((o for o in outputs if o.get("kind") == "primary"), None)
        if not primary_output:
            raise ValueError("No primary output specified in spec")

        output_rel_path = primary_output.get("path", "output.glb")
        output_path = out_root / output_rel_path
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Apply bake settings from rig_setup or export settings
        bake_settings = rig_setup.get("bake")
        export_settings = params.get("export", {})

        if bake_settings:
            # Use explicit bake settings
            bake_animation(armature, bake_settings, 1, frame_count)
        elif export_settings.get("bake_transforms", True):
            # Legacy bake behavior
            bpy.context.view_layer.objects.active = armature
            bpy.ops.object.mode_set(mode='POSE')
            bpy.ops.pose.select_all(action='SELECT')
            bpy.ops.nla.bake(
                frame_start=1,
                frame_end=frame_count,
                only_selected=False,
                visual_keying=True,
                clear_constraints=False,
                use_current_action=True,
                bake_types={'POSE'}
            )
            bpy.ops.object.mode_set(mode='OBJECT')

        # Compute metrics
        metrics = compute_animation_metrics(armature, action)

        # Add IK-specific metrics
        metrics["ik_chain_count"] = len(rig_setup.get("presets", [])) + len(rig_setup.get("ik_chains", []))
        metrics["ik_keyframe_count"] = len(ik_keyframes)
        metrics["procedural_layer_count"] = len(procedural_layers)
        metrics["phase_count"] = len(phases)
        metrics["pose_count"] = len(poses)

        # Export GLB with animation and tangents if requested
        export_tangents = export_settings.get("tangents", False)
        export_glb(output_path, include_armature=True, include_animation=True, export_tangents=export_tangents)

        # Save .blend file if requested (from params or export settings)
        blend_rel_path = None
        save_blend = params.get("save_blend", False) or export_settings.get("save_blend", False)
        if save_blend:
            blend_rel_path = output_rel_path.replace(".glb", ".blend")
            blend_path = out_root / blend_rel_path
            bpy.ops.wm.save_as_mainfile(filepath=str(blend_path))

        duration_ms = int((time.time() - start_time) * 1000)
        write_report(report_path, ok=True, metrics=metrics,
                     output_path=output_rel_path, blend_path=blend_rel_path,
                     duration_ms=duration_ms)

    except Exception as e:
        write_report(report_path, ok=False, error=str(e))
        raise


# =============================================================================
# Main Entry Point
# =============================================================================

def main() -> int:
    """Main entry point."""
    # Parse arguments after '--'
    try:
        argv = sys.argv[sys.argv.index("--") + 1:]
    except ValueError:
        argv = []

    parser = argparse.ArgumentParser(description="SpecCade Blender Entrypoint")
    parser.add_argument("--mode", required=True,
                        choices=["static_mesh", "skeletal_mesh", "animation", "rigged_animation"],
                        help="Generation mode")
    parser.add_argument("--spec", required=True, type=Path,
                        help="Path to spec JSON file")
    parser.add_argument("--out-root", required=True, type=Path,
                        help="Output root directory")
    parser.add_argument("--report", required=True, type=Path,
                        help="Path for report JSON output")

    args = parser.parse_args(argv)

    # Ensure Blender is available
    if not BLENDER_AVAILABLE:
        write_report(args.report, ok=False,
                     error="This script must be run inside Blender")
        return 1

    # Load spec
    try:
        with open(args.spec, 'r') as f:
            spec = json.load(f)
    except Exception as e:
        write_report(args.report, ok=False, error=f"Failed to load spec: {e}")
        return 1

    # Clear and setup scene
    clear_scene()
    setup_scene()

    # Dispatch to handler
    handlers = {
        "static_mesh": handle_static_mesh,
        "skeletal_mesh": handle_skeletal_mesh,
        "animation": handle_animation,
        "rigged_animation": handle_rigged_animation,
    }

    handler = handlers.get(args.mode)
    if not handler:
        write_report(args.report, ok=False, error=f"Unknown mode: {args.mode}")
        return 1

    try:
        handler(spec, args.out_root, args.report)
        return 0
    except Exception as e:
        # Report already written in handler
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
