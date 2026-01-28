# SpecCade Blender Package

Modular Python package for Blender asset generation. Executed via `blender --background --python entrypoint.py`.

## Module Structure

```
speccade/
├── __init__.py           # Package exports (main only)
├── main.py               # CLI parsing + handler dispatch
├── report.py             # Report generation
│
│   # Scene & Primitives
├── scene.py              # clear_scene, setup_scene, create_primitive
├── normals.py            # apply_normals_settings
├── materials.py          # create_material, apply_materials
├── modifiers.py          # apply_modifier, apply_all_modifiers
├── uv_mapping.py         # UV projection, texel density, lightmaps
├── metrics.py            # Mesh/skeletal/animation metrics
│
│   # Skeleton Domain
├── skeleton_presets.py   # HUMANOID_*_BONES constants
├── skeleton.py           # create_armature, apply_skeleton_overrides
├── body_parts.py         # Body part mesh creation, skinning
│
│   # Rigging Domain
├── ik_fk.py              # IK chains, IK/FK switching, rig setup
├── constraints.py        # Bone constraints (hinge, ball, aim, twist)
├── controls.py           # Space switch, fingers, foot system
│
│   # Animation Domain
├── animation.py          # Animation creation, baking, root motion
├── rig_config.py         # Widgets, bone collections, colors
│
│   # Export Domain
├── export.py             # GLB export, LOD chains, collision meshes
├── rendering.py          # Camera, preview frames, atlas packing
│
│   # Handlers (entry points by mode)
├── handlers_mesh.py      # static_mesh, modular_kit, organic_sculpt, boolean_kit
├── handlers_skeletal.py  # skeletal_mesh, animation, rigged_animation
└── handlers_render.py    # mesh_to_sprite, validation_grid
```

## Import Hierarchy

Modules are organized in dependency levels to prevent circular imports:

```
Level 0 (only bpy/stdlib):
    report, scene, normals, skeleton_presets

Level 1 (imports Level 0):
    materials, modifiers, uv_mapping

Level 2 (imports Level 0-1):
    metrics, skeleton, body_parts

Level 3 (imports Level 0-2):
    ik_fk, constraints, controls

Level 4 (imports Level 0-3):
    animation, rig_config

Level 5 (imports Level 0-4):
    export, rendering

Level 6 (imports Level 0-5):
    handlers_mesh, handlers_skeletal, handlers_render

Level 7 (imports Level 6):
    main
```

## Usage

The package is invoked through `entrypoint.py`:

```bash
blender --background --factory-startup --python entrypoint.py -- \
    --mode <mode> --spec <path> --out-root <path> --report <path>
```

Available modes:
- `static_mesh` - Static mesh generation
- `modular_kit` - Wall/pipe/door kit generation
- `organic_sculpt` - Metaball-based organic meshes
- `shrinkwrap` - Armor/clothing wrapping
- `boolean_kit` - Boolean kitbashing
- `skeletal_mesh` - Rigged character meshes
- `animation` - FK animation clips
- `rigged_animation` - Full IK/FK animation with preview
- `mesh_to_sprite` - Sprite sheet from mesh
- `validation_grid` - 6-view validation PNG

## Module Line Counts

| Module | Lines | Target |
|--------|------:|-------:|
| report.py | 44 | 80 |
| scene.py | 59 | 150 |
| materials.py | 62 | 210 |
| modifiers.py | 91 | 320 |
| normals.py | 106 | 100 |
| main.py | 109 | 100 |
| skeleton_presets.py | 165 | 200 |
| uv_mapping.py | 253 | 350 |
| skeleton.py | 284 | 400 |
| controls.py | 298 | 400 |
| constraints.py | 518 | 400 |
| ik_fk.py | 530 | 400 |
| rig_config.py | 549 | 400 |
| metrics.py | 554 | 560 |
| handlers_skeletal.py | 592 | 400 |
| animation.py | 620 | 600 |
| handlers_render.py | 649 | 500 |
| rendering.py | 668 | 400 |
| body_parts.py | 779 | 400 |
| export.py | 1008 | 600 |
| handlers_mesh.py | 1467 | 500 |

## Adding New Functionality

1. Identify the appropriate module based on domain
2. Respect the import hierarchy (don't import from higher levels)
3. Keep functions focused and well-documented
4. Update `handlers_*.py` if adding new modes
5. Update `main.py` to register new handlers
