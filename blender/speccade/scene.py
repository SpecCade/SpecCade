"""
SpecCade Scene Setup Module

This module handles Blender scene initialization and primitive mesh creation.
It provides functions to clear the scene and create basic mesh primitives.
"""

from typing import List

# Blender modules - only available when running inside Blender
try:
    import bpy
    BLENDER_AVAILABLE = True
except ImportError:
    BLENDER_AVAILABLE = False


def clear_scene() -> None:
    """Clear the Blender scene."""
    bpy.ops.wm.read_factory_settings(use_empty=True)


def setup_scene() -> None:
    """Set up the scene for export."""
    # Ensure we have a scene
    if not bpy.context.scene:
        bpy.ops.scene.new(type='NEW')


# Dictionary mapping primitive type names to their creation functions
PRIMITIVE_CREATORS = {
    "cube": lambda dims: bpy.ops.mesh.primitive_cube_add(size=1),
    "sphere": lambda dims: bpy.ops.mesh.primitive_uv_sphere_add(radius=0.5, segments=32, ring_count=16),
    "cylinder": lambda dims: bpy.ops.mesh.primitive_cylinder_add(radius=0.5, depth=1, vertices=32),
    "cone": lambda dims: bpy.ops.mesh.primitive_cone_add(radius1=0.5, radius2=0, depth=1, vertices=32),
    "torus": lambda dims: bpy.ops.mesh.primitive_torus_add(major_radius=0.5, minor_radius=0.125),
    "plane": lambda dims: bpy.ops.mesh.primitive_plane_add(size=1),
    "ico_sphere": lambda dims: bpy.ops.mesh.primitive_ico_sphere_add(radius=0.5, subdivisions=2),
}


def create_primitive(primitive_type: str, dimensions: List[float]) -> 'bpy.types.Object':
    """Create a mesh primitive."""
    primitive_type = primitive_type.lower().replace("_", "")

    if primitive_type == "icosphere":
        primitive_type = "ico_sphere"

    if primitive_type not in PRIMITIVE_CREATORS:
        raise ValueError(f"Unknown primitive type: {primitive_type}")

    PRIMITIVE_CREATORS[primitive_type](dimensions)
    obj = bpy.context.active_object

    # Scale to dimensions
    obj.scale = (dimensions[0], dimensions[1], dimensions[2])
    bpy.ops.object.transform_apply(scale=True)

    return obj
