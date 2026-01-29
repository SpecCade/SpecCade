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

    def _apply_rotation_scale(obj) -> None:
        _ensure_object_mode()
        _select_only([obj], active=obj)
        bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)

    def _remove_segment_caps(obj, *, cap_start: bool, cap_end: bool) -> None:
        if cap_start and cap_end:
            return
        if getattr(obj, 'type', None) != 'MESH':
            return

        mesh = obj.data
        bm = bmesh.new()
        try:
            bm.from_mesh(mesh)
            bm.verts.ensure_lookup_table()
            bm.faces.ensure_lookup_table()

            if not bm.verts or not bm.faces:
                return

            z_min = min(v.co.z for v in bm.verts)
            z_max = max(v.co.z for v in bm.verts)
            dz = float(z_max - z_min)
            eps = max(1e-6, dz * 1e-4)

            faces_to_delete = []
            for f in bm.faces:
                zs = [v.co.z for v in f.verts]
                if not zs:
                    continue
                if (not cap_start) and all(abs(z - z_min) <= eps for z in zs):
                    faces_to_delete.append(f)
                    continue
                if (not cap_end) and all(abs(z - z_max) <= eps for z in zs):
                    faces_to_delete.append(f)
                    continue

            if faces_to_delete:
                bmesh.ops.delete(bm, geom=faces_to_delete, context='FACES')
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

    bone_meshes = _resolve_mirrors(params.get('bone_meshes', {}) or {})
    if not bone_meshes:
        raise ValueError("params.bone_meshes is required")

    _ensure_object_mode()

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

        kind, segments = _parse_profile(mesh_spec.get('profile'))

        radius_val = _resolve_bone_relative_length(
            mesh_spec.get('profile_radius', 0.15),
            bone_length=length,
        )
        if isinstance(radius_val, tuple):
            rx, ry = radius_val
        else:
            rx, ry = (radius_val, radius_val)
        rx = float(rx)
        ry = float(ry)

        if not math.isfinite(rx) or rx <= 0.0:
            raise ValueError(f"profile_radius.x must be > 0 for '{bone_name}', got {rx!r}")
        if not math.isfinite(ry) or ry <= 0.0:
            raise ValueError(f"profile_radius.y must be > 0 for '{bone_name}', got {ry!r}")

        if kind == 'circle':
            _primitive_add_with_uvs(
                bpy.ops.mesh.primitive_cylinder_add,
                vertices=int(segments),
                radius=1.0,
                depth=1.0,
                location=mid,
            )
            obj = _require_active_mesh(context='primitive_cylinder_add')
            obj.scale = (rx, ry, length)
        elif kind in ('square', 'rectangle'):
            _primitive_add_with_uvs(
                bpy.ops.mesh.primitive_cube_add,
                size=1.0,
                location=mid,
            )
            obj = _require_active_mesh(context='primitive_cube_add')
            obj.scale = (rx * 2.0, ry * 2.0, length)
        else:
            raise ValueError(f"unsupported profile kind: {kind!r}")

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

        _remove_segment_caps(obj, cap_start=cap_start, cap_end=cap_end)

        obj.name = f"Segment_{bone_name}"

        obj.rotation_mode = 'QUATERNION'
        base_q = axis_n.to_track_quat('Z', 'Y')
        obj.rotation_quaternion = base_q

        translate = _parse_vec3(mesh_spec.get('translate'), name=f"bone_meshes['{bone_name}'].translate")
        if translate is not None:
            tx, ty, tz = translate
            # Bone-relative translation is in bone-local axes and scales by bone length.
            offset_local = Vector((tx * length, ty * length, tz * length))
            offset_world = _quat_rotate_vec(base_q, offset_local)
            obj.location = obj.location + offset_world

        rotate = _parse_vec3(mesh_spec.get('rotate'), name=f"bone_meshes['{bone_name}'].rotate")
        if rotate is not None:
            rx, ry, rz = rotate
            rot_q = Euler(
                (math.radians(rx), math.radians(ry), math.radians(rz)),
                'XYZ',
            ).to_quaternion()
            obj.rotation_quaternion = _quat_mul(base_q, rot_q)

        _apply_rotation_scale(obj)

        vg = obj.vertex_groups.get(bone_name)
        if vg is None:
            vg = obj.vertex_groups.new(name=bone_name)
        indices = [v.index for v in obj.data.vertices]
        if indices:
            vg.add(indices, 1.0, 'REPLACE')

        segment_objs.append(obj)

    if not segment_objs:
        raise ValueError("No segments generated")

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

    _parent_mesh_to_armature(combined, armature)
    _ensure_uvs(combined)
    _recalculate_normals(combined)

    return combined
