"""
Animation module for SpecCade Blender asset generation.

This module handles procedural animation layers, pose and phase systems,
animation baking, root motion settings, and animation creation from keyframe
specifications.
"""

import math
from typing import Any, Dict, List, Optional

try:
    import bpy
    from mathutils import Euler, Vector
except ImportError:
    bpy = None  # type: ignore
    Euler = None  # type: ignore
    Vector = None  # type: ignore


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
    fps = params.get("fps", 30)
    keyframes = params.get("keyframes", [])
    interpolation = params.get("interpolation", "linear").upper()

    # Set scene FPS
    bpy.context.scene.render.fps = fps

    # Calculate frame range - support both duration_frames and duration_seconds
    duration_frames = params.get("duration_frames")
    duration_seconds = params.get("duration_seconds")
    if duration_frames:
        frame_count = duration_frames
    elif duration_seconds:
        frame_count = int(duration_seconds * fps)
    else:
        frame_count = 30  # Default to 1 second at 30fps
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
                # Insert keyframe with the value we just set
                # Do NOT use INSERTKEY_VISUAL - it reads the visual pose which is
                # rest pose in Blender 5.0 background mode
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
# Export Helpers
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
