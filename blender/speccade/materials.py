"""
Materials module for SpecCade Blender asset generation.

This module handles material creation and application for mesh objects,
including PBR material properties like base color, metallic, roughness,
and emissive settings.
"""

from typing import Dict, List

try:
    import bpy
except ImportError:
    bpy = None  # type: ignore


def create_material(name: str, spec: Dict) -> 'bpy.types.Material':
    """Create a material from spec."""
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True

    # Get principled BSDF node
    nodes = mat.node_tree.nodes
    principled = nodes.get("Principled BSDF")
    if not principled:
        return mat

    # Base color
    if "base_color" in spec:
        color = spec["base_color"]
        if len(color) == 3:
            color = color + [1.0]
        principled.inputs["Base Color"].default_value = color

    # Metallic
    if "metallic" in spec:
        principled.inputs["Metallic"].default_value = spec["metallic"]

    # Roughness
    if "roughness" in spec:
        principled.inputs["Roughness"].default_value = spec["roughness"]

    # Emissive
    if "emissive" in spec:
        emissive = spec["emissive"]
        strength = spec.get("emissive_strength", 1.0)
        # Blender 4.0+ uses "Emission Color"
        if "Emission Color" in principled.inputs:
            principled.inputs["Emission Color"].default_value = emissive + [1.0]
            principled.inputs["Emission Strength"].default_value = strength
        elif "Emission" in principled.inputs:
            principled.inputs["Emission"].default_value = emissive + [1.0]

    return mat


def apply_materials(obj: 'bpy.types.Object', material_slots: List[Dict]) -> None:
    """Apply materials to an object.

    Slot-replacing and idempotent with respect to obj.data.materials:
    - Ensure the object has exactly N material slots.
    - Set obj.data.materials[i] for each slot.
    - Do not append repeatedly across calls.
    """

    if bpy is None:  # pragma: no cover
        raise RuntimeError("Blender Python (bpy) is required")

    if obj is None or getattr(obj, "type", None) != "MESH":
        return

    if material_slots is None:
        material_slots = []
    if not isinstance(material_slots, list):
        raise TypeError("material_slots must be a list")

    mats = obj.data.materials
    desired = len(material_slots)

    # Resize to exactly desired slots.
    try:
        while len(mats) > desired:
            mats.pop()
    except Exception:
        # Older Blender APIs can throw if pop() is unavailable; fall back to clear().
        try:
            mats.clear()
        except Exception:
            while len(mats) > 0:
                mats.pop()

    while len(mats) < desired:
        mats.append(None)

    for i, slot_spec in enumerate(material_slots):
        if not isinstance(slot_spec, dict):
            raise TypeError(f"material_slots[{i}] must be a dict")
        name = slot_spec.get("name", f"Material_{i}")
        if not isinstance(name, str) or not name:
            name = f"Material_{i}"
        mat = create_material(name, slot_spec)
        mats[i] = mat
