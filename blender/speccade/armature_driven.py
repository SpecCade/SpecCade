"""Armature-driven skeletal mesh generation.

This module is the dedicated home for the Blender implementation of the
`skeletal_mesh.armature_driven_v1` recipe.
"""


def build_armature_driven_character_mesh(*, armature, params: dict, out_root) -> "bpy.types.Object":
    """Build a character mesh driven by an existing armature.

    Args:
        armature: Blender armature object.
        params: Recipe params for `skeletal_mesh.armature_driven_v1`.
        out_root: Spec output root (Path-like).

    Returns:
        The generated mesh object.
    """

    raise NotImplementedError
