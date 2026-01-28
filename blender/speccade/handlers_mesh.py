"""
SpecCade Mesh Handlers Module

This module handles mesh generation modes including static meshes, modular kits,
organic sculpts, shrinkwrap, and boolean kit operations.

Supported handlers:
- handle_static_mesh: Static mesh generation with modifiers, LOD, and collision.
- handle_modular_kit: Modular kit generation (walls, pipes, doors).
- handle_organic_sculpt: Organic sculpt mesh generation using metaballs.
- handle_shrinkwrap: Shrinkwrap mesh generation for armor/clothing wrapping.
- handle_boolean_kit: Boolean kitbashing for hard-surface modeling.
"""

import math
import time
from pathlib import Path
from typing import Any, Dict, Optional

# Blender modules - only available when running inside Blender
try:
    import bpy
    import bmesh
    from mathutils import Euler, Vector
    BLENDER_AVAILABLE = True
except ImportError:
    bpy = None  # type: ignore
    bmesh = None  # type: ignore
    Vector = None  # type: ignore
    Euler = None  # type: ignore
    BLENDER_AVAILABLE = False

from .report import write_report
from .scene import create_primitive
from .modifiers import apply_modifier, apply_all_modifiers
from .uv_mapping import apply_uv_projection
from .normals import apply_normals_settings
from .materials import apply_materials
from .metrics import compute_mesh_metrics
from .export import (
    export_glb,
    generate_lod_chain,
    export_glb_with_lods,
    generate_collision_mesh,
    export_collision_mesh,
    analyze_navmesh,
    bake_textures,
)


# =============================================================================
# Static Mesh Handler
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


# =============================================================================
# Modular Kit Handler
# =============================================================================

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


# =============================================================================
# Wall Kit Functions
# =============================================================================

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


# =============================================================================
# Pipe Kit Functions
# =============================================================================

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


# =============================================================================
# Door Kit Functions
# =============================================================================

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


# =============================================================================
# Organic Sculpt Handler
# =============================================================================

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


# =============================================================================
# Shrinkwrap Handler
# =============================================================================

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


# =============================================================================
# Mesh Import/Creation Utilities
# =============================================================================

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


# =============================================================================
# Shrinkwrap Validation Utilities
# =============================================================================

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
    non_manifold_edges = _count_non_manifold_edges(mesh)
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


def _count_non_manifold_edges(mesh: 'bpy.types.Mesh') -> int:
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
