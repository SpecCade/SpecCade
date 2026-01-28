"""
SpecCade Export Module

This module contains all export-related functions for SpecCade asset generation,
including GLB export, LOD generation, collision mesh generation, navmesh analysis,
and texture baking.

Functionality includes:
- GLB export with configurable options
- LOD (Level of Detail) mesh generation with decimation
- Collision mesh generation (convex hull, simplified mesh, box)
- Navmesh analysis with walkability classification
- Texture baking (normal, AO, curvature maps)

The module handles cross-version Blender API compatibility and provides
deterministic output for reproducible asset generation.
"""

import math
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

# Blender modules - only available when running inside Blender
try:
    import bpy
    import bmesh
    from mathutils import Vector
    BLENDER_AVAILABLE = True
except ImportError:
    BLENDER_AVAILABLE = False

# Import from sibling modules
from .metrics import compute_mesh_metrics


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


# =============================================================================
# Texture Baking
# =============================================================================

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
