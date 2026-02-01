"""Armature-driven skeletal mesh generation.

This module is the dedicated home for the Blender implementation of the
`skeletal_mesh.armature_driven_v1` recipe.
"""

from __future__ import annotations

import copy
import math
import re
from typing import TYPE_CHECKING

if TYPE_CHECKING:  # pragma: no cover
    import bpy  # type: ignore


_PROFILE_RE = re.compile(r"^(circle|hexagon)\((\d+)\)$")


# ============================================================================
# Step-Based Extrusion Helpers
# ============================================================================


def _parse_step_scale(value) -> tuple[float, float]:
    """Parse step scale value to (sx, sy) tuple."""
    if value is None:
        return (1.0, 1.0)
    if isinstance(value, (int, float)):
        s = float(value)
        return (s, s)
    if isinstance(value, (list, tuple)) and len(value) == 2:
        return (float(value[0]), float(value[1]))
    raise TypeError(f"scale must be a number or [x, y], got {type(value).__name__}: {value!r}")


def _parse_step_tilt(value) -> tuple[float, float]:
    """Parse step tilt value to (x_deg, y_deg) tuple."""
    if value is None:
        return (0.0, 0.0)
    if isinstance(value, (int, float)):
        return (float(value), 0.0)
    if isinstance(value, (list, tuple)) and len(value) == 2:
        return (float(value[0]), float(value[1]))
    raise TypeError(f"tilt must be a number or [x, y], got {type(value).__name__}: {value!r}")


def _parse_step_bulge(value) -> tuple[float, float]:
    """Parse step bulge value to (side, front_back) tuple."""
    if value is None:
        return (1.0, 1.0)
    if isinstance(value, (int, float)):
        b = float(value)
        return (b, b)
    if isinstance(value, (list, tuple)) and len(value) == 2:
        return (float(value[0]), float(value[1]))
    raise TypeError(f"bulge must be a number or [side, fb], got {type(value).__name__}: {value!r}")


def _normalize_extrusion_step(step) -> dict:
    """Normalize step to full dict form."""
    if isinstance(step, (int, float)):
        return {"extrude": float(step)}
    if isinstance(step, dict):
        return step
    raise TypeError(f"extrusion step must be a number or dict, got {type(step).__name__}: {step!r}")


def _parse_profile(profile: str | None) -> tuple[str, int]:
    """Parse a profile string used by armature-driven recipes.

    Supported forms:
    - None -> ("circle", 12)
    - "circle(N)" -> ("circle", N)
    - "hexagon(N)" -> ("circle", N)
    - "square" -> ("square", 4)
    - "rectangle" -> ("rectangle", 4)
    """

    if profile is None:
        return ("circle", 12)

    if not isinstance(profile, str):
        raise TypeError(f"profile must be a str or None, got {type(profile).__name__}: {profile!r}")

    s = profile.strip()
    if s == "square":
        return ("square", 4)
    if s == "rectangle":
        return ("rectangle", 4)

    m = _PROFILE_RE.match(s)
    if m is not None:
        kind, n_s = m.group(1), m.group(2)
        n = int(n_s)
        if n < 3:
            raise ValueError(f"profile segments must be >= 3, got {n}")
        if kind in ("circle", "hexagon"):
            return ("circle", n)

    raise ValueError(
        f"unknown profile string: {profile!r}; expected one of: None, 'square', 'rectangle', 'circle(N)', 'hexagon(N)'"
    )


def are_profiles_compatible(count_a: int, count_b: int) -> bool:
    """Check if two profile segment counts are compatible for bridging.

    Compatible means: exact match OR one is exactly 2x the other.
    """
    if count_a == count_b:
        return True
    if count_a == 2 * count_b or count_b == 2 * count_a:
        return True
    return False


def get_bridge_pairs(bone_hierarchy: dict, bone_meshes: dict) -> list[tuple[str, str]]:
    """Determine which bone pairs should have their edge loops bridged.

    A bridge is created when:
    1. Parent bone's mesh has connect_end="bridge"
    2. Child bone's mesh has connect_start="bridge"
    3. Both bones have mesh definitions (not just mirrors)

    Args:
        bone_hierarchy: Dict mapping bone_name -> {"parent": str|None, "children": [str]}
        bone_meshes: Dict mapping bone_name -> bone mesh definition dict

    Returns:
        List of (parent_bone, child_bone) tuples to bridge.
    """
    pairs = []

    for child_name, info in bone_hierarchy.items():
        parent_name = info.get("parent")
        if parent_name is None:
            continue

        parent_mesh = bone_meshes.get(parent_name)
        child_mesh = bone_meshes.get(child_name)

        if parent_mesh is None or child_mesh is None:
            continue

        # Skip mirror references - they resolve to their target's settings
        if isinstance(parent_mesh, dict) and "mirror" in parent_mesh:
            continue
        if isinstance(child_mesh, dict) and "mirror" in child_mesh:
            continue

        parent_connect_end = parent_mesh.get("connect_end")
        child_connect_start = child_mesh.get("connect_start")

        if parent_connect_end == "bridge" and child_connect_start == "bridge":
            pairs.append((parent_name, child_name))

    return pairs


def get_bridge_head_vgroup_name(bone_name: str) -> str:
    """Get the vertex group name for a bone's head (start) edge loop."""
    return f"_bridge_head_{bone_name}"


def get_bridge_tail_vgroup_name(bone_name: str) -> str:
    """Get the vertex group name for a bone's tail (end) edge loop."""
    return f"_bridge_tail_{bone_name}"


def _build_bone_hierarchy(armature) -> dict:
    """Build bone hierarchy dict from armature.

    Returns:
        Dict mapping bone_name -> {"parent": str|None, "children": [str]}
    """
    hierarchy = {}

    for bone in armature.data.bones:
        parent_name = bone.parent.name if bone.parent else None
        hierarchy[bone.name] = {
            "parent": parent_name,
            "children": [],
        }

    # Fill in children
    for bone_name, info in hierarchy.items():
        parent_name = info["parent"]
        if parent_name and parent_name in hierarchy:
            hierarchy[parent_name]["children"].append(bone_name)

    return hierarchy


def _perform_bridge_operations(
    *,
    bpy_module,
    bmesh_module,
    mesh_obj,
    bridge_pairs: list[tuple[str, str]],
    armature,
) -> None:
    """Bridge edge loops between connected bone pairs.

    Args:
        bpy_module: Blender bpy module
        bmesh_module: Blender bmesh module
        mesh_obj: The combined mesh object
        bridge_pairs: List of (parent_bone, child_bone) tuples to bridge
    """
    if not bridge_pairs:
        return

    bpy = bpy_module

    # Enter edit mode
    bpy.context.view_layer.objects.active = mesh_obj
    bpy.ops.object.mode_set(mode='EDIT')

    try:
        for parent_bone, child_bone in bridge_pairs:
            tail_vg = get_bridge_tail_vgroup_name(parent_bone)
            head_vg = get_bridge_head_vgroup_name(child_bone)

            # Check if both vertex groups exist
            if tail_vg not in mesh_obj.vertex_groups:
                continue
            if head_vg not in mesh_obj.vertex_groups:
                continue

            # Deselect all
            bpy.ops.mesh.select_all(action='DESELECT')

            # Select vertices in parent's tail group
            bpy.ops.object.mode_set(mode='OBJECT')
            tail_vg_idx = mesh_obj.vertex_groups[tail_vg].index
            for v in mesh_obj.data.vertices:
                for g in v.groups:
                    if g.group == tail_vg_idx:
                        v.select = True
                        break

            # Also select vertices in child's head group
            head_vg_idx = mesh_obj.vertex_groups[head_vg].index
            for v in mesh_obj.data.vertices:
                for g in v.groups:
                    if g.group == head_vg_idx:
                        v.select = True
                        break

            bpy.ops.object.mode_set(mode='EDIT')

            # Bridge the selected edge loops
            try:
                bpy.ops.mesh.bridge_edge_loops()
            except RuntimeError as e:
                # Bridge can fail if selection isn't valid edge loops
                import sys
                print(f"Warning: bridge_edge_loops failed for {parent_bone}->{child_bone}: {e}", file=sys.stderr)

        # Clean up: merge very close vertices at bridge points
        bpy.ops.mesh.select_all(action='SELECT')
        bpy.ops.mesh.remove_doubles(threshold=0.0001)

    finally:
        bpy.ops.object.mode_set(mode='OBJECT')

    # Blend weights for bridged regions
    for parent_bone, child_bone in bridge_pairs:
        _blend_bridge_weights(
            bpy_module=bpy,
            mesh_obj=mesh_obj,
            parent_bone=parent_bone,
            child_bone=child_bone,
            armature=armature,
        )

    # Remove temporary bridge vertex groups
    for parent_bone, child_bone in bridge_pairs:
        for vg_name in [
            get_bridge_tail_vgroup_name(parent_bone),
            get_bridge_head_vgroup_name(child_bone),
        ]:
            if vg_name in mesh_obj.vertex_groups:
                mesh_obj.vertex_groups.remove(mesh_obj.vertex_groups[vg_name])


def _blend_bridge_weights(
    *,
    bpy_module,
    mesh_obj,
    parent_bone: str,
    child_bone: str,
    armature,
) -> None:
    """Blend skin weights for vertices in the bridge region.

    Vertices in the bridge get interpolated weights between parent and child bones
    based on their position along the bridge axis.
    """
    bpy = bpy_module

    # Get bone positions for interpolation
    parent_bone_obj = armature.data.bones.get(parent_bone)
    child_bone_obj = armature.data.bones.get(child_bone)

    if not parent_bone_obj or not child_bone_obj:
        return

    # Parent tail and child head positions in armature space
    parent_tail = armature.matrix_world @ parent_bone_obj.tail_local
    child_head = armature.matrix_world @ child_bone_obj.head_local

    # Bridge axis
    bridge_vec = child_head - parent_tail
    bridge_len = bridge_vec.length

    if bridge_len < 1e-6:
        return  # Bones are at same position

    bridge_dir = bridge_vec.normalized()

    # Get or create vertex groups for the bones
    parent_vg = mesh_obj.vertex_groups.get(parent_bone)
    child_vg = mesh_obj.vertex_groups.get(child_bone)

    if not parent_vg:
        parent_vg = mesh_obj.vertex_groups.new(name=parent_bone)
    if not child_vg:
        child_vg = mesh_obj.vertex_groups.new(name=child_bone)

    # Find vertices in bridge region (between parent tail and child head)
    mesh = mesh_obj.data
    world_matrix = mesh_obj.matrix_world

    for v in mesh.vertices:
        v_world = world_matrix @ v.co

        # Project vertex onto bridge axis
        to_vert = v_world - parent_tail
        proj_dist = to_vert.dot(bridge_dir)

        # Check if vertex is in bridge region
        if proj_dist < -0.001 or proj_dist > bridge_len + 0.001:
            continue  # Outside bridge region

        # Calculate blend factor (0 = parent, 1 = child)
        t = max(0.0, min(1.0, proj_dist / bridge_len))

        parent_weight = 1.0 - t
        child_weight = t

        # Apply blended weights
        parent_vg.add([v.index], parent_weight, 'REPLACE')
        child_vg.add([v.index], child_weight, 'REPLACE')


def _resolve_mirrors(defs: dict) -> dict:
    """Resolve `{ "mirror": "other" }` entries to concrete dicts.

    This is a pure-Python helper used by the Blender backend config layer.
    """

    if defs is None:
        return {}

    if not isinstance(defs, dict):
        raise TypeError("defs must be a dict")

    visiting: set[str] = set()
    visited: dict[str, dict] = {}

    def resolve_base(key: str) -> dict:
        if key in visited:
            return visited[key]

        if key in visiting:
            raise ValueError(f"mirror cycle detected at '{key}'")

        if key not in defs:
            raise ValueError(f"mirror target '{key}' not found")

        value = defs[key]
        if not isinstance(value, dict):
            raise TypeError(f"defs['{key}'] must be a dict")

        visiting.add(key)
        try:
            if "mirror" in value:
                if set(value.keys()) != {"mirror"}:
                    raise ValueError(
                        f"mirror-only dict must be exactly {{'mirror': ...}} for '{key}'"
                    )
                target = value.get("mirror")
                if not isinstance(target, str):
                    raise TypeError(f"defs['{key}']['mirror'] must be a str")
                base = resolve_base(target)
            else:
                base = copy.deepcopy(value)

            visited[key] = base
            return base
        finally:
            visiting.remove(key)

    resolved: dict[str, dict] = {}
    for key in sorted(defs.keys()):
        if not isinstance(key, str):
            raise TypeError("defs keys must be str")
        resolved[key] = copy.deepcopy(resolve_base(key))
    return resolved


def _resolve_bone_relative_length(
    value, *, bone_length: float
) -> float | tuple[float, float]:
    """Decode BoneRelativeLength values.

    Supported forms:
    - number: relative length, scaled by bone_length -> float
    - [x, y] / (x, y): relative2 length, scaled by bone_length -> (float, float)
    - {"absolute": a}: absolute length -> float
    """

    if isinstance(bone_length, bool) or not isinstance(bone_length, (int, float)):
        raise TypeError(
            f"bone_length must be a number, got {type(bone_length).__name__}: {bone_length!r}"
        )
    bone_length_f = float(bone_length)
    if not math.isfinite(bone_length_f) or bone_length_f <= 0.0:
        raise ValueError(
            f"bone_length must be a finite, positive number, got {bone_length_f!r}"
        )

    if isinstance(value, bool):
        raise TypeError(f"value must be a number/dict/list/tuple, got bool: {value!r}")

    if isinstance(value, (int, float)):
        out = float(value) * bone_length_f
        if not math.isfinite(out) or out <= 0.0:
            raise ValueError(
                f"resolved length must be a finite, positive number, got {out!r}"
            )
        return out

    if isinstance(value, (list, tuple)):
        if len(value) != 2:
            raise TypeError("relative2 value must be a list/tuple of length 2")
        x, y = value
        if isinstance(x, bool) or not isinstance(x, (int, float)):
            raise TypeError("relative2[0] must be a number")
        if isinstance(y, bool) or not isinstance(y, (int, float)):
            raise TypeError("relative2[1] must be a number")
        out_x = float(x) * bone_length_f
        out_y = float(y) * bone_length_f
        if not math.isfinite(out_x) or out_x <= 0.0:
            raise ValueError(
                f"resolved relative2[0] length must be a finite, positive number, got {out_x!r}"
            )
        if not math.isfinite(out_y) or out_y <= 0.0:
            raise ValueError(
                f"resolved relative2[1] length must be a finite, positive number, got {out_y!r}"
            )
        return (out_x, out_y)

    if isinstance(value, dict):
        if set(value.keys()) != {"absolute"}:
            keys = sorted(value.keys())
            raise ValueError(
                f"absolute form dict must have exactly {{'absolute'}}, got keys {keys!r}"
            )
        a = value.get("absolute")
        if isinstance(a, bool) or not isinstance(a, (int, float)):
            raise TypeError(
                f"absolute value must be a number, got {type(a).__name__}: {a!r}"
            )
        out = float(a)
        if not math.isfinite(out) or out <= 0.0:
            raise ValueError(
                f"resolved length must be a finite, positive number, got {out!r}"
            )
        return out

    raise TypeError(
        f"unsupported BoneRelativeLength form: {type(value).__name__}: {value!r}"
    )


def _resolve_profile_radius(mesh_spec: dict, *, bone_length: float) -> tuple[float, float]:
    """Resolve profile_radius to (rx, ry) tuple in absolute units."""
    radius_val = _resolve_bone_relative_length(
        mesh_spec.get('profile_radius', 0.15),
        bone_length=bone_length,
    )
    if isinstance(radius_val, tuple):
        rx, ry = radius_val
    else:
        rx, ry = (radius_val, radius_val)
    rx = float(rx)
    ry = float(ry)

    if not math.isfinite(rx) or rx <= 0.0:
        raise ValueError(f"profile_radius.x must be > 0, got {rx!r}")
    if not math.isfinite(ry) or ry <= 0.0:
        raise ValueError(f"profile_radius.y must be > 0, got {ry!r}")

    return (rx, ry)


def _build_mesh_with_steps(
    *,
    bpy_module,
    bmesh_module,
    bone_name: str,
    bone_length: float,
    head_w,
    base_q,
    profile: tuple[str, int],
    profile_radius: tuple[float, float],
    steps: list,
    cap_start: bool,
    cap_end: bool,
    ensure_object_mode,
    select_only,
    apply_scale_only,
    apply_rotation_only,
    quat_rotate_vec,
    connect_start: str | None = None,
    connect_end: str | None = None,
) -> object:
    """Build mesh using step-based extrusion along bone axis.

    True box modeling: start with a profile, extrude, modify, extrude, modify.
    All distances are bone-relative, so resizing the skeleton preserves silhouette.

    Args:
        bpy_module: Blender bpy module.
        bmesh_module: Blender bmesh module.
        bone_name: Name of the bone (for error messages).
        bone_length: Length of the bone in world units.
        head_w: Bone head position in world coordinates.
        base_q: Base quaternion rotation aligning Z to bone axis.
        profile: Tuple of (kind, segments) from _parse_profile.
        profile_radius: Tuple of (rx, ry) radii in absolute units.
        steps: List of extrusion step definitions.
        cap_start: Whether to keep the cap at bone head.
        cap_end: Whether to keep the cap at bone tail.
        ensure_object_mode: Callback to ensure object mode.
        select_only: Callback to select objects.
        apply_scale_only: Callback to apply scale transform.
        apply_rotation_only: Callback to apply rotation transform.
        quat_rotate_vec: Callback to rotate vector by quaternion.

    Returns:
        The generated mesh object.
    """
    from mathutils import Vector

    bpy = bpy_module
    bmesh = bmesh_module
    kind, segments = profile
    rx, ry = profile_radius

    # Create initial profile at bone head as a filled polygon (NGON)
    # The circle primitive creates vertices at radius, we scale afterward.
    bpy.ops.mesh.primitive_circle_add(
        vertices=int(segments),
        radius=1.0,
        fill_type='NGON',
        location=head_w,
    )
    obj = bpy.context.active_object
    if obj is None:
        raise RuntimeError("primitive_circle_add: expected an active object, got None")

    # Apply elliptical radius via scale
    obj.scale = (rx, ry, 1.0)
    apply_scale_only(obj)

    # Align circle normal (local Z) with bone axis
    obj.rotation_mode = 'QUATERNION'
    obj.rotation_quaternion = base_q
    apply_rotation_only(obj)

    obj.name = f"Segment_{bone_name}"

    # Process each extrusion step
    for si, step in enumerate(steps):
        step = _normalize_extrusion_step(step)

        extrude_frac = step.get("extrude", 0.1)
        if not isinstance(extrude_frac, (int, float)) or extrude_frac <= 0:
            raise ValueError(
                f"bone_meshes['{bone_name}'].extrusion_steps[{si}].extrude must be > 0, got {extrude_frac!r}"
            )
        extrude_dist = float(extrude_frac) * bone_length

        sx, sy = _parse_step_scale(step.get("scale"))
        translate = step.get("translate", [0, 0, 0])
        if translate is None:
            translate = [0, 0, 0]
        rotate_deg = float(step.get("rotate", 0) or 0)
        tilt_x, tilt_y = _parse_step_tilt(step.get("tilt"))
        bulge_x, bulge_y = _parse_step_bulge(step.get("bulge"))

        ensure_object_mode()
        select_only([obj], active=obj)
        bpy.ops.object.mode_set(mode='EDIT')

        # Select top faces for extrusion (faces with highest Z in local space)
        mesh = obj.data
        bm = bmesh.from_edit_mesh(mesh)
        bm.faces.ensure_lookup_table()

        if not bm.faces:
            raise RuntimeError(f"bone_meshes['{bone_name}'].extrusion_steps[{si}]: mesh has no faces")

        # Find the top faces (highest local Z after transforms are applied)
        # Since we already applied rotation, the mesh is now in world space oriented along bone.
        # In edit mode, co gives local coordinates. Z axis is along the bone.
        bm.verts.ensure_lookup_table()
        if bm.verts:
            z_coords = [v.co.z for v in bm.verts]
            z_max = max(z_coords)
            z_range = z_max - min(z_coords)
            eps = max(1e-6, z_range * 0.01)

            # Deselect all, then select top faces
            for f in bm.faces:
                f.select = False
            for e in bm.edges:
                e.select = False
            for v in bm.verts:
                v.select = False

            for f in bm.faces:
                if all(abs(v.co.z - z_max) <= eps for v in f.verts):
                    f.select = True

            bmesh.update_edit_mesh(mesh)

        # Extrude along local Z (bone axis)
        bpy.ops.mesh.extrude_region_move(
            TRANSFORM_OT_translate={'value': (0, 0, extrude_dist)}
        )

        # Apply scale combined with bulge
        final_sx = sx * bulge_x
        final_sy = sy * bulge_y
        if final_sx != 1.0 or final_sy != 1.0:
            bpy.ops.transform.resize(
                value=(final_sx, final_sy, 1.0),
                orient_type='LOCAL',
            )

        # Apply translation (bone-relative, scaled by bone_length)
        if any(t != 0 for t in translate):
            tx = float(translate[0]) * bone_length
            ty = float(translate[1]) * bone_length
            tz = float(translate[2]) * bone_length
            bpy.ops.transform.translate(
                value=(tx, ty, tz),
                orient_type='LOCAL',
            )

        # Apply Z-axis rotation
        if rotate_deg != 0:
            bpy.ops.transform.rotate(
                value=math.radians(rotate_deg),
                orient_axis='Z',
                orient_type='LOCAL',
            )

        # Apply tilt (X/Y rotation)
        if tilt_x != 0:
            bpy.ops.transform.rotate(
                value=math.radians(tilt_x),
                orient_axis='X',
                orient_type='LOCAL',
            )
        if tilt_y != 0:
            bpy.ops.transform.rotate(
                value=math.radians(tilt_y),
                orient_axis='Y',
                orient_type='LOCAL',
            )

        bpy.ops.object.mode_set(mode='OBJECT')

    # Handle end caps: remove bottom (start) or top (end) faces if needed
    ensure_object_mode()
    if not cap_start or not cap_end:
        mesh = obj.data
        bm = bmesh.new()
        try:
            bm.from_mesh(mesh)
            bm.verts.ensure_lookup_table()
            bm.faces.ensure_lookup_table()

            if bm.verts and bm.faces:
                z_coords = [v.co.z for v in bm.verts]
                z_min = min(z_coords)
                z_max = max(z_coords)
                z_range = z_max - z_min
                eps = max(1e-6, z_range * 0.01)

                faces_to_delete = []
                for f in bm.faces:
                    zs = [v.co.z for v in f.verts]
                    if not zs:
                        continue
                    if not cap_start and all(abs(z - z_min) <= eps for z in zs):
                        faces_to_delete.append(f)
                    elif not cap_end and all(abs(z - z_max) <= eps for z in zs):
                        faces_to_delete.append(f)

                if faces_to_delete:
                    bmesh.ops.delete(bm, geom=faces_to_delete, context='FACES')
                    bm.to_mesh(mesh)
        finally:
            bm.free()
            try:
                mesh.update()
            except Exception:
                pass

    # Track edge loops for bridging via vertex groups
    if connect_start == "bridge" or connect_end == "bridge":
        mesh = obj.data
        bm = bmesh_module.new()
        try:
            bm.from_mesh(mesh)
            bm.verts.ensure_lookup_table()

            if bm.verts:
                z_coords = [v.co.z for v in bm.verts]
                z_min = min(z_coords)
                z_max = max(z_coords)
                z_range = z_max - z_min
                eps = max(1e-6, z_range * 0.01)

                head_verts = []
                tail_verts = []

                for v in bm.verts:
                    if abs(v.co.z - z_min) <= eps:
                        head_verts.append(v.index)
                    elif abs(v.co.z - z_max) <= eps:
                        tail_verts.append(v.index)

                bm.free()

                # Create vertex groups in object mode
                if connect_start == "bridge" and head_verts:
                    vg_name = get_bridge_head_vgroup_name(bone_name)
                    vg = obj.vertex_groups.new(name=vg_name)
                    vg.add(head_verts, 1.0, 'REPLACE')

                if connect_end == "bridge" and tail_verts:
                    vg_name = get_bridge_tail_vgroup_name(bone_name)
                    vg = obj.vertex_groups.new(name=vg_name)
                    vg.add(tail_verts, 1.0, 'REPLACE')
            else:
                bm.free()
        except Exception:
            try:
                bm.free()
            except Exception:
                pass
            raise

    return obj


def build_armature_driven_character_mesh(*, armature, params: dict, out_root) -> object:
    """Build a character mesh driven by an existing armature.

    Args:
        armature: Blender armature object.
        params: Recipe params for `skeletal_mesh.armature_driven_v1`.
        out_root: Spec output root (Path-like).

    Returns:
        The generated mesh object.
    """

    try:
        import bpy  # type: ignore
        import bmesh  # type: ignore
        from mathutils import Euler, Vector  # type: ignore
    except Exception as e:  # pragma: no cover
        raise RuntimeError("Blender Python (bpy) is required") from e

    if armature is None:
        raise TypeError("armature is required")
    if getattr(armature, 'type', None) != 'ARMATURE':
        raise TypeError(
            f"armature must be a Blender ARMATURE object, got {getattr(armature, 'type', None)!r}"
        )

    def _select_only(objs, *, active=None) -> None:
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

    def _apply_scale_only(obj) -> None:
        _ensure_object_mode()
        _select_only([obj], active=obj)
        bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)

    def _apply_rotation_only(obj) -> None:
        _ensure_object_mode()
        _select_only([obj], active=obj)
        bpy.ops.object.transform_apply(location=False, rotation=True, scale=False)

    def _apply_modifier(obj, *, modifier_name: str) -> None:
        if not isinstance(modifier_name, str) or not modifier_name:
            raise TypeError("modifier_name must be a non-empty str")
        _ensure_object_mode()
        _select_only([obj], active=obj)
        bpy.ops.object.modifier_apply(modifier=modifier_name)

    def _subdivide_all_edges(obj, *, cuts: int) -> None:
        if getattr(obj, 'type', None) != 'MESH':
            return
        if isinstance(cuts, bool) or not isinstance(cuts, int):
            raise TypeError(f"subdivide.cuts must be an int, got {type(cuts).__name__}: {cuts!r}")
        if cuts < 1:
            raise ValueError(f"subdivide.cuts must be >= 1, got {cuts!r}")

        mesh = obj.data
        bm = bmesh.new()
        try:
            bm.from_mesh(mesh)
            bm.verts.ensure_lookup_table()
            bm.edges.ensure_lookup_table()
            if not bm.edges:
                return

            # Subdivide deterministically by operating on the current edge list.
            bmesh.ops.subdivide_edges(
                bm,
                edges=list(bm.edges),
                cuts=cuts,
                use_grid_fill=True,
                smooth=0.0,
            )
            bm.to_mesh(mesh)
        finally:
            bm.free()
            try:
                mesh.update()
            except Exception:
                pass

    def _quat_mul(a, b):
        # Blender versions differ on whether Quaternion supports @ for multiplication.
        try:
            return a @ b
        except TypeError:
            return a * b

    def _quat_rotate_vec(q, v):
        # Blender versions differ on whether Quaternion supports @ for rotating vectors.
        try:
            return q @ v
        except TypeError:
            return q * v

    def _parse_vec3(value, *, name: str) -> tuple[float, float, float] | None:
        if value is None:
            return None
        if not isinstance(value, (list, tuple)) or len(value) != 3:
            raise TypeError(f"{name} must be a 3-element list/tuple")
        x, y, z = value
        for c in (x, y, z):
            if isinstance(c, bool) or not isinstance(c, (int, float)):
                raise TypeError(f"{name} components must be numbers")

        xf, yf, zf = (float(x), float(y), float(z))
        if not (math.isfinite(xf) and math.isfinite(yf) and math.isfinite(zf)):
            raise ValueError(
                f"{name} components must be finite numbers, got ({xf!r}, {yf!r}, {zf!r})"
            )

        return (xf, yf, zf)

    def _parse_finite_number(value, *, name: str, default: float | None = None) -> float:
        if value is None:
            if default is None:
                raise TypeError(f"{name} is required")
            return float(default)
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            raise TypeError(f"{name} must be a number")
        out = float(value)
        if not math.isfinite(out):
            raise ValueError(f"{name} must be a finite number, got {out!r}")
        return out

    def _recalculate_normals(obj) -> None:
        _ensure_object_mode()
        _select_only([obj], active=obj)
        bpy.ops.object.mode_set(mode='EDIT')
        bpy.ops.mesh.select_all(action='SELECT')
        bpy.ops.mesh.normals_make_consistent(inside=False)
        bpy.ops.object.mode_set(mode='OBJECT')

    def _ensure_armature_modifier(mesh_obj, armature_obj) -> None:
        for mod in mesh_obj.modifiers:
            if mod.type == 'ARMATURE':
                mod.object = armature_obj
                return
        mod = mesh_obj.modifiers.new(name='Armature', type='ARMATURE')
        mod.object = armature_obj

    def _parent_mesh_to_armature(mesh_obj, armature_obj) -> None:
        _ensure_object_mode()
        _select_only([mesh_obj, armature_obj], active=armature_obj)
        bpy.ops.object.parent_set(type='ARMATURE')
        _ensure_armature_modifier(mesh_obj, armature_obj)

    def _ensure_uvs(mesh_obj) -> None:
        if len(mesh_obj.data.uv_layers) != 0:
            return

        mesh_obj.data.uv_layers.new(name='UVMap')

        _ensure_object_mode()
        _select_only([mesh_obj], active=mesh_obj)
        try:
            bpy.ops.object.mode_set(mode='EDIT')
            bpy.ops.mesh.select_all(action='SELECT')
            try:
                bpy.ops.uv.smart_project(island_margin=0.02)
            except Exception as e:
                print(
                    f"WARN: uv.smart_project failed for '{mesh_obj.name}', continuing without unwrap: {e.__class__.__name__}: {e}"
                )
        finally:
            try:
                bpy.ops.object.mode_set(mode='OBJECT')
            except Exception:
                # In background mode, mode switching can fail if no active object.
                pass

    def _primitive_add_with_uvs(add_op, **kwargs) -> None:
        try:
            add_op(calc_uvs=True, **kwargs)
        except TypeError:
            # Older Blender builds do not support calc_uvs.
            add_op(**kwargs)

    def _join_into(base_obj, other_obj) -> object:
        _ensure_object_mode()
        _select_only([base_obj, other_obj], active=base_obj)
        bpy.ops.object.join()
        joined = bpy.context.active_object
        if joined is None:
            raise RuntimeError("object.join: expected an active object, got None")
        return joined

    def _create_primitive_mesh(*, primitive: str, location_w) -> object:
        prim = primitive.strip().lower()
        if prim == 'cube':
            _primitive_add_with_uvs(bpy.ops.mesh.primitive_cube_add, size=1.0, location=location_w)
            return _require_active_mesh(context='primitive_cube_add(attachment)')
        if prim in ('sphere', 'uv_sphere'):
            # Fixed tessellation for determinism.
            _primitive_add_with_uvs(
                bpy.ops.mesh.primitive_uv_sphere_add,
                radius=0.5,
                segments=16,
                ring_count=8,
                location=location_w,
            )
            return _require_active_mesh(context='primitive_uv_sphere_add(attachment)')
        if prim == 'ico_sphere':
            _primitive_add_with_uvs(
                bpy.ops.mesh.primitive_ico_sphere_add,
                radius=0.5,
                subdivisions=2,
                location=location_w,
            )
            return _require_active_mesh(context='primitive_ico_sphere_add(attachment)')
        if prim == 'cylinder':
            _primitive_add_with_uvs(
                bpy.ops.mesh.primitive_cylinder_add,
                radius=0.5,
                depth=1.0,
                vertices=16,
                location=location_w,
            )
            return _require_active_mesh(context='primitive_cylinder_add(attachment)')
        if prim == 'cone':
            _primitive_add_with_uvs(
                bpy.ops.mesh.primitive_cone_add,
                radius1=0.5,
                depth=1.0,
                vertices=16,
                location=location_w,
            )
            return _require_active_mesh(context='primitive_cone_add(attachment)')
        if prim == 'torus':
            _primitive_add_with_uvs(
                bpy.ops.mesh.primitive_torus_add,
                major_radius=0.35,
                minor_radius=0.15,
                major_segments=24,
                minor_segments=12,
                location=location_w,
            )
            return _require_active_mesh(context='primitive_torus_add(attachment)')
        if prim == 'plane':
            _primitive_add_with_uvs(bpy.ops.mesh.primitive_plane_add, size=1.0, location=location_w)
            return _require_active_mesh(context='primitive_plane_add(attachment)')
        raise ValueError(
            f"unsupported attachment primitive: {primitive!r}; expected one of: cube, sphere, ico_sphere, cylinder, cone, torus, plane"
        )

    def _require_active_mesh(*, context: str):
        obj = bpy.context.active_object
        if obj is None:
            raise RuntimeError(f"{context}: expected an active object, got None")
        if getattr(obj, 'type', None) != 'MESH':
            raise RuntimeError(
                f"{context}: expected a MESH active object, got {getattr(obj, 'type', None)!r}"
            )
        return obj

    if not isinstance(params, dict):
        raise TypeError("params must be a dict")

    material_slots = None
    material_slot_count = None
    if 'material_slots' in params:
        material_slots = params.get('material_slots')
        if material_slots is None:
            material_slots = []
        if not isinstance(material_slots, list):
            raise TypeError("params.material_slots must be a list")
        material_slot_count = len(material_slots)

    placeholder_materials = None

    def _parse_material_index(value, *, default: int, name: str) -> int:
        if value is None:
            idx = int(default)
        else:
            if isinstance(value, bool) or not isinstance(value, int):
                raise TypeError(f"{name} must be an int")
            idx = int(value)

        if idx < 0:
            raise ValueError(f"{name} must be >= 0, got {idx!r}")

        if material_slot_count is not None:
            # If explicit slots are provided, enforce index bounds.
            if material_slot_count == 0:
                if idx != 0:
                    raise ValueError(
                        f"{name}={idx} out of range for material_slots length {material_slot_count}"
                    )
            else:
                if idx >= material_slot_count:
                    raise ValueError(
                        f"{name}={idx} out of range for material_slots length {material_slot_count}"
                    )

        return idx

    def _set_all_poly_material_index(mesh_obj, *, material_index: int) -> None:
        if getattr(mesh_obj, 'type', None) != 'MESH':
            return
        _ensure_material_slot_placeholders(mesh_obj)
        mesh = mesh_obj.data
        try:
            polys = mesh.polygons
        except Exception:
            return
        for p in polys:
            p.material_index = int(material_index)

    def _ensure_material_slot_placeholders(mesh_obj) -> None:
        """Ensure stable material slots for polygon material_index and joins.

        Blender's join operation can collapse empty/None material slots.
        To keep per-face material_index stable across joins, assign lightweight
        placeholder Material objects per slot index.

        The outer handler later calls apply_materials() to replace these slots
        with recipe materials.
        """

        if material_slot_count is None:
            return
        if getattr(mesh_obj, 'type', None) != 'MESH':
            return

        nonlocal placeholder_materials

        desired = int(material_slot_count)
        mats = mesh_obj.data.materials

        if desired <= 0:
            while len(mats) > 0:
                mats.pop()
            return

        if placeholder_materials is None:
            placeholder_materials = []

        while len(placeholder_materials) < desired:
            idx = len(placeholder_materials)
            name = f"__speccade_material_slot_{idx}"
            mat = bpy.data.materials.get(name)
            if mat is None:
                mat = bpy.data.materials.new(name=name)
                try:
                    mat.use_nodes = False
                except Exception:
                    pass
            placeholder_materials.append(mat)

        while len(mats) > desired:
            mats.pop()
        while len(mats) < desired:
            mats.append(None)

        for i in range(desired):
            mats[i] = placeholder_materials[i]

    bool_shapes = _resolve_mirrors(params.get('bool_shapes', {}) or {})
    bone_meshes = _resolve_mirrors(params.get('bone_meshes', {}) or {})
    if not bone_meshes:
        raise ValueError("params.bone_meshes is required")

    _ensure_object_mode()

    bool_shape_objs = {}

    # Create boolean target shapes up-front so they can be referenced by modifiers.
    for shape_key, shape_spec in sorted(bool_shapes.items(), key=lambda kv: kv[0]):
        if not isinstance(shape_key, str) or not shape_key:
            raise TypeError(f"bool_shapes keys must be non-empty str, got {shape_key!r}")
        if not isinstance(shape_spec, dict):
            raise TypeError(f"bool_shapes['{shape_key}'] must be a dict")

        primitive = shape_spec.get('primitive')
        if not isinstance(primitive, str) or not primitive:
            raise TypeError(f"bool_shapes['{shape_key}'].primitive must be a non-empty str")
        primitive_lc = primitive.strip().lower()

        position = _parse_vec3(shape_spec.get('position'), name=f"bool_shapes['{shape_key}'].position")
        if position is None:
            raise TypeError(f"bool_shapes['{shape_key}'].position is required")
        dimensions = _parse_vec3(shape_spec.get('dimensions'), name=f"bool_shapes['{shape_key}'].dimensions")
        if dimensions is None:
            raise TypeError(f"bool_shapes['{shape_key}'].dimensions is required")

        px, py, pz = position
        dx, dy, dz = dimensions
        if dx <= 0.0 or dy <= 0.0 or dz <= 0.0:
            raise ValueError(
                f"bool_shapes['{shape_key}'].dimensions must be > 0, got ({dx!r}, {dy!r}, {dz!r})"
            )

        # Determine placement frame.
        bone_name = shape_spec.get('bone')
        if bone_name is not None and (not isinstance(bone_name, str) or not bone_name):
            raise TypeError(f"bool_shapes['{shape_key}'].bone must be a non-empty str")

        if bone_name is not None:
            bone = armature.data.bones.get(bone_name)
            if bone is None:
                raise ValueError(f"bool_shapes['{shape_key}'].bone '{bone_name}' not found in armature")

            head_w = armature.matrix_world @ bone.head_local
            tail_w = armature.matrix_world @ bone.tail_local
            axis = tail_w - head_w
            length = float(axis.length)
            if not math.isfinite(length) or length < 1e-6:
                raise ValueError(f"bone '{bone_name}' has near-zero length: {length!r}")
            axis_n = axis.normalized()
            base_q = axis_n.to_track_quat('Z', 'Y')

            offset_local = Vector((px * length, py * length, pz * length))
            location_w = head_w + _quat_rotate_vec(base_q, offset_local)
            scale = (dx * length, dy * length, dz * length)
            rotation_q = base_q
        else:
            location_w = armature.matrix_world @ Vector((px, py, pz))
            scale = (dx, dy, dz)
            try:
                rotation_q = armature.matrix_world.to_quaternion()
            except Exception:
                rotation_q = None

        # Create primitive in world space with deterministic sizing.
        if primitive_lc == 'cube':
            bpy.ops.mesh.primitive_cube_add(size=1.0, location=location_w)
            obj = _require_active_mesh(context='primitive_cube_add(bool_shape)')
        elif primitive_lc in ('sphere', 'uv_sphere'):
            bpy.ops.mesh.primitive_uv_sphere_add(radius=0.5, location=location_w)
            obj = _require_active_mesh(context='primitive_uv_sphere_add(bool_shape)')
        elif primitive_lc == 'cylinder':
            bpy.ops.mesh.primitive_cylinder_add(radius=0.5, depth=1.0, location=location_w)
            obj = _require_active_mesh(context='primitive_cylinder_add(bool_shape)')
        else:
            raise ValueError(
                f"bool_shapes['{shape_key}'].primitive unsupported: {primitive!r}; expected 'cube'|'sphere'|'cylinder'"
            )

        obj.name = f"BoolShape_{shape_key}"
        obj.scale = scale
        _apply_scale_only(obj)

        if rotation_q is not None:
            obj.rotation_mode = 'QUATERNION'
            obj.rotation_quaternion = rotation_q
            _apply_rotation_only(obj)

        try:
            obj.hide_set(True)
        except Exception:
            try:
                obj.hide = True
            except Exception:
                pass
        try:
            obj.hide_render = True
        except Exception:
            pass

        bool_shape_objs[shape_key] = obj

    segment_objs = []

    for bone_name, mesh_spec in sorted(bone_meshes.items(), key=lambda kv: kv[0]):
        if not isinstance(bone_name, str) or not bone_name:
            raise TypeError(f"bone_meshes keys must be non-empty str, got {bone_name!r}")
        if not isinstance(mesh_spec, dict):
            raise TypeError(f"bone_meshes['{bone_name}'] must be a dict")

        bone = armature.data.bones.get(bone_name)
        if bone is None:
            raise ValueError(f"bone '{bone_name}' not found in armature")

        head_w = armature.matrix_world @ bone.head_local
        tail_w = armature.matrix_world @ bone.tail_local
        axis = tail_w - head_w
        length = float(axis.length)
        if not math.isfinite(length) or length < 1e-6:
            raise ValueError(f"bone '{bone_name}' has near-zero length: {length!r}")
        axis_n = axis.normalized()
        mid = (head_w + tail_w) * 0.5

        base_q = axis_n.to_track_quat('Z', 'Y')

        kind, segments = _parse_profile(mesh_spec.get('profile'))
        rx, ry = _resolve_profile_radius(mesh_spec, bone_length=length)

        cap_start = mesh_spec.get('cap_start', True)
        cap_end = mesh_spec.get('cap_end', True)
        if not isinstance(cap_start, bool):
            raise TypeError(
                f"bone_meshes['{bone_name}'].cap_start must be a bool, got {type(cap_start).__name__}: {cap_start!r}"
            )
        if not isinstance(cap_end, bool):
            raise TypeError(
                f"bone_meshes['{bone_name}'].cap_end must be a bool, got {type(cap_end).__name__}: {cap_end!r}"
            )

        bone_material_index = _parse_material_index(
            mesh_spec.get('material_index'),
            default=0,
            name=f"bone_meshes['{bone_name}'].material_index",
        )

        extrusion_steps = mesh_spec.get('extrusion_steps', [])
        if not extrusion_steps:
            raise ValueError(
                f"bone_meshes['{bone_name}'].extrusion_steps is required"
            )

        connect_start = mesh_spec.get("connect_start")
        connect_end = mesh_spec.get("connect_end")

        obj = _build_mesh_with_steps(
            bpy_module=bpy,
            bmesh_module=bmesh,
            bone_name=bone_name,
            bone_length=length,
            head_w=head_w,
            base_q=base_q,
            profile=(kind, segments),
            profile_radius=(rx, ry),
            steps=extrusion_steps,
            cap_start=cap_start,
            cap_end=cap_end,
            ensure_object_mode=_ensure_object_mode,
            select_only=_select_only,
            apply_scale_only=_apply_scale_only,
            apply_rotation_only=_apply_rotation_only,
            quat_rotate_vec=_quat_rotate_vec,
            connect_start=connect_start,
            connect_end=connect_end,
        )
        _ensure_material_slot_placeholders(obj)

        _set_all_poly_material_index(obj, material_index=bone_material_index)

        segment_origin_w = mid

        def _bone_local_head_to_segment_world(v_rel_head) -> Vector:
            v = _parse_vec3(v_rel_head, name='attachment vec3')
            if v is None:
                raise TypeError("attachment vec3 is required")
            x, y, z = v
            # Bone-relative -> absolute (meters), head-origin.
            local = Vector((x * length, y * length, z * length))
            # Convert to segment-mid origin.
            local = local - Vector((0.0, 0.0, length * 0.5))
            return segment_origin_w + _quat_rotate_vec(base_q, local)

        # Attachments: additional geometry joined into this segment.
        attachments_spec = mesh_spec.get('attachments', []) or []
        if attachments_spec is not None:
            if not isinstance(attachments_spec, list):
                raise TypeError(f"bone_meshes['{bone_name}'].attachments must be a list")

            for ai, att in enumerate(attachments_spec):
                if not isinstance(att, dict):
                    raise TypeError(f"bone_meshes['{bone_name}'].attachments[{ai}] must be a dict")

                # Primitive attachment (untagged form).
                if 'primitive' in att:
                    att_material_index = _parse_material_index(
                        att.get('material_index'),
                        default=bone_material_index,
                        name=f"bone_meshes['{bone_name}'].attachments[{ai}].material_index",
                    )

                    primitive = att.get('primitive')
                    if not isinstance(primitive, str) or not primitive:
                        raise TypeError(
                            f"bone_meshes['{bone_name}'].attachments[{ai}].primitive must be a non-empty str"
                        )

                    dimensions = _parse_vec3(att.get('dimensions'), name=f"bone_meshes['{bone_name}'].attachments[{ai}].dimensions")
                    if dimensions is None:
                        raise TypeError(
                            f"bone_meshes['{bone_name}'].attachments[{ai}].dimensions is required"
                        )
                    dx, dy, dz = dimensions
                    if dx <= 0.0 or dy <= 0.0 or dz <= 0.0:
                        raise ValueError(
                            f"bone_meshes['{bone_name}'].attachments[{ai}].dimensions must be > 0, got ({dx!r}, {dy!r}, {dz!r})"
                        )

                    offset = att.get('offset', [0.0, 0.0, 0.0])
                    location_w = _bone_local_head_to_segment_world(offset)

                    attach_obj = _create_primitive_mesh(primitive=primitive, location_w=location_w)
                    attach_obj.name = f"Attachment_{bone_name}_{ai}"

                    _ensure_material_slot_placeholders(attach_obj)

                    # Bone-relative dimensions -> absolute scale.
                    attach_obj.scale = (dx * length, dy * length, dz * length)
                    _apply_scale_only(attach_obj)

                    rot = _parse_vec3(att.get('rotation'), name=f"bone_meshes['{bone_name}'].attachments[{ai}].rotation")
                    attach_obj.rotation_mode = 'QUATERNION'
                    if rot is not None:
                        rx, ry, rz = rot
                        rot_q = Euler(
                            (math.radians(rx), math.radians(ry), math.radians(rz)),
                            'XYZ',
                        ).to_quaternion()
                        attach_obj.rotation_quaternion = _quat_mul(base_q, rot_q)
                    else:
                        attach_obj.rotation_quaternion = base_q
                    _apply_rotation_only(attach_obj)

                    _set_all_poly_material_index(attach_obj, material_index=att_material_index)

                    obj = _join_into(obj, attach_obj)
                    continue

                # Extrude attachment (tagged as {"extrude": {...}}).
                if 'extrude' in att:
                    att_material_index = _parse_material_index(
                        att.get('material_index'),
                        default=bone_material_index,
                        name=f"bone_meshes['{bone_name}'].attachments[{ai}].material_index",
                    )

                    extr = att.get('extrude')
                    if not isinstance(extr, dict):
                        raise TypeError(
                            f"bone_meshes['{bone_name}'].attachments[{ai}].extrude must be a dict"
                        )

                    start = extr.get('start')
                    end = extr.get('end')
                    if start is None or end is None:
                        raise TypeError(
                            f"bone_meshes['{bone_name}'].attachments[{ai}].extrude.start and .end are required"
                        )

                    start_w = _bone_local_head_to_segment_world(start)
                    end_w = _bone_local_head_to_segment_world(end)
                    axis_w = end_w - start_w
                    dist = float(axis_w.length)
                    if not math.isfinite(dist) or dist < 1e-6:
                        raise ValueError(
                            f"bone_meshes['{bone_name}'].attachments[{ai}].extrude has near-zero length: {dist!r}"
                        )

                    axis_n = axis_w.normalized()
                    loc_w = (start_w + end_w) * 0.5

                    kind_e, segments_e = _parse_profile(extr.get('profile'))
                    radius_val_e = _resolve_bone_relative_length(
                        extr.get('profile_radius', 0.15),
                        bone_length=length,
                    )
                    if isinstance(radius_val_e, tuple):
                        rx_e, ry_e = radius_val_e
                    else:
                        rx_e, ry_e = (radius_val_e, radius_val_e)
                    rx_e = float(rx_e)
                    ry_e = float(ry_e)
                    if rx_e <= 0.0 or ry_e <= 0.0:
                        raise ValueError(
                            f"bone_meshes['{bone_name}'].attachments[{ai}].extrude.profile_radius must be > 0"
                        )

                    if kind_e == 'circle':
                        _primitive_add_with_uvs(
                            bpy.ops.mesh.primitive_cylinder_add,
                            vertices=int(segments_e),
                            radius=1.0,
                            depth=1.0,
                            location=loc_w,
                        )
                        attach_obj = _require_active_mesh(context='primitive_cylinder_add(extrude_attachment)')
                        attach_obj.scale = (rx_e, ry_e, dist)
                    elif kind_e in ('square', 'rectangle'):
                        _primitive_add_with_uvs(
                            bpy.ops.mesh.primitive_cube_add,
                            size=1.0,
                            location=loc_w,
                        )
                        attach_obj = _require_active_mesh(context='primitive_cube_add(extrude_attachment)')
                        attach_obj.scale = (rx_e * 2.0, ry_e * 2.0, dist)
                    else:
                        raise ValueError(f"unsupported extrude profile kind: {kind_e!r}")

                    attach_obj.name = f"AttachmentExtrude_{bone_name}_{ai}"
                    _apply_scale_only(attach_obj)

                    _ensure_material_slot_placeholders(attach_obj)

                    taper_e = _parse_finite_number(
                        extr.get('taper'),
                        name=f"bone_meshes['{bone_name}'].attachments[{ai}].extrude.taper",
                        default=1.0,
                    )
                    if taper_e <= 0.0:
                        raise ValueError(
                            f"bone_meshes['{bone_name}'].attachments[{ai}].extrude.taper must be > 0, got {taper_e!r}"
                        )
                    _deform_taper_bulge_twist(attach_obj, taper=taper_e, bulge_points=[], twist_deg=0.0)

                    attach_obj.rotation_mode = 'QUATERNION'
                    attach_obj.rotation_quaternion = axis_n.to_track_quat('Z', 'Y')
                    _apply_rotation_only(attach_obj)

                    _set_all_poly_material_index(attach_obj, material_index=att_material_index)

                    obj = _join_into(obj, attach_obj)
                    continue

                # Asset attachment (untagged form; key is 'asset').
                if 'asset' in att:
                    att_material_index = _parse_material_index(
                        att.get('material_index'),
                        default=bone_material_index,
                        name=f"bone_meshes['{bone_name}'].attachments[{ai}].material_index",
                    )

                    from pathlib import Path

                    asset = att.get('asset')
                    if not isinstance(asset, str) or not asset:
                        raise TypeError(
                            f"bone_meshes['{bone_name}'].attachments[{ai}].asset must be a non-empty str"
                        )

                    p = Path(asset)
                    candidates = []
                    if p.suffix:
                        candidates.append(p)
                    else:
                        candidates.append(p.with_suffix('.glb'))
                        candidates.append(p.with_suffix('.gltf'))
                    candidates = [c if c.is_absolute() else (Path(out_root) / c) for c in candidates]
                    src = next((c for c in candidates if c.exists()), None)
                    if src is None:
                        print(
                            f"WARN: asset attachment source not found for '{bone_name}' attachments[{ai}]: {asset!r}; skipping"
                        )
                        continue

                    _ensure_object_mode()
                    bpy.ops.object.select_all(action='DESELECT')
                    bpy.ops.import_scene.gltf(filepath=str(src))

                    imported_meshes = [o for o in bpy.context.selected_objects if o.type == 'MESH']
                    if not imported_meshes:
                        print(
                            f"WARN: asset attachment import produced no meshes for '{bone_name}' attachments[{ai}]: {src}"
                        )
                        continue

                    imported_meshes.sort(key=lambda o: o.name)
                    if len(imported_meshes) > 1:
                        _select_only(imported_meshes, active=imported_meshes[0])
                        bpy.ops.object.join()
                        attach_obj = bpy.context.active_object
                    else:
                        attach_obj = imported_meshes[0]
                    if attach_obj is None or getattr(attach_obj, 'type', None) != 'MESH':
                        print(
                            f"WARN: asset attachment join failed for '{bone_name}' attachments[{ai}]: {src}"
                        )
                        continue

                    attach_obj.name = f"AttachmentAsset_{bone_name}_{ai}"

                    _ensure_material_slot_placeholders(attach_obj)

                    offset = att.get('offset', [0.0, 0.0, 0.0])
                    attach_obj.location = _bone_local_head_to_segment_world(offset)

                    s = att.get('scale', 1.0)
                    if isinstance(s, bool) or not isinstance(s, (int, float)):
                        raise TypeError(
                            f"bone_meshes['{bone_name}'].attachments[{ai}].scale must be a number"
                        )
                    s_f = float(s)
                    if not math.isfinite(s_f) or s_f <= 0.0:
                        raise ValueError(
                            f"bone_meshes['{bone_name}'].attachments[{ai}].scale must be a finite, positive number"
                        )
                    attach_obj.scale = (s_f, s_f, s_f)
                    _apply_scale_only(attach_obj)

                    rot = _parse_vec3(att.get('rotation'), name=f"bone_meshes['{bone_name}'].attachments[{ai}].rotation")
                    attach_obj.rotation_mode = 'QUATERNION'
                    if rot is not None:
                        rx, ry, rz = rot
                        rot_q = Euler(
                            (math.radians(rx), math.radians(ry), math.radians(rz)),
                            'XYZ',
                        ).to_quaternion()
                        attach_obj.rotation_quaternion = _quat_mul(base_q, rot_q)
                    else:
                        attach_obj.rotation_quaternion = base_q
                    _apply_rotation_only(attach_obj)

                    _set_all_poly_material_index(attach_obj, material_index=att_material_index)

                    obj = _join_into(obj, attach_obj)
                    continue

        # Apply per-segment modifiers (in-order).
        modifiers_spec = mesh_spec.get('modifiers', []) or []
        if not isinstance(modifiers_spec, list):
            raise TypeError(f"bone_meshes['{bone_name}'].modifiers must be a list")

        for mi, entry in enumerate(modifiers_spec):
            if not isinstance(entry, dict):
                raise TypeError(f"bone_meshes['{bone_name}'].modifiers[{mi}] must be a dict")

            recognized = [k for k in ('bool', 'bevel', 'subdivide') if k in entry]
            if not recognized:
                # Unknown / unsupported modifier entry; ignore for forward-compat.
                continue
            if len(recognized) != 1:
                raise ValueError(
                    f"bone_meshes['{bone_name}'].modifiers[{mi}] must contain exactly one recognized modifier key, got {sorted(recognized)!r}"
                )
            kind = recognized[0]

            if kind == 'bool':
                bool_spec = entry.get('bool')
                if not isinstance(bool_spec, dict):
                    raise TypeError(
                        f"bone_meshes['{bone_name}'].modifiers[{mi}].bool must be a dict"
                    )

                operation = bool_spec.get('operation', 'difference')
                if not isinstance(operation, str) or not operation:
                    raise TypeError(
                        f"bone_meshes['{bone_name}'].modifiers[{mi}].bool.operation must be a non-empty str"
                    )
                op_lc = operation.strip().lower()
                if op_lc in ('subtract', 'difference'):
                    blender_op = 'DIFFERENCE'
                elif op_lc == 'union':
                    blender_op = 'UNION'
                elif op_lc in ('intersect', 'intersection'):
                    blender_op = 'INTERSECT'
                else:
                    raise ValueError(
                        f"bone_meshes['{bone_name}'].modifiers[{mi}].bool.operation unsupported: {operation!r}"
                    )

                target = bool_spec.get('target')
                if not isinstance(target, str) or not target:
                    raise TypeError(
                        f"bone_meshes['{bone_name}'].modifiers[{mi}].bool.target must be a non-empty str"
                    )
                target_obj = bool_shape_objs.get(target)
                if target_obj is None:
                    raise ValueError(
                        f"bone_meshes['{bone_name}'].modifiers[{mi}].bool.target '{target}' not found in bool_shapes"
                    )

                mod = obj.modifiers.new(name=f"Bool_{mi}_{target}", type='BOOLEAN')
                mod.operation = blender_op
                mod.object = target_obj
                try:
                    mod.solver = 'EXACT'
                except Exception:
                    pass
                _apply_modifier(obj, modifier_name=mod.name)

            elif kind == 'bevel':
                bevel_spec = entry.get('bevel')
                if not isinstance(bevel_spec, dict):
                    raise TypeError(
                        f"bone_meshes['{bone_name}'].modifiers[{mi}].bevel must be a dict"
                    )

                width_raw = bevel_spec.get('width')
                if width_raw is None:
                    raise TypeError(
                        f"bone_meshes['{bone_name}'].modifiers[{mi}].bevel.width is required"
                    )
                width_abs = _resolve_bone_relative_length(width_raw, bone_length=length)
                if isinstance(width_abs, tuple):
                    raise TypeError(
                        f"bone_meshes['{bone_name}'].modifiers[{mi}].bevel.width must resolve to a scalar, got {width_abs!r}"
                    )

                segments = bevel_spec.get('segments', 1)
                if isinstance(segments, bool) or not isinstance(segments, int):
                    raise TypeError(
                        f"bone_meshes['{bone_name}'].modifiers[{mi}].bevel.segments must be an int"
                    )
                if segments < 1:
                    raise ValueError(
                        f"bone_meshes['{bone_name}'].modifiers[{mi}].bevel.segments must be >= 1, got {segments!r}"
                    )

                mod = obj.modifiers.new(name=f"Bevel_{mi}", type='BEVEL')
                mod.width = float(width_abs)
                mod.segments = int(segments)
                try:
                    mod.limit_method = 'NONE'
                except Exception:
                    pass
                _apply_modifier(obj, modifier_name=mod.name)

            elif kind == 'subdivide':
                subdiv_spec = entry.get('subdivide')
                if not isinstance(subdiv_spec, dict):
                    raise TypeError(
                        f"bone_meshes['{bone_name}'].modifiers[{mi}].subdivide must be a dict"
                    )
                cuts = subdiv_spec.get('cuts')
                if cuts is None:
                    raise TypeError(
                        f"bone_meshes['{bone_name}'].modifiers[{mi}].subdivide.cuts is required"
                    )
                _subdivide_all_edges(obj, cuts=cuts)

        vg = obj.vertex_groups.get(bone_name)
        if vg is None:
            vg = obj.vertex_groups.new(name=bone_name)
        indices = [v.index for v in obj.data.vertices]
        if indices:
            vg.add(indices, 1.0, 'REPLACE')

        segment_objs.append(obj)

    if not segment_objs:
        raise ValueError("No segments generated")

    # Boolean target shapes should not be exported.
    for shape_key in sorted(bool_shape_objs.keys()):
        obj = bool_shape_objs.get(shape_key)
        if obj is None:
            continue
        try:
            bpy.data.objects.remove(obj, do_unlink=True)
        except Exception:
            pass

    _ensure_object_mode()
    # Deterministic join order: preserve the segment creation order.
    combined = segment_objs[0]
    for other in segment_objs[1:]:
        _ensure_object_mode()
        _select_only([combined, other], active=combined)
        bpy.ops.object.join()
        combined = bpy.context.active_object
        if combined is None:
            raise RuntimeError("object.join: expected an active object, got None")

    if getattr(combined, 'type', None) != 'MESH':
        raise RuntimeError(
            f"object.join: expected a MESH active object, got {getattr(combined, 'type', None)!r}"
        )
    combined.name = 'Character'

    # Determine bridge pairs and perform bridging
    bone_hierarchy = _build_bone_hierarchy(armature)
    bridge_pairs = get_bridge_pairs(bone_hierarchy, bone_meshes)

    if bridge_pairs:
        _perform_bridge_operations(
            bpy_module=bpy,
            bmesh_module=bmesh,
            mesh_obj=combined,
            bridge_pairs=bridge_pairs,
            armature=armature,
        )

    _ensure_material_slot_placeholders(combined)

    _parent_mesh_to_armature(combined, armature)
    _ensure_uvs(combined)
    _recalculate_normals(combined)

    return combined
