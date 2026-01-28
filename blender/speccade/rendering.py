"""
SpecCade Rendering Module

This module contains all rendering-related functions for SpecCade asset generation,
including camera setup, lighting presets, animation preview rendering, and
atlas compositing.

Functionality includes:
- Preview camera setup with multiple angle presets
- Stick figure visualization for skeletal animation previews
- Animation preview frame rendering
- Lighting setup with various presets (three_point, rim, flat, validation, dramatic, studio)
- Frame packing and atlas image creation

The module is designed for both real-time previews and batch rendering
operations in Blender's background mode.
"""

import math
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

# Blender modules - only available when running inside Blender
try:
    import bpy
    import bmesh
    from mathutils import Euler, Vector
    BLENDER_AVAILABLE = True
except ImportError:
    BLENDER_AVAILABLE = False


def _iter_action_fcurves(action: Any):
    """
    Return an iterable of fcurves for an action if available.

    Blender's animation API has evolved:
    - Blender < 5.0: fcurves directly on Action
    - Blender >= 5.0: fcurves in Channelbags within Layers/Strips

    We try both old and new APIs.
    """
    # Try direct fcurves first (Blender < 5.0 or simple actions)
    if hasattr(action, 'fcurves') and action.fcurves:
        return action.fcurves

    # Try Blender 5.0 layered action structure
    if hasattr(action, 'layers'):
        fcurves = []
        for layer in action.layers:
            for strip in layer.strips:
                # In Blender 5.0, strips have channelbags accessed via slots
                # Try multiple access patterns
                if hasattr(strip, 'channelbags'):
                    for channelbag in strip.channelbags:
                        if hasattr(channelbag, 'fcurves'):
                            fcurves.extend(channelbag.fcurves)
                elif hasattr(strip, 'channelbag') and strip.channelbag:
                    if hasattr(strip.channelbag, 'fcurves'):
                        fcurves.extend(strip.channelbag.fcurves)
                # Try action_slot pattern
                if hasattr(strip, 'action') and strip.action:
                    if hasattr(strip.action, 'fcurves') and strip.action.fcurves:
                        fcurves.extend(strip.action.fcurves)
        if fcurves:
            return fcurves

    return []


# =============================================================================
# Animation Preview GIF Rendering
# =============================================================================

def setup_preview_camera(
    armature: 'bpy.types.Object',
    camera_preset: str,
    distance: float = 5.0
) -> 'bpy.types.Object':
    """
    Set up a camera for animation preview rendering.

    Args:
        armature: The armature to frame.
        camera_preset: Camera angle preset name.
        distance: Distance from character.

    Returns:
        The camera object.
    """
    # Calculate armature bounds center
    bounds_min = Vector((float('inf'), float('inf'), float('inf')))
    bounds_max = Vector((float('-inf'), float('-inf'), float('-inf')))

    for bone in armature.data.bones:
        head_world = armature.matrix_world @ bone.head_local
        tail_world = armature.matrix_world @ bone.tail_local
        for pos in [head_world, tail_world]:
            bounds_min.x = min(bounds_min.x, pos.x)
            bounds_min.y = min(bounds_min.y, pos.y)
            bounds_min.z = min(bounds_min.z, pos.z)
            bounds_max.x = max(bounds_max.x, pos.x)
            bounds_max.y = max(bounds_max.y, pos.y)
            bounds_max.z = max(bounds_max.z, pos.z)

    center = (bounds_min + bounds_max) / 2
    height = bounds_max.z - bounds_min.z

    # Create camera
    cam_data = bpy.data.cameras.new("PreviewCamera")
    cam_data.lens = 50  # Standard lens
    camera = bpy.data.objects.new("PreviewCamera", cam_data)
    bpy.context.collection.objects.link(camera)

    # Position camera based on preset
    preset = camera_preset.lower().replace("-", "_")
    if preset == "three_quarter":
        angle = math.radians(45)
        cam_x = center.x + distance * math.sin(angle)
        cam_y = center.y - distance * math.cos(angle)
        cam_z = center.z + height * 0.3
    elif preset == "front":
        cam_x = center.x
        cam_y = center.y - distance
        cam_z = center.z + height * 0.2
    elif preset == "side":
        cam_x = center.x + distance
        cam_y = center.y
        cam_z = center.z + height * 0.2
    elif preset == "back":
        cam_x = center.x
        cam_y = center.y + distance
        cam_z = center.z + height * 0.2
    elif preset == "top":
        cam_x = center.x
        cam_y = center.y
        cam_z = center.z + distance * 1.5
    else:
        # Default to three_quarter
        angle = math.radians(45)
        cam_x = center.x + distance * math.sin(angle)
        cam_y = center.y - distance * math.cos(angle)
        cam_z = center.z + height * 0.3

    camera.location = Vector((cam_x, cam_y, cam_z))

    # Point camera at center
    direction = center - camera.location
    rot_quat = direction.to_track_quat('-Z', 'Y')
    camera.rotation_euler = rot_quat.to_euler()

    # Set as active camera
    bpy.context.scene.camera = camera

    return camera


def create_stick_figure_material() -> 'bpy.types.Material':
    """Create a simple diffuse material for the stick figure visualization."""
    mat = bpy.data.materials.new("StickFigureMat")
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    nodes.clear()

    # Use diffuse BSDF for better visibility in renders
    bsdf = nodes.new('ShaderNodeBsdfDiffuse')
    bsdf.inputs['Color'].default_value = (0.9, 0.8, 0.6, 1.0)  # Warm beige

    output = nodes.new('ShaderNodeOutputMaterial')
    links.new(bsdf.outputs['BSDF'], output.inputs['Surface'])

    return mat


def create_stick_figure_for_frame(armature: 'bpy.types.Object', mat: 'bpy.types.Material',
                                   bone_radius: float = 0.03) -> 'bpy.types.Object':
    """
    Create a stick figure mesh based on the current pose of the armature.

    This creates the geometry at the exact world positions of the pose bones
    for the current frame, ensuring the visualization matches the animation.

    Args:
        armature: The armature object.
        mat: Material to apply to the mesh.
        bone_radius: Radius of the bone visualizations.

    Returns:
        A single mesh object representing the stick figure.
    """
    mesh = bpy.data.meshes.new("StickFigureMesh")
    bm = bmesh.new()

    # Build a cache of posed bone matrices computed from rotation_euler values
    # This is needed because Blender 5.0 background mode doesn't update pbone.matrix
    # when we set rotation_euler via fcurve evaluation
    posed_matrices = {}

    def compute_posed_matrix(pbone):
        """Compute the posed matrix for a bone from its rotation_euler."""
        if pbone.name in posed_matrices:
            return posed_matrices[pbone.name]

        # Get the bone's rest pose matrix (in armature local space)
        rest_matrix = pbone.bone.matrix_local.copy()

        # Create the pose transform from rotation_euler
        pose_rot = pbone.rotation_euler.to_matrix().to_4x4()

        if pbone.parent:
            # Get parent's posed matrix
            parent_posed = compute_posed_matrix(pbone.parent)
            # Get parent's rest matrix
            parent_rest = pbone.parent.bone.matrix_local.copy()
            # Compute the relative rest offset from parent to this bone
            parent_rest_inv = parent_rest.inverted()
            local_rest = parent_rest_inv @ rest_matrix
            # Final posed matrix = parent_posed @ local_rest @ pose_rotation
            posed = parent_posed @ local_rest @ pose_rot
        else:
            # Root bone: just apply pose rotation to rest matrix
            posed = rest_matrix @ pose_rot

        posed_matrices[pbone.name] = posed
        return posed

    # Compute all posed matrices
    for pbone in armature.pose.bones:
        compute_posed_matrix(pbone)

    # Now create the stick figure using computed posed matrices
    for pbone in armature.pose.bones:
        posed_matrix = posed_matrices.get(pbone.name, pbone.matrix)
        bone_world_matrix = armature.matrix_world @ posed_matrix
        head = bone_world_matrix @ Vector((0, 0, 0))
        tail = bone_world_matrix @ Vector((0, pbone.length, 0))
        length = (tail - head).length

        if length < 0.001:
            continue

        # Create cube at head position (simpler than cylinders for debugging)
        vert_count_before = len(bm.verts)
        bmesh.ops.create_cube(bm, size=bone_radius * 2)
        bm.verts.ensure_lookup_table()
        for i in range(vert_count_before, len(bm.verts)):
            bm.verts[i].co += head

        # Create cube at tail position
        vert_count_before = len(bm.verts)
        bmesh.ops.create_cube(bm, size=bone_radius)
        bm.verts.ensure_lookup_table()
        for i in range(vert_count_before, len(bm.verts)):
            bm.verts[i].co += tail

    bm.to_mesh(mesh)
    bm.free()

    obj = bpy.data.objects.new("StickFigure", mesh)
    bpy.context.collection.objects.link(obj)
    mesh.materials.append(mat)

    return obj


def render_animation_preview_frames(
    armature: 'bpy.types.Object',
    output_dir: Path,
    preview_config: Dict[str, Any],
    frame_start: int,
    frame_end: int,
    fps: int,
    keyframe_times: Optional[List[float]] = None
) -> Dict[str, Any]:
    """
    Render animation preview as PNG frames.

    Creates a stick figure visualization mesh for the armature bones since
    armatures are not visible in production renders. Uses regular EEVEE
    rendering which works in background mode.

    Args:
        armature: The armature being animated.
        output_dir: Output directory for PNG frames.
        preview_config: Preview configuration from spec.
        frame_start: First frame to render.
        frame_end: Last frame to render.
        fps: Frames per second.
        keyframe_times: Optional list of keyframe times (in seconds) to render.
                       If provided, only these frames are rendered.

    Returns:
        Dictionary with preview metrics and frame paths.
    """
    scene = bpy.context.scene

    # Get config values with defaults
    camera_preset = preview_config.get("camera", "three_quarter")
    size = preview_config.get("size", [256, 256])
    frame_step = max(1, preview_config.get("frame_step", 2))
    background = preview_config.get("background", [0.2, 0.2, 0.2, 1.0])

    # Create material for stick figures
    mat = create_stick_figure_material()

    # Setup camera
    camera = setup_preview_camera(armature, camera_preset)

    # Add a sun lamp for lighting (needed for diffuse material)
    light_data = bpy.data.lights.new(name="PreviewSun", type='SUN')
    light_data.energy = 2.0
    sun = bpy.data.objects.new("PreviewSun", light_data)
    bpy.context.collection.objects.link(sun)
    sun.rotation_euler = (math.radians(45), 0, math.radians(45))

    # Configure render settings
    scene.render.resolution_x = size[0]
    scene.render.resolution_y = size[1]
    scene.render.resolution_percentage = 100
    scene.render.film_transparent = False

    # Set background color using world nodes
    world = bpy.data.worlds.new("PreviewWorld")
    world.use_nodes = True
    bg_node = world.node_tree.nodes.get("Background")
    if bg_node:
        bg_node.inputs['Color'].default_value = (background[0], background[1], background[2], 1.0)
    scene.world = world

    # Configure render engine for fast preview
    if hasattr(scene.render, 'engine'):
        if 'BLENDER_EEVEE_NEXT' in dir(bpy.types):
            scene.render.engine = 'BLENDER_EEVEE_NEXT'
        else:
            scene.render.engine = 'BLENDER_EEVEE'

    # Low samples for speed
    if hasattr(scene, 'eevee'):
        scene.eevee.taa_render_samples = 16

    # Set output format to PNG
    scene.render.image_settings.file_format = 'PNG'
    scene.render.image_settings.color_mode = 'RGB'

    # Ensure output directory exists and resolve to absolute path
    output_dir = output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    # Determine which frames to render
    if keyframe_times:
        # Convert keyframe times to frame numbers
        frames_to_render = sorted(set(
            max(1, int(t * fps) + 1) for t in keyframe_times
        ))
    else:
        # Fall back to frame_step based rendering
        frames_to_render = list(range(frame_start, frame_end + 1, frame_step))

    # Check if animation data exists and has fcurves
    if armature.animation_data and armature.animation_data.action:
        action = armature.animation_data.action
        fcurves = list(_iter_action_fcurves(action))
        print(f"Animation has {len(fcurves)} fcurves")
        if fcurves:
            print(f"Sample fcurve: {fcurves[0].data_path}, keyframes: {len(fcurves[0].keyframe_points)}")
    else:
        print("WARNING: No animation data or action on armature!")

    # Render each frame
    # After baking, animation is in simple keyframes that should evaluate correctly
    rendered_frames = []
    for frame in frames_to_render:
        scene.frame_set(frame, subframe=0.0)

        # Force full depsgraph update including animation evaluation
        depsgraph = bpy.context.evaluated_depsgraph_get()
        depsgraph.update()

        # Manually evaluate fcurves for this frame to ensure animation is applied
        # This is a workaround for Blender 5.0's background mode evaluation issues
        eval_errors = []
        eval_success = 0
        if armature.animation_data and armature.animation_data.action:
            action = armature.animation_data.action
            for fcurve in _iter_action_fcurves(action):
                value = fcurve.evaluate(frame)
                data_path = fcurve.data_path
                array_index = fcurve.array_index
                try:
                    # data_path is like 'pose.bones["bone_name"].rotation_euler'
                    armature.path_resolve(data_path)[array_index] = value
                    eval_success += 1
                except Exception as e:
                    eval_errors.append(f"{data_path}[{array_index}]: {e}")


        # Force pose bone matrix recalculation
        # Enter and exit pose mode to trigger matrix updates
        bpy.context.view_layer.objects.active = armature
        if bpy.context.mode != 'POSE':
            bpy.ops.object.mode_set(mode='POSE')
        bpy.ops.pose.select_all(action='SELECT')
        bpy.ops.object.mode_set(mode='OBJECT')
        bpy.context.view_layer.update()

        # Create stick figure for current pose
        stick_figure = create_stick_figure_for_frame(armature, mat)

        frame_path = output_dir / f"frame_{frame:04d}.png"
        scene.render.filepath = str(frame_path)
        bpy.ops.render.render(write_still=True)
        rendered_frames.append(f"frame_{frame:04d}.png")
        print(f"Rendered frame {frame}")

        # Clean up stick figure for this frame
        bpy.data.objects.remove(stick_figure, do_unlink=True)

    # Clean up
    bpy.data.objects.remove(camera, do_unlink=True)
    if "PreviewCamera" in bpy.data.cameras:
        bpy.data.cameras.remove(bpy.data.cameras["PreviewCamera"])
    bpy.data.objects.remove(sun, do_unlink=True)
    if "PreviewSun" in bpy.data.lights:
        bpy.data.lights.remove(bpy.data.lights["PreviewSun"])
    bpy.data.worlds.remove(world)
    bpy.data.materials.remove(mat)

    return {
        "preview_frames": rendered_frames,
        "frames_rendered": len(rendered_frames),
        "size": size,
        "camera_preset": camera_preset,
        "keyframes_rendered": keyframe_times is not None,
    }


# =============================================================================
# Lighting Setup
# =============================================================================

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


# =============================================================================
# Atlas Compositing
# =============================================================================

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
