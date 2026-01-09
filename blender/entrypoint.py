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
                 duration_ms: Optional[int] = None) -> None:
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
    if duration_ms is not None:
        report["duration_ms"] = duration_ms
    if BLENDER_AVAILABLE:
        report["blender_version"] = bpy.app.version_string

    with open(report_path, 'w') as f:
        json.dump(report, f, indent=2)


def compute_mesh_metrics(obj: 'bpy.types.Object') -> Dict[str, Any]:
    """Compute metrics for a mesh object."""
    # Ensure we're working with evaluated mesh data
    depsgraph = bpy.context.evaluated_depsgraph_get()
    obj_eval = obj.evaluated_get(depsgraph)
    mesh = obj_eval.to_mesh()

    # Triangle count
    triangle_count = sum(len(p.vertices) - 2 for p in mesh.polygons)

    # Bounding box
    bbox_min = [float('inf')] * 3
    bbox_max = [float('-inf')] * 3
    for v in mesh.vertices:
        co = obj.matrix_world @ v.co
        for i in range(3):
            bbox_min[i] = min(bbox_min[i], co[i])
            bbox_max[i] = max(bbox_max[i], co[i])

    # UV island count
    uv_island_count = 0
    if mesh.uv_layers:
        uv_layer = mesh.uv_layers.active
        if uv_layer:
            # Simple approximation: count connected UV components
            uv_island_count = count_uv_islands(mesh, uv_layer)

    # Vertex count
    vertex_count = len(mesh.vertices)

    # Material slot count
    material_slot_count = len(obj.material_slots)

    obj_eval.to_mesh_clear()

    return {
        "triangle_count": triangle_count,
        "bounding_box": {
            "min": bbox_min,
            "max": bbox_max
        },
        "uv_island_count": uv_island_count,
        "vertex_count": vertex_count,
        "material_slot_count": material_slot_count
    }


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


def compute_skeletal_mesh_metrics(obj: 'bpy.types.Object', armature: 'bpy.types.Object') -> Dict[str, Any]:
    """Compute metrics for a skeletal mesh."""
    mesh_metrics = compute_mesh_metrics(obj)

    # Add bone count
    bone_count = len(armature.data.bones)

    # Compute max bone influences
    max_influences = 0
    if obj.vertex_groups:
        for v in obj.data.vertices:
            influences = sum(1 for g in v.groups if g.weight > 0.001)
            max_influences = max(max_influences, influences)

    mesh_metrics["bone_count"] = bone_count
    mesh_metrics["max_bone_influences"] = max_influences

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

def apply_uv_projection(obj: 'bpy.types.Object', projection: str) -> None:
    """Apply UV projection to an object."""
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')

    projection = projection.lower()

    if projection == "box":
        bpy.ops.uv.cube_project()
    elif projection == "cylinder":
        bpy.ops.uv.cylinder_project()
    elif projection == "sphere":
        bpy.ops.uv.sphere_project()
    elif projection == "smart":
        bpy.ops.uv.smart_project()
    elif projection == "lightmap":
        bpy.ops.uv.lightmap_pack()
    else:
        # Default to smart project
        bpy.ops.uv.smart_project()

    bpy.ops.object.mode_set(mode='OBJECT')


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
               include_animation: bool = False) -> None:
    """Export scene to GLB format."""
    export_settings = {
        'filepath': str(output_path),
        'export_format': 'GLB',
        'export_apply': True,
        'export_texcoords': True,
        'export_normals': True,
        'export_colors': True,
    }

    if include_animation:
        export_settings['export_animations'] = True
        export_settings['export_current_frame'] = False
    else:
        export_settings['export_animations'] = False

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

        # Compute metrics before export
        metrics = compute_mesh_metrics(obj)

        # Export
        export_glb(output_path)

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

        # Create armature
        skeleton_preset = params.get("skeleton_preset", "humanoid_basic_v1")
        armature = create_armature(skeleton_preset)

        # Create body parts
        body_parts = params.get("body_parts", [])
        mesh_objects = []
        for part_spec in body_parts:
            mesh_obj = create_body_part(armature, part_spec)
            mesh_objects.append((mesh_obj, part_spec.get("bone")))

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

            # Export
            export_glb(output_path, include_armature=True)

            duration_ms = int((time.time() - start_time) * 1000)
            write_report(report_path, ok=True, metrics=metrics,
                         output_path=output_rel_path, duration_ms=duration_ms)
        else:
            raise ValueError("No body parts specified")

    except Exception as e:
        write_report(report_path, ok=False, error=str(e))
        raise


def handle_animation(spec: Dict, out_root: Path, report_path: Path) -> None:
    """Handle animation generation."""
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

        # Export with animation
        export_glb(output_path, include_armature=True, include_animation=True)

        duration_ms = int((time.time() - start_time) * 1000)
        write_report(report_path, ok=True, metrics=metrics,
                     output_path=output_rel_path, duration_ms=duration_ms)

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
                        choices=["static_mesh", "skeletal_mesh", "animation"],
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
