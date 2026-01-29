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

    def _deform_taper_bulge_twist(
        obj,
        *,
        taper: float = 1.0,
        bulge_points: list[tuple[float, float]] | None = None,
        twist_deg: float = 0.0,
    ) -> None:
        if getattr(obj, 'type', None) != 'MESH':
            return

        taper_f = float(taper)
        if not math.isfinite(taper_f) or taper_f <= 0.0:
            raise ValueError(f"taper must be a finite, positive number, got {taper!r}")

        twist_f = float(twist_deg)
        if not math.isfinite(twist_f):
            raise ValueError(f"twist must be a finite number, got {twist_deg!r}")

        pts: list[tuple[float, float]] = []
        if bulge_points:
            for at, scale in bulge_points:
                at_f = float(at)
                scale_f = float(scale)
                if not math.isfinite(at_f):
                    raise ValueError(f"bulge.at must be finite, got {at!r}")
                if not math.isfinite(scale_f) or scale_f <= 0.0:
                    raise ValueError(f"bulge.scale must be a finite, positive number, got {scale!r}")
                # Spec says clamp and linearly interpolate.
                if at_f < 0.0:
                    at_f = 0.0
                if at_f > 1.0:
                    at_f = 1.0
                pts.append((at_f, scale_f))

        pts.sort(key=lambda p: p[0])

        def bulge_scale(t: float) -> float:
            if not pts:
                return 1.0
            if t <= pts[0][0]:
                return pts[0][1]
            for i in range(1, len(pts)):
                at1, s1 = pts[i]
                at0, s0 = pts[i - 1]
                if t <= at1:
                    denom = at1 - at0
                    if abs(denom) < 1e-12:
                        return s1
                    u = (t - at0) / denom
                    return s0 + (s1 - s0) * u
            return pts[-1][1]

        mesh = obj.data
        bm = bmesh.new()
        try:
            bm.from_mesh(mesh)
            bm.verts.ensure_lookup_table()
            if not bm.verts:
                return

            z_min = min(v.co.z for v in bm.verts)
            z_max = max(v.co.z for v in bm.verts)
            dz = float(z_max - z_min)
            if not math.isfinite(dz) or abs(dz) < 1e-12:
                return

            for v in bm.verts:
                t = (float(v.co.z) - float(z_min)) / dz
                if t < 0.0:
                    t = 0.0
                if t > 1.0:
                    t = 1.0

                s = (1.0 + (taper_f - 1.0) * t) * bulge_scale(t)

                x = float(v.co.x) * s
                y = float(v.co.y) * s

                if twist_f != 0.0:
                    a = math.radians(twist_f * t)
                    ca = math.cos(a)
                    sa = math.sin(a)
                    x, y = (x * ca - y * sa, x * sa + y * ca)

                v.co.x = x
                v.co.y = y

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

    def _parse_bulge_points(value, *, name: str) -> list[tuple[float, float]]:
        if value is None:
            return []
        if not isinstance(value, list):
            raise TypeError(f"{name} must be a list")
        out: list[tuple[float, float]] = []
        for i, item in enumerate(value):
            if not isinstance(item, dict):
                raise TypeError(f"{name}[{i}] must be a dict")
            if set(item.keys()) != {"at", "scale"}:
                keys = sorted(item.keys())
                raise ValueError(f"{name}[{i}] must have exactly keys ['at', 'scale'], got {keys!r}")
            at = _parse_finite_number(item.get("at"), name=f"{name}[{i}].at")
            scale = _parse_finite_number(item.get("scale"), name=f"{name}[{i}].scale")
            if scale <= 0.0:
                raise ValueError(f"{name}[{i}].scale must be > 0, got {scale!r}")
            out.append((at, scale))
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

        # Apply scale early so taper/bulge/twist operates on real dimensions.
        _apply_scale_only(obj)

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

        taper = _parse_finite_number(
            mesh_spec.get('taper'),
            name=f"bone_meshes['{bone_name}'].taper",
            default=1.0,
        )
        if taper <= 0.0:
            raise ValueError(
                f"bone_meshes['{bone_name}'].taper must be > 0, got {taper!r}"
            )

        bulge_points = _parse_bulge_points(
            mesh_spec.get('bulge'),
            name=f"bone_meshes['{bone_name}'].bulge",
        )

        twist = _parse_finite_number(
            mesh_spec.get('twist'),
            name=f"bone_meshes['{bone_name}'].twist",
            default=0.0,
        )

        _deform_taper_bulge_twist(
            obj,
            taper=taper,
            bulge_points=bulge_points,
            twist_deg=twist,
        )

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

        _apply_rotation_only(obj)

        segment_origin_w = obj.location.copy()

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

                    obj = _join_into(obj, attach_obj)
                    continue

                # Extrude attachment (tagged as {"extrude": {...}}).
                if 'extrude' in att:
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

                    obj = _join_into(obj, attach_obj)
                    continue

                # Asset attachment (untagged form; key is 'asset').
                if 'asset' in att:
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

    _parent_mesh_to_armature(combined, armature)
    _ensure_uvs(combined)
    _recalculate_normals(combined)

    return combined
