"""
Rig configuration module for SpecCade Blender asset generation.

This module handles animator rig configuration including custom bone shapes
(widgets), bone collections for organization, and color-coded bones for
easier animation workflows.
"""

from typing import Any, Dict, List, Optional, Tuple

try:
    import bpy
    import bmesh
except ImportError:
    bpy = None  # type: ignore
    bmesh = None  # type: ignore


# =============================================================================
# Constants
# =============================================================================

# Widget collection name (hidden from viewport)
WIDGET_COLLECTION_NAME = "Widgets"

# Standard widget shapes
WIDGET_SHAPES = {
    "wire_circle": "WGT_circle",
    "wire_cube": "WGT_cube",
    "wire_sphere": "WGT_sphere",
    "wire_diamond": "WGT_diamond",
    "custom_mesh": "WGT_custom",
}

# Standard bone colors (L=blue, R=red, center=yellow)
BONE_COLORS = {
    "left": (0.2, 0.4, 1.0),      # Blue
    "right": (1.0, 0.3, 0.3),     # Red
    "center": (1.0, 0.9, 0.2),    # Yellow
}


# =============================================================================
# Widget Shapes
# =============================================================================

def create_widget_shapes(armature: 'bpy.types.Object') -> Dict[str, 'bpy.types.Object']:
    """
    Create the 5 standard widget shapes for bone visualization.

    Widget shapes are created in a hidden collection and can be assigned
    to bones as custom shapes for better visual feedback during animation.

    Args:
        armature: The armature object (used to scale widgets appropriately).

    Returns:
        Dictionary mapping widget style names to their mesh objects.
    """
    widgets = {}

    # Create or get the widgets collection
    widget_collection = bpy.data.collections.get(WIDGET_COLLECTION_NAME)
    if not widget_collection:
        widget_collection = bpy.data.collections.new(WIDGET_COLLECTION_NAME)
        bpy.context.scene.collection.children.link(widget_collection)

    # Hide the widget collection from viewport
    widget_collection.hide_viewport = True
    widget_collection.hide_render = True

    # Calculate base scale from armature
    # Average bone length gives us a reasonable widget scale
    avg_bone_length = 0.1  # Default
    if armature.data.bones:
        total_length = sum(bone.length for bone in armature.data.bones)
        avg_bone_length = total_length / len(armature.data.bones)
    widget_scale = avg_bone_length * 0.5

    # Create Wire Circle widget
    if WIDGET_SHAPES["wire_circle"] not in bpy.data.objects:
        bpy.ops.mesh.primitive_circle_add(
            vertices=32,
            radius=widget_scale,
            fill_type='NOTHING',
            enter_editmode=False
        )
        circle = bpy.context.active_object
        circle.name = WIDGET_SHAPES["wire_circle"]
        circle.display_type = 'WIRE'
        # Move to widget collection
        for col in circle.users_collection:
            col.objects.unlink(circle)
        widget_collection.objects.link(circle)
        widgets["wire_circle"] = circle
    else:
        widgets["wire_circle"] = bpy.data.objects[WIDGET_SHAPES["wire_circle"]]

    # Create Wire Cube widget
    if WIDGET_SHAPES["wire_cube"] not in bpy.data.objects:
        bpy.ops.mesh.primitive_cube_add(size=widget_scale * 2)
        cube = bpy.context.active_object
        cube.name = WIDGET_SHAPES["wire_cube"]
        cube.display_type = 'WIRE'
        # Move to widget collection
        for col in cube.users_collection:
            col.objects.unlink(cube)
        widget_collection.objects.link(cube)
        widgets["wire_cube"] = cube
    else:
        widgets["wire_cube"] = bpy.data.objects[WIDGET_SHAPES["wire_cube"]]

    # Create Wire Sphere widget
    # Use lower polygon count for wireframe display (efficiency)
    if WIDGET_SHAPES["wire_sphere"] not in bpy.data.objects:
        bpy.ops.mesh.primitive_uv_sphere_add(
            radius=widget_scale,
            segments=12,
            ring_count=6
        )
        sphere = bpy.context.active_object
        sphere.name = WIDGET_SHAPES["wire_sphere"]
        sphere.display_type = 'WIRE'
        # Move to widget collection
        for col in sphere.users_collection:
            col.objects.unlink(sphere)
        widget_collection.objects.link(sphere)
        widgets["wire_sphere"] = sphere
    else:
        widgets["wire_sphere"] = bpy.data.objects[WIDGET_SHAPES["wire_sphere"]]

    # Create Wire Diamond widget (octahedron)
    if WIDGET_SHAPES["wire_diamond"] not in bpy.data.objects:
        # Create a diamond shape using bmesh
        mesh = bpy.data.meshes.new(WIDGET_SHAPES["wire_diamond"])
        diamond = bpy.data.objects.new(WIDGET_SHAPES["wire_diamond"], mesh)

        bm = bmesh.new()
        # Diamond vertices: top, bottom, and 4 around the middle
        top = bm.verts.new((0, 0, widget_scale))
        bottom = bm.verts.new((0, 0, -widget_scale))
        mid_verts = [
            bm.verts.new((widget_scale * 0.7, 0, 0)),
            bm.verts.new((0, widget_scale * 0.7, 0)),
            bm.verts.new((-widget_scale * 0.7, 0, 0)),
            bm.verts.new((0, -widget_scale * 0.7, 0)),
        ]
        bm.verts.ensure_lookup_table()

        # Create faces
        for i in range(4):
            next_i = (i + 1) % 4
            bm.faces.new([top, mid_verts[i], mid_verts[next_i]])
            bm.faces.new([bottom, mid_verts[next_i], mid_verts[i]])

        bm.to_mesh(mesh)
        bm.free()

        diamond.display_type = 'WIRE'
        widget_collection.objects.link(diamond)
        widgets["wire_diamond"] = diamond
    else:
        widgets["wire_diamond"] = bpy.data.objects[WIDGET_SHAPES["wire_diamond"]]

    # Create Custom Mesh placeholder (empty mesh that can be replaced)
    if WIDGET_SHAPES["custom_mesh"] not in bpy.data.objects:
        mesh = bpy.data.meshes.new(WIDGET_SHAPES["custom_mesh"])
        custom = bpy.data.objects.new(WIDGET_SHAPES["custom_mesh"], mesh)
        custom.display_type = 'WIRE'
        widget_collection.objects.link(custom)
        widgets["custom_mesh"] = custom
    else:
        widgets["custom_mesh"] = bpy.data.objects[WIDGET_SHAPES["custom_mesh"]]

    return widgets


# =============================================================================
# Bone Collections
# =============================================================================

def organize_bone_collections(
    armature: 'bpy.types.Object',
    collections_config: Optional[List[Dict]] = None
) -> Dict[str, List[str]]:
    """
    Organize bones into collections (IK Controls, FK Controls, Deform, Mechanism).

    Bone collections help animators by grouping related bones and allowing
    them to be shown/hidden as needed.

    Args:
        armature: The armature object.
        collections_config: Optional list of collection configurations. Each entry:
            - name: Collection name
            - bones: List of bone names
            - visible: Whether collection is visible (default True)
            - selectable: Whether bones are selectable (default True)

    Returns:
        Dictionary mapping collection names to lists of bone names.
    """
    # Default collections if none provided
    if collections_config is None:
        collections_config = [
            {
                "name": "IK Controls",
                "bones": [],  # Will be auto-populated
                "visible": True,
                "selectable": True,
            },
            {
                "name": "FK Controls",
                "bones": [],  # Will be auto-populated
                "visible": True,
                "selectable": True,
            },
            {
                "name": "Deform",
                "bones": [],  # Will be auto-populated
                "visible": False,
                "selectable": False,
            },
            {
                "name": "Mechanism",
                "bones": [],  # Will be auto-populated
                "visible": False,
                "selectable": False,
            },
        ]

    # Ensure we're in object mode
    if bpy.context.mode != 'OBJECT':
        bpy.ops.object.mode_set(mode='OBJECT')

    bpy.context.view_layer.objects.active = armature

    # Get all bone names
    all_bones = [bone.name for bone in armature.data.bones]

    # Auto-categorize bones if collections have empty bone lists
    ik_patterns = ["ik_", "pole_", "target_"]
    fk_patterns = ["fk_", "ctrl_"]
    deform_patterns = ["def_", "deform_"]
    mechanism_patterns = ["mch_", "mechanism_", "helper_"]

    # Track which bones are assigned
    assigned_bones = set()
    result = {}

    for config in collections_config:
        coll_name = config.get("name", "Collection")
        bones = config.get("bones", [])
        visible = config.get("visible", True)
        selectable = config.get("selectable", True)

        # If no bones specified, auto-categorize based on collection name
        if not bones:
            if "IK" in coll_name.upper():
                bones = [b for b in all_bones if any(b.lower().startswith(p) for p in ik_patterns) and b not in assigned_bones]
            elif "FK" in coll_name.upper():
                bones = [b for b in all_bones if any(b.lower().startswith(p) for p in fk_patterns) and b not in assigned_bones]
                # Also include main skeleton bones that aren't IK or mechanism
                for b in all_bones:
                    if b not in assigned_bones and b not in bones and not any(b.lower().startswith(p) for p in ik_patterns + mechanism_patterns + deform_patterns):
                        bones.append(b)
            elif "DEFORM" in coll_name.upper():
                bones = [b for b in all_bones if any(b.lower().startswith(p) for p in deform_patterns) and b not in assigned_bones]
            elif "MECHANISM" in coll_name.upper():
                bones = [b for b in all_bones if any(b.lower().startswith(p) for p in mechanism_patterns) and b not in assigned_bones]

        # Filter to only existing bones
        bones = [b for b in bones if b in all_bones]
        assigned_bones.update(bones)

        # Create bone collection in Blender 4.0+
        # Note: Blender 4.0 introduced bone collections, replacing bone groups
        use_bone_collections = hasattr(armature.data, 'collections')

        if use_bone_collections:
            try:
                # Blender 4.0+ bone collections API
                bone_coll = armature.data.collections.get(coll_name)
                if not bone_coll:
                    bone_coll = armature.data.collections.new(coll_name)

                # Set visibility
                bone_coll.is_visible = visible

                # Assign bones to collection
                for bone_name in bones:
                    bone = armature.data.bones.get(bone_name)
                    if bone:
                        bone_coll.assign(bone)

            except (AttributeError, RuntimeError) as e:
                print(f"Warning: Failed to create bone collection '{coll_name}': {e}")
                use_bone_collections = False

        if not use_bone_collections:
            # Fallback for older Blender versions using bone groups
            if hasattr(armature.pose, 'bone_groups'):
                bpy.ops.object.mode_set(mode='POSE')

                # Create bone group
                bone_group = armature.pose.bone_groups.get(coll_name)
                if not bone_group:
                    bone_group = armature.pose.bone_groups.new(name=coll_name)

                # Assign bones to group
                for bone_name in bones:
                    pose_bone = armature.pose.bones.get(bone_name)
                    if pose_bone:
                        pose_bone.bone_group = bone_group

                bpy.ops.object.mode_set(mode='OBJECT')

        result[coll_name] = bones

    return result


# =============================================================================
# Bone Colors
# =============================================================================

def apply_bone_colors(
    armature: 'bpy.types.Object',
    color_scheme: str = "standard",
    custom_colors: Optional[Dict] = None
) -> None:
    """
    Apply color coding to bones (L=blue, R=red, center=yellow).

    Color-coded bones help animators quickly identify left vs right side
    bones and center bones.

    Args:
        armature: The armature object.
        color_scheme: One of "standard", "custom", or "per_bone".
        custom_colors: For "custom" scheme: dict with "left", "right", "center" colors.
                      For "per_bone" scheme: dict mapping bone names to (r, g, b) tuples.
    """
    if bpy.context.mode != 'OBJECT':
        bpy.ops.object.mode_set(mode='OBJECT')

    bpy.context.view_layer.objects.active = armature

    def get_bone_color(bone_name: str) -> Tuple[float, float, float]:
        """Determine color for a bone based on its name and the color scheme."""
        if color_scheme == "per_bone" and custom_colors:
            if bone_name in custom_colors:
                c = custom_colors[bone_name]
                return (c.get("r", 1.0), c.get("g", 1.0), c.get("b", 1.0))
            # Fall back to standard for unlisted bones
            return BONE_COLORS["center"]

        if color_scheme == "custom" and custom_colors:
            if bone_name.endswith("_l") or bone_name.endswith("_L"):
                c = custom_colors.get("left", {})
                return (c.get("r", 0.2), c.get("g", 0.4), c.get("b", 1.0))
            elif bone_name.endswith("_r") or bone_name.endswith("_R"):
                c = custom_colors.get("right", {})
                return (c.get("r", 1.0), c.get("g", 0.3), c.get("b", 0.3))
            else:
                c = custom_colors.get("center", {})
                return (c.get("r", 1.0), c.get("g", 0.9), c.get("b", 0.2))

        # Standard scheme
        if bone_name.endswith("_l") or bone_name.endswith("_L"):
            return BONE_COLORS["left"]
        elif bone_name.endswith("_r") or bone_name.endswith("_R"):
            return BONE_COLORS["right"]
        else:
            return BONE_COLORS["center"]

    # Apply colors to bones
    # Blender 4.0+ uses bone.color for individual bone colors
    try:
        if hasattr(armature.data.bones[0], 'color') if armature.data.bones else False:
            # Blender 4.0+ bone color API
            for bone in armature.data.bones:
                color = get_bone_color(bone.name)
                bone.color.palette = 'CUSTOM'
                bone.color.custom.normal = (*color, 1.0)  # RGBA
                bone.color.custom.select = tuple(min(c + 0.2, 1.0) for c in color) + (1.0,)
                bone.color.custom.active = tuple(min(c + 0.4, 1.0) for c in color) + (1.0,)
    except (AttributeError, IndexError):
        # Fallback for older Blender versions using bone groups with colors
        if hasattr(armature.pose, 'bone_groups'):
            bpy.ops.object.mode_set(mode='POSE')

            # Create color groups
            color_groups = {}
            for color_name, color_value in BONE_COLORS.items():
                group = armature.pose.bone_groups.get(f"Color_{color_name}")
                if not group:
                    group = armature.pose.bone_groups.new(name=f"Color_{color_name}")
                group.color_set = 'CUSTOM'
                # Set theme colors
                group.colors.normal = color_value
                group.colors.select = tuple(min(c + 0.2, 1.0) for c in color_value)
                group.colors.active = tuple(min(c + 0.4, 1.0) for c in color_value)
                color_groups[color_name] = group

            # Assign bones to color groups
            for pose_bone in armature.pose.bones:
                color = get_bone_color(pose_bone.name)
                if color == BONE_COLORS["left"]:
                    pose_bone.bone_group = color_groups["left"]
                elif color == BONE_COLORS["right"]:
                    pose_bone.bone_group = color_groups["right"]
                else:
                    pose_bone.bone_group = color_groups["center"]

            bpy.ops.object.mode_set(mode='OBJECT')


# =============================================================================
# Widget Assignment
# =============================================================================

def assign_widget_to_bone(
    armature: 'bpy.types.Object',
    bone_name: str,
    widget_style: str,
    widgets: Dict[str, 'bpy.types.Object']
) -> bool:
    """
    Assign a widget shape to a specific bone.

    Args:
        armature: The armature object.
        bone_name: Name of the bone to assign the widget to.
        widget_style: One of "wire_circle", "wire_cube", "wire_sphere",
                     "wire_diamond", or "custom_mesh".
        widgets: Dictionary of created widget objects.

    Returns:
        True if widget was assigned successfully.
    """
    if widget_style not in widgets:
        print(f"Warning: Unknown widget style '{widget_style}'")
        return False

    widget = widgets[widget_style]
    pose_bone = armature.pose.bones.get(bone_name)

    if not pose_bone:
        print(f"Warning: Bone '{bone_name}' not found in armature")
        return False

    pose_bone.custom_shape = widget
    return True


# =============================================================================
# Main Entry Point
# =============================================================================

def apply_animator_rig_config(
    armature: 'bpy.types.Object',
    animator_rig_config: Dict
) -> Dict[str, Any]:
    """
    Apply animator rig configuration to an armature.

    This is the main entry point for setting up visual aids for animators.

    Args:
        armature: The armature object.
        animator_rig_config: Configuration dictionary with keys:
            - collections: bool - Whether to organize bone collections
            - shapes: bool - Whether to add custom bone shapes
            - colors: bool - Whether to color-code bones
            - display: str - Armature display type (OCTAHEDRAL, STICK, etc.)
            - widget_style: str - Default widget style for control bones
            - bone_collections: list - Custom bone collection definitions
            - bone_colors: dict - Bone color scheme configuration

    Returns:
        Dictionary with results from each operation.
    """
    result = {
        "widgets_created": False,
        "collections_created": False,
        "colors_applied": False,
    }

    # Set armature display type
    display_type = animator_rig_config.get("display", "OCTAHEDRAL")
    armature.data.display_type = display_type

    # Create widget shapes if enabled
    widgets = {}
    if animator_rig_config.get("shapes", True):
        widgets = create_widget_shapes(armature)
        result["widgets_created"] = True
        widgets_assigned = 0
        widgets_failed = 0

        # Assign default widget to control bones
        widget_style = animator_rig_config.get("widget_style", "wire_circle")
        for bone in armature.data.bones:
            # Assign widgets to IK targets (diamond) and other controls (specified style)
            success = False
            if bone.name.startswith("ik_"):
                success = assign_widget_to_bone(armature, bone.name, "wire_diamond", widgets)
            elif bone.name.startswith("pole_"):
                success = assign_widget_to_bone(armature, bone.name, "wire_sphere", widgets)
            elif not bone.name.startswith(("def_", "mch_", "deform_", "mechanism_")):
                success = assign_widget_to_bone(armature, bone.name, widget_style, widgets)
            else:
                continue  # Skip deform/mechanism bones, don't count as failure

            if success:
                widgets_assigned += 1
            else:
                widgets_failed += 1

        result["widgets_assigned"] = widgets_assigned
        if widgets_failed > 0:
            print(f"Warning: {widgets_failed} widget assignments failed")

    # Organize bone collections if enabled
    if animator_rig_config.get("collections", True):
        collections_config = animator_rig_config.get("bone_collections", None)
        organize_bone_collections(armature, collections_config)
        result["collections_created"] = True

    # Apply bone colors if enabled
    if animator_rig_config.get("colors", True):
        bone_colors = animator_rig_config.get("bone_colors", {})
        scheme = bone_colors.get("scheme", "standard")

        custom_colors = None
        if scheme == "custom":
            custom_colors = {
                "left": bone_colors.get("left", {}),
                "right": bone_colors.get("right", {}),
                "center": bone_colors.get("center", {}),
            }
        elif scheme == "per_bone":
            custom_colors = bone_colors.get("colors", {})

        apply_bone_colors(armature, scheme, custom_colors)
        result["colors_applied"] = True

    return result
