"""
SpecCade Skeletal Mesh and Animation Handlers

This module contains handler functions for skeletal mesh and animation generation:
- handle_skeletal_mesh: Generate rigged character meshes with body parts and skinning
- handle_animation: Generate simple FK animations with keyframes
- handle_rigged_animation: Generate animations with IK support and procedural layers
- handle_animation_helpers: Generate procedural locomotion animations using presets
"""

import math
import time
from pathlib import Path
from typing import Any, Dict, List, Optional

# Blender modules - only available when running inside Blender
try:
    import bpy
    from mathutils import Euler, Vector
    BLENDER_AVAILABLE = True
except ImportError:
    bpy = None  # type: ignore
    Euler = None  # type: ignore
    Vector = None  # type: ignore
    BLENDER_AVAILABLE = False

# Internal module imports
from .report import write_report
from .scene import clear_scene, setup_scene
from .skeleton_presets import SKELETON_PRESETS
from .skeleton import create_armature, apply_skeleton_overrides, create_custom_skeleton
from .skeletal_mesh_rework import classify_skeletal_mesh_kind, compute_safe_rename_plan
from .ik_fk import apply_rig_setup
from .animation import (
    create_animation,
    apply_procedural_layers,
    apply_poses_and_phases,
    bake_animation,
    apply_root_motion_settings,
)
from .rig_config import apply_animator_rig_config
from .metrics import compute_skeletal_mesh_metrics, compute_animation_metrics
from .export import export_glb
from .rendering import render_animation_preview_frames
from .materials import apply_materials


def _select_only(objs: List['bpy.types.Object'], *, active: Optional['bpy.types.Object'] = None) -> None:
    bpy.ops.object.select_all(action='DESELECT')
    for obj in objs:
        obj.select_set(True)
    if active is not None:
        bpy.context.view_layer.objects.active = active


def _ensure_object_mode() -> None:
    try:
        if bpy.context.mode != 'OBJECT':
            bpy.ops.object.mode_set(mode='OBJECT')
    except Exception:
        # In background mode, mode switching can fail if no active object.
        pass


def _recalculate_normals(mesh_obj: 'bpy.types.Object') -> None:
    _ensure_object_mode()
    _select_only([mesh_obj], active=mesh_obj)
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.mesh.normals_make_consistent(inside=False)
    bpy.ops.object.mode_set(mode='OBJECT')


def _assign_all_vertices_to_group(mesh_obj: 'bpy.types.Object', group_name: str, *, weight: float = 1.0) -> None:
    vg = mesh_obj.vertex_groups.get(group_name)
    if vg is None:
        vg = mesh_obj.vertex_groups.new(name=group_name)
    indices = [v.index for v in mesh_obj.data.vertices]
    if indices:
        vg.add(indices, weight, 'REPLACE')


def _limit_vertex_group_influences(mesh_obj: 'bpy.types.Object', limit: int) -> None:
    """Limit number of groups per vertex (best effort)."""
    limit = max(1, int(limit))
    _ensure_object_mode()
    _select_only([mesh_obj], active=mesh_obj)

    try:
        bpy.ops.object.vertex_group_limit_total(limit=limit)
        bpy.ops.object.vertex_group_normalize_all(lock_active=False)
        return
    except Exception:
        # Fallback: manual pruning.
        pass

    for v in mesh_obj.data.vertices:
        groups = [(g.group, g.weight) for g in v.groups]
        if len(groups) <= limit:
            continue
        groups.sort(key=lambda t: t[1], reverse=True)
        keep = groups[:limit]
        remove = groups[limit:]

        for group_idx, _w in remove:
            try:
                mesh_obj.vertex_groups[group_idx].remove([v.index])
            except Exception:
                continue

        if limit == 1 and keep:
            keep_group_idx, _keep_w = keep[0]
            try:
                mesh_obj.vertex_groups[keep_group_idx].add([v.index], 1.0, 'REPLACE')
            except Exception:
                pass


def _ensure_armature_modifier(mesh_obj: 'bpy.types.Object', armature_obj: 'bpy.types.Object') -> None:
    for mod in mesh_obj.modifiers:
        if mod.type == 'ARMATURE':
            mod.object = armature_obj
            return
    mod = mesh_obj.modifiers.new(name='Armature', type='ARMATURE')
    mod.object = armature_obj


def _parent_mesh_to_armature(mesh_obj: 'bpy.types.Object', armature_obj: 'bpy.types.Object', *, auto_weights: bool) -> None:
    _ensure_object_mode()
    _select_only([mesh_obj, armature_obj], active=armature_obj)

    # Clear any existing armature binding from imported meshes.
    if mesh_obj.parent and mesh_obj.parent.type == 'ARMATURE':
        mesh_obj.parent = None
    for mod in list(mesh_obj.modifiers):
        if mod.type == 'ARMATURE':
            mesh_obj.modifiers.remove(mod)

    if auto_weights:
        bpy.ops.object.parent_set(type='ARMATURE_AUTO')
    else:
        bpy.ops.object.parent_set(type='ARMATURE')

    _ensure_armature_modifier(mesh_obj, armature_obj)


def _merge_vertex_group_weights(mesh_obj: 'bpy.types.Object', src_name: str, dst_name: str) -> None:
    src = mesh_obj.vertex_groups.get(src_name)
    if src is None:
        return
    dst = mesh_obj.vertex_groups.get(dst_name)
    if dst is None:
        dst = mesh_obj.vertex_groups.new(name=dst_name)

    src_idx = src.index
    for v in mesh_obj.data.vertices:
        for g in v.groups:
            if g.group == src_idx:
                dst.add([v.index], g.weight, 'ADD')
                break

    mesh_obj.vertex_groups.remove(src)


def _apply_vertex_group_map(mesh_obj: 'bpy.types.Object', vertex_group_map: Dict[str, str]) -> None:
    """Apply mesh vertex-group -> bone-name mapping.

    Best-effort rules:
    - Missing source groups are ignored.
    - If the destination group already exists and is not being renamed away, we
      merge weights and delete the source group.
    - Cycles / chains are handled via temporary names.
    """
    if not vertex_group_map:
        return

    existing = [vg.name for vg in mesh_obj.vertex_groups]
    existing_set = set(existing)
    filtered = {
        src: dst
        for src, dst in sorted((vertex_group_map or {}).items(), key=lambda kv: kv[0])
        if isinstance(src, str)
        and isinstance(dst, str)
        and src
        and dst
        and src in existing_set
        and src != dst
    }
    if not filtered:
        return

    sources = set(filtered.keys())

    rename_only: Dict[str, str] = {}
    merges: List[tuple[str, str]] = []

    # Handle many-to-one maps deterministically by merging weights into the
    # destination group when the destination is not being renamed away.
    by_dst: Dict[str, List[str]] = {}
    for src, dst in filtered.items():
        by_dst.setdefault(dst, []).append(src)

    for dst, srcs in sorted(by_dst.items(), key=lambda kv: kv[0]):
        if len(srcs) <= 1:
            continue
        if dst in sources:
            raise ValueError(
                f"vertex_group_map maps multiple sources to '{dst}', but '{dst}' is also a source"
            )
        for src in sorted(srcs):
            merges.append((src, dst))

    for src, dst in filtered.items():
        if (src, dst) in merges:
            continue
        if dst in existing_set and dst not in sources:
            merges.append((src, dst))
        else:
            rename_only[src] = dst

    for src, dst in merges:
        _merge_vertex_group_weights(mesh_obj, src, dst)
        existing_set.discard(src)
        existing_set.add(dst)

    if not rename_only:
        return

    plan = compute_safe_rename_plan(rename_only, existing_names=[vg.name for vg in mesh_obj.vertex_groups])
    for src, dst in plan:
        vg = mesh_obj.vertex_groups.get(src)
        if vg is None:
            continue
        vg.name = dst


def handle_skeletal_mesh(spec: Dict, out_root: Path, report_path: Path) -> None:
    """Handle skeletal mesh generation."""
    start_time = time.time()

    try:
        if not BLENDER_AVAILABLE:
            raise RuntimeError("Blender Python (bpy) is required for skeletal mesh generation")

        recipe = spec.get("recipe", {})
        params = recipe.get("params", {})
        kind = classify_skeletal_mesh_kind(recipe.get("kind", ""))

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

        combined_mesh = None

        if kind == "skeletal_mesh.armature_driven_v1":
            bone_meshes = params.get("bone_meshes", {})
            if not bone_meshes:
                raise ValueError("armature_driven_v1 requires params.bone_meshes")

            bone_entries = []
            if isinstance(bone_meshes, dict):
                for bone_name, mesh_spec in bone_meshes.items():
                    bone_entries.append((bone_name, mesh_spec or {}))
            elif isinstance(bone_meshes, list):
                for entry in bone_meshes:
                    if isinstance(entry, dict) and entry.get("bone"):
                        bone_entries.append((entry.get("bone"), entry))
            else:
                raise ValueError("params.bone_meshes must be an object or array")

            bone_entries.sort(key=lambda t: t[0])

            mesh_objs = []
            for bone_name, mesh_spec in bone_entries:
                bone = armature.data.bones.get(bone_name)
                if bone is None:
                    print(f"Warning: bone_meshes refers to missing bone: {bone_name}")
                    continue

                head_w = armature.matrix_world @ bone.head_local
                tail_w = armature.matrix_world @ bone.tail_local
                axis = (tail_w - head_w)
                length = float(axis.length)
                if length <= 1e-6:
                    print(f"Warning: bone has near-zero length: {bone_name}")
                    continue

                radius_spec = mesh_spec.get("profile_radius", 0.15)
                if isinstance(radius_spec, dict) and "absolute" in radius_spec:
                    radius = float(radius_spec["absolute"])
                elif isinstance(radius_spec, (list, tuple)) and len(radius_spec) >= 2:
                    radius = float(radius_spec[0] + radius_spec[1]) * 0.5 * length
                else:
                    radius = float(radius_spec) * length

                vertices = 12
                profile = mesh_spec.get("profile")
                if isinstance(profile, str) and "(" in profile and profile.endswith(")"):
                    try:
                        vertices = int(profile.split("(", 1)[1][:-1])
                    except Exception:
                        vertices = 12

                bpy.ops.mesh.primitive_cylinder_add(
                    radius=radius,
                    depth=length,
                    vertices=max(3, vertices),
                    location=(head_w + tail_w) * 0.5,
                )
                seg_obj = bpy.context.active_object
                seg_obj.name = f"BoneMesh_{bone_name}"

                axis_n = axis.normalized()
                seg_obj.rotation_euler = axis_n.to_track_quat('Z', 'Y').to_euler()
                bpy.context.view_layer.objects.active = seg_obj
                bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)

                _assign_all_vertices_to_group(seg_obj, bone_name, weight=1.0)
                mesh_objs.append(seg_obj)

            if not mesh_objs:
                raise ValueError("armature_driven_v1 produced no meshes (check bone_meshes)")

            if len(mesh_objs) > 1:
                _select_only(mesh_objs, active=mesh_objs[0])
                bpy.ops.object.join()
                combined_mesh = bpy.context.active_object
            else:
                combined_mesh = mesh_objs[0]
            combined_mesh.name = "Character"

            _recalculate_normals(combined_mesh)
            _parent_mesh_to_armature(combined_mesh, armature, auto_weights=False)

        elif kind == "skeletal_mesh.skinned_mesh_v1":
            mesh_file = params.get("mesh_file")
            mesh_asset = params.get("mesh_asset")
            if not mesh_file and not mesh_asset:
                raise ValueError("skinned_mesh_v1 requires params.mesh_file or params.mesh_asset")

            mesh_path = None
            if mesh_file:
                p = Path(mesh_file)
                mesh_path = p if p.is_absolute() else (out_root / p)
            else:
                p = Path(str(mesh_asset))
                candidates = []
                if p.suffix:
                    candidates.append(p)
                candidates.append(p.with_suffix('.glb'))
                candidates.append(p.with_suffix('.gltf'))
                candidates = [c if c.is_absolute() else (out_root / c) for c in candidates]
                mesh_path = next((c for c in candidates if c.exists()), None)

            if mesh_path is None or not mesh_path.exists():
                raise ValueError(f"Mesh file not found for skinned_mesh_v1: {mesh_file or mesh_asset}")

            _ensure_object_mode()
            bpy.ops.object.select_all(action='DESELECT')
            bpy.ops.import_scene.gltf(filepath=str(mesh_path))

            imported_meshes = [o for o in bpy.context.selected_objects if o.type == 'MESH']
            imported_armatures = [o for o in bpy.context.selected_objects if o.type == 'ARMATURE']
            for other_arm in imported_armatures:
                bpy.data.objects.remove(other_arm, do_unlink=True)

            if not imported_meshes:
                raise ValueError(f"No mesh objects found in imported file: {mesh_path}")

            if len(imported_meshes) > 1:
                _select_only(imported_meshes, active=imported_meshes[0])
                bpy.ops.object.join()
                combined_mesh = bpy.context.active_object
            else:
                combined_mesh = imported_meshes[0]

            combined_mesh.name = "Character"

            binding = params.get("binding", {}) or {}
            mode = binding.get("mode", "auto_weights")

            if mode == "auto_weights":
                _parent_mesh_to_armature(combined_mesh, armature, auto_weights=True)
                max_infl = binding.get("max_bone_influences", 4)
                _limit_vertex_group_influences(combined_mesh, int(max_infl))

            elif mode == "rigid":
                vertex_group_map = binding.get("vertex_group_map", {}) or {}
                _apply_vertex_group_map(combined_mesh, vertex_group_map)
                if not combined_mesh.vertex_groups:
                    raise ValueError(
                        "binding.mode=rigid requires vertex groups on the source mesh"
                    )
                _limit_vertex_group_influences(combined_mesh, 1)
                _parent_mesh_to_armature(combined_mesh, armature, auto_weights=False)

            else:
                raise ValueError(f"Unknown binding.mode: {mode}")

        else:
            # classify_skeletal_mesh_kind() should have rejected this already.
            raise ValueError(f"Unhandled skeletal mesh kind: {kind}")

        if combined_mesh is None:
            raise ValueError("Failed to produce a character mesh")

        # Apply materials (if any)
        material_slots = params.get("material_slots", [])
        apply_materials(combined_mesh, material_slots)

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
        if export_settings.get("save_blend", False):
            blend_rel_path = output_rel_path.replace(".glb", ".blend")
            blend_path = out_root / blend_rel_path
            bpy.ops.wm.save_as_mainfile(filepath=str(blend_path))

        duration_ms = int((time.time() - start_time) * 1000)
        write_report(
            report_path,
            ok=True,
            metrics=metrics,
            output_path=output_rel_path,
            blend_path=blend_rel_path,
            duration_ms=duration_ms,
        )

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

        # Apply finger keyframes
        # Property names match setup_finger_controls(): curl_{name}, spread_{name}, curl_{name}_{finger}
        finger_keyframes = params.get("finger_keyframes", [])
        for finger_kf in finger_keyframes:
            time_sec = finger_kf.get("time", 0)
            frame_index = int(time_sec * fps)
            if frame_count > 0:
                frame_index = max(0, min(frame_index, frame_count - 1))
            frame = 1 + frame_index
            controls_name = finger_kf.get("controls", "")
            pose = finger_kf.get("pose", {})

            # Global curl and spread
            curl = pose.get("curl", 0.0)
            spread = pose.get("spread", 0.0)

            # Set curl property and keyframe
            curl_prop = f"curl_{controls_name}"
            if curl_prop in armature:
                armature[curl_prop] = curl
                armature.keyframe_insert(data_path=f'["{curl_prop}"]', frame=frame)

            # Set spread property and keyframe
            spread_prop = f"spread_{controls_name}"
            if spread_prop in armature:
                armature[spread_prop] = spread
                armature.keyframe_insert(data_path=f'["{spread_prop}"]', frame=frame)

            # Per-finger curl overrides (e.g., index_curl, thumb_curl)
            for finger in ["thumb", "index", "middle", "ring", "pinky"]:
                finger_curl_key = f"{finger}_curl"
                if finger_curl_key in pose:
                    prop_name = f"curl_{controls_name}_{finger}"
                    if prop_name in armature:
                        armature[prop_name] = pose[finger_curl_key]
                        armature.keyframe_insert(data_path=f'["{prop_name}"]', frame=frame)

        # Get output path
        outputs = spec.get("outputs", [])
        primary_output = next((o for o in outputs if o.get("kind") == "primary"), None)
        if not primary_output:
            raise ValueError("No primary output specified in spec")

        output_rel_path = primary_output.get("path", "output.glb")
        output_path = out_root / output_rel_path
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Render preview frames BEFORE baking (baking overwrites animation in Blender 5.0 bg mode)
        preview_rel_dir = None
        preview_config = params.get("preview")
        if preview_config:
            preview_rel_dir = output_rel_path.replace(".glb", "_preview")
            preview_dir = out_root / preview_rel_dir

            # Extract keyframe times from params for keyframe-only rendering
            keyframe_times = None
            keyframes_param = params.get("keyframes", [])
            ik_keyframes_param = params.get("ik_keyframes", [])
            if keyframes_param or ik_keyframes_param:
                all_times = set()
                for kf in keyframes_param:
                    if "time" in kf:
                        all_times.add(kf["time"])
                for kf in ik_keyframes_param:
                    if "time" in kf:
                        all_times.add(kf["time"])
                if all_times:
                    keyframe_times = sorted(all_times)

            try:
                preview_result = render_animation_preview_frames(
                    armature,
                    preview_dir,
                    preview_config,
                    1,  # frame_start
                    frame_count,  # frame_end
                    fps,
                    keyframe_times=keyframe_times
                )
                print(f"Rendered {len(preview_result.get('frames', []))} preview frames")
            except Exception as e:
                print(f"Warning: Failed to render preview: {e}")
                import traceback
                traceback.print_exc()

        # Apply bake settings from rig_setup or export settings
        bake_settings = rig_setup.get("bake")
        export_settings = params.get("export", {})

        if bake_settings:
            # Use explicit bake settings
            bake_animation(armature, bake_settings, 1, frame_count)
        elif export_settings.get("bake_transforms", True):
            # Legacy bake behavior
            # NOTE: visual_keying=False to bake the action keyframes directly,
            # not the visual pose (which is rest pose in Blender 5.0 background mode)
            bpy.context.view_layer.objects.active = armature
            bpy.ops.object.mode_set(mode='POSE')
            bpy.ops.pose.select_all(action='SELECT')
            bpy.ops.nla.bake(
                frame_start=1,
                frame_end=frame_count,
                only_selected=False,
                visual_keying=False,  # Changed from True to preserve animation in bg mode
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

        # Preview was rendered before baking - just add metrics if available
        # (preview_rel_dir is already set from earlier)

        duration_ms = int((time.time() - start_time) * 1000)
        write_report(report_path, ok=True, metrics=metrics,
                     output_path=output_rel_path, blend_path=blend_rel_path,
                     preview_path=preview_rel_dir,
                     duration_ms=duration_ms)

    except Exception as e:
        write_report(report_path, ok=False, error=str(e))
        raise


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
