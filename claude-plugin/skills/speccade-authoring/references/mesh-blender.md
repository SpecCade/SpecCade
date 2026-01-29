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
Rigged characters (armature + mesh).

Recipe kinds:

- `skeletal_mesh.armature_driven_v1`
- `skeletal_mesh.skinned_mesh_v1`

```json
{
  "asset_type": "skeletal_mesh",
  "recipe": {
    "kind": "skeletal_mesh.armature_driven_v1",
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

Skeletal meshes are produced via Blender (Tier 2) and validated by metrics.

Two workflows:

- `skeletal_mesh.armature_driven_v1`: build mesh from a skeleton (rigid skinning)
- `skeletal_mesh.skinned_mesh_v1`: bind an existing mesh to a skeleton (rigid or auto weights)

Minimal recipe example (armature-driven):

```json
{
  "recipe": {
    "kind": "skeletal_mesh.armature_driven_v1",
    "params": {
      "skeleton_preset": "humanoid_basic_v1",
      "bone_meshes": {
        "spine": { "profile": "hexagon(8)", "profile_radius": 0.15, "taper": 0.9 }
      }
    }
  }
}
```

Minimal recipe example (skinned mesh):

```json
{
  "recipe": {
    "kind": "skeletal_mesh.skinned_mesh_v1",
    "params": {
      "mesh_file": "assets/character.glb",
      "skeleton_preset": "humanoid_basic_v1",
      "binding": { "mode": "rigid" }
    }
  }
}
```

Full parameter reference: `docs/spec-reference/character.md`

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
    "kind": "skeletal_mesh.armature_driven_v1",
    "params": {
      "skeleton_preset": "humanoid_basic_v1",
      "bone_meshes": {
        "spine": { "profile": "hexagon(8)", "profile_radius": 0.15, "taper": 0.9 },
        "head": { "profile": "circle(12)", "profile_radius": 0.12 }
      },
      "material_slots": [
        { "name": "skin", "base_color": [0.9, 0.75, 0.65, 1.0], "roughness": 0.8 }
      ]
    }
  }
}
```

## Troubleshooting

**"Blender not found"**: Add Blender to PATH or set `BLENDER_PATH` env var.

**"Generation timeout"**: Complex meshes may need longer timeout. Use `--timeout` flag.

**"Metric validation failed"**: Check triangle count and bounds match expectations.

**"Different output each run"**: Tier 2 is not byte-identical. Use metric validation instead.
