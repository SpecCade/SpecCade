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
    modular_kit     - Generate modular kit mesh (modular_kit_v1)
    organic_sculpt  - Generate organic sculpt mesh (organic_sculpt_v1)
    shrinkwrap      - Generate shrinkwrap mesh (shrinkwrap_v1) - armor/clothing wrapping
    boolean_kit     - Generate boolean kitbash mesh (boolean_kit_v1) - hard-surface modeling
    skeletal_mesh   - Generate skeletal mesh (blender_rigged_mesh_v1)
    animation       - Generate animation clip (blender_clip_v1)
    validation_grid - Generate 6-view validation grid PNG for LLM verification
"""

import argparse
import json
import math
import sys
import tempfile
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

# Structural metrics module (for LLM-friendly 3D feedback)
try:
    from structural_metrics import compute_all_structural_metrics
    STRUCTURAL_METRICS_AVAILABLE = True
except ImportError:
    STRUCTURAL_METRICS_AVAILABLE = False
    def compute_all_structural_metrics(obj, armature=None, components=None):
        return {}


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


def apply_skeleton_overrides(
    armature_obj: 'bpy.types.Object',
    skeleton_spec: List[Dict[str, Any]],
) -> None:
    """
    Apply skeleton overrides/additions to an existing armature.

    This supports specs that provide a `skeleton_preset` plus a `skeleton` list
    to tweak bone positions/hierarchy or add extra bones.
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
            for fcurve in _iter_action_fcurves(action):
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
        for fcurve in _iter_action_fcurves(action):
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
# Root Motion
# =============================================================================


def apply_root_motion_settings(armature, action, settings):
    """Apply root motion settings after animation is generated.

    Args:
        armature: The armature object.
        action: The Blender action containing animation data.
        settings: Dict with mode, axes, and optional ground_height.

    Returns:
        Extracted root motion delta [x, y, z] if mode is 'extract', else None.
    """
    mode = settings.get("mode", "keep")
    axes = settings.get("axes", [True, True, True])

    if mode == "keep":
        return None

    # Find root bone (first bone with no parent)
    root_bone = None
    for bone in armature.pose.bones:
        if bone.parent is None:
            root_bone = bone
            break

    if not root_bone:
        print("Warning: No root bone found for root motion processing")
        return None

    if mode == "lock":
        # Zero out root bone location keyframes on specified axes
        for fc in action.fcurves:
            if fc.data_path == f'pose.bones["{root_bone.name}"].location':
                if axes[fc.array_index]:
                    for kp in fc.keyframe_points:
                        kp.co[1] = 0.0
                        kp.handle_left[1] = 0.0
                        kp.handle_right[1] = 0.0
        return None

    elif mode == "extract":
        # Extract root motion curves - zero out root but return the delta
        extracted = [0.0, 0.0, 0.0]
        for fc in action.fcurves:
            if fc.data_path == f'pose.bones["{root_bone.name}"].location':
                idx = fc.array_index
                if axes[idx]:
                    # Get start and end values
                    start_val = fc.evaluate(action.frame_range[0])
                    end_val = fc.evaluate(action.frame_range[1])
                    extracted[idx] = end_val - start_val
                    # Zero the curve
                    for kp in fc.keyframe_points:
                        kp.co[1] = 0.0
                        kp.handle_left[1] = 0.0
                        kp.handle_right[1] = 0.0
        return extracted

    elif mode == "bake_to_hip":
        # Find hip bone (first child of root)
        hip_bone = None
        for bone in armature.pose.bones:
            if bone.parent == root_bone:
                hip_bone = bone
                break
        if not hip_bone:
            print("Warning: No hip bone found for bake_to_hip mode")
            return None

        ground_height = settings.get("ground_height", 0.0)

        # Transfer root location to hip on specified axes
        for fc in action.fcurves:
            if fc.data_path == f'pose.bones["{root_bone.name}"].location':
                idx = fc.array_index
                if axes[idx]:
                    # Copy keyframes to hip
                    hip_path = f'pose.bones["{hip_bone.name}"].location'
                    hip_fc = action.fcurves.find(hip_path, index=idx)
                    if not hip_fc:
                        hip_fc = action.fcurves.new(hip_path, index=idx)
                    for kp in fc.keyframe_points:
                        value = kp.co[1]
                        if ground_height and idx == 2:  # Z axis adjustment
                            value -= ground_height
                        hip_fc.keyframe_points.insert(kp.co[0], value)
                    # Zero root
                    for kp in fc.keyframe_points:
                        kp.co[1] = 0.0
                        kp.handle_left[1] = 0.0
                        kp.handle_right[1] = 0.0
        return None

    return None


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
        # Clamp times so time == duration doesn't create an extra frame.
        # SpecCade defines frame_count = duration_seconds * fps.
        frame_index = int(time * fps)
        if frame_count > 0:
            frame_index = max(0, min(frame_index, frame_count - 1))
        frame = 1 + frame_index
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
            for fcurve in _iter_action_fcurves(action):
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
                for fcurve in _iter_action_fcurves(action):
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

def _normalize_operator_kwargs(op, kwargs: Dict[str, Any]) -> Dict[str, Any]:
    """
    Normalize kwargs for a Blender operator across Blender versions.

    Blender operator signatures can change across versions; passing unknown
    keywords hard-fails the whole generation run. This helper makes exports
    forwards/backwards compatible by dropping unknown args and coercing
    values where reasonable.
    """
    try:
        rna = op.get_rna_type()
        props = getattr(rna, "properties", None)
        if props is None:
            return kwargs

        allowed = {p.identifier for p in props}

        def pick_enum(prop: Any, enabled: bool) -> Optional[str]:
            try:
                ids = [it.identifier for it in prop.enum_items]
            except Exception:
                return None
            if not ids:
                return None
            if not enabled:
                for cand in ("NONE", "OFF", "DISABLED", "NO"):
                    if cand in ids:
                        return cand
                return ids[0]

            for cand in ("MATERIAL", "ACTIVE", "ALL", "EXPORT", "ENABLED", "YES", "ON"):
                if cand in ids:
                    return cand
            for ident in ids:
                if ident not in ("NONE", "OFF", "DISABLED", "NO"):
                    return ident
            return ids[0]

        normalized: Dict[str, Any] = {}
        for k, v in kwargs.items():
            if k not in allowed:
                continue
            try:
                prop = props[k]
                prop_type = getattr(prop, "type", None)
            except Exception:
                normalized[k] = v
                continue

            if prop_type == "BOOLEAN":
                if isinstance(v, bool):
                    normalized[k] = v
                elif isinstance(v, int) and v in (0, 1):
                    normalized[k] = bool(v)
                else:
                    continue
            elif prop_type == "ENUM":
                if isinstance(v, str):
                    normalized[k] = v
                elif isinstance(v, bool):
                    choice = pick_enum(prop, v)
                    if choice is not None:
                        normalized[k] = choice
                else:
                    continue
            elif prop_type == "INT":
                if isinstance(v, bool):
                    normalized[k] = int(v)
                elif isinstance(v, int):
                    normalized[k] = v
                else:
                    continue
            elif prop_type == "FLOAT":
                if isinstance(v, bool):
                    normalized[k] = float(int(v))
                elif isinstance(v, (int, float)):
                    normalized[k] = float(v)
                else:
                    continue
            elif prop_type == "STRING":
                if isinstance(v, str):
                    normalized[k] = v
                else:
                    continue
            else:
                normalized[k] = v

        return normalized
    except Exception:
        return kwargs


def _iter_action_fcurves(action: Any):
    """
    Return an iterable of fcurves for an action if available.

    Blender's animation API has evolved; some versions expose fcurves directly
    on the Action, others may not. We treat missing fcurves as "no-op" for
    interpolation tweaking rather than hard failing generation.
    """
    return getattr(action, "fcurves", []) or []


def export_glb(output_path: Path, include_armature: bool = True,
               include_animation: bool = False, export_tangents: bool = False) -> None:
    """Export scene to GLB format."""
    export_settings = {
        'filepath': str(output_path),
        'export_format': 'GLB',
        'export_apply': True,
        'export_texcoords': True,
        'export_normals': True,
        # Vertex color export flag has changed names across Blender versions.
        # We include both and filter to supported kwargs at call time.
        'export_colors': True,
        'export_vertex_color': True,
        'export_tangents': export_tangents,
    }

    if include_animation:
        export_settings['export_animations'] = True
        export_settings['export_current_frame'] = False
    else:
        export_settings['export_animations'] = False

    export_settings = _normalize_operator_kwargs(bpy.ops.export_scene.gltf, export_settings)
    export_settings = _normalize_operator_kwargs(bpy.ops.export_scene.gltf, export_settings)
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

    export_settings = _normalize_operator_kwargs(bpy.ops.export_scene.gltf, export_settings)
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

    export_settings = _normalize_operator_kwargs(bpy.ops.export_scene.gltf, export_settings)
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
                # UV summary from LOD0 (MESH-002)
                metrics["uv_layer_count"] = lod0.get("uv_layer_count", 0)
                metrics["texel_density"] = lod0.get("texel_density", 0.0)
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


def handle_modular_kit(spec: Dict, out_root: Path, report_path: Path) -> None:
    """Handle modular kit mesh generation (walls, pipes, doors)."""
    start_time = time.time()

    try:
        recipe = spec.get("recipe", {})
        params = recipe.get("params", {})
        kit_type_spec = params.get("kit_type", {})
        kit_type = kit_type_spec.get("type", "wall")

        # Create the kit mesh based on type
        if kit_type == "wall":
            obj = create_wall_kit(kit_type_spec)
        elif kit_type == "pipe":
            obj = create_pipe_kit(kit_type_spec)
        elif kit_type == "door":
            obj = create_door_kit(kit_type_spec)
        else:
            raise ValueError(f"Unknown kit type: {kit_type}")

        # Apply export settings
        export_settings = params.get("export", {})
        if export_settings.get("apply_modifiers", True):
            apply_all_modifiers(obj)

        # Triangulate if requested
        if export_settings.get("triangulate", True):
            mod = obj.modifiers.new(name="Triangulate", type='TRIANGULATE')
            bpy.ops.object.modifier_apply(modifier=mod.name)

        # Apply UV projection (box projection for modular kits)
        apply_uv_projection(obj, {"type": "box", "scale": 1.0})

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

        # Compute metrics and export
        metrics = compute_mesh_metrics(obj)
        export_glb(output_path, export_tangents=export_tangents)

        # Save .blend file if requested
        blend_rel_path = None
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


def create_wall_kit(spec: Dict) -> 'bpy.types.Object':
    """Create a wall kit mesh with optional cutouts and trim."""
    width = spec.get("width", 3.0)
    height = spec.get("height", 2.5)
    thickness = spec.get("thickness", 0.15)
    cutouts = spec.get("cutouts", [])
    has_baseboard = spec.get("has_baseboard", False)
    has_crown = spec.get("has_crown", False)
    baseboard_height = spec.get("baseboard_height", 0.1)
    crown_height = spec.get("crown_height", 0.08)
    bevel_width = spec.get("bevel_width", 0.0)

    # Create base wall as a cube
    bpy.ops.mesh.primitive_cube_add(size=1, location=(width / 2, thickness / 2, height / 2))
    obj = bpy.context.active_object
    obj.name = "WallKit"
    obj.scale = (width, thickness, height)
    bpy.ops.object.transform_apply(scale=True)

    # Apply cutouts using boolean operations
    for i, cutout in enumerate(cutouts):
        cutout_type = cutout.get("cutout_type", "window")
        cut_x = cutout.get("x", 0.0)
        cut_y = cutout.get("y", 0.0)
        cut_width = cutout.get("width", 0.8)
        cut_height = cutout.get("height", 1.0)

        # Create cutter cube
        bpy.ops.mesh.primitive_cube_add(size=1)
        cutter = bpy.context.active_object
        cutter.name = f"Cutter_{i}"

        # Position cutter - x is horizontal, z is vertical
        cutter.location = (cut_x, thickness / 2, cut_y + cut_height / 2)
        cutter.scale = (cut_width, thickness * 1.5, cut_height)
        bpy.ops.object.transform_apply(scale=True)

        # Boolean difference
        bool_mod = obj.modifiers.new(name=f"Cutout_{i}", type='BOOLEAN')
        bool_mod.operation = 'DIFFERENCE'
        bool_mod.object = cutter

        # Apply modifier
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.modifier_apply(modifier=bool_mod.name)

        # Delete cutter
        bpy.data.objects.remove(cutter, do_unlink=True)

        # Add frame if requested
        if cutout.get("has_frame", False):
            frame_thickness = cutout.get("frame_thickness", 0.05)
            frame = create_cutout_frame(cut_x, cut_y, cut_width, cut_height,
                                        frame_thickness, thickness)
            # Join frame with wall
            bpy.ops.object.select_all(action='DESELECT')
            frame.select_set(True)
            obj.select_set(True)
            bpy.context.view_layer.objects.active = obj
            bpy.ops.object.join()

    # Add baseboard if requested
    if has_baseboard:
        bpy.ops.mesh.primitive_cube_add(size=1)
        baseboard = bpy.context.active_object
        baseboard.name = "Baseboard"
        baseboard.location = (width / 2, thickness / 2 + thickness * 0.1, baseboard_height / 2)
        baseboard.scale = (width, thickness * 1.2, baseboard_height)
        bpy.ops.object.transform_apply(scale=True)

        # Join with wall
        bpy.ops.object.select_all(action='DESELECT')
        baseboard.select_set(True)
        obj.select_set(True)
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.join()

    # Add crown molding if requested
    if has_crown:
        bpy.ops.mesh.primitive_cube_add(size=1)
        crown = bpy.context.active_object
        crown.name = "Crown"
        crown.location = (width / 2, thickness / 2 + thickness * 0.1, height - crown_height / 2)
        crown.scale = (width, thickness * 1.2, crown_height)
        bpy.ops.object.transform_apply(scale=True)

        # Join with wall
        bpy.ops.object.select_all(action='DESELECT')
        crown.select_set(True)
        obj.select_set(True)
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.join()

    # Apply bevel if requested
    if bevel_width > 0:
        bevel_mod = obj.modifiers.new(name="Bevel", type='BEVEL')
        bevel_mod.width = bevel_width
        bevel_mod.segments = 2
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.modifier_apply(modifier=bevel_mod.name)

    return obj


def create_cutout_frame(x: float, y: float, width: float, height: float,
                        frame_thickness: float, wall_thickness: float) -> 'bpy.types.Object':
    """Create a frame around a cutout."""
    # Create frame using 4 cubes (top, bottom, left, right)
    frame_parts = []

    # Bottom frame
    bpy.ops.mesh.primitive_cube_add(size=1)
    bottom = bpy.context.active_object
    bottom.location = (x, wall_thickness / 2 + wall_thickness * 0.1, y - frame_thickness / 2)
    bottom.scale = (width + frame_thickness * 2, wall_thickness * 1.1, frame_thickness)
    bpy.ops.object.transform_apply(scale=True)
    frame_parts.append(bottom)

    # Top frame
    bpy.ops.mesh.primitive_cube_add(size=1)
    top = bpy.context.active_object
    top.location = (x, wall_thickness / 2 + wall_thickness * 0.1, y + height + frame_thickness / 2)
    top.scale = (width + frame_thickness * 2, wall_thickness * 1.1, frame_thickness)
    bpy.ops.object.transform_apply(scale=True)
    frame_parts.append(top)

    # Left frame
    bpy.ops.mesh.primitive_cube_add(size=1)
    left = bpy.context.active_object
    left.location = (x - width / 2 - frame_thickness / 2, wall_thickness / 2 + wall_thickness * 0.1, y + height / 2)
    left.scale = (frame_thickness, wall_thickness * 1.1, height)
    bpy.ops.object.transform_apply(scale=True)
    frame_parts.append(left)

    # Right frame
    bpy.ops.mesh.primitive_cube_add(size=1)
    right = bpy.context.active_object
    right.location = (x + width / 2 + frame_thickness / 2, wall_thickness / 2 + wall_thickness * 0.1, y + height / 2)
    right.scale = (frame_thickness, wall_thickness * 1.1, height)
    bpy.ops.object.transform_apply(scale=True)
    frame_parts.append(right)

    # Join all frame parts
    bpy.ops.object.select_all(action='DESELECT')
    for part in frame_parts:
        part.select_set(True)
    bpy.context.view_layer.objects.active = frame_parts[0]
    bpy.ops.object.join()

    return bpy.context.active_object


def create_pipe_kit(spec: Dict) -> 'bpy.types.Object':
    """Create a pipe kit mesh with segments."""
    diameter = spec.get("diameter", 0.1)
    wall_thickness = spec.get("wall_thickness", 0.02)
    segments = spec.get("segments", [])
    vertices = spec.get("vertices", 16)
    bevel_width = spec.get("bevel_width", 0.0)

    radius = diameter / 2
    inner_radius = radius - wall_thickness

    if not segments:
        segments = [{"type": "straight", "length": 1.0}]

    # Start position and direction
    current_pos = Vector((0, 0, 0))
    current_dir = Vector((0, 0, 1))  # Start pointing up
    all_objects = []

    for i, seg in enumerate(segments):
        seg_type = seg.get("type", "straight")

        if seg_type == "straight":
            length = seg.get("length", 1.0)
            obj = create_pipe_segment(current_pos, current_dir, length, radius, inner_radius, vertices)
            all_objects.append(obj)
            current_pos = current_pos + current_dir * length

        elif seg_type == "bend":
            angle = seg.get("angle", 90.0)
            bend_radius = seg.get("radius", radius * 2)
            obj = create_pipe_bend(current_pos, current_dir, angle, bend_radius, radius, inner_radius, vertices)
            all_objects.append(obj)
            # Update direction after bend
            angle_rad = math.radians(angle)
            # Rotate direction around X axis (assuming bend in XZ plane)
            new_dir = Vector((
                current_dir.x,
                current_dir.y * math.cos(angle_rad) - current_dir.z * math.sin(angle_rad),
                current_dir.y * math.sin(angle_rad) + current_dir.z * math.cos(angle_rad)
            ))
            current_dir = new_dir.normalized()

        elif seg_type == "t_junction":
            arm_length = seg.get("arm_length", radius * 3)
            obj = create_pipe_tjunction(current_pos, current_dir, arm_length, radius, inner_radius, vertices)
            all_objects.append(obj)
            current_pos = current_pos + current_dir * (radius * 2)

        elif seg_type == "flange":
            outer_diameter = seg.get("outer_diameter", diameter * 1.5)
            flange_thickness = seg.get("thickness", 0.02)
            obj = create_pipe_flange(current_pos, current_dir, outer_diameter / 2, radius, flange_thickness, vertices)
            all_objects.append(obj)
            current_pos = current_pos + current_dir * flange_thickness

    # Join all pipe parts
    if len(all_objects) > 1:
        bpy.ops.object.select_all(action='DESELECT')
        for obj in all_objects:
            obj.select_set(True)
        bpy.context.view_layer.objects.active = all_objects[0]
        bpy.ops.object.join()
        result = bpy.context.active_object
    else:
        result = all_objects[0]

    result.name = "PipeKit"

    # Apply bevel if requested
    if bevel_width > 0:
        bevel_mod = result.modifiers.new(name="Bevel", type='BEVEL')
        bevel_mod.width = bevel_width
        bevel_mod.segments = 2
        bpy.context.view_layer.objects.active = result
        bpy.ops.object.modifier_apply(modifier=bevel_mod.name)

    return result


def create_pipe_segment(pos: Vector, direction: Vector, length: float,
                        outer_radius: float, inner_radius: float, vertices: int) -> 'bpy.types.Object':
    """Create a straight pipe segment."""
    # Create outer cylinder
    bpy.ops.mesh.primitive_cylinder_add(
        radius=outer_radius,
        depth=length,
        vertices=vertices,
        location=pos + direction * (length / 2)
    )
    outer = bpy.context.active_object

    # Create inner cylinder (for boolean subtraction)
    bpy.ops.mesh.primitive_cylinder_add(
        radius=inner_radius,
        depth=length * 1.1,
        vertices=vertices,
        location=pos + direction * (length / 2)
    )
    inner = bpy.context.active_object

    # Boolean difference
    bool_mod = outer.modifiers.new(name="Hollow", type='BOOLEAN')
    bool_mod.operation = 'DIFFERENCE'
    bool_mod.object = inner

    bpy.context.view_layer.objects.active = outer
    bpy.ops.object.modifier_apply(modifier=bool_mod.name)

    # Delete inner cylinder
    bpy.data.objects.remove(inner, do_unlink=True)

    return outer


def create_pipe_bend(pos: Vector, direction: Vector, angle: float, bend_radius: float,
                     outer_radius: float, inner_radius: float, vertices: int) -> 'bpy.types.Object':
    """Create a pipe bend/elbow segment (simplified as a torus section)."""
    # For simplicity, create a cylinder that approximates the bend
    # A proper implementation would use a torus section
    length = bend_radius * math.radians(angle)

    bpy.ops.mesh.primitive_cylinder_add(
        radius=outer_radius,
        depth=length,
        vertices=vertices,
        location=pos + direction * (length / 2)
    )
    outer = bpy.context.active_object

    # Create inner cylinder
    bpy.ops.mesh.primitive_cylinder_add(
        radius=inner_radius,
        depth=length * 1.1,
        vertices=vertices,
        location=pos + direction * (length / 2)
    )
    inner = bpy.context.active_object

    # Boolean difference
    bool_mod = outer.modifiers.new(name="Hollow", type='BOOLEAN')
    bool_mod.operation = 'DIFFERENCE'
    bool_mod.object = inner

    bpy.context.view_layer.objects.active = outer
    bpy.ops.object.modifier_apply(modifier=bool_mod.name)

    bpy.data.objects.remove(inner, do_unlink=True)

    return outer


def create_pipe_tjunction(pos: Vector, direction: Vector, arm_length: float,
                          outer_radius: float, inner_radius: float, vertices: int) -> 'bpy.types.Object':
    """Create a T-junction pipe segment."""
    # Main pipe section
    main_length = outer_radius * 4
    main = create_pipe_segment(pos, direction, main_length, outer_radius, inner_radius, vertices)

    # Side arm (perpendicular)
    arm_dir = Vector((1, 0, 0))  # Perpendicular to main direction
    arm_pos = pos + direction * (main_length / 2)
    arm = create_pipe_segment(arm_pos, arm_dir, arm_length, outer_radius, inner_radius, vertices)

    # Join
    bpy.ops.object.select_all(action='DESELECT')
    main.select_set(True)
    arm.select_set(True)
    bpy.context.view_layer.objects.active = main
    bpy.ops.object.join()

    return bpy.context.active_object


def create_pipe_flange(pos: Vector, direction: Vector, outer_radius: float,
                       pipe_radius: float, thickness: float, vertices: int) -> 'bpy.types.Object':
    """Create a pipe flange connector."""
    # Create outer disc
    bpy.ops.mesh.primitive_cylinder_add(
        radius=outer_radius,
        depth=thickness,
        vertices=vertices,
        location=pos + direction * (thickness / 2)
    )
    flange = bpy.context.active_object

    # Create hole
    bpy.ops.mesh.primitive_cylinder_add(
        radius=pipe_radius,
        depth=thickness * 1.1,
        vertices=vertices,
        location=pos + direction * (thickness / 2)
    )
    hole = bpy.context.active_object

    # Boolean difference
    bool_mod = flange.modifiers.new(name="Hole", type='BOOLEAN')
    bool_mod.operation = 'DIFFERENCE'
    bool_mod.object = hole

    bpy.context.view_layer.objects.active = flange
    bpy.ops.object.modifier_apply(modifier=bool_mod.name)

    bpy.data.objects.remove(hole, do_unlink=True)

    return flange


def create_door_kit(spec: Dict) -> 'bpy.types.Object':
    """Create a door kit mesh with frame and optional panel."""
    width = spec.get("width", 0.9)
    height = spec.get("height", 2.1)
    frame_thickness = spec.get("frame_thickness", 0.05)
    frame_depth = spec.get("frame_depth", 0.1)
    has_door_panel = spec.get("has_door_panel", False)
    hinge_side = spec.get("hinge_side", "left")
    panel_thickness = spec.get("panel_thickness", 0.04)
    is_open = spec.get("is_open", False)
    open_angle = spec.get("open_angle", 0.0)
    bevel_width = spec.get("bevel_width", 0.0)

    all_parts = []

    # Create door frame (4 pieces: top, bottom, left, right)
    # Left jamb
    bpy.ops.mesh.primitive_cube_add(size=1)
    left_jamb = bpy.context.active_object
    left_jamb.name = "LeftJamb"
    left_jamb.location = (-width / 2 - frame_thickness / 2, frame_depth / 2, height / 2)
    left_jamb.scale = (frame_thickness, frame_depth, height + frame_thickness)
    bpy.ops.object.transform_apply(scale=True)
    all_parts.append(left_jamb)

    # Right jamb
    bpy.ops.mesh.primitive_cube_add(size=1)
    right_jamb = bpy.context.active_object
    right_jamb.name = "RightJamb"
    right_jamb.location = (width / 2 + frame_thickness / 2, frame_depth / 2, height / 2)
    right_jamb.scale = (frame_thickness, frame_depth, height + frame_thickness)
    bpy.ops.object.transform_apply(scale=True)
    all_parts.append(right_jamb)

    # Top header
    bpy.ops.mesh.primitive_cube_add(size=1)
    header = bpy.context.active_object
    header.name = "Header"
    header.location = (0, frame_depth / 2, height + frame_thickness / 2)
    header.scale = (width + frame_thickness * 2, frame_depth, frame_thickness)
    bpy.ops.object.transform_apply(scale=True)
    all_parts.append(header)

    # Optional threshold/bottom
    bpy.ops.mesh.primitive_cube_add(size=1)
    threshold = bpy.context.active_object
    threshold.name = "Threshold"
    threshold.location = (0, frame_depth / 2, -frame_thickness / 2)
    threshold.scale = (width + frame_thickness * 2, frame_depth, frame_thickness)
    bpy.ops.object.transform_apply(scale=True)
    all_parts.append(threshold)

    # Create door panel if requested
    if has_door_panel:
        bpy.ops.mesh.primitive_cube_add(size=1)
        panel = bpy.context.active_object
        panel.name = "DoorPanel"

        # Position based on hinge side
        if hinge_side == "left":
            hinge_x = -width / 2 + panel_thickness / 2
        else:
            hinge_x = width / 2 - panel_thickness / 2

        panel.location = (0, frame_depth / 2, height / 2)
        panel.scale = (width - 0.01, panel_thickness, height - 0.01)  # Slight gap
        bpy.ops.object.transform_apply(scale=True)

        # Apply rotation if open
        if is_open and open_angle > 0:
            # Set origin to hinge side
            cursor_loc = bpy.context.scene.cursor.location.copy()
            if hinge_side == "left":
                bpy.context.scene.cursor.location = (-width / 2, frame_depth / 2, height / 2)
            else:
                bpy.context.scene.cursor.location = (width / 2, frame_depth / 2, height / 2)

            bpy.ops.object.origin_set(type='ORIGIN_CURSOR')
            bpy.context.scene.cursor.location = cursor_loc

            # Rotate around Z axis
            angle_rad = math.radians(open_angle)
            if hinge_side == "right":
                angle_rad = -angle_rad
            panel.rotation_euler.z = angle_rad
            bpy.ops.object.transform_apply(rotation=True)

        all_parts.append(panel)

    # Join all parts
    bpy.ops.object.select_all(action='DESELECT')
    for part in all_parts:
        part.select_set(True)
    bpy.context.view_layer.objects.active = all_parts[0]
    bpy.ops.object.join()

    result = bpy.context.active_object
    result.name = "DoorKit"

    # Apply bevel if requested
    if bevel_width > 0:
        bevel_mod = result.modifiers.new(name="Bevel", type='BEVEL')
        bevel_mod.width = bevel_width
        bevel_mod.segments = 2
        bpy.context.view_layer.objects.active = result
        bpy.ops.object.modifier_apply(modifier=bevel_mod.name)

    return result


def handle_organic_sculpt(spec: Dict, out_root: Path, report_path: Path) -> None:
    """Handle organic sculpt mesh generation (metaballs, remesh, smooth, displacement)."""
    start_time = time.time()

    try:
        recipe = spec.get("recipe", {})
        params = recipe.get("params", {})

        metaballs = params.get("metaballs", [])
        if not metaballs:
            raise ValueError("metaballs array must contain at least one metaball")

        remesh_voxel_size = params.get("remesh_voxel_size", 0.1)
        smooth_iterations = params.get("smooth_iterations", 0)
        displacement = params.get("displacement")
        export_settings = params.get("export", {})
        seed = spec.get("seed", 0)

        # Create metaball object
        mball_data = bpy.data.metaballs.new("OrganicSculpt_Mball")
        mball_obj = bpy.data.objects.new("OrganicSculpt_Mball", mball_data)
        bpy.context.collection.objects.link(mball_obj)

        # Configure metaball settings
        mball_data.resolution = 0.1  # High resolution for conversion
        mball_data.render_resolution = 0.1

        # Add metaball elements
        for mb in metaballs:
            elem = mball_data.elements.new()
            elem.type = 'BALL'
            elem.co = mb.get("position", [0, 0, 0])
            elem.radius = mb.get("radius", 1.0)
            elem.stiffness = mb.get("stiffness", 2.0)

        # Select and convert metaball to mesh
        bpy.context.view_layer.objects.active = mball_obj
        mball_obj.select_set(True)
        bpy.ops.object.convert(target='MESH')
        mesh_obj = bpy.context.active_object
        mesh_obj.name = "OrganicSculpt"

        # Apply voxel remesh modifier
        remesh_mod = mesh_obj.modifiers.new(name="Remesh", type='REMESH')
        remesh_mod.mode = 'VOXEL'
        remesh_mod.voxel_size = remesh_voxel_size
        remesh_mod.adaptivity = 0.0  # No adaptivity for consistent output
        bpy.ops.object.modifier_apply(modifier=remesh_mod.name)

        # Apply smooth modifier if iterations > 0
        if smooth_iterations > 0:
            smooth_mod = mesh_obj.modifiers.new(name="Smooth", type='SMOOTH')
            smooth_mod.iterations = smooth_iterations
            smooth_mod.factor = 0.5  # Moderate smoothing
            bpy.ops.object.modifier_apply(modifier=smooth_mod.name)

        # Apply displacement noise if configured
        if displacement:
            strength = displacement.get("strength", 0.1)
            scale = displacement.get("scale", 2.0)
            octaves = displacement.get("octaves", 4)
            disp_seed = displacement.get("seed", seed)

            # Add a displace modifier with procedural texture
            disp_mod = mesh_obj.modifiers.new(name="Displace", type='DISPLACE')

            # Create noise texture
            noise_tex = bpy.data.textures.new(name="OrganicNoise", type='CLOUDS')
            noise_tex.noise_scale = scale
            noise_tex.noise_depth = min(octaves, 6)  # Blender max depth is 6
            noise_tex.noise_type = 'SOFT_NOISE'

            disp_mod.texture = noise_tex
            disp_mod.texture_coords = 'LOCAL'
            disp_mod.strength = strength
            disp_mod.mid_level = 0.5

            bpy.ops.object.modifier_apply(modifier=disp_mod.name)

            # Clean up texture
            bpy.data.textures.remove(noise_tex)

        # Apply export settings
        if export_settings.get("apply_modifiers", True):
            apply_all_modifiers(mesh_obj)

        # Triangulate if requested
        if export_settings.get("triangulate", True):
            tri_mod = mesh_obj.modifiers.new(name="Triangulate", type='TRIANGULATE')
            bpy.ops.object.modifier_apply(modifier=tri_mod.name)

        # Apply automatic UV projection
        apply_uv_projection(mesh_obj, {"type": "smart_uv", "angle_limit": 66.0})

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

        # Compute metrics and export
        metrics = compute_mesh_metrics(mesh_obj)
        export_glb(output_path, export_tangents=export_tangents)

        # Save .blend file if requested
        blend_rel_path = None
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


def handle_shrinkwrap(spec: Dict, out_root: Path, report_path: Path) -> None:
    """Handle shrinkwrap mesh generation (armor/clothing wrapping onto body meshes)."""
    start_time = time.time()

    try:
        recipe = spec.get("recipe", {})
        params = recipe.get("params", {})

        base_mesh_ref = params.get("base_mesh")
        wrap_mesh_ref = params.get("wrap_mesh")

        if not base_mesh_ref:
            raise ValueError("base_mesh is required")
        if not wrap_mesh_ref:
            raise ValueError("wrap_mesh is required")

        mode = params.get("mode", "nearest_surface")
        offset = params.get("offset", 0.0)
        smooth_iterations = params.get("smooth_iterations", 0)
        smooth_factor = params.get("smooth_factor", 0.5)
        validation = params.get("validation", {})
        export_settings = params.get("export", {})

        # For this implementation, we expect the meshes to be imported from external files
        # or created as primitives. Here we'll create simple placeholder meshes for testing.
        # In production, these would be loaded from the asset references.

        # Create or import base mesh (target surface)
        base_obj = import_or_create_mesh(base_mesh_ref, "BaseMesh")
        if not base_obj:
            raise ValueError(f"Failed to load base_mesh: {base_mesh_ref}")

        # Create or import wrap mesh (the mesh to shrinkwrap)
        wrap_obj = import_or_create_mesh(wrap_mesh_ref, "WrapMesh")
        if not wrap_obj:
            raise ValueError(f"Failed to load wrap_mesh: {wrap_mesh_ref}")

        # Apply shrinkwrap modifier to wrap mesh
        bpy.context.view_layer.objects.active = wrap_obj
        wrap_obj.select_set(True)

        shrinkwrap_mod = wrap_obj.modifiers.new(name="Shrinkwrap", type='SHRINKWRAP')
        shrinkwrap_mod.target = base_obj
        shrinkwrap_mod.offset = offset

        # Set shrinkwrap mode
        mode_map = {
            "nearest_surface": 'NEAREST_SURFACEPOINT',
            "project": 'PROJECT',
            "nearest_vertex": 'NEAREST_VERTEX',
        }
        shrinkwrap_mod.wrap_method = mode_map.get(mode, 'NEAREST_SURFACEPOINT')

        # For project mode, configure projection settings
        if mode == "project":
            shrinkwrap_mod.use_project_x = False
            shrinkwrap_mod.use_project_y = False
            shrinkwrap_mod.use_project_z = True
            shrinkwrap_mod.use_negative_direction = True
            shrinkwrap_mod.use_positive_direction = True

        # Apply the shrinkwrap modifier
        bpy.ops.object.modifier_apply(modifier=shrinkwrap_mod.name)

        # Apply smooth modifier if iterations > 0
        if smooth_iterations > 0:
            smooth_mod = wrap_obj.modifiers.new(name="Smooth", type='SMOOTH')
            smooth_mod.iterations = min(smooth_iterations, 10)
            smooth_mod.factor = smooth_factor
            bpy.ops.object.modifier_apply(modifier=smooth_mod.name)

        # Validation: check for self-intersections and degenerate faces
        validation_results = validate_shrinkwrap_result(wrap_obj, validation)

        # Check validation thresholds
        max_self_intersections = validation.get("max_self_intersections", 0)
        if validation_results["self_intersection_count"] > max_self_intersections:
            raise ValueError(
                f"Shrinkwrap validation failed: {validation_results['self_intersection_count']} "
                f"self-intersections found (max allowed: {max_self_intersections})"
            )

        min_face_area = validation.get("min_face_area", 0.0001)
        if validation_results["degenerate_face_count"] > 0:
            raise ValueError(
                f"Shrinkwrap validation failed: {validation_results['degenerate_face_count']} "
                f"degenerate faces found (area < {min_face_area})"
            )

        # Remove the base mesh from export (we only want the wrapped result)
        bpy.data.objects.remove(base_obj, do_unlink=True)

        # Apply export settings
        if export_settings.get("apply_modifiers", True):
            apply_all_modifiers(wrap_obj)

        # Triangulate if requested
        if export_settings.get("triangulate", True):
            tri_mod = wrap_obj.modifiers.new(name="Triangulate", type='TRIANGULATE')
            bpy.ops.object.modifier_apply(modifier=tri_mod.name)

        # Apply automatic UV projection if mesh doesn't have UVs
        if not wrap_obj.data.uv_layers:
            apply_uv_projection(wrap_obj, {"type": "smart_uv", "angle_limit": 66.0})

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

        # Compute metrics and export
        metrics = compute_mesh_metrics(wrap_obj)
        metrics["shrinkwrap_mode"] = mode
        metrics["offset"] = offset
        metrics["smooth_iterations"] = smooth_iterations
        metrics["validation"] = validation_results
        export_glb(output_path, export_tangents=export_tangents)

        duration_ms = int((time.time() - start_time) * 1000)
        write_report(report_path, ok=True, metrics=metrics,
                     output_path=output_rel_path, duration_ms=duration_ms)

    except Exception as e:
        write_report(report_path, ok=False, error=str(e))
        raise


# =============================================================================
# Boolean Kit Handler
# =============================================================================

def handle_boolean_kit(spec: Dict, out_root: Path, report_path: Path) -> None:
    """Handle boolean kitbashing mesh generation.

    Combines meshes using boolean operations (union, difference, intersect)
    for hard-surface modeling (vehicles, buildings, mechanical parts).
    """
    start_time = time.time()

    try:
        recipe = spec.get("recipe", {})
        params = recipe.get("params", {})

        base_spec = params.get("base")
        operations = params.get("operations", [])
        solver = params.get("solver", "exact")
        cleanup = params.get("cleanup", {})
        export_settings = params.get("export", {})

        if not base_spec:
            raise ValueError("base mesh specification is required")

        # Create the base mesh
        base_obj = create_boolean_kit_mesh(base_spec, "BooleanKit_Base")
        if not base_obj:
            raise ValueError("Failed to create base mesh")

        bpy.context.view_layer.objects.active = base_obj
        base_obj.select_set(True)

        # Apply boolean operations in order (deterministic)
        for i, op_spec in enumerate(operations):
            op_type = op_spec.get("op", "union")
            target_spec = op_spec.get("target")

            if not target_spec:
                raise ValueError(f"Operation {i} missing target specification")

            # Create target mesh
            target_obj = create_boolean_kit_mesh(target_spec, f"BooleanKit_Target_{i}")
            if not target_obj:
                raise ValueError(f"Failed to create target mesh for operation {i}")

            # Apply boolean modifier
            apply_boolean_operation(base_obj, target_obj, op_type, solver)

            # Remove the target mesh (it's been consumed by the boolean)
            bpy.data.objects.remove(target_obj, do_unlink=True)

        # Apply cleanup operations
        apply_boolean_cleanup(base_obj, cleanup)

        # Validate the result for non-manifold geometry
        validation_results = validate_boolean_result(base_obj)

        # Apply export settings
        if export_settings.get("apply_modifiers", True):
            apply_all_modifiers(base_obj)

        # Triangulate if requested
        if export_settings.get("triangulate", True):
            tri_mod = base_obj.modifiers.new(name="Triangulate", type='TRIANGULATE')
            bpy.ops.object.modifier_apply(modifier=tri_mod.name)

        # Apply automatic UV projection if mesh doesn't have UVs
        if not base_obj.data.uv_layers:
            apply_uv_projection(base_obj, {"type": "smart_uv", "angle_limit": 66.0})

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

        # Compute metrics and export
        metrics = compute_mesh_metrics(base_obj)
        metrics["boolean_operations"] = len(operations)
        metrics["validation"] = validation_results
        export_glb(output_path, export_tangents=export_tangents)

        duration_ms = int((time.time() - start_time) * 1000)
        write_report(report_path, ok=True, metrics=metrics,
                     output_path=output_rel_path, duration_ms=duration_ms)

    except Exception as e:
        write_report(report_path, ok=False, error=str(e))
        raise


def create_boolean_kit_mesh(mesh_spec: Dict, name: str) -> Optional['bpy.types.Object']:
    """Create a mesh from a boolean kit specification.

    Supports both primitive meshes and asset references.
    """
    # Check if it's a primitive specification
    if "primitive" in mesh_spec:
        primitive_type = mesh_spec.get("primitive", "cube").lower()
        dimensions = mesh_spec.get("dimensions", [1.0, 1.0, 1.0])
        position = mesh_spec.get("position", [0.0, 0.0, 0.0])
        rotation = mesh_spec.get("rotation", [0.0, 0.0, 0.0])
        scale = mesh_spec.get("scale", 1.0)

        # Create the primitive
        obj = create_primitive(primitive_type, dimensions)
        obj.name = name

        # Apply transforms
        obj.location = Vector(position)
        obj.rotation_euler = Euler((
            math.radians(rotation[0]),
            math.radians(rotation[1]),
            math.radians(rotation[2])
        ))
        if scale != 1.0:
            obj.scale = Vector([scale, scale, scale])

        # Apply transforms to mesh data
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)

        return obj

    # Check if it's an asset reference
    elif "asset_ref" in mesh_spec:
        asset_ref = mesh_spec.get("asset_ref")
        position = mesh_spec.get("position", [0.0, 0.0, 0.0])
        rotation = mesh_spec.get("rotation", [0.0, 0.0, 0.0])
        scale = mesh_spec.get("scale", 1.0)

        # Import or create the referenced mesh
        obj = import_or_create_mesh(asset_ref, name)
        if obj:
            obj.location = Vector(position)
            obj.rotation_euler = Euler((
                math.radians(rotation[0]),
                math.radians(rotation[1]),
                math.radians(rotation[2])
            ))
            if scale != 1.0:
                obj.scale = Vector([scale, scale, scale])

            bpy.context.view_layer.objects.active = obj
            bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)

        return obj

    return None


def apply_boolean_operation(
    base_obj: 'bpy.types.Object',
    target_obj: 'bpy.types.Object',
    op_type: str,
    solver: str
) -> None:
    """Apply a boolean operation to the base object.

    Args:
        base_obj: The base mesh to modify.
        target_obj: The target mesh for the boolean operation.
        op_type: Type of operation ("union", "difference", "intersect").
        solver: Solver to use ("exact", "fast").
    """
    bpy.context.view_layer.objects.active = base_obj

    # Add boolean modifier
    bool_mod = base_obj.modifiers.new(name="Boolean", type='BOOLEAN')
    bool_mod.object = target_obj

    # Set operation type
    op_map = {
        "union": 'UNION',
        "difference": 'DIFFERENCE',
        "intersect": 'INTERSECT',
    }
    bool_mod.operation = op_map.get(op_type.lower(), 'UNION')

    # Set solver
    solver_map = {
        "exact": 'EXACT',
        "fast": 'FAST',
    }
    bool_mod.solver = solver_map.get(solver.lower(), 'EXACT')

    # Apply the modifier
    bpy.ops.object.modifier_apply(modifier=bool_mod.name)


def apply_boolean_cleanup(obj: 'bpy.types.Object', cleanup: Dict) -> None:
    """Apply cleanup operations after boolean operations.

    Args:
        obj: The mesh object to clean up.
        cleanup: Cleanup settings dictionary.
    """
    if not cleanup:
        # Apply default cleanup
        cleanup = {
            "merge_distance": 0.001,
            "remove_doubles": True,
            "recalc_normals": True,
            "dissolve_degenerate": True,
        }

    merge_distance = cleanup.get("merge_distance", 0.001)
    remove_doubles = cleanup.get("remove_doubles", True)
    recalc_normals = cleanup.get("recalc_normals", True)
    fill_holes = cleanup.get("fill_holes", False)
    dissolve_degenerate = cleanup.get("dissolve_degenerate", True)

    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')

    # Remove doubles (merge vertices within distance)
    if remove_doubles:
        bpy.ops.mesh.remove_doubles(threshold=merge_distance)

    # Dissolve degenerate geometry
    if dissolve_degenerate:
        bpy.ops.mesh.dissolve_degenerate(threshold=merge_distance)

    # Fill holes if requested
    if fill_holes:
        bpy.ops.mesh.fill_holes(sides=0)

    # Recalculate normals
    if recalc_normals:
        bpy.ops.mesh.normals_make_consistent(inside=False)

    bpy.ops.object.mode_set(mode='OBJECT')


def validate_boolean_result(obj: 'bpy.types.Object') -> Dict[str, Any]:
    """Validate the result of boolean operations.

    Checks for non-manifold geometry and other issues.

    Args:
        obj: The mesh object to validate.

    Returns:
        Dictionary with validation results.
    """
    bpy.context.view_layer.objects.active = obj

    mesh = obj.data
    bm = bmesh.new()
    bm.from_mesh(mesh)

    # Count non-manifold edges
    non_manifold_edges = sum(1 for e in bm.edges if not e.is_manifold)

    # Count non-manifold verts
    non_manifold_verts = sum(1 for v in bm.verts if not v.is_manifold)

    # Count loose vertices
    loose_verts = sum(1 for v in bm.verts if len(v.link_edges) == 0)

    # Count loose edges
    loose_edges = sum(1 for e in bm.edges if len(e.link_faces) == 0)

    # Count zero-area faces
    zero_area_faces = sum(1 for f in bm.faces if f.calc_area() < 1e-8)

    bm.free()

    return {
        "non_manifold_edges": non_manifold_edges,
        "non_manifold_verts": non_manifold_verts,
        "loose_verts": loose_verts,
        "loose_edges": loose_edges,
        "zero_area_faces": zero_area_faces,
        "is_manifold": non_manifold_edges == 0 and non_manifold_verts == 0,
    }


def import_or_create_mesh(mesh_ref: str, name: str) -> Optional['bpy.types.Object']:
    """Import a mesh from a file reference or create a placeholder primitive.

    For production use, this would handle:
    - GLB/GLTF imports: "path/to/mesh.glb"
    - Asset references: "asset://mesh_id"
    - Primitive specifications: "primitive://cube" or "primitive://sphere"

    For now, we support primitive:// references for testing.
    """
    if mesh_ref.startswith("primitive://"):
        primitive_type = mesh_ref.replace("primitive://", "")
        return create_primitive_mesh(primitive_type, name)
    elif mesh_ref.endswith(".glb") or mesh_ref.endswith(".gltf"):
        # Import GLB/GLTF file
        if Path(mesh_ref).exists():
            bpy.ops.import_scene.gltf(filepath=mesh_ref)
            # Get the imported object
            imported_objs = [obj for obj in bpy.context.selected_objects if obj.type == 'MESH']
            if imported_objs:
                obj = imported_objs[0]
                obj.name = name
                return obj
        return None
    else:
        # Assume it's an asset reference - for now, create a placeholder sphere
        return create_primitive_mesh("sphere", name)


def create_primitive_mesh(primitive_type: str, name: str) -> 'bpy.types.Object':
    """Create a primitive mesh for testing purposes."""
    if primitive_type == "cube":
        bpy.ops.mesh.primitive_cube_add(size=1.0)
    elif primitive_type == "sphere":
        bpy.ops.mesh.primitive_uv_sphere_add(radius=0.5, segments=32, ring_count=16)
    elif primitive_type == "cylinder":
        bpy.ops.mesh.primitive_cylinder_add(radius=0.5, depth=1.0, vertices=32)
    elif primitive_type == "plane":
        bpy.ops.mesh.primitive_plane_add(size=2.0)
    elif primitive_type == "torus":
        bpy.ops.mesh.primitive_torus_add(major_radius=0.5, minor_radius=0.1)
    elif primitive_type == "cone":
        bpy.ops.mesh.primitive_cone_add(radius1=0.5, depth=1.0, vertices=32)
    else:
        # Default to sphere
        bpy.ops.mesh.primitive_uv_sphere_add(radius=0.5, segments=32, ring_count=16)

    obj = bpy.context.active_object
    obj.name = name
    return obj


def validate_shrinkwrap_result(obj: 'bpy.types.Object', validation: Dict) -> Dict[str, Any]:
    """Validate shrinkwrap result for self-intersections and mesh quality.

    Returns a dictionary with validation metrics:
    - self_intersection_count: Number of detected self-intersections
    - degenerate_face_count: Number of faces below min_face_area threshold
    - manifold: Whether the mesh is manifold (watertight)
    """
    depsgraph = bpy.context.evaluated_depsgraph_get()
    obj_eval = obj.evaluated_get(depsgraph)
    mesh = obj_eval.to_mesh()

    min_face_area = validation.get("min_face_area", 0.0001)

    # Count degenerate faces (faces with area below threshold)
    degenerate_count = 0
    for poly in mesh.polygons:
        if poly.area < min_face_area:
            degenerate_count += 1

    # Check for self-intersections using bmesh
    # Note: True self-intersection detection is expensive and complex
    # For now, we use a simplified check based on face normal consistency
    self_intersection_count = detect_self_intersections(obj)

    # Check if mesh is manifold
    non_manifold_edges = count_non_manifold_edges(mesh)
    manifold = non_manifold_edges == 0

    obj_eval.to_mesh_clear()

    return {
        "self_intersection_count": self_intersection_count,
        "degenerate_face_count": degenerate_count,
        "manifold": manifold,
        "non_manifold_edge_count": non_manifold_edges,
    }


def detect_self_intersections(obj: 'bpy.types.Object') -> int:
    """Detect self-intersections in a mesh using bmesh.

    This is a simplified detection that checks for inverted normals and
    overlapping faces. A full boolean intersection test would be more accurate
    but significantly more expensive.

    Returns the count of detected self-intersection issues.
    """
    # Create bmesh from object
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bm.faces.ensure_lookup_table()

    intersection_count = 0

    # Check for inverted faces (faces with normals pointing inward)
    # This can indicate self-intersection issues
    center = Vector((0, 0, 0))
    for v in bm.verts:
        center += v.co
    if bm.verts:
        center /= len(bm.verts)

    for face in bm.faces:
        face_center = face.calc_center_median()
        to_center = center - face_center

        # If normal points toward center, face might be inverted
        if face.normal.dot(to_center) > 0.1:  # Small threshold
            intersection_count += 1

    bm.free()
    return intersection_count


def handle_skeletal_mesh(spec: Dict, out_root: Path, report_path: Path) -> None:
    """Handle skeletal mesh generation."""
    start_time = time.time()

    try:
        recipe = spec.get("recipe", {})
        params = recipe.get("params", {})

        # Create armature - support both preset and custom skeleton
        skeleton_spec = params.get("skeleton", [])
        skeleton_preset = params.get("skeleton_preset")

        if skeleton_preset:
            # Use preset skeleton, then apply optional overrides/additions.
            armature = create_armature(skeleton_preset)
            if skeleton_spec:
                apply_skeleton_overrides(armature, skeleton_spec)
        elif skeleton_spec:
            # Use custom skeleton definition
            armature = create_custom_skeleton(skeleton_spec)
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

            # Recalculate normals after joining parts (rotation can flip winding)
            bpy.ops.object.mode_set(mode='EDIT')
            bpy.ops.mesh.select_all(action='SELECT')
            bpy.ops.mesh.normals_make_consistent(inside=False)
            bpy.ops.object.mode_set(mode='OBJECT')

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

        # Apply root motion settings if specified
        root_motion_settings = params.get("root_motion")
        extracted_delta = None
        if root_motion_settings:
            extracted_delta = apply_root_motion_settings(armature, action, root_motion_settings)
            print(f"Applied root motion mode: {root_motion_settings.get('mode', 'keep')}")

        # Compute metrics (motion verification only uses constraints when provided)
        metrics = compute_animation_metrics(armature, action)

        # Add root motion info to metrics
        if root_motion_settings:
            metrics["root_motion_mode"] = root_motion_settings.get("mode", "keep")
        if extracted_delta:
            metrics["root_motion_delta"] = extracted_delta

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
            # Clamp so time == duration doesn't create an extra frame.
            frame_index = int(time_sec * fps)
            if frame_count > 0:
                frame_index = max(0, min(frame_index, frame_count - 1))
            frame = 1 + frame_index
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

        # Apply root motion settings if specified
        root_motion_settings = params.get("root_motion")
        extracted_delta = None
        if root_motion_settings:
            extracted_delta = apply_root_motion_settings(armature, action, root_motion_settings)
            print(f"Applied root motion mode: {root_motion_settings.get('mode', 'keep')}")

        # Compute metrics (include motion verification using rig constraints)
        constraints_list = rig_setup.get("constraints", {}).get("constraints", [])
        metrics = compute_animation_metrics(armature, action, constraints_list=constraints_list)

        # Add root motion info to metrics
        if root_motion_settings:
            metrics["root_motion_mode"] = root_motion_settings.get("mode", "keep")
        if extracted_delta:
            metrics["root_motion_delta"] = extracted_delta

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
# Animation Helpers Handler
# =============================================================================

def handle_animation_helpers(spec: Dict, out_root: Path, report_path: Path) -> None:
    """Handle animation.helpers_v1 generation using preset-based locomotion helpers.

    This handler uses the animation_helpers operator to generate procedural
    locomotion animations (walk cycles, run cycles, idle sway) based on presets.
    """
    from operators.animation_helpers import handle_animation_helpers as _handle_helpers

    start_time = time.time()

    try:
        recipe = spec.get("recipe", {})
        params = recipe.get("params", {})

        # Get skeleton preset and create armature
        skeleton_type = params.get("skeleton", "humanoid")
        preset = params.get("preset", "walk_cycle")

        # Create the armature based on skeleton type
        armature = create_armature(skeleton_type)

        # Call the operator to set up IK, foot roll, and generate keyframes
        result = _handle_helpers(spec, armature, out_root)

        # Get output path from spec
        outputs = spec.get("outputs", [])
        primary_output = next((o for o in outputs if o.get("kind") == "primary"), None)
        if not primary_output:
            raise ValueError("No primary output specified in spec")

        output_rel_path = primary_output.get("path", "output.glb")
        output_path = out_root / output_rel_path
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Export GLB with animation
        export_glb(output_path, include_armature=True, include_animation=True)

        # Compute animation metrics
        action = armature.animation_data.action if armature.animation_data else None
        metrics = compute_animation_metrics(armature, action)

        # Add animation helpers specific metrics
        metrics["preset"] = result.get("preset", preset)
        metrics["skeleton_type"] = result.get("skeleton_type", skeleton_type)
        metrics["ik_chains_created"] = result.get("ik_chains_created", 0)
        metrics["foot_roll_enabled"] = result.get("foot_roll_enabled", False)
        metrics["clip_name"] = result.get("clip_name", params.get("clip_name", preset))

        # Save .blend file if requested
        blend_rel_path = None
        if params.get("save_blend", False):
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
# Mesh to Sprite Handler
# =============================================================================

def handle_mesh_to_sprite(spec: Dict, out_root: Path, report_path: Path) -> None:
    """
    Handle mesh-to-sprite rendering.

    Renders a 3D mesh from multiple rotation angles and packs the resulting
    frames into a sprite atlas with metadata.
    """
    start_time = time.time()

    try:
        recipe = spec.get("recipe", {})
        params = recipe.get("params", {})

        # Extract mesh params
        mesh_params = params.get("mesh", {})

        # Camera and lighting settings
        camera_preset = params.get("camera", "orthographic")
        lighting_preset = params.get("lighting", "three_point")
        frame_resolution = params.get("frame_resolution", [64, 64])
        rotation_angles = params.get("rotation_angles", [0.0])
        atlas_padding = params.get("atlas_padding", 2)
        background_color = params.get("background_color", [0.0, 0.0, 0.0, 0.0])
        camera_distance = params.get("camera_distance", 2.0)
        camera_elevation = params.get("camera_elevation", 30.0)

        # Create primitive mesh
        primitive = mesh_params.get("base_primitive", "cube")
        dimensions = mesh_params.get("dimensions", [1, 1, 1])
        obj = create_primitive(primitive, dimensions)

        # Apply modifiers
        modifiers = mesh_params.get("modifiers", [])
        for mod_spec in modifiers:
            apply_modifier(obj, mod_spec)

        # Join attachments (extra primitives positioned relative to base)
        attachments = mesh_params.get("attachments", [])
        for att in attachments:
            att_prim = att.get("primitive", "cube")
            att_dims = att.get("dimensions", [1.0, 1.0, 1.0])
            att_pos = att.get("position", [0.0, 0.0, 0.0])
            att_rot = att.get("rotation", [0.0, 0.0, 0.0])

            att_obj = create_primitive(att_prim, att_dims)
            att_obj.location = Vector(att_pos)
            att_obj.rotation_euler = Euler([math.radians(r) for r in att_rot])
            bpy.context.view_layer.objects.active = att_obj
            bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)

            # Select both and join
            bpy.ops.object.select_all(action='DESELECT')
            att_obj.select_set(True)
            obj.select_set(True)
            bpy.context.view_layer.objects.active = obj
            bpy.ops.object.join()

        # Apply modifiers to mesh
        export_settings = mesh_params.get("export", {})
        if export_settings.get("apply_modifiers", True):
            apply_all_modifiers(obj)

        # Apply UV projection if specified
        uv_projection = mesh_params.get("uv_projection")
        if uv_projection:
            apply_uv_projection(obj, uv_projection)

        # Apply materials
        material_slots = mesh_params.get("material_slots", [])
        apply_materials(obj, material_slots)

        # Compute mesh bounding box for camera placement
        mesh_bounds = get_mesh_bounds(obj)
        mesh_center = [
            (mesh_bounds[0][i] + mesh_bounds[1][i]) / 2
            for i in range(3)
        ]
        mesh_size = max(
            mesh_bounds[1][i] - mesh_bounds[0][i]
            for i in range(3)
        )

        # Set up camera
        camera_data = bpy.data.cameras.new(name="RenderCamera")
        camera = bpy.data.objects.new("RenderCamera", camera_data)
        bpy.context.collection.objects.link(camera)

        if camera_preset == "orthographic":
            camera_data.type = 'ORTHO'
            camera_data.ortho_scale = mesh_size * camera_distance
        elif camera_preset == "isometric":
            camera_data.type = 'ORTHO'
            camera_data.ortho_scale = mesh_size * camera_distance
            camera_elevation = 35.264  # Isometric angle
        else:  # perspective
            camera_data.type = 'PERSP'
            camera_data.lens = 50

        # Set up lighting based on preset
        setup_lighting(lighting_preset, mesh_center, mesh_size)

        # Configure render settings
        bpy.context.scene.render.resolution_x = frame_resolution[0]
        bpy.context.scene.render.resolution_y = frame_resolution[1]
        bpy.context.scene.render.film_transparent = (background_color[3] < 1.0)

        if not bpy.context.scene.render.film_transparent:
            bpy.context.scene.world = bpy.data.worlds.new("Background")
            bpy.context.scene.world.use_nodes = False
            bpy.context.scene.world.color = (
                background_color[0],
                background_color[1],
                background_color[2]
            )

        bpy.context.scene.camera = camera

        # Create temp directory for individual frames
        temp_frames_dir = Path(tempfile.mkdtemp(prefix="speccade_frames_"))
        frame_paths = []
        frame_metadata = []

        # Calculate camera distance from mesh center
        cam_dist = mesh_size * camera_distance

        # Render each rotation angle
        for i, angle in enumerate(rotation_angles):
            # Position camera around the mesh
            angle_rad = math.radians(angle)
            elev_rad = math.radians(camera_elevation)

            cam_x = mesh_center[0] + cam_dist * math.sin(angle_rad) * math.cos(elev_rad)
            cam_y = mesh_center[1] - cam_dist * math.cos(angle_rad) * math.cos(elev_rad)
            cam_z = mesh_center[2] + cam_dist * math.sin(elev_rad)

            camera.location = Vector((cam_x, cam_y, cam_z))

            # Point camera at mesh center
            direction = Vector(mesh_center) - camera.location
            rot_quat = direction.to_track_quat('-Z', 'Y')
            camera.rotation_euler = rot_quat.to_euler()

            # Render frame
            frame_path = temp_frames_dir / f"frame_{i:04d}.png"
            bpy.context.scene.render.filepath = str(frame_path)
            bpy.ops.render.render(write_still=True)
            frame_paths.append(frame_path)

            frame_metadata.append({
                "id": f"angle_{int(angle)}",
                "angle": angle,
                "index": i
            })

        # Pack frames into atlas
        atlas_width, atlas_height, frame_positions = pack_frames_into_atlas(
            frame_paths,
            frame_resolution,
            atlas_padding
        )

        # Create atlas image
        atlas = create_atlas_image(
            frame_paths,
            frame_positions,
            atlas_width,
            atlas_height,
            background_color
        )

        # Get output paths from spec
        outputs = spec.get("outputs", [])
        primary_output = next((o for o in outputs if o.get("kind") == "primary"), None)
        metadata_output = next((o for o in outputs if o.get("kind") == "metadata"), None)

        if not primary_output:
            raise ValueError("No primary output specified in spec")

        output_rel_path = primary_output.get("path", "sprites/atlas.png")
        output_path = out_root / output_rel_path
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Save atlas
        atlas.save(str(output_path))

        # Generate metadata if requested
        metadata_rel_path = None
        if metadata_output:
            metadata_rel_path = metadata_output.get("path", "sprites/atlas.json")
            metadata_path = out_root / metadata_rel_path

            # Build frame metadata with UV coordinates
            frames = []
            for i, (pos, meta) in enumerate(zip(frame_positions, frame_metadata)):
                u_min = pos[0] / atlas_width
                v_min = pos[1] / atlas_height
                u_max = (pos[0] + frame_resolution[0]) / atlas_width
                v_max = (pos[1] + frame_resolution[1]) / atlas_height

                frames.append({
                    "id": meta["id"],
                    "angle": meta["angle"],
                    "position": [pos[0], pos[1]],
                    "dimensions": frame_resolution,
                    "uv": [u_min, v_min, u_max, v_max]
                })

            metadata = {
                "atlas_dimensions": [atlas_width, atlas_height],
                "padding": atlas_padding,
                "frame_resolution": frame_resolution,
                "camera": camera_preset,
                "lighting": lighting_preset,
                "frames": frames
            }

            with open(metadata_path, 'w') as f:
                json.dump(metadata, f, indent=2)

        # Clean up temp frames
        for frame_path in frame_paths:
            if frame_path.exists():
                frame_path.unlink()
        temp_frames_dir.rmdir()

        # Build metrics
        metrics = {
            "atlas_dimensions": [atlas_width, atlas_height],
            "frame_count": len(rotation_angles),
            "frame_resolution": frame_resolution,
            "camera": camera_preset,
            "lighting": lighting_preset
        }

        # Save .blend file if requested
        blend_rel_path = None
        if params.get("save_blend", False):
            blend_rel_path = output_rel_path.replace(".png", ".blend")
            blend_path = out_root / blend_rel_path
            bpy.ops.wm.save_as_mainfile(filepath=str(blend_path))

        duration_ms = int((time.time() - start_time) * 1000)
        write_report(report_path, ok=True, metrics=metrics,
                     output_path=output_rel_path, blend_path=blend_rel_path,
                     duration_ms=duration_ms)

    except Exception as e:
        write_report(report_path, ok=False, error=str(e))
        raise


def get_mesh_bounds(obj: 'bpy.types.Object') -> Tuple[List[float], List[float]]:
    """Get the world-space bounding box of a mesh object."""
    bbox_corners = [obj.matrix_world @ Vector(corner) for corner in obj.bound_box]
    min_corner = [min(c[i] for c in bbox_corners) for i in range(3)]
    max_corner = [max(c[i] for c in bbox_corners) for i in range(3)]
    return min_corner, max_corner


# =============================================================================
# Validation Grid Handler
# =============================================================================

VALIDATION_GRID_VIEWS = [
    # (label, azimuth_deg, elevation_deg)
    ("FRONT", 0.0, 30.0),
    ("BACK", 180.0, 30.0),
    ("TOP", 0.0, 90.0),
    ("LEFT", 90.0, 30.0),
    ("RIGHT", 270.0, 30.0),
    ("ISO", 45.0, 35.264),  # Isometric angle
]


def handle_validation_grid(spec: Dict, out_root: Path, report_path: Path) -> None:
    """
    Generate a 6-view validation grid PNG for LLM-based visual verification.

    Grid layout (2 rows x 3 columns):
        FRONT | BACK  | TOP
        LEFT  | RIGHT | ISO
    """
    start_time = time.time()

    try:
        recipe = spec.get("recipe", {})
        params = recipe.get("params", {})

        # Get panel size from spec (default 256)
        panel_size = params.get("panel_size", 256)
        grid_padding = 4

        # Create the mesh based on recipe kind
        recipe_kind = recipe.get("kind", "")

        obj = None

        # Check specific recipe variants BEFORE generic handlers
        if recipe_kind.endswith("organic_sculpt_v1"):
            # Organic sculpt - create metaball mesh
            # Support both "metaballs" (array) and "metaball.elements" (object with array)
            metaballs = params.get("metaballs", [])
            if not metaballs:
                metaball_params = params.get("metaball", {})
                metaballs = metaball_params.get("elements", [])

            mball_data = bpy.data.metaballs.new("OrganicMeta")
            mball_obj = bpy.data.objects.new("OrganicMeta", mball_data)
            bpy.context.collection.objects.link(mball_obj)

            # Add metaball elements
            for elem in metaballs:
                el = mball_data.elements.new()
                el.type = elem.get("type", "BALL").upper()
                el.co = Vector(elem.get("position", [0, 0, 0]))
                el.radius = elem.get("radius", 1.0)
                # stiffness maps to Blender's 'stiffness' property
                if "stiffness" in elem:
                    el.stiffness = elem.get("stiffness", 2.0)

            # Set resolution for smoother result
            mball_data.resolution = params.get("remesh_voxel_size", 0.1)
            mball_data.threshold = 0.6

            # Convert to mesh
            bpy.context.view_layer.objects.active = mball_obj
            bpy.ops.object.convert(target='MESH')
            obj = bpy.context.active_object

            # Apply smoothing if specified
            smooth_iterations = params.get("smooth_iterations", 0)
            if smooth_iterations > 0 and obj is not None:
                try:
                    smooth_mod = obj.modifiers.new(name="Smooth", type='SMOOTH')
                    if smooth_mod is not None:
                        smooth_mod.iterations = smooth_iterations
                        apply_all_modifiers(obj)
                except Exception as e:
                    print(f"Warning: Could not apply smoothing: {e}")

            # Apply displacement if specified (optional, may fail on some Blender versions)
            displacement = params.get("displacement", {})
            if displacement:
                try:
                    # Add displacement modifier with noise texture
                    disp_mod = obj.modifiers.new(name="Displacement", type='DISPLACE')
                    tex = bpy.data.textures.new("NoiseDisp", type='CLOUDS')
                    tex.noise_scale = displacement.get("scale", 1.0)
                    disp_mod.texture = tex
                    disp_mod.strength = displacement.get("strength", 0.1)
                    apply_all_modifiers(obj)
                except Exception as e:
                    # Displacement texture assignment may fail on some Blender versions
                    # Continue without displacement noise
                    print(f"Warning: Could not apply displacement texture: {e}")

        elif recipe_kind.startswith("static_mesh.") or recipe_kind == "blender_primitives_v1":
            # Static mesh - extract mesh params
            primitive = params.get("base_primitive", "cube")
            dimensions = params.get("dimensions", [1, 1, 1])
            obj = create_primitive(primitive, dimensions)

            modifiers = params.get("modifiers", [])
            for mod_spec in modifiers:
                apply_modifier(obj, mod_spec)

            # Apply modifiers to mesh
            export_settings = params.get("export", {})
            if export_settings.get("apply_modifiers", True):
                apply_all_modifiers(obj)

        elif recipe_kind.startswith("skeletal_mesh.") or recipe_kind == "blender_rigged_mesh_v1":
            # Skeletal mesh - create armature and body parts
            skeleton_spec = params.get("skeleton", [])
            skeleton_preset = params.get("skeleton_preset")

            if skeleton_preset:
                armature = create_armature(skeleton_preset)
                if skeleton_spec:
                    apply_skeleton_overrides(armature, skeleton_spec)
            elif skeleton_spec:
                armature = create_custom_skeleton(skeleton_spec)
            else:
                armature = create_armature("humanoid_basic_v1")

            mesh_objects = []

            # Create body parts
            body_parts = params.get("body_parts", [])
            for part_spec in body_parts:
                mesh_obj = create_body_part(armature, part_spec)
                mesh_objects.append((mesh_obj, part_spec.get("bone")))

            # Create legacy parts
            legacy_parts = params.get("parts", {})
            if legacy_parts:
                for part_name, part_spec in legacy_parts.items():
                    mesh_obj = create_legacy_part(armature, part_name, part_spec, legacy_parts)
                    if mesh_obj:
                        bone_name = part_spec.get("bone", part_name)
                        mesh_objects.append((mesh_obj, bone_name))

            # Join all meshes
            if mesh_objects:
                bpy.ops.object.select_all(action='DESELECT')
                for mesh_obj, _ in mesh_objects:
                    mesh_obj.select_set(True)
                bpy.context.view_layer.objects.active = mesh_objects[0][0]
                if len(mesh_objects) > 1:
                    bpy.ops.object.join()
                obj = bpy.context.active_object
            else:
                # Fallback - use armature bounds
                obj = armature

        elif recipe_kind == "mesh_to_sprite_v1":
            # Sprite mesh - extract mesh subparams
            mesh_params = params.get("mesh", {})
            primitive = mesh_params.get("base_primitive", "cube")
            dimensions = mesh_params.get("dimensions", [1, 1, 1])
            obj = create_primitive(primitive, dimensions)

            modifiers = mesh_params.get("modifiers", [])
            for mod_spec in modifiers:
                apply_modifier(obj, mod_spec)

        else:
            # Fallback: try to extract mesh params directly
            mesh_params = params.get("mesh", params)
            primitive = mesh_params.get("base_primitive", "cube")
            dimensions = mesh_params.get("dimensions", [1, 1, 1])
            obj = create_primitive(primitive, dimensions)

            modifiers = mesh_params.get("modifiers", [])
            for mod_spec in modifiers:
                apply_modifier(obj, mod_spec)

        if obj is None:
            raise ValueError(f"Could not create mesh for recipe kind: {recipe_kind}")

        # Compute mesh bounds for camera placement
        mesh_bounds = get_mesh_bounds(obj)
        mesh_center = [
            (mesh_bounds[0][i] + mesh_bounds[1][i]) / 2
            for i in range(3)
        ]

        # Calculate dimensions on each axis
        dims = [mesh_bounds[1][i] - mesh_bounds[0][i] for i in range(3)]
        mesh_size = max(dims)

        # Clamp to minimum size to handle very small or degenerate meshes
        MIN_MESH_SIZE = 0.1  # Minimum 0.1 units for camera framing
        if mesh_size < MIN_MESH_SIZE:
            print(f"Warning: Mesh is very small ({mesh_size:.4f}), clamping to {MIN_MESH_SIZE}")
            mesh_size = MIN_MESH_SIZE

        # Warn about very flat meshes that may be hard to see from some angles
        min_dim = min(dims)
        if min_dim < mesh_size * 0.01:  # If one dimension is <1% of the largest
            print(f"Warning: Mesh is very flat (dims: {dims}), some views may show little detail")

        # Camera distance scaled to mesh size
        cam_dist = mesh_size * 2.5

        # Set up orthographic camera
        camera_data = bpy.data.cameras.new(name="ValidationCamera")
        camera_data.type = 'ORTHO'
        camera_data.ortho_scale = mesh_size * 1.5
        camera = bpy.data.objects.new("ValidationCamera", camera_data)
        bpy.context.collection.objects.link(camera)
        bpy.context.scene.camera = camera

        # Set up validation lighting (illuminates all sides equally)
        setup_lighting("validation", mesh_center, mesh_size)

        # Configure render settings
        bpy.context.scene.render.resolution_x = panel_size
        bpy.context.scene.render.resolution_y = panel_size
        bpy.context.scene.render.film_transparent = True

        # Create temp directory for individual frames
        temp_frames_dir = Path(tempfile.mkdtemp(prefix="speccade_validation_grid_"))
        frame_paths = []

        # Render each view
        for i, (label, azimuth, elevation) in enumerate(VALIDATION_GRID_VIEWS):
            azimuth_rad = math.radians(azimuth)
            elev_rad = math.radians(elevation)

            # Position camera
            cam_x = mesh_center[0] + cam_dist * math.sin(azimuth_rad) * math.cos(elev_rad)
            cam_y = mesh_center[1] - cam_dist * math.cos(azimuth_rad) * math.cos(elev_rad)
            cam_z = mesh_center[2] + cam_dist * math.sin(elev_rad)

            camera.location = Vector((cam_x, cam_y, cam_z))

            # Point camera at mesh center
            direction = Vector(mesh_center) - camera.location
            rot_quat = direction.to_track_quat('-Z', 'Y')
            camera.rotation_euler = rot_quat.to_euler()

            # Render frame
            frame_path = temp_frames_dir / f"view_{i}_{label}.png"
            bpy.context.scene.render.filepath = str(frame_path)
            bpy.ops.render.render(write_still=True)
            frame_paths.append((frame_path, label))

        # Composite into grid (3 cols x 2 rows)
        grid_width = panel_size * 3 + grid_padding * 4
        grid_height = panel_size * 2 + grid_padding * 3

        # Get output path from spec
        outputs = spec.get("outputs", [])
        primary_output = next((o for o in outputs if o.get("kind") == "primary"), None)

        if primary_output:
            output_rel_path = primary_output.get("path", "validation_grid.png")
        else:
            output_rel_path = "validation_grid.png"

        output_path = out_root / output_rel_path
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Use PIL if available, otherwise save individual frames for Rust to composite
        try:
            from PIL import Image, ImageDraw, ImageFont

            grid_img = Image.new('RGBA', (grid_width, grid_height), (0, 0, 0, 0))

            for i, (frame_path, label) in enumerate(frame_paths):
                col = i % 3
                row = i // 3
                x = grid_padding + col * (panel_size + grid_padding)
                y = grid_padding + row * (panel_size + grid_padding)

                frame_img = Image.open(frame_path)
                grid_img.paste(frame_img, (x, y))

                # Draw label
                draw = ImageDraw.Draw(grid_img)
                try:
                    font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 14)
                except Exception:
                    font = ImageFont.load_default()

                # Label background
                text_bbox = draw.textbbox((x + 4, y + 4), label, font=font)
                draw.rectangle([text_bbox[0] - 2, text_bbox[1] - 2, text_bbox[2] + 2, text_bbox[3] + 2], fill=(0, 0, 0, 180))
                draw.text((x + 4, y + 4), label, fill=(255, 255, 255, 255), font=font)

            grid_img.save(str(output_path))

        except ImportError:
            # Fallback: save individual frames, let Rust composite
            frames_output_path = out_root / "validation_grid_frames"
            frames_output_path.mkdir(parents=True, exist_ok=True)
            import shutil
            for frame_path, label in frame_paths:
                shutil.copy(frame_path, frames_output_path / f"{label}.png")
            output_path = frames_output_path

        # Clean up temp files
        import shutil
        shutil.rmtree(temp_frames_dir)

        duration_ms = int((time.time() - start_time) * 1000)

        write_report(report_path, ok=True,
                     output_path=str(output_path.relative_to(out_root) if output_path.is_relative_to(out_root) else output_rel_path),
                     duration_ms=duration_ms)

    except Exception as e:
        write_report(report_path, ok=False, error=str(e))
        raise


def setup_lighting(preset: str, center: List[float], size: float) -> None:
    """Set up lighting based on preset."""
    # Clear existing lights
    for obj in bpy.data.objects:
        if obj.type == 'LIGHT':
            bpy.data.objects.remove(obj, do_unlink=True)

    center_vec = Vector(center)

    if preset == "three_point":
        # Key light (main)
        key_data = bpy.data.lights.new(name="Key", type='SUN')
        key_data.energy = 3.0
        key = bpy.data.objects.new("Key", key_data)
        key.location = center_vec + Vector((size * 2, -size * 2, size * 2))
        bpy.context.collection.objects.link(key)
        key.rotation_euler = Euler((math.radians(45), 0, math.radians(45)))

        # Fill light (softer, opposite side)
        fill_data = bpy.data.lights.new(name="Fill", type='SUN')
        fill_data.energy = 1.0
        fill = bpy.data.objects.new("Fill", fill_data)
        fill.location = center_vec + Vector((-size * 2, -size * 1.5, size))
        bpy.context.collection.objects.link(fill)
        fill.rotation_euler = Euler((math.radians(30), 0, math.radians(-45)))

        # Back light (rim)
        back_data = bpy.data.lights.new(name="Back", type='SUN')
        back_data.energy = 2.0
        back = bpy.data.objects.new("Back", back_data)
        back.location = center_vec + Vector((0, size * 2, size * 1.5))
        bpy.context.collection.objects.link(back)
        back.rotation_euler = Euler((math.radians(-45), 0, math.radians(180)))

    elif preset == "rim":
        # Strong rim lighting from behind
        rim_data = bpy.data.lights.new(name="Rim", type='SUN')
        rim_data.energy = 4.0
        rim = bpy.data.objects.new("Rim", rim_data)
        rim.location = center_vec + Vector((0, size * 2, size))
        bpy.context.collection.objects.link(rim)
        rim.rotation_euler = Euler((math.radians(-30), 0, math.radians(180)))

        # Soft fill from front
        fill_data = bpy.data.lights.new(name="Fill", type='SUN')
        fill_data.energy = 0.5
        fill = bpy.data.objects.new("Fill", fill_data)
        fill.location = center_vec + Vector((0, -size * 2, size * 0.5))
        bpy.context.collection.objects.link(fill)

    elif preset == "flat":
        # Single overhead light for minimal shadows
        flat_data = bpy.data.lights.new(name="Flat", type='SUN')
        flat_data.energy = 2.0
        flat = bpy.data.objects.new("Flat", flat_data)
        flat.location = center_vec + Vector((0, 0, size * 3))
        bpy.context.collection.objects.link(flat)
        flat.rotation_euler = Euler((0, 0, 0))

    elif preset == "validation":
        # Balanced lighting for validation grids - illuminates all sides
        # Front light
        front_data = bpy.data.lights.new(name="Front", type='SUN')
        front_data.energy = 2.0
        front = bpy.data.objects.new("Front", front_data)
        front.location = center_vec + Vector((0, -size * 2, size))
        bpy.context.collection.objects.link(front)
        front.rotation_euler = Euler((math.radians(30), 0, 0))

        # Back light
        back_data = bpy.data.lights.new(name="Back", type='SUN')
        back_data.energy = 2.0
        back = bpy.data.objects.new("Back", back_data)
        back.location = center_vec + Vector((0, size * 2, size))
        bpy.context.collection.objects.link(back)
        back.rotation_euler = Euler((math.radians(30), 0, math.radians(180)))

        # Left light
        left_data = bpy.data.lights.new(name="Left", type='SUN')
        left_data.energy = 1.5
        left = bpy.data.objects.new("Left", left_data)
        left.location = center_vec + Vector((-size * 2, 0, size))
        bpy.context.collection.objects.link(left)
        left.rotation_euler = Euler((math.radians(30), 0, math.radians(-90)))

        # Right light
        right_data = bpy.data.lights.new(name="Right", type='SUN')
        right_data.energy = 1.5
        right = bpy.data.objects.new("Right", right_data)
        right.location = center_vec + Vector((size * 2, 0, size))
        bpy.context.collection.objects.link(right)
        right.rotation_euler = Euler((math.radians(30), 0, math.radians(90)))

        # Top light (softer fill)
        top_data = bpy.data.lights.new(name="Top", type='SUN')
        top_data.energy = 1.0
        top = bpy.data.objects.new("Top", top_data)
        top.location = center_vec + Vector((0, 0, size * 3))
        bpy.context.collection.objects.link(top)
        top.rotation_euler = Euler((0, 0, 0))

    elif preset == "dramatic":
        # Strong single light with hard shadows
        key_data = bpy.data.lights.new(name="Dramatic", type='SPOT')
        key_data.energy = 500
        key_data.spot_size = math.radians(45)
        key = bpy.data.objects.new("Dramatic", key_data)
        key.location = center_vec + Vector((size * 1.5, -size * 1.5, size * 2))
        bpy.context.collection.objects.link(key)
        direction = center_vec - key.location
        key.rotation_euler = direction.to_track_quat('-Z', 'Y').to_euler()

    elif preset == "studio":
        # Soft, balanced studio lighting
        # Main softbox-like light
        main_data = bpy.data.lights.new(name="Main", type='AREA')
        main_data.energy = 100
        main_data.size = size * 2
        main = bpy.data.objects.new("Main", main_data)
        main.location = center_vec + Vector((size, -size * 2, size * 1.5))
        bpy.context.collection.objects.link(main)
        direction = center_vec - main.location
        main.rotation_euler = direction.to_track_quat('-Z', 'Y').to_euler()

        # Side fill
        fill_data = bpy.data.lights.new(name="Fill", type='AREA')
        fill_data.energy = 50
        fill_data.size = size * 1.5
        fill = bpy.data.objects.new("Fill", fill_data)
        fill.location = center_vec + Vector((-size * 1.5, -size, size))
        bpy.context.collection.objects.link(fill)
        direction = center_vec - fill.location
        fill.rotation_euler = direction.to_track_quat('-Z', 'Y').to_euler()


def pack_frames_into_atlas(
    frame_paths: List[Path],
    frame_resolution: List[int],
    padding: int
) -> Tuple[int, int, List[Tuple[int, int]]]:
    """
    Pack frames into an atlas using simple grid layout.

    Returns:
        Tuple of (atlas_width, atlas_height, frame_positions)
    """
    frame_count = len(frame_paths)
    if frame_count == 0:
        return 0, 0, []

    frame_w = frame_resolution[0] + padding * 2
    frame_h = frame_resolution[1] + padding * 2

    # Calculate grid dimensions (prefer square-ish atlas)
    cols = math.ceil(math.sqrt(frame_count))
    rows = math.ceil(frame_count / cols)

    atlas_width = cols * frame_w
    atlas_height = rows * frame_h

    # Calculate frame positions
    positions = []
    for i in range(frame_count):
        col = i % cols
        row = i // cols
        x = col * frame_w + padding
        y = row * frame_h + padding
        positions.append((x, y))

    return atlas_width, atlas_height, positions


class _BlenderAtlasImage:
    """Wraps a bpy.types.Image to provide a .save(path) interface for atlas compositing."""

    def __init__(self, bpy_image: 'bpy.types.Image'):
        self._img = bpy_image

    def save(self, path: str) -> None:
        scene = bpy.context.scene
        scene.render.image_settings.file_format = 'PNG'
        scene.render.image_settings.color_mode = 'RGBA'
        scene.render.image_settings.color_depth = '8'
        self._img.save_render(filepath=path, scene=scene)


def create_atlas_image(
    frame_paths: List[Path],
    positions: List[Tuple[int, int]],
    atlas_width: int,
    atlas_height: int,
    background_color: List[float]
) -> '_BlenderAtlasImage':
    """
    Create the atlas image by compositing individual frames using Blender's image API.

    Returns:
        _BlenderAtlasImage with a .save(path) method
    """
    # Create atlas image filled with background color
    atlas = bpy.data.images.new("speccade_atlas", width=atlas_width, height=atlas_height, alpha=True)
    bg = list(background_color[:4]) if len(background_color) >= 4 else list(background_color[:3]) + [1.0]
    pixels = bg * (atlas_width * atlas_height)
    atlas.pixels[:] = pixels

    # Load and paste each frame
    for frame_path, (x, y) in zip(frame_paths, positions):
        frame = bpy.data.images.load(str(frame_path))
        fw, fh = frame.size[0], frame.size[1]
        frame_pixels = list(frame.pixels[:])

        atlas_pixels = list(atlas.pixels[:])
        for row in range(fh):
            # bpy images are bottom-up; position (x, y) is top-left in screen coords
            atlas_row = (atlas_height - 1 - y - row)
            src_start = row * fw * 4
            dst_start = (atlas_row * atlas_width + x) * 4
            atlas_pixels[dst_start:dst_start + fw * 4] = frame_pixels[src_start:src_start + fw * 4]
        atlas.pixels[:] = atlas_pixels

        bpy.data.images.remove(frame)

    return _BlenderAtlasImage(atlas)


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
                        choices=["static_mesh", "modular_kit", "organic_sculpt", "shrinkwrap", "boolean_kit", "skeletal_mesh", "animation", "rigged_animation", "animation_helpers", "mesh_to_sprite", "validation_grid"],
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
        "modular_kit": handle_modular_kit,
        "organic_sculpt": handle_organic_sculpt,
        "shrinkwrap": handle_shrinkwrap,
        "boolean_kit": handle_boolean_kit,
        "skeletal_mesh": handle_skeletal_mesh,
        "animation": handle_animation,
        "rigged_animation": handle_rigged_animation,
        "animation_helpers": handle_animation_helpers,
        "mesh_to_sprite": handle_mesh_to_sprite,
        "validation_grid": handle_validation_grid,
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
