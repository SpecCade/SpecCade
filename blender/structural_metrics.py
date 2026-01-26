#!/usr/bin/env python3
"""
Structural Metrics for SpecCade Mesh Generation.

This module computes non-opinionated structural metrics for generated meshes,
enabling LLMs to self-correct 3D output by comparing geometric properties
against stated intent.

Metrics include:
- Geometry: extent, aspect ratios, centroid, convex hull ratio
- Symmetry: axis reflection scores
- Component: per-part breakdown and adjacency
- Skeletal: bone coverage and symmetry
- Scale: dimension checks against reference sizes

Usage:
    from structural_metrics import compute_all_structural_metrics
    metrics = compute_all_structural_metrics(obj, armature=armature, components=objects)
"""

import math
from typing import Any, Dict, List, Optional, Tuple

# Blender modules - only available when running inside Blender
try:
    import bpy
    import bmesh
    from mathutils import Matrix, Vector
    from mathutils.bvhtree import BVHTree
    BLENDER_AVAILABLE = True
except ImportError:
    BLENDER_AVAILABLE = False


# =============================================================================
# Geometry Metrics
# =============================================================================

def compute_geometry_metrics(obj: 'bpy.types.Object') -> Dict[str, Any]:
    """
    Compute extent, aspect ratios, centroid, convex hull ratio for a mesh object.

    Args:
        obj: The mesh object to analyze.

    Returns:
        Dictionary containing:
        - extent: bounding box dimensions [x, y, z]
        - aspect_ratios: {xy, xz, yz}
        - dominant_axis: "X", "Y", or "Z"
        - elongation: longest/shortest
        - centroid: mesh center of mass [x, y, z]
        - centroid_normalized: centroid as fraction of bbox [0-1, 0-1, 0-1]
        - convex_hull_ratio: mesh volume / convex hull volume
    """
    if not BLENDER_AVAILABLE:
        return {}

    if obj.type != 'MESH' or not obj.data.vertices:
        return _empty_geometry_metrics()

    # Get evaluated mesh data
    depsgraph = bpy.context.evaluated_depsgraph_get()
    obj_eval = obj.evaluated_get(depsgraph)
    mesh = obj_eval.to_mesh()

    if not mesh.vertices:
        obj_eval.to_mesh_clear()
        return _empty_geometry_metrics()

    # Compute bounding box in world space
    bbox_min = [float('inf')] * 3
    bbox_max = [float('-inf')] * 3

    for v in mesh.vertices:
        co = obj.matrix_world @ v.co
        for i in range(3):
            bbox_min[i] = min(bbox_min[i], co[i])
            bbox_max[i] = max(bbox_max[i], co[i])

    # Compute extent
    extent = [bbox_max[i] - bbox_min[i] for i in range(3)]

    # Handle degenerate cases
    if any(e < 1e-8 for e in extent):
        obj_eval.to_mesh_clear()
        return _empty_geometry_metrics()

    # Compute aspect ratios
    aspect_ratios = {
        "xy": extent[0] / extent[1] if extent[1] > 1e-8 else 0.0,
        "xz": extent[0] / extent[2] if extent[2] > 1e-8 else 0.0,
        "yz": extent[1] / extent[2] if extent[2] > 1e-8 else 0.0,
    }

    # Determine dominant axis
    axis_names = ["X", "Y", "Z"]
    dominant_idx = extent.index(max(extent))
    dominant_axis = axis_names[dominant_idx]

    # Compute elongation (longest / shortest)
    sorted_extent = sorted([e for e in extent if e > 1e-8])
    if len(sorted_extent) >= 2:
        elongation = sorted_extent[-1] / sorted_extent[0]
    else:
        elongation = 1.0

    # Compute centroid (center of mass approximation using vertex positions)
    centroid = _compute_mesh_centroid(mesh, obj.matrix_world)

    # Normalize centroid to bbox fraction [0-1]
    centroid_normalized = [
        (centroid[i] - bbox_min[i]) / extent[i] if extent[i] > 1e-8 else 0.5
        for i in range(3)
    ]

    # Compute convex hull ratio
    convex_hull_ratio = _compute_convex_hull_ratio(mesh, obj.matrix_world)

    obj_eval.to_mesh_clear()

    return {
        "extent": [round(e, 6) for e in extent],
        "aspect_ratios": {k: round(v, 4) for k, v in aspect_ratios.items()},
        "dominant_axis": dominant_axis,
        "elongation": round(elongation, 4),
        "centroid": [round(c, 6) for c in centroid],
        "centroid_normalized": [round(c, 4) for c in centroid_normalized],
        "convex_hull_ratio": round(convex_hull_ratio, 4),
    }


def _empty_geometry_metrics() -> Dict[str, Any]:
    """Return empty geometry metrics for invalid/empty meshes."""
    return {
        "extent": [0.0, 0.0, 0.0],
        "aspect_ratios": {"xy": 0.0, "xz": 0.0, "yz": 0.0},
        "dominant_axis": "Z",
        "elongation": 1.0,
        "centroid": [0.0, 0.0, 0.0],
        "centroid_normalized": [0.5, 0.5, 0.5],
        "convex_hull_ratio": 0.0,
    }


def _compute_mesh_centroid(
    mesh: 'bpy.types.Mesh',
    matrix_world: 'Matrix'
) -> List[float]:
    """Compute mesh centroid in world space."""
    if not mesh.vertices:
        return [0.0, 0.0, 0.0]

    centroid = Vector((0.0, 0.0, 0.0))
    for v in mesh.vertices:
        centroid += matrix_world @ v.co
    centroid /= len(mesh.vertices)

    return [centroid.x, centroid.y, centroid.z]


def _compute_convex_hull_ratio(
    mesh: 'bpy.types.Mesh',
    matrix_world: 'Matrix'
) -> float:
    """
    Compute ratio of mesh volume to convex hull volume.

    A value close to 1.0 means the mesh is nearly convex.
    A value close to 0.0 means the mesh has many concavities.
    """
    if not mesh.vertices or len(mesh.vertices) < 4:
        return 0.0

    # Create bmesh for volume calculation
    bm = bmesh.new()
    bm.from_mesh(mesh)

    # Transform to world space
    bmesh.ops.transform(bm, matrix=matrix_world, verts=bm.verts)

    # Compute original mesh volume (absolute value to handle inverted normals)
    mesh_volume = abs(bm.calc_volume())

    if mesh_volume < 1e-10:
        bm.free()
        return 0.0

    # Compute convex hull
    try:
        hull_result = bmesh.ops.convex_hull(bm, input=bm.verts)
        # The convex hull operation modifies bm in place for new geometry
        # We need to recalculate volume on the hull

        # Create a new bmesh from the hull geometry
        hull_bm = bmesh.new()
        hull_verts = hull_result.get("geom", [])

        # Extract vertices from hull result
        hull_vert_coords = []
        for elem in hull_verts:
            if isinstance(elem, bmesh.types.BMVert):
                hull_vert_coords.append(elem.co.copy())

        if len(hull_vert_coords) < 4:
            bm.free()
            hull_bm.free()
            return 1.0  # Degenerate case, assume convex

        # Build hull mesh from vertices
        for co in hull_vert_coords:
            hull_bm.verts.new(co)
        hull_bm.verts.ensure_lookup_table()

        # Compute convex hull of the extracted points
        if len(hull_bm.verts) >= 4:
            bmesh.ops.convex_hull(hull_bm, input=hull_bm.verts)
            hull_volume = abs(hull_bm.calc_volume())
        else:
            hull_volume = mesh_volume

        hull_bm.free()

    except Exception:
        # Fallback: assume ratio of 1.0 if hull computation fails
        bm.free()
        return 1.0

    bm.free()

    if hull_volume < 1e-10:
        return 1.0

    ratio = mesh_volume / hull_volume
    return min(ratio, 1.0)  # Cap at 1.0 to handle floating point errors


# =============================================================================
# Symmetry Metrics
# =============================================================================

def compute_symmetry_metrics(obj: 'bpy.types.Object') -> Dict[str, Any]:
    """
    Compute axis reflection symmetry scores (0-1) for X, Y, Z axes.

    Samples vertices, reflects across each axis, finds nearest vertex,
    and measures distance. Score = 1 - (avg_distance / bbox_diagonal).

    Args:
        obj: The mesh object to analyze.

    Returns:
        Dictionary containing:
        - x_axis: symmetry score for X axis reflection (0-1)
        - y_axis: symmetry score for Y axis reflection (0-1)
        - z_axis: symmetry score for Z axis reflection (0-1)
    """
    if not BLENDER_AVAILABLE:
        return {"x_axis": 0.0, "y_axis": 0.0, "z_axis": 0.0}

    if obj.type != 'MESH' or not obj.data.vertices:
        return {"x_axis": 0.0, "y_axis": 0.0, "z_axis": 0.0}

    # Get evaluated mesh data
    depsgraph = bpy.context.evaluated_depsgraph_get()
    obj_eval = obj.evaluated_get(depsgraph)
    mesh = obj_eval.to_mesh()

    if not mesh.vertices or len(mesh.vertices) < 2:
        obj_eval.to_mesh_clear()
        return {"x_axis": 0.0, "y_axis": 0.0, "z_axis": 0.0}

    # Get world-space vertices
    world_verts = [obj.matrix_world @ v.co for v in mesh.vertices]

    # Compute bounding box
    bbox_min = Vector((float('inf'), float('inf'), float('inf')))
    bbox_max = Vector((float('-inf'), float('-inf'), float('-inf')))
    for co in world_verts:
        for i in range(3):
            bbox_min[i] = min(bbox_min[i], co[i])
            bbox_max[i] = max(bbox_max[i], co[i])

    # Compute diagonal for normalization
    diagonal = (bbox_max - bbox_min).length
    if diagonal < 1e-8:
        obj_eval.to_mesh_clear()
        return {"x_axis": 1.0, "y_axis": 1.0, "z_axis": 1.0}

    # Compute center for reflection
    center = (bbox_min + bbox_max) * 0.5

    # Build BVH tree for efficient nearest-vertex queries
    bvh = BVHTree.FromPolygons(
        [v.co for v in mesh.vertices],
        [[v for v in p.vertices] for p in mesh.polygons] if mesh.polygons else []
    )

    # If no polygons, use point-based approach
    if not mesh.polygons:
        # Fall back to brute-force for point clouds
        obj_eval.to_mesh_clear()
        return _compute_symmetry_brute_force(world_verts, center, diagonal)

    # Sample vertices for symmetry check (limit for performance)
    max_samples = min(500, len(world_verts))
    sample_step = max(1, len(world_verts) // max_samples)
    sampled_verts = world_verts[::sample_step]

    # Compute symmetry for each axis
    x_score = _compute_axis_symmetry(sampled_verts, center, 0, world_verts, diagonal)
    y_score = _compute_axis_symmetry(sampled_verts, center, 1, world_verts, diagonal)
    z_score = _compute_axis_symmetry(sampled_verts, center, 2, world_verts, diagonal)

    obj_eval.to_mesh_clear()

    return {
        "x_axis": round(x_score, 4),
        "y_axis": round(y_score, 4),
        "z_axis": round(z_score, 4),
    }


def _compute_axis_symmetry(
    sampled_verts: List['Vector'],
    center: 'Vector',
    axis: int,
    all_verts: List['Vector'],
    diagonal: float
) -> float:
    """Compute symmetry score for a single axis."""
    if not sampled_verts or diagonal < 1e-8:
        return 0.0

    total_distance = 0.0
    valid_samples = 0

    for v in sampled_verts:
        # Reflect vertex across axis through center
        reflected = v.copy()
        reflected[axis] = 2 * center[axis] - v[axis]

        # Find nearest vertex to reflected position
        min_dist = float('inf')
        for other in all_verts:
            dist = (reflected - other).length
            min_dist = min(min_dist, dist)

        total_distance += min_dist
        valid_samples += 1

    if valid_samples == 0:
        return 0.0

    avg_distance = total_distance / valid_samples
    score = 1.0 - (avg_distance / diagonal)
    return max(0.0, min(1.0, score))


def _compute_symmetry_brute_force(
    world_verts: List['Vector'],
    center: 'Vector',
    diagonal: float
) -> Dict[str, float]:
    """Brute-force symmetry computation for meshes without polygons."""
    if not world_verts or diagonal < 1e-8:
        return {"x_axis": 0.0, "y_axis": 0.0, "z_axis": 0.0}

    max_samples = min(200, len(world_verts))
    sample_step = max(1, len(world_verts) // max_samples)
    sampled = world_verts[::sample_step]

    scores = {}
    for axis, axis_name in enumerate(["x_axis", "y_axis", "z_axis"]):
        scores[axis_name] = _compute_axis_symmetry(
            sampled, center, axis, world_verts, diagonal
        )

    return {k: round(v, 4) for k, v in scores.items()}


# =============================================================================
# Component Metrics
# =============================================================================

def compute_component_metrics(objects: List['bpy.types.Object']) -> Dict[str, Any]:
    """
    Compute per-component breakdown and adjacency for multi-part meshes.

    Args:
        objects: List of mesh objects representing components.

    Returns:
        Dictionary containing:
        - components: list of {name, bounds_min, bounds_max, volume_fraction,
                               centroid, triangle_count}
        - adjacency: list of {part_a, part_b, distance}
    """
    if not BLENDER_AVAILABLE:
        return {"components": [], "adjacency": []}

    if not objects:
        return {"components": [], "adjacency": []}

    # Filter to mesh objects only
    mesh_objects = [obj for obj in objects if obj.type == 'MESH']
    if not mesh_objects:
        return {"components": [], "adjacency": []}

    # Compute per-component info
    components = []
    total_volume = 0.0
    component_volumes = []

    for obj in mesh_objects:
        info = _compute_component_info(obj)
        components.append(info)
        component_volumes.append(info.get("_volume", 0.0))
        total_volume += info.get("_volume", 0.0)

    # Compute volume fractions
    for i, comp in enumerate(components):
        if total_volume > 1e-10:
            comp["volume_fraction"] = round(component_volumes[i] / total_volume, 4)
        else:
            comp["volume_fraction"] = 1.0 / len(components) if components else 0.0
        # Remove internal _volume key
        comp.pop("_volume", None)

    # Compute adjacency between components
    adjacency = _compute_component_adjacency(mesh_objects)

    return {
        "components": components,
        "adjacency": adjacency,
    }


def _compute_component_info(obj: 'bpy.types.Object') -> Dict[str, Any]:
    """Compute info for a single component."""
    if obj.type != 'MESH' or not obj.data.vertices:
        return {
            "name": obj.name,
            "bounds_min": [0.0, 0.0, 0.0],
            "bounds_max": [0.0, 0.0, 0.0],
            "volume_fraction": 0.0,
            "centroid": [0.0, 0.0, 0.0],
            "triangle_count": 0,
            "_volume": 0.0,
        }

    # Get evaluated mesh
    depsgraph = bpy.context.evaluated_depsgraph_get()
    obj_eval = obj.evaluated_get(depsgraph)
    mesh = obj_eval.to_mesh()

    # Compute bounds
    bbox_min = [float('inf')] * 3
    bbox_max = [float('-inf')] * 3
    centroid = Vector((0.0, 0.0, 0.0))

    for v in mesh.vertices:
        co = obj.matrix_world @ v.co
        centroid += co
        for i in range(3):
            bbox_min[i] = min(bbox_min[i], co[i])
            bbox_max[i] = max(bbox_max[i], co[i])

    if mesh.vertices:
        centroid /= len(mesh.vertices)

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

    # Compute volume using bmesh
    bm = bmesh.new()
    bm.from_mesh(mesh)
    bmesh.ops.transform(bm, matrix=obj.matrix_world, verts=bm.verts)
    volume = abs(bm.calc_volume())
    bm.free()

    obj_eval.to_mesh_clear()

    # Handle degenerate bounds
    if any(bbox_min[i] > bbox_max[i] for i in range(3)):
        bbox_min = [0.0, 0.0, 0.0]
        bbox_max = [0.0, 0.0, 0.0]

    return {
        "name": obj.name,
        "bounds_min": [round(b, 6) for b in bbox_min],
        "bounds_max": [round(b, 6) for b in bbox_max],
        "volume_fraction": 0.0,  # Filled in later
        "centroid": [round(centroid.x, 6), round(centroid.y, 6), round(centroid.z, 6)],
        "triangle_count": triangle_count,
        "_volume": volume,
    }


def _compute_component_adjacency(
    objects: List['bpy.types.Object']
) -> List[Dict[str, Any]]:
    """
    Compute adjacency/distance between component pairs.

    Positive distance = gap between parts.
    Negative distance = overlap.
    """
    adjacency = []

    if len(objects) < 2:
        return adjacency

    # Compute bounding boxes for all objects
    bboxes = {}
    for obj in objects:
        if obj.type != 'MESH' or not obj.data.vertices:
            continue

        bbox_min = [float('inf')] * 3
        bbox_max = [float('-inf')] * 3
        for v in obj.data.vertices:
            co = obj.matrix_world @ v.co
            for i in range(3):
                bbox_min[i] = min(bbox_min[i], co[i])
                bbox_max[i] = max(bbox_max[i], co[i])

        if not any(bbox_min[i] > bbox_max[i] for i in range(3)):
            bboxes[obj.name] = (bbox_min, bbox_max)

    # Compute pairwise distances
    names = list(bboxes.keys())
    for i in range(len(names)):
        for j in range(i + 1, len(names)):
            name_a = names[i]
            name_b = names[j]
            bbox_a = bboxes[name_a]
            bbox_b = bboxes[name_b]

            distance = _bbox_distance(bbox_a, bbox_b)
            adjacency.append({
                "part_a": name_a,
                "part_b": name_b,
                "distance": round(distance, 6),
            })

    return adjacency


def _bbox_distance(
    bbox_a: Tuple[List[float], List[float]],
    bbox_b: Tuple[List[float], List[float]]
) -> float:
    """
    Compute signed distance between two bounding boxes.

    Positive = gap, Negative = overlap.
    """
    min_a, max_a = bbox_a
    min_b, max_b = bbox_b

    # Compute overlap/gap per axis
    gaps = []
    for i in range(3):
        # Gap on this axis (positive = separation, negative = overlap)
        gap = max(min_a[i], min_b[i]) - min(max_a[i], max_b[i])
        gaps.append(gap)

    # If any axis has positive gap, boxes are separated
    # Return the Euclidean distance of the gap
    if all(g <= 0 for g in gaps):
        # Boxes overlap - return negative of minimum penetration
        return min(gaps)  # Most negative = deepest overlap

    # Boxes are separated - compute Euclidean gap distance
    positive_gaps = [max(0, g) for g in gaps]
    return math.sqrt(sum(g * g for g in positive_gaps))


# =============================================================================
# Skeletal Metrics
# =============================================================================

def compute_skeletal_metrics(
    armature: 'bpy.types.Object',
    mesh: 'bpy.types.Object'
) -> Dict[str, Any]:
    """
    Compute bone coverage and symmetry for rigged meshes.

    Args:
        armature: The armature object.
        mesh: The mesh object with vertex groups.

    Returns:
        Dictionary containing:
        - hierarchy_depth: max bone chain length
        - terminal_bones: leaf bone names
        - bone_coverage: per-bone {bone_name, bone_length, mesh_length_along_bone,
                                   coverage_ratio, mesh_radius_avg}
        - bone_symmetry: for L/R pairs {bone_left, bone_right, length_ratio, radius_ratio}
    """
    if not BLENDER_AVAILABLE:
        return _empty_skeletal_metrics()

    if not armature or armature.type != 'ARMATURE':
        return _empty_skeletal_metrics()

    if not mesh or mesh.type != 'MESH':
        return _empty_skeletal_metrics()

    bones = armature.data.bones

    if not bones:
        return _empty_skeletal_metrics()

    # Compute hierarchy depth
    hierarchy_depth = _compute_hierarchy_depth(bones)

    # Find terminal bones (leaves)
    terminal_bones = [b.name for b in bones if not b.children]

    # Compute bone coverage
    bone_coverage = _compute_bone_coverage(armature, mesh)

    # Compute bone pair symmetry
    bone_symmetry = _compute_bone_symmetry(bones, bone_coverage)

    return {
        "hierarchy_depth": hierarchy_depth,
        "terminal_bones": terminal_bones,
        "bone_coverage": bone_coverage,
        "bone_symmetry": bone_symmetry,
    }


def _empty_skeletal_metrics() -> Dict[str, Any]:
    """Return empty skeletal metrics."""
    return {
        "hierarchy_depth": 0,
        "terminal_bones": [],
        "bone_coverage": [],
        "bone_symmetry": [],
    }


def _compute_hierarchy_depth(bones: 'bpy.types.ArmatureBones') -> int:
    """Compute maximum bone chain length."""
    max_depth = 0

    for bone in bones:
        depth = 1
        parent = bone.parent
        while parent:
            depth += 1
            parent = parent.parent
        max_depth = max(max_depth, depth)

    return max_depth


def _compute_bone_coverage(
    armature: 'bpy.types.Object',
    mesh_obj: 'bpy.types.Object'
) -> List[Dict[str, Any]]:
    """Compute per-bone mesh coverage."""
    coverage = []

    if not mesh_obj.vertex_groups:
        return coverage

    # Get mesh data
    mesh = mesh_obj.data

    # Build vertex group name to index mapping
    vg_names = {vg.name: vg.index for vg in mesh_obj.vertex_groups}

    for bone in armature.data.bones:
        if bone.name not in vg_names:
            continue

        vg_index = vg_names[bone.name]

        # Find vertices in this group with significant weight
        influenced_verts = []
        for v in mesh.vertices:
            for g in v.groups:
                if g.group == vg_index and g.weight > 0.1:
                    # Transform to world space
                    world_co = mesh_obj.matrix_world @ v.co
                    influenced_verts.append(world_co)
                    break

        if not influenced_verts:
            continue

        # Get bone world-space head and tail
        bone_head = armature.matrix_world @ bone.head_local
        bone_tail = armature.matrix_world @ bone.tail_local
        bone_length = (bone_tail - bone_head).length

        if bone_length < 1e-6:
            continue

        bone_dir = (bone_tail - bone_head).normalized()

        # Project vertices onto bone axis
        projections = []
        radii = []
        for v in influenced_verts:
            # Vector from bone head to vertex
            to_vert = v - bone_head
            # Projection along bone
            proj = to_vert.dot(bone_dir)
            projections.append(proj)
            # Perpendicular distance (radius)
            proj_point = bone_head + bone_dir * proj
            radius = (v - proj_point).length
            radii.append(radius)

        if not projections:
            continue

        # Mesh extent along bone
        min_proj = min(projections)
        max_proj = max(projections)
        mesh_length = max_proj - min_proj

        # Coverage ratio
        coverage_ratio = mesh_length / bone_length if bone_length > 0 else 0.0

        # Average radius
        avg_radius = sum(radii) / len(radii) if radii else 0.0

        coverage.append({
            "bone_name": bone.name,
            "bone_length": round(bone_length, 6),
            "mesh_length_along_bone": round(mesh_length, 6),
            "coverage_ratio": round(coverage_ratio, 4),
            "mesh_radius_avg": round(avg_radius, 6),
        })

    return coverage


def _compute_bone_symmetry(
    bones: 'bpy.types.ArmatureBones',
    coverage: List[Dict[str, Any]]
) -> List[Dict[str, Any]]:
    """Compute symmetry for L/R bone pairs."""
    symmetry = []

    # Build coverage lookup
    coverage_lookup = {c["bone_name"]: c for c in coverage}

    # Common L/R suffixes
    left_suffixes = [".L", "_L", ".l", "_l", "_left", ".left"]
    right_suffixes = [".R", "_R", ".r", "_r", "_right", ".right"]

    # Find pairs
    processed = set()
    for bone in bones:
        if bone.name in processed:
            continue

        # Check if this is a left bone
        left_name = None
        right_name = None
        base_name = None

        for l_suf, r_suf in zip(left_suffixes, right_suffixes):
            if bone.name.endswith(l_suf):
                left_name = bone.name
                base_name = bone.name[:-len(l_suf)]
                right_name = base_name + r_suf
                break
            elif bone.name.endswith(r_suf):
                right_name = bone.name
                base_name = bone.name[:-len(r_suf)]
                left_name = base_name + l_suf
                break

        if not left_name or not right_name:
            continue

        # Check if both bones exist
        left_bone = bones.get(left_name)
        right_bone = bones.get(right_name)

        if not left_bone or not right_bone:
            continue

        processed.add(left_name)
        processed.add(right_name)

        # Get bone lengths
        left_length = left_bone.length
        right_length = right_bone.length

        length_ratio = min(left_length, right_length) / max(left_length, right_length) \
            if max(left_length, right_length) > 1e-6 else 1.0

        # Get mesh radii from coverage
        left_cov = coverage_lookup.get(left_name, {})
        right_cov = coverage_lookup.get(right_name, {})

        left_radius = left_cov.get("mesh_radius_avg", 0.0)
        right_radius = right_cov.get("mesh_radius_avg", 0.0)

        radius_ratio = min(left_radius, right_radius) / max(left_radius, right_radius) \
            if max(left_radius, right_radius) > 1e-6 else 1.0

        symmetry.append({
            "bone_left": left_name,
            "bone_right": right_name,
            "length_ratio": round(length_ratio, 4),
            "radius_ratio": round(radius_ratio, 4),
        })

    return symmetry


# =============================================================================
# Scale Reference
# =============================================================================

def compute_scale_reference(obj: 'bpy.types.Object') -> Dict[str, Any]:
    """
    Compute scale reference metrics.

    Args:
        obj: The mesh object to analyze.

    Returns:
        Dictionary containing:
        - longest_dimension_m: longest bounding box dimension in meters
        - volume_m3: total mesh volume in cubic meters
        - fits_in_1m_cube: whether the mesh fits in a 1m cube
        - fits_in_10cm_cube: whether the mesh fits in a 10cm cube
    """
    if not BLENDER_AVAILABLE:
        return _empty_scale_reference()

    if obj.type != 'MESH' or not obj.data.vertices:
        return _empty_scale_reference()

    # Get evaluated mesh
    depsgraph = bpy.context.evaluated_depsgraph_get()
    obj_eval = obj.evaluated_get(depsgraph)
    mesh = obj_eval.to_mesh()

    if not mesh.vertices:
        obj_eval.to_mesh_clear()
        return _empty_scale_reference()

    # Compute bounding box
    bbox_min = [float('inf')] * 3
    bbox_max = [float('-inf')] * 3

    for v in mesh.vertices:
        co = obj.matrix_world @ v.co
        for i in range(3):
            bbox_min[i] = min(bbox_min[i], co[i])
            bbox_max[i] = max(bbox_max[i], co[i])

    extent = [bbox_max[i] - bbox_min[i] for i in range(3)]
    longest_dimension = max(extent)

    # Compute volume using bmesh
    bm = bmesh.new()
    bm.from_mesh(mesh)
    bmesh.ops.transform(bm, matrix=obj.matrix_world, verts=bm.verts)
    volume = abs(bm.calc_volume())
    bm.free()

    obj_eval.to_mesh_clear()

    # Determine fits_in checks
    fits_in_1m = all(e <= 1.0 for e in extent)
    fits_in_10cm = all(e <= 0.1 for e in extent)

    return {
        "longest_dimension_m": round(longest_dimension, 6),
        "volume_m3": round(volume, 9),
        "fits_in_1m_cube": fits_in_1m,
        "fits_in_10cm_cube": fits_in_10cm,
    }


def _empty_scale_reference() -> Dict[str, Any]:
    """Return empty scale reference metrics."""
    return {
        "longest_dimension_m": 0.0,
        "volume_m3": 0.0,
        "fits_in_1m_cube": True,
        "fits_in_10cm_cube": True,
    }


# =============================================================================
# Main Entry Point
# =============================================================================

def compute_all_structural_metrics(
    obj: 'bpy.types.Object',
    armature: Optional['bpy.types.Object'] = None,
    components: Optional[List['bpy.types.Object']] = None
) -> Dict[str, Any]:
    """
    Compute all structural metrics for an object.

    Args:
        obj: The primary mesh object to analyze.
        armature: Optional armature for skeletal metrics.
        components: Optional list of component objects for multi-part analysis.

    Returns:
        Dictionary containing all structural metrics:
        - geometry: extent, aspect ratios, centroid, convex hull ratio
        - symmetry: axis reflection scores
        - component: per-part breakdown and adjacency (if components provided)
        - skeletal: bone coverage and symmetry (if armature provided)
        - scale: dimension reference checks
    """
    if not BLENDER_AVAILABLE:
        return {}

    result = {}

    # Geometry metrics (always computed)
    result["geometry"] = compute_geometry_metrics(obj)

    # Symmetry metrics (always computed)
    result["symmetry"] = compute_symmetry_metrics(obj)

    # Component metrics (if multiple components provided)
    if components and len(components) > 1:
        result["component"] = compute_component_metrics(components)
    elif components and len(components) == 1:
        # Single component - still provide structure but simplified
        result["component"] = compute_component_metrics(components)

    # Skeletal metrics (if armature provided)
    if armature and armature.type == 'ARMATURE':
        result["skeletal"] = compute_skeletal_metrics(armature, obj)

    # Scale reference (always computed)
    result["scale"] = compute_scale_reference(obj)

    return result
