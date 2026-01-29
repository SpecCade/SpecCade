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

    raise NotImplementedError
