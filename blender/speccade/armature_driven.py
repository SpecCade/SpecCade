"""Armature-driven skeletal mesh generation.

This module is the dedicated home for the Blender implementation of the
`skeletal_mesh.armature_driven_v1` recipe.
"""

from __future__ import annotations

import copy
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
