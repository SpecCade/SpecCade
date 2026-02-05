"""
SpecCade Render Handlers Module

This module contains render-related handler functions for Blender asset generation.
It handles mesh-to-sprite rendering and validation grid generation for visual
verification of 3D assets.

Key functions:
- handle_mesh_to_sprite(): Render a 3D mesh from multiple angles into a sprite atlas
- handle_validation_grid(): Generate a 6-view validation grid PNG for visual QA
- get_mesh_bounds(): Compute world-space bounding box of a mesh object
- create_skeleton_debug_mesh(): Generate debug mesh visualization of skeleton bones
"""

import json
import math
import shutil
import tempfile
import time
from pathlib import Path
from typing import Any, Dict, List, Tuple

# Blender modules - only available when running inside Blender
try:
    import bpy
    from mathutils import Euler, Vector
    BLENDER_AVAILABLE = True
except ImportError:
    BLENDER_AVAILABLE = False

# Local imports from speccade package
from .report import write_report
from .scene import clear_scene, setup_scene, create_primitive
from .metrics import compute_mesh_metrics
from .rendering import setup_lighting, pack_frames_into_atlas, create_atlas_image
from .modifiers import apply_modifier, apply_all_modifiers
from .materials import apply_materials
from .uv_mapping import apply_uv_projection
from .skeleton import create_armature, apply_skeleton_overrides, create_custom_skeleton
from .skeleton_presets import SKELETON_PRESETS
from .body_parts import create_body_part, create_extrusion_part
from .armature_driven import build_armature_driven_character_mesh
from .handlers_mesh import create_wall_kit, create_pipe_kit, create_door_kit


# =============================================================================
# Validation Grid Configuration
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


def create_skeleton_debug_mesh(armature: 'bpy.types.Object') -> 'bpy.types.Object':
    """
    Create a debug mesh visualization of an armature skeleton.

    Generates capsule-like geometry for each bone, useful for previewing
    skeletal animations without a full character mesh.

    Args:
        armature: The armature object to visualize.

    Returns:
        A mesh object representing the skeleton debug visualization.
    """
    # Collect all bone data in object mode
    bone_data = []
    for bone in armature.data.bones:
        # Get bone world positions
        head_world = armature.matrix_world @ bone.head_local
        tail_world = armature.matrix_world @ bone.tail_local
        length = (tail_world - head_world).length

        # Determine bone radius based on length (thicker for longer bones)
        # Use a minimum radius to ensure visibility
        radius = max(length * 0.08, 0.02)

        bone_data.append({
            'name': bone.name,
            'head': head_world.copy(),
            'tail': tail_world.copy(),
            'length': length,
            'radius': radius,
        })

    if not bone_data:
        # No bones - create a small placeholder cube
        bpy.ops.mesh.primitive_cube_add(size=0.1, location=(0, 0, 0))
        return bpy.context.active_object

    # Create combined mesh for all bone visualizations
    import bmesh
    bm = bmesh.new()

    for bd in bone_data:
        head = bd['head']
        tail = bd['tail']
        length = bd['length']
        radius = bd['radius']

        if length < 0.001:
            # Degenerate bone - skip or add a small sphere
            continue

        # Create a cylinder along the bone direction
        direction = (tail - head).normalized()

        # Find perpendicular vectors for cylinder orientation
        up = Vector((0, 0, 1))
        if abs(direction.dot(up)) > 0.99:
            up = Vector((1, 0, 0))
        right = direction.cross(up).normalized()
        forward = right.cross(direction).normalized()

        # Create cylinder vertices (8 segments)
        segments = 8
        verts_bottom = []
        verts_top = []

        for i in range(segments):
            angle = (i / segments) * 2 * math.pi
            offset = (math.cos(angle) * right + math.sin(angle) * forward) * radius

            v_bottom = bm.verts.new(head + offset)
            v_top = bm.verts.new(tail + offset)
            verts_bottom.append(v_bottom)
            verts_top.append(v_top)

        # Create faces
        bm.verts.ensure_lookup_table()

        # Side faces
        for i in range(segments):
            next_i = (i + 1) % segments
            try:
                bm.faces.new([
                    verts_bottom[i],
                    verts_bottom[next_i],
                    verts_top[next_i],
                    verts_top[i],
                ])
            except ValueError:
                pass  # Face already exists

        # Cap faces (simplified - just close the ends)
        try:
            bm.faces.new(verts_bottom)
        except ValueError:
            pass
        try:
            bm.faces.new(list(reversed(verts_top)))
        except ValueError:
            pass

    # Create mesh from bmesh
    mesh_data = bpy.data.meshes.new("SkeletonDebugMesh")
    bm.to_mesh(mesh_data)
    bm.free()

    # Create object
    debug_obj = bpy.data.objects.new("SkeletonDebug", mesh_data)
    bpy.context.collection.objects.link(debug_obj)

    # Apply smooth shading
    for poly in debug_obj.data.polygons:
        poly.use_smooth = True

    # Add a simple material for visibility
    mat = bpy.data.materials.new("SkeletonDebugMaterial")
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    principled = nodes.get("Principled BSDF")
    if principled:
        # Light blue-gray color for bone visualization
        principled.inputs["Base Color"].default_value = (0.6, 0.7, 0.8, 1.0)
        principled.inputs["Roughness"].default_value = 0.5
    debug_obj.data.materials.append(mat)

    return debug_obj


# =============================================================================
# Mesh to Sprite Handler
# =============================================================================

def handle_mesh_to_sprite(spec: Dict, out_root: Path, report_path: Path) -> None:
    """
    Handle mesh-to-sprite rendering.

    Renders a 3D mesh from multiple rotation angles and packs the resulting
    frames into a sprite atlas with metadata.

    Args:
        spec: The specification dictionary containing recipe and params.
        out_root: Root directory for output files.
        report_path: Path to write the generation report.
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
    """
    Get the world-space bounding box of a mesh object.

    Args:
        obj: The Blender object to compute bounds for.

    Returns:
        A tuple of (min_corner, max_corner) where each is a list of [x, y, z]
        coordinates in world space.
    """
    bbox_corners = [obj.matrix_world @ Vector(corner) for corner in obj.bound_box]
    min_corner = [min(c[i] for c in bbox_corners) for i in range(3)]
    max_corner = [max(c[i] for c in bbox_corners) for i in range(3)]
    return min_corner, max_corner


# =============================================================================
# Validation Grid Handler
# =============================================================================

def handle_validation_grid(
    spec: Dict,
    out_root: Path,
    report_path: Path,
    create_body_part_fn=None,
    create_extrusion_part_fn=None,
) -> None:
    """
    Generate a 6-view validation grid PNG for LLM-based visual verification.

    Grid layout (2 rows x 3 columns):
        FRONT | BACK  | TOP
        LEFT  | RIGHT | ISO

    Args:
        spec: The specification dictionary containing recipe and params.
        out_root: Root directory for output files.
        report_path: Path to write the generation report.
        create_body_part_fn: Optional function to create body parts for skeletal meshes.
        create_extrusion_part_fn: Optional function to create extrusion parts for skeletal meshes.
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

        elif recipe_kind == "static_mesh.modular_kit_v1":
            # Modular kit - walls, pipes, doors
            kit_type_spec = params.get("kit_type", {})
            kit_type = kit_type_spec.get("type", "wall")

            if kit_type == "wall":
                obj = create_wall_kit(kit_type_spec)
            elif kit_type == "pipe":
                obj = create_pipe_kit(kit_type_spec)
            elif kit_type == "door":
                obj = create_door_kit(kit_type_spec)
            else:
                raise ValueError(f"Unknown modular kit type: {kit_type}")

            # Apply export settings
            export_settings = params.get("export", {})
            if export_settings.get("apply_modifiers", True):
                apply_all_modifiers(obj)

        elif recipe_kind.startswith("static_mesh.") or recipe_kind == "blender_primitives_v1":
            # Static mesh - extract mesh params
            primitive = params.get("base_primitive", "cube")
            dimensions = params.get("dimensions", [1, 1, 1])
            obj = create_primitive(primitive, dimensions)

            modifiers = params.get("modifiers", [])
            for mod_spec in modifiers:
                apply_modifier(obj, mod_spec)

            # Join attachments (extra primitives positioned relative to base)
            attachments = params.get("attachments", [])
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
            export_settings = params.get("export", {})
            if export_settings.get("apply_modifiers", True):
                apply_all_modifiers(obj)

        elif recipe_kind.startswith("skeletal_mesh."):
            # Skeletal mesh preview helpers (not the main generation handler).
            # We intentionally only support the new recipe kinds.
            skeleton_spec = params.get("skeleton", [])
            skeleton_preset = params.get("skeleton_preset")

            if skeleton_preset:
                armature = create_armature(skeleton_preset)
                if skeleton_spec:
                    apply_skeleton_overrides(armature, skeleton_spec)
            elif skeleton_spec:
                armature = create_custom_skeleton(skeleton_spec)
            else:
                armature = create_armature("humanoid_connected_v1")

            if recipe_kind == "skeletal_mesh.armature_driven_v1":
                # Reuse the shared builder so previews match generation behavior.
                # Note: asset attachments may not exist; the builder already warns/skips.
                obj = build_armature_driven_character_mesh(
                    armature=armature,
                    params=params,
                    out_root=out_root,
                )

            elif recipe_kind == "skeletal_mesh.skinned_mesh_v1":
                mesh_file = params.get("mesh_file")
                mesh_asset = params.get("mesh_asset")
                mesh_ref = mesh_file or mesh_asset

                if mesh_ref and isinstance(mesh_ref, str):
                    p = Path(mesh_ref)
                    candidates = [p]
                    if not p.suffix:
                        candidates.append(p.with_suffix('.glb'))
                        candidates.append(p.with_suffix('.gltf'))
                    mesh_path = next((c for c in candidates if c.exists()), None)
                    if mesh_path is not None:
                        bpy.ops.object.select_all(action='DESELECT')
                        bpy.ops.import_scene.gltf(filepath=str(mesh_path))
                        imported_meshes = [o for o in bpy.context.selected_objects if o.type == 'MESH']
                        if imported_meshes:
                            bpy.context.view_layer.objects.active = imported_meshes[0]
                            if len(imported_meshes) > 1:
                                bpy.ops.object.join()
                            obj = bpy.context.active_object
                        else:
                            obj = armature
                    else:
                        obj = armature
                else:
                    obj = armature

            else:
                raise ValueError(
                    f"Unsupported skeletal_mesh recipe kind for preview: {recipe_kind}"
                )

        elif recipe_kind.startswith("skeletal_animation."):
            # Skeletal animation preview - create debug mesh from skeleton preset
            skeleton_preset = params.get("skeleton_preset", None)
            skeleton_param = params.get("skeleton", None)

            # Handle helpers_v1 format where "skeleton" is a string preset name
            if isinstance(skeleton_param, str):
                # Map short names to full preset names
                skeleton_map = {
                    "humanoid": "humanoid_connected_v1",
                    "quadruped": "quadruped_basic_v1",
                }
                skeleton_preset = skeleton_map.get(skeleton_param, skeleton_param)
                skeleton_spec = []
            else:
                skeleton_spec = skeleton_param if skeleton_param else []

            # Create armature from preset or custom skeleton
            if skeleton_preset:
                armature = create_armature(skeleton_preset)
                if skeleton_spec and isinstance(skeleton_spec, list):
                    apply_skeleton_overrides(armature, skeleton_spec)
            elif skeleton_spec and isinstance(skeleton_spec, list):
                armature = create_custom_skeleton(skeleton_spec)
            else:
                # Default to humanoid connected if no skeleton info provided
                armature = create_armature("humanoid_connected_v1")

            # Create debug mesh visualization of the skeleton
            obj = create_skeleton_debug_mesh(armature)

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
            for frame_path, label in frame_paths:
                shutil.copy(frame_path, frames_output_path / f"{label}.png")
            output_path = frames_output_path

        # Clean up temp files
        shutil.rmtree(temp_frames_dir)

        duration_ms = int((time.time() - start_time) * 1000)

        write_report(report_path, ok=True,
                     output_path=str(output_path.relative_to(out_root) if output_path.is_relative_to(out_root) else output_rel_path),
                     duration_ms=duration_ms)

    except Exception as e:
        write_report(report_path, ok=False, error=str(e))
        raise
