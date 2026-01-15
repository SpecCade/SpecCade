# Blender Mesh Backends

SpecCade uses Blender for 3D asset generation (Tier 2 backends). Output is GLB (glTF Binary) format. These backends are metric-validated, not byte-identical.

## Requirements

- Blender 3.x or 4.x installed
- `blender` command in PATH
- Run `speccade doctor` to verify setup

## Asset Types

### Static Mesh
Non-animated 3D models.

```json
{
  "asset_type": "static_mesh",
  "recipe": {
    "kind": "static_mesh.blender_primitives_v1",
    "params": { ... }
  }
}
```

### Skeletal Mesh
Rigged characters with bones.

```json
{
  "asset_type": "skeletal_mesh",
  "recipe": {
    "kind": "skeletal_mesh.blender_rigged_mesh_v1",
    "params": { ... }
  }
}
```

### Skeletal Animation
Animation clips for skeletal meshes.

```json
{
  "asset_type": "skeletal_animation",
  "recipe": {
    "kind": "skeletal_animation.blender_clip_v1",
    "params": { ... }
  }
}
```

## Static Mesh Primitives

### Available Primitives

```json
{
  "recipe": {
    "kind": "static_mesh.blender_primitives_v1",
    "params": {
      "primitive": "cube",   // See types below
      "size": [1.0, 1.0, 1.0],
      "subdivisions": 1,
      "smooth_shading": true,
      "material": { ... }
    }
  }
}
```

**Primitive types:**
- `cube` - Box with size [x, y, z]
- `sphere` - UV sphere with segments/rings
- `cylinder` - Cylinder with radius/depth
- `cone` - Cone with vertices/radius
- `torus` - Donut shape with major/minor radius
- `plane` - Flat quad
- `grid` - Subdivided plane
- `monkey` - Suzanne (test model)

### Primitive Parameters

**Cube:**
```json
{ "primitive": "cube", "size": [1.0, 2.0, 0.5] }
```

**Sphere:**
```json
{
  "primitive": "sphere",
  "radius": 1.0,
  "segments": 32,
  "rings": 16
}
```

**Cylinder:**
```json
{
  "primitive": "cylinder",
  "radius": 0.5,
  "depth": 2.0,
  "vertices": 32,
  "cap_fill": "ngon"  // none, ngon, triangle_fan
}
```

**Torus:**
```json
{
  "primitive": "torus",
  "major_radius": 1.0,
  "minor_radius": 0.25,
  "major_segments": 48,
  "minor_segments": 12
}
```

### Materials

Basic PBR material setup:

```json
{
  "material": {
    "name": "Metal",
    "base_color": [0.8, 0.8, 0.8, 1.0],
    "metallic": 1.0,
    "roughness": 0.3,
    "emission": [0.0, 0.0, 0.0, 1.0],
    "emission_strength": 0.0
  }
}
```

## Skeletal Mesh

Rigged character mesh with armature.

```json
{
  "recipe": {
    "kind": "skeletal_mesh.blender_rigged_mesh_v1",
    "params": {
      "mesh": {
        "primitive": "cube",
        "size": [1.0, 2.0, 0.5]
      },
      "armature": {
        "bones": [
          { "name": "root", "head": [0, 0, 0], "tail": [0, 0, 1], "parent": null },
          { "name": "spine", "head": [0, 0, 1], "tail": [0, 0, 2], "parent": "root" },
          { "name": "head", "head": [0, 0, 2], "tail": [0, 0, 2.5], "parent": "spine" }
        ]
      },
      "auto_weights": true  // Automatic weight painting
    }
  }
}
```

### Bone Structure

```json
{
  "bones": [
    {
      "name": "bone_name",
      "head": [x, y, z],      // Bone start position
      "tail": [x, y, z],      // Bone end position
      "parent": "parent_name", // null for root
      "connected": false,     // Connect to parent tail
      "inherit_rotation": true,
      "inherit_scale": "full" // full, fix_shear, none
    }
  ]
}
```

## Skeletal Animation

Animation clips for rigged meshes.

```json
{
  "recipe": {
    "kind": "skeletal_animation.blender_clip_v1",
    "params": {
      "name": "walk_cycle",
      "fps": 30,
      "frame_start": 0,
      "frame_end": 30,
      "loop": true,
      "keyframes": [
        {
          "bone": "spine",
          "frame": 0,
          "location": [0, 0, 0],
          "rotation_euler": [0, 0, 0],
          "scale": [1, 1, 1]
        },
        {
          "bone": "spine",
          "frame": 15,
          "rotation_euler": [0.1, 0, 0]
        }
      ]
    }
  }
}
```

### Keyframe Types

**Transform keyframe:**
```json
{
  "bone": "bone_name",
  "frame": 10,
  "location": [x, y, z],        // Optional
  "rotation_euler": [x, y, z],  // Optional (radians)
  "rotation_quaternion": [w, x, y, z], // Alternative to euler
  "scale": [x, y, z]            // Optional
}
```

**Interpolation:**
```json
{
  "bone": "spine",
  "frame": 0,
  "rotation_euler": [0, 0, 0],
  "interpolation": "bezier"  // constant, linear, bezier
}
```

## Validation Metrics

Tier 2 backends validate by metrics, not byte-identical:

```json
{
  "validation": {
    "triangle_count": { "min": 100, "max": 10000 },
    "bounds": {
      "min": [-2, -2, -2],
      "max": [2, 2, 2]
    },
    "bone_count": { "min": 1, "max": 100 }
  }
}
```

## Example: Simple Character

```json
{
  "spec_version": 1,
  "asset_id": "simple_character",
  "asset_type": "skeletal_mesh",
  "license": "CC0-1.0",
  "seed": 100,
  "outputs": [{ "kind": "primary", "format": "glb", "path": "character.glb" }],
  "recipe": {
    "kind": "skeletal_mesh.blender_rigged_mesh_v1",
    "params": {
      "mesh": {
        "primitive": "cube",
        "size": [0.5, 0.3, 1.8]
      },
      "armature": {
        "bones": [
          { "name": "root", "head": [0, 0, 0], "tail": [0, 0, 0.2], "parent": null },
          { "name": "spine_01", "head": [0, 0, 0.2], "tail": [0, 0, 0.6], "parent": "root" },
          { "name": "spine_02", "head": [0, 0, 0.6], "tail": [0, 0, 1.0], "parent": "spine_01" },
          { "name": "neck", "head": [0, 0, 1.0], "tail": [0, 0, 1.2], "parent": "spine_02" },
          { "name": "head", "head": [0, 0, 1.2], "tail": [0, 0, 1.5], "parent": "neck" }
        ]
      },
      "auto_weights": true,
      "material": {
        "name": "Skin",
        "base_color": [0.9, 0.75, 0.65, 1.0],
        "roughness": 0.8
      }
    }
  }
}
```

## Troubleshooting

**"Blender not found"**: Add Blender to PATH or set `BLENDER_PATH` env var.

**"Generation timeout"**: Complex meshes may need longer timeout. Use `--timeout` flag.

**"Metric validation failed"**: Check triangle count and bounds match expectations.

**"Different output each run"**: Tier 2 is not byte-identical. Use metric validation instead.
