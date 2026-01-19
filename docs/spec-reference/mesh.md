# Static Mesh Specs

Static meshes are non-deforming 3D geometry for props, environments, and other objects.

| Property | Value |
|----------|-------|
| Asset Type | `static_mesh` |
| Recipe Kind | `static_mesh.blender_primitives_v1` |
| Output Formats | `glb` |
| Determinism | Tier 2 (metric validation) |

## SSOT (Source Of Truth)

- Rust types: `crates/speccade-spec/src/recipe/mesh/`
- Golden specs: `golden/speccade/specs/static_mesh/`
- CLI validation: `speccade validate --spec file.json`

## Recipe Parameters

The `static_mesh.blender_primitives_v1` recipe builds meshes from Blender primitives with modifiers.

### Main Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `base_primitive` | string | Yes | Base mesh primitive (see primitives below) |
| `dimensions` | `[f64; 3]` | Yes | Dimensions [X, Y, Z] in Blender units |
| `modifiers` | array | No | List of modifiers to apply (see modifiers below) |
| `uv_projection` | string/object | No | UV unwrapping method (see UV projection below) |
| `normals` | object | No | Normals automation settings (see normals below) |
| `material_slots` | array | No | Material definitions (see materials below) |
| `export` | object | No | GLB export settings (see export below) |
| `constraints` | object | No | Validation constraints (see constraints below) |
| `lod_chain` | object | No | LOD chain settings for multi-LOD export (see LOD chain below) |

### Primitives

| Value | Description |
|-------|-------------|
| `cube` | Cube/box |
| `sphere` | UV sphere |
| `cylinder` | Cylinder |
| `cone` | Cone |
| `torus` | Torus |
| `plane` | Plane |
| `ico_sphere` | Icosphere |

### Modifiers

Modifiers are applied in order. Each modifier has a `type` field.

#### Bevel

```json
{
  "type": "bevel",
  "width": 0.02,
  "segments": 2,
  "angle_limit": 0.785
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `width` | f64 | Yes | Bevel width |
| `segments` | u8 | Yes | Number of segments |
| `angle_limit` | f64 | No | Angle limit in radians (only bevel edges below this angle) |

#### Subdivision

```json
{
  "type": "subdivision",
  "levels": 2,
  "render_levels": 3
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `levels` | u8 | Yes | Subdivision levels for viewport |
| `render_levels` | u8 | Yes | Subdivision levels for render |

#### Decimate

```json
{
  "type": "decimate",
  "ratio": 0.5
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `ratio` | f64 | Yes | Decimate ratio (0.0 to 1.0) |

#### Mirror

```json
{
  "type": "mirror",
  "axis_x": true,
  "axis_y": false,
  "axis_z": false
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `axis_x` | bool | false | Mirror along X axis |
| `axis_y` | bool | false | Mirror along Y axis |
| `axis_z` | bool | false | Mirror along Z axis |

#### Array

```json
{
  "type": "array",
  "count": 5,
  "offset": [1.0, 0.0, 0.0]
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `count` | u32 | Yes | Number of copies |
| `offset` | `[f64; 3]` | Yes | Offset between copies |

#### Solidify

```json
{
  "type": "solidify",
  "thickness": 0.1,
  "offset": 0.0
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `thickness` | f64 | Yes | Shell thickness |
| `offset` | f64 | Yes | Offset (-1 to 1) |

#### Edge Split

```json
{
  "type": "edge_split",
  "angle": 30.0
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `angle` | f64 | Yes | Split angle in degrees |

### UV Projection

UV projection can be a simple string or an object with settings.

**Simple form:**

```json
"uv_projection": "smart"
```

**Object form:**

```json
"uv_projection": {
  "method": "smart",
  "angle_limit": 66.0,
  "cube_size": 1.0
}
```

**Extended form with texel density and lightmap UVs:**

```json
"uv_projection": {
  "method": "smart",
  "angle_limit": 66.0,
  "texel_density": 512.0,
  "uv_margin": 0.002,
  "lightmap_uv": true
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `method` | string | `smart` | UV projection method (see methods below) |
| `angle_limit` | f64 | - | Angle limit in degrees (for smart projection) |
| `cube_size` | f64 | - | Cube size (for box projection) |
| `texel_density` | f64 | - | Target texel density in pixels per world unit |
| `uv_margin` | f64 | 0.001 | UV island margin/padding (0.0 to 1.0) |
| `lightmap_uv` | bool | false | Generate secondary UV channel for lightmaps |

| Method | Description |
|--------|-------------|
| `box` | Box/cube projection |
| `cylinder` | Cylinder projection |
| `sphere` | Sphere projection |
| `smart` | Smart UV project |
| `lightmap` | Lightmap pack |

#### Texel Density

The `texel_density` parameter specifies the target pixel-per-unit ratio for the UVs. When specified, UVs are scaled after unwrapping to achieve the target density. This helps ensure consistent texture resolution across different meshes.

For example, a `texel_density` of 512.0 means 512 pixels per world unit (assuming a 1024x1024 texture).

#### Lightmap UVs

When `lightmap_uv` is true, a secondary UV channel named "UVMap_Lightmap" is generated using lightmap packing. This is useful for:
- Lightmap baking (static lighting)
- Ambient occlusion maps
- Any per-surface data that needs non-overlapping UVs

### Normals Settings

Normals automation settings control how vertex normals are calculated for the mesh.

```json
"normals": {
  "preset": "auto_smooth",
  "angle": 30.0,
  "keep_sharp": true
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `preset` | string | Yes | Normals preset (see presets below) |
| `angle` | f64 | No | Angle threshold in degrees (default: 30.0) |
| `keep_sharp` | bool | No | Preserve existing sharp edges (default: true) |

#### Normals Presets

| Preset | Description |
|--------|-------------|
| `flat` | Flat shading - each face has its own normal direction, creating a faceted appearance |
| `smooth` | Smooth shading - normals are interpolated across faces for a smooth appearance |
| `auto_smooth` | Auto-smooth based on angle threshold - edges sharper than the angle are hard, others are smooth |
| `weighted_normals` | Weighted normals based on face area - larger faces contribute more to vertex normals |
| `hard_edge_by_angle` | Mark edges as sharp if angle exceeds threshold, then apply smooth shading |

#### Examples

**Flat shading for low-poly style:**

```json
"normals": {
  "preset": "flat"
}
```

**Auto-smooth with 45 degree threshold:**

```json
"normals": {
  "preset": "auto_smooth",
  "angle": 45.0
}
```

**Weighted normals for hard-surface models:**

```json
"normals": {
  "preset": "weighted_normals",
  "keep_sharp": true
}
```

### Material Slots

```json
"material_slots": [
  {
    "name": "Metal",
    "base_color": [0.8, 0.8, 0.9, 1.0],
    "metallic": 1.0,
    "roughness": 0.2
  }
]
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Material name |
| `base_color` | `[f64; 4]` | No | RGBA color (0.0 to 1.0) |
| `metallic` | f64 | No | Metallic value (0.0 to 1.0) |
| `roughness` | f64 | No | Roughness value (0.0 to 1.0) |
| `emissive` | `[f64; 3]` | No | Emissive RGB color |
| `emissive_strength` | f64 | No | Emissive strength |

### Export Settings

```json
"export": {
  "apply_modifiers": true,
  "triangulate": true,
  "include_normals": true,
  "include_uvs": true,
  "include_vertex_colors": false,
  "tangents": false
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `apply_modifiers` | bool | true | Apply modifiers before export |
| `triangulate` | bool | true | Triangulate mesh |
| `include_normals` | bool | true | Include vertex normals |
| `include_uvs` | bool | true | Include UV coordinates |
| `include_vertex_colors` | bool | false | Include vertex colors |
| `tangents` | bool | false | Export tangents for normal mapping |

### Constraints

Constraints define validation limits. Reports include metrics for verification.

```json
"constraints": {
  "max_triangles": 1000,
  "max_materials": 4,
  "max_vertices": 2000
}
```

| Field | Type | Description |
|-------|------|-------------|
| `max_triangles` | u32 | Maximum triangle count |
| `max_materials` | u32 | Maximum material count |
| `max_vertices` | u32 | Maximum vertex count |

### LOD Chain

LOD (Level of Detail) chain settings enable generation of multiple mesh LODs at different triangle counts. Each LOD is exported as a separate mesh in the GLB file (e.g., "Mesh_LOD0", "Mesh_LOD1", etc.).

```json
"lod_chain": {
  "levels": [
    { "level": 0, "target_tris": null },
    { "level": 1, "target_tris": 500 },
    { "level": 2, "target_tris": 100 }
  ],
  "decimate_method": "collapse"
}
```

#### LOD Chain Settings

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `levels` | array | Yes | List of LOD level specifications |
| `decimate_method` | string | No | Decimation method: `collapse` (default) or `planar` |

#### LOD Level Specification

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `level` | u8 | Yes | LOD level index (0 = highest detail) |
| `target_tris` | u32 | No | Target triangle count. If `null`, mesh is not decimated (original) |

#### Decimation Methods

| Method | Description |
|--------|-------------|
| `collapse` | Edge collapse decimation (default). Best quality for organic meshes. |
| `planar` | Planar decimation (dissolve). Good for architectural/mechanical meshes with flat surfaces. |

#### LOD Generation Notes

- LOD0 should typically have `target_tris: null` to preserve the original mesh
- The decimator targets the specified triangle count but may not hit it exactly
- UVs and materials are preserved across all LOD levels
- Each LOD is exported as a separate mesh object in the GLB file

#### Example with LODs

```json
{
  "spec_version": 1,
  "asset_id": "prop_with_lods",
  "asset_type": "static_mesh",
  "license": "CC0-1.0",
  "seed": 7001,
  "outputs": [
    { "kind": "primary", "format": "glb", "path": "prop_with_lods.glb" }
  ],
  "recipe": {
    "kind": "static_mesh.blender_primitives_v1",
    "params": {
      "base_primitive": "ico_sphere",
      "dimensions": [1.0, 1.0, 1.0],
      "modifiers": [
        { "type": "subdivision", "levels": 2, "render_levels": 2 }
      ],
      "lod_chain": {
        "levels": [
          { "level": 0, "target_tris": null },
          { "level": 1, "target_tris": 500 },
          { "level": 2, "target_tris": 100 }
        ]
      }
    }
  }
}
```

## Example Spec

```json
{
  "spec_version": 1,
  "asset_id": "simple_cube",
  "asset_type": "static_mesh",
  "license": "CC0-1.0",
  "seed": 6001,
  "description": "Simple beveled cube",
  "outputs": [
    {
      "kind": "primary",
      "format": "glb",
      "path": "simple_cube.glb"
    }
  ],
  "recipe": {
    "kind": "static_mesh.blender_primitives_v1",
    "params": {
      "base_primitive": "cube",
      "dimensions": [1.0, 1.0, 1.0],
      "modifiers": [
        {
          "type": "bevel",
          "width": 0.02,
          "segments": 2
        }
      ],
      "uv_projection": "box",
      "normals": {
        "preset": "auto_smooth",
        "angle": 30.0
      },
      "export": {
        "apply_modifiers": true,
        "triangulate": true,
        "include_normals": true,
        "include_uvs": true
      }
    }
  }
}
```

## Output Metrics

Generation produces a report with mesh metrics:

| Metric | Description |
|--------|-------------|
| `vertex_count` | Number of vertices |
| `face_count` | Number of faces |
| `triangle_count` | Number of triangles |
| `quad_count` | Number of quads |
| `quad_percentage` | Percentage of quads (0-100) |
| `manifold` | Whether mesh is watertight |
| `non_manifold_edge_count` | Non-manifold edge count |
| `degenerate_face_count` | Degenerate face count |
| `uv_island_count` | Number of UV islands |
| `uv_coverage` | UV coverage ratio (0-1) |
| `uv_overlap_percentage` | UV overlap percentage (0-100) |
| `uv_layer_count` | Number of UV layers |
| `texel_density` | Average texel density (pixels per world unit at 1024x1024) |
| `bounding_box` | Axis-aligned bounding box |
| `material_slot_count` | Number of materials |

### LOD-Specific Metrics

When `lod_chain` is specified, the report includes additional per-LOD metrics:

| Metric | Description |
|--------|-------------|
| `lod_count` | Total number of LOD levels generated |
| `lod_levels` | Array of per-LOD metrics (see below) |

Each entry in `lod_levels` includes:

| Metric | Description |
|--------|-------------|
| `lod_level` | LOD level index (0, 1, 2, ...) |
| `vertex_count` | Vertex count for this LOD |
| `face_count` | Face count for this LOD |
| `triangle_count` | Triangle count for this LOD |
| `target_tris` | Target triangle count (if specified) |
| `simplification_ratio` | Actual ratio vs original (1.0 = no reduction) |
| `bounding_box` | Bounding box for this LOD |

## Post-Generation Verification

Use `speccade verify` to validate metrics against constraints:

```bash
speccade verify --report output.report.json --constraints constraints.json
```

See [../README.md](../README.md) for constraint definitions.

## See Also

- [Character Specs](character.md) - Skeletal meshes with armatures
- [Animation Specs](animation.md) - Skeletal animations
