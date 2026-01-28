"""
SpecCade Metrics Module

This module contains all mesh and animation metrics computation functions
extracted from the main entrypoint. These functions compute comprehensive
metrics for static meshes, skeletal meshes, and animations.

The metrics include:
- Topology metrics (vertex count, face count, triangles, quads)
- Manifold metrics (non-manifold edges, degenerate faces)
- UV metrics (coverage, overlap, island count, texel density)
- Skeletal metrics (bone count, max influences, weight normalization)
- Animation metrics (frame count, duration, motion verification)
"""

import math
from typing import Any, Dict, List, Optional, Tuple

# Blender modules - only available when running inside Blender
try:
    import bpy
    import bmesh
    from mathutils import Euler, Matrix, Vector
    BLENDER_AVAILABLE = True
except ImportError:
    BLENDER_AVAILABLE = False

# Structural metrics module (for LLM-friendly 3D feedback)
try:
    from structural_metrics import compute_all_structural_metrics
    STRUCTURAL_METRICS_AVAILABLE = True
except ImportError:
    STRUCTURAL_METRICS_AVAILABLE = False
    def compute_all_structural_metrics(obj, armature=None, components=None):
        return {}


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

    result = {
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

    # Add structural metrics for LLM-friendly feedback
    if STRUCTURAL_METRICS_AVAILABLE:
        structural = compute_all_structural_metrics(obj)
        if structural:
            result["structural"] = structural

    return result


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

    # Enhance structural metrics with skeletal data if available
    if STRUCTURAL_METRICS_AVAILABLE and "structural" in mesh_metrics:
        skeletal_structural = compute_all_structural_metrics(obj, armature=armature)
        if skeletal_structural:
            mesh_metrics["structural"] = skeletal_structural

    return mesh_metrics


def _wrap_degrees(angle_deg: float) -> float:
    """Wrap an angle to [-180, 180] degrees."""
    while angle_deg > 180.0:
        angle_deg -= 360.0
    while angle_deg < -180.0:
        angle_deg += 360.0
    return angle_deg


def _pose_bone_local_euler_degrees(pose_bone: 'bpy.types.PoseBone') -> Tuple[float, float, float]:
    """
    Get pose bone local rotation (matrix_basis) as XYZ Euler degrees.

    We intentionally use matrix_basis (local) to align with how LIMIT_ROTATION
    constraints are configured in LOCAL space.
    """
    try:
        e = pose_bone.matrix_basis.to_euler('XYZ')
        return (
            math.degrees(e.x),
            math.degrees(e.y),
            math.degrees(e.z),
        )
    except Exception:
        e = pose_bone.rotation_euler
        return (
            math.degrees(e.x),
            math.degrees(e.y),
            math.degrees(e.z),
        )


def compute_motion_verification_metrics(
    armature: 'bpy.types.Object',
    frame_start: int,
    frame_end: int,
    constraints_list: Optional[List[Dict[str, Any]]] = None,
) -> Dict[str, Any]:
    """
    Compute motion verification metrics (MESHVER-005).

    - hinge_axis_violations: rotation leakage on locked axes for hinge joints
    - range_violations: rotations outside declared limits for constrained joints
    - velocity_spikes: root motion direction flips between consecutive frames
    - root_motion_delta: root bone head delta from first->last frame
    """
    constraints_list = constraints_list or []

    hinge_axis_violations = 0
    range_violations = 0
    velocity_spikes = 0
    root_motion_delta = [0.0, 0.0, 0.0]

    if frame_end < frame_start:
        return {
            "hinge_axis_violations": 0,
            "range_violations": 0,
            "velocity_spikes": 0,
            "root_motion_delta": root_motion_delta,
        }

    # Pre-filter to hinge constraints (highest-leverage / clearest semantics)
    hinge_constraints = [
        c for c in constraints_list
        if c.get("type", "").lower() == "hinge" and c.get("bone")
    ]

    # Root bone selection for motion metrics
    root_pose_bone = armature.pose.bones.get("root")
    if root_pose_bone is None and armature.pose.bones:
        root_pose_bone = armature.pose.bones[0]

    # Tolerances
    lock_axis_tolerance_deg = 0.25
    range_tolerance_deg = 0.25
    velocity_epsilon = 1e-6

    # Ensure pose mode for consistent pose evaluation
    bpy.ops.object.select_all(action='DESELECT')
    armature.select_set(True)
    bpy.context.view_layer.objects.active = armature
    bpy.ops.object.mode_set(mode='POSE')

    root_positions: List[Tuple[float, float, float]] = []

    for frame in range(frame_start, frame_end + 1):
        bpy.context.scene.frame_set(frame)
        bpy.context.view_layer.update()

        # Root motion samples
        if root_pose_bone is not None:
            root_world = armature.matrix_world @ root_pose_bone.head
            root_positions.append((root_world.x, root_world.y, root_world.z))

        # Constraint checks
        for c in hinge_constraints:
            bone_name = c.get("bone")
            pose_bone = armature.pose.bones.get(bone_name)
            if pose_bone is None:
                continue

            axis = c.get("axis", "X").upper()
            min_angle = float(c.get("min_angle", 0.0))
            max_angle = float(c.get("max_angle", 160.0))

            rx, ry, rz = _pose_bone_local_euler_degrees(pose_bone)
            angles = {
                "X": _wrap_degrees(rx),
                "Y": _wrap_degrees(ry),
                "Z": _wrap_degrees(rz),
            }

            # Range check on hinge axis
            hinge_angle = angles.get(axis, 0.0)
            if hinge_angle < (min_angle - range_tolerance_deg) or hinge_angle > (max_angle + range_tolerance_deg):
                range_violations += 1

            # Leakage on locked axes
            for ax in ("X", "Y", "Z"):
                if ax == axis:
                    continue
                if abs(angles.get(ax, 0.0)) > lock_axis_tolerance_deg:
                    hinge_axis_violations += 1

    bpy.ops.object.mode_set(mode='OBJECT')

    # Root motion delta and velocity spikes
    if len(root_positions) >= 2:
        start = root_positions[0]
        end = root_positions[-1]
        root_motion_delta = [end[0] - start[0], end[1] - start[1], end[2] - start[2]]

        for i in range(2, len(root_positions)):
            p0 = root_positions[i - 2]
            p1 = root_positions[i - 1]
            p2 = root_positions[i]

            v_prev = (p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2])
            v_cur = (p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2])

            dot = v_prev[0] * v_cur[0] + v_prev[1] * v_cur[1] + v_prev[2] * v_cur[2]
            mag_prev = math.sqrt(v_prev[0] ** 2 + v_prev[1] ** 2 + v_prev[2] ** 2)
            mag_cur = math.sqrt(v_cur[0] ** 2 + v_cur[1] ** 2 + v_cur[2] ** 2)

            if mag_prev > velocity_epsilon and mag_cur > velocity_epsilon and dot < -velocity_epsilon:
                velocity_spikes += 1

    return {
        "hinge_axis_violations": hinge_axis_violations,
        "range_violations": range_violations,
        "velocity_spikes": velocity_spikes,
        "root_motion_delta": root_motion_delta,
    }


def compute_animation_metrics(
    armature: 'bpy.types.Object',
    action: 'bpy.types.Action',
    constraints_list: Optional[List[Dict[str, Any]]] = None,
) -> Dict[str, Any]:
    """Compute metrics for an animation."""
    bone_count = len(armature.data.bones)

    # Get frame range
    frame_start = int(action.frame_range[0])
    frame_end = int(action.frame_range[1])
    frame_count = frame_end - frame_start + 1

    # Get FPS from scene
    fps = bpy.context.scene.render.fps
    duration_seconds = frame_count / fps

    metrics = {
        "bone_count": bone_count,
        "animation_frame_count": frame_count,
        "animation_duration_seconds": duration_seconds
    }

    # Motion verification metrics (optional; depends on rig_setup constraints).
    metrics.update(
        compute_motion_verification_metrics(
            armature,
            frame_start=frame_start,
            frame_end=frame_end,
            constraints_list=constraints_list,
        )
    )

    return metrics
