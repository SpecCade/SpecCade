"""Armature-driven skeletal mesh generation.

This module is the dedicated home for the Blender implementation of the
`skeletal_mesh.armature_driven_v1` recipe.
"""

from __future__ import annotations

import copy
import math
from typing import TYPE_CHECKING

if TYPE_CHECKING:  # pragma: no cover
    import bpy  # type: ignore


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


def _resolve_bone_relative_length(value, *, bone_length: float):
    """Decode BoneRelativeLength values.

    Supported forms:
    - number: relative length, scaled by bone_length
    - [x, y]: relative2 length, scaled by bone_length
    - {"absolute": a}: absolute length
    """

    if isinstance(bone_length, bool) or not isinstance(bone_length, (int, float)):
        raise TypeError("bone_length must be a float")
    bone_length_f = float(bone_length)
    if not math.isfinite(bone_length_f) or bone_length_f <= 0.0:
        raise ValueError("bone_length must be a finite, positive number")

    if isinstance(value, bool):
        raise TypeError("value must not be a bool")

    if isinstance(value, (int, float)):
        out = float(value) * bone_length_f
        if not math.isfinite(out) or out <= 0.0:
            raise ValueError("resolved length must be a finite, positive number")
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
            raise ValueError("resolved length must be a finite, positive number")
        if not math.isfinite(out_y) or out_y <= 0.0:
            raise ValueError("resolved length must be a finite, positive number")
        return (out_x, out_y)

    if isinstance(value, dict):
        if set(value.keys()) != {"absolute"}:
            raise TypeError("absolute form must be exactly {'absolute': ...}")
        a = value.get("absolute")
        if isinstance(a, bool) or not isinstance(a, (int, float)):
            raise TypeError("absolute value must be a number")
        out = float(a)
        if not math.isfinite(out) or out <= 0.0:
            raise ValueError("resolved length must be a finite, positive number")
        return out

    raise TypeError("unsupported BoneRelativeLength form")


def build_armature_driven_character_mesh(*, armature, params: dict, out_root) -> object:
    """Build a character mesh driven by an existing armature.

    Args:
        armature: Blender armature object.
        params: Recipe params for `skeletal_mesh.armature_driven_v1`.
        out_root: Spec output root (Path-like).

    Returns:
        The generated mesh object.
    """

    raise NotImplementedError
