"""Pure-Python helpers for the skeletal mesh rework.

This module must not import Blender (`bpy`). It exists so we can unit-test the
non-Blender logic (kind dispatch, vertex-group mapping plans) with `unittest`.
"""

from __future__ import annotations

from typing import Dict, Iterable, List, Sequence, Tuple


SUPPORTED_SKELETAL_MESH_KINDS = {
    "skeletal_mesh.armature_driven_v1",
    "skeletal_mesh.skinned_mesh_v1",
}


def classify_skeletal_mesh_kind(kind: str) -> str:
    """Validate and return the skeletal-mesh recipe kind.

    Returns the original kind string for supported kinds.
    """
    if not isinstance(kind, str) or not kind:
        raise ValueError("recipe.kind must be a non-empty string")

    if kind in SUPPORTED_SKELETAL_MESH_KINDS:
        return kind

    if kind.startswith("skeletal_mesh."):
        supported = ", ".join(sorted(SUPPORTED_SKELETAL_MESH_KINDS))
        raise ValueError(
            f"Unsupported skeletal_mesh recipe kind: {kind}. Supported kinds: {supported}"
        )

    raise ValueError(f"Not a skeletal_mesh recipe kind: {kind}")


def compute_safe_rename_plan(
    vertex_group_map: Dict[str, str],
    *,
    existing_names: Sequence[str],
    temp_prefix: str = "__tmp_vg__",
) -> List[Tuple[str, str]]:
    """Compute a rename plan that avoids clobbering existing names.

    The returned plan is a list of (src, dst) renames. When applying the plan in
    order, each step guarantees:

    - `src` exists in the current name set
    - `dst` does not exist in the current name set

    If a destination already exists and is NOT also a source being renamed away,
    this function raises. (The Blender-side implementation can handle that case
    by merging weights instead of renaming.)
    """
    current_names = set(existing_names)

    # Filter to sources that exist, drop no-ops.
    mapping: Dict[str, str] = {
        src: dst
        for src, dst in (vertex_group_map or {}).items()
        if src in current_names and src != dst and isinstance(dst, str) and dst
    }

    if not mapping:
        return []

    # This helper is only for renames. Many-to-one mappings must be handled via
    # merge semantics in Blender, not by renaming.
    by_dst: Dict[str, List[str]] = {}
    for src, dst in mapping.items():
        by_dst.setdefault(dst, []).append(src)
    dup_dsts = sorted(dst for dst, srcs in by_dst.items() if len(srcs) > 1)
    if dup_dsts:
        raise ValueError(
            "Multiple vertex groups map to the same destination (merge required): "
            + ", ".join(dup_dsts)
        )

    sources = set(mapping.keys())

    # If a destination exists but is not being renamed away, we cannot rename
    # without a collision.
    blocking = sorted(
        dst for dst in mapping.values() if dst in current_names and dst not in sources
    )
    if blocking:
        raise ValueError(
            "Destination vertex groups already exist and are not being remapped: "
            + ", ".join(blocking)
        )

    def _fresh_temp(base: str) -> str:
        candidate = f"{temp_prefix}{base}"
        if candidate not in current_names:
            return candidate
        i = 1
        while True:
            candidate_i = f"{candidate}_{i}"
            if candidate_i not in current_names:
                return candidate_i
            i += 1

    plan: List[Tuple[str, str]] = []

    # Break cycles / chains by temporarily moving any source whose destination is
    # also a source name.
    temp_mapping: Dict[str, str] = dict(mapping)
    for src, dst in list(mapping.items()):
        if dst in sources:
            tmp = _fresh_temp(src)
            plan.append((src, tmp))
            current_names.remove(src)
            current_names.add(tmp)
            del temp_mapping[src]
            temp_mapping[tmp] = dst

    mapping = temp_mapping

    # Topologically apply remaining renames: always rename a source whose
    # destination is not another remaining source.
    while mapping:
        remaining_sources = set(mapping.keys())
        progressed = False
        for src, dst in list(mapping.items()):
            if dst in remaining_sources:
                continue
            if src not in current_names:
                # Should not happen if plan application matches.
                raise ValueError(f"Internal error: missing source during planning: {src}")
            if dst in current_names:
                raise ValueError(
                    f"Internal error: destination already exists during planning: {dst}"
                )
            plan.append((src, dst))
            current_names.remove(src)
            current_names.add(dst)
            del mapping[src]
            progressed = True

        if not progressed:
            # This would imply a cycle we failed to break.
            raise ValueError("Failed to compute rename plan (cycle detected)")

    return plan
