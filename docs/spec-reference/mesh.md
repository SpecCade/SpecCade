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
| `collision_mesh` | object | No | Collision mesh generation settings (see collision mesh below) |
| `navmesh` | object | No | Navmesh analysis settings for walkability metadata (see navmesh below) |
| `baking` | object | No | Texture baking settings for normal/AO/curvature maps (see baking below) |

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

### Collision Mesh

Collision mesh settings enable generation of simplified collision geometry alongside the primary mesh. The collision mesh is exported as a separate GLB file with a configurable suffix.

```json
"collision_mesh": {
  "collision_type": "convex_hull",
  "output_suffix": "_col"
}
```

#### Collision Mesh Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `collision_type` | string | `convex_hull` | Type of collision mesh to generate (see types below) |
| `target_faces` | u32 | - | Target face count for simplified mesh type only |
| `output_suffix` | string | `_col` | Suffix appended to output filename for collision mesh |

#### Collision Types

| Type | Description |
|------|-------------|
| `convex_hull` | Convex hull collision (default). Fast collision detection, wraps around the mesh. Good for most props. |
| `simplified_mesh` | Simplified mesh collision using decimation. Preserves general shape with reduced triangles. Good for complex concave objects. |
| `box` | Axis-aligned bounding box collision. Fastest collision detection but least accurate. Good for rectangular objects. |

#### Collision Mesh Examples

**Convex hull (default):**

```json
"collision_mesh": {
  "collision_type": "convex_hull",
  "output_suffix": "_col"
}
```

**Simplified mesh with target face count:**

```json
"collision_mesh": {
  "collision_type": "simplified_mesh",
  "target_faces": 64,
  "output_suffix": "_col"
}
```

**Box collision:**

```json
"collision_mesh": {
  "collision_type": "box",
  "output_suffix": "_box"
}
```

#### Example with Collision Mesh

```json
{
  "spec_version": 1,
  "asset_id": "prop_with_collision",
  "asset_type": "static_mesh",
  "license": "CC0-1.0",
  "seed": 7002,
  "outputs": [
    { "kind": "primary", "format": "glb", "path": "prop.glb" }
  ],
  "recipe": {
    "kind": "static_mesh.blender_primitives_v1",
    "params": {
      "base_primitive": "ico_sphere",
      "dimensions": [1.0, 1.0, 1.0],
      "modifiers": [
        { "type": "subdivision", "levels": 2, "render_levels": 2 }
      ],
      "collision_mesh": {
        "collision_type": "convex_hull",
        "output_suffix": "_col"
      }
    }
  }
}
```

This will generate two files:
- `prop.glb` - The primary mesh (high detail)
- `prop_col.glb` - The collision mesh (convex hull)

#### Collision Mesh Notes

- The collision mesh is generated from the primary mesh before LOD chain processing
- Materials and UVs are not included in collision mesh exports
- The collision mesh filename uses the primary output path with the configured suffix
- Box collision is the fastest but least accurate; convex hull provides a good balance
- For complex concave objects, use `simplified_mesh` with an appropriate `target_faces` count

### Navmesh

Navmesh settings enable analysis of mesh geometry for walkability classification. This produces metadata about which surfaces are walkable (based on slope angle) and optionally detects potential stair geometry.

**Note:** This produces classification metadata only, not actual navmesh/pathfinding mesh generation. Use this metadata to inform game engine navmesh baking or level design.

```json
"navmesh": {
  "walkable_slope_max": 45.0,
  "stair_detection": true,
  "stair_step_height": 0.3
}
```

#### Navmesh Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `walkable_slope_max` | f64 | 45.0 | Maximum slope angle in degrees for a surface to be considered walkable |
| `stair_detection` | bool | false | Enable detection of potential stair geometry |
| `stair_step_height` | f64 | - | Step height threshold for stair detection (only used if stair_detection is true) |

#### Walkability Classification

A face is classified as walkable if its surface normal is within `walkable_slope_max` degrees of vertical (Z-up). Faces with steeper slopes are classified as non-walkable.

For example, with the default `walkable_slope_max: 45.0`:
- A perfectly horizontal floor (0 degrees from vertical) is walkable
- A 30-degree ramp is walkable
- A 60-degree slope is non-walkable
- A vertical wall (90 degrees) is non-walkable

#### Stair Detection

When `stair_detection` is enabled, the analyzer looks for clusters of walkable surfaces at regular height intervals matching the `stair_step_height` threshold. This helps identify potential stair geometry in the mesh.

#### Navmesh Examples

**Basic walkability analysis:**

```json
"navmesh": {
  "walkable_slope_max": 45.0
}
```

**Strict walkability with stair detection:**

```json
"navmesh": {
  "walkable_slope_max": 30.0,
  "stair_detection": true,
  "stair_step_height": 0.25
}
```

#### Example with Navmesh Analysis

```json
{
  "spec_version": 1,
  "asset_id": "level_floor",
  "asset_type": "static_mesh",
  "license": "CC0-1.0",
  "seed": 7003,
  "outputs": [
    { "kind": "primary", "format": "glb", "path": "level_floor.glb" }
  ],
  "recipe": {
    "kind": "static_mesh.blender_primitives_v1",
    "params": {
      "base_primitive": "plane",
      "dimensions": [10.0, 10.0, 0.0],
      "navmesh": {
        "walkable_slope_max": 45.0,
        "stair_detection": false
      }
    }
  }
}
```

#### Navmesh Notes

- Navmesh analysis is performed on the original mesh before collision mesh or LOD chain processing
- The analysis uses world-space face normals, accounting for object rotation
- Stair detection uses height clustering to identify step-like patterns
- Results are metadata only; actual navmesh generation requires game engine tools (e.g., Recast/Detour)

### Baking

Baking settings enable generation of texture maps (normal, AO, curvature) from the mesh geometry. Uses Cycles CPU renderer for deterministic output.

```json
"baking": {
  "bake_types": ["normal", "ao"],
  "ray_distance": 0.1,
  "margin": 16,
  "resolution": [1024, 1024]
}
```

#### Baking Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `bake_types` | array | - | Types of maps to bake (required, see types below) |
| `ray_distance` | f64 | 0.1 | Ray casting distance for high-to-low baking |
| `margin` | u32 | 16 | Dilation in pixels for mip-safe edges |
| `resolution` | `[u32; 2]` | `[1024, 1024]` | Resolution of baked textures [width, height] |
| `high_poly_source` | string | - | Optional path to high-poly mesh for baking |

#### Bake Types

| Type | Description |
|------|-------------|
| `normal` | Tangent-space normal map |
| `ao` | Ambient occlusion map |
| `curvature` | Curvature map (convex/concave edges via Geometry Pointiness) |
| `combined` | Combined/full lighting bake |

#### Baking Examples

**Normal and AO baking:**

```json
"baking": {
  "bake_types": ["normal", "ao"],
  "ray_distance": 0.1,
  "margin": 16,
  "resolution": [1024, 1024]
}
```

**High-to-low baking with external high-poly source:**

```json
"baking": {
  "bake_types": ["normal"],
  "ray_distance": 0.2,
  "margin": 32,
  "resolution": [2048, 2048],
  "high_poly_source": "meshes/sculpt_high.glb"
}
```

**Curvature baking for edge wear/highlighting:**

```json
"baking": {
  "bake_types": ["curvature"],
  "margin": 16,
  "resolution": [512, 512]
}
```

#### Example with Baking

```json
{
  "spec_version": 1,
  "asset_id": "baked_prop",
  "asset_type": "static_mesh",
  "license": "CC0-1.0",
  "seed": 7004,
  "outputs": [
    { "kind": "primary", "format": "glb", "path": "prop.glb" }
  ],
  "recipe": {
    "kind": "static_mesh.blender_primitives_v1",
    "params": {
      "base_primitive": "cube",
      "dimensions": [1.0, 1.0, 1.0],
      "modifiers": [
        { "type": "bevel", "width": 0.05, "segments": 3 }
      ],
      "uv_projection": "smart",
      "baking": {
        "bake_types": ["normal", "ao"],
        "ray_distance": 0.1,
        "margin": 16,
        "resolution": [1024, 1024]
      }
    }
  }
}
```

This will generate:
- `prop.glb` - The primary mesh
- `baked_prop_normal.png` - Normal map (1024x1024)
- `baked_prop_ao.png` - Ambient occlusion map (1024x1024)

#### Baking Notes

- Baking is performed before LOD chain or collision mesh generation
- If the mesh has no UVs, smart UV projection is applied automatically
- Uses Cycles CPU renderer for deterministic output
- Margin (dilation) extends texture edges for mip-safe sampling
- For high-to-low baking, the high_poly_source mesh is imported, baked, then removed
- Curvature is baked using Blender's Geometry > Pointiness attribute

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

### Collision Mesh Metrics

When `collision_mesh` is specified, the report includes collision mesh metrics:

| Metric | Description |
|--------|-------------|
| `collision_mesh_path` | Filename of the exported collision mesh |
| `collision_mesh` | Object containing collision mesh metrics (see below) |

The `collision_mesh` object includes:

| Metric | Description |
|--------|-------------|
| `collision_type` | Type of collision mesh generated |
| `output_suffix` | Suffix used for the collision mesh filename |
| `vertex_count` | Vertex count of the collision mesh |
| `face_count` | Face count of the collision mesh |
| `triangle_count` | Triangle count of the collision mesh |
| `target_faces` | Target face count (for simplified_mesh type only) |
| `bounding_box` | Bounding box of the collision mesh |

### Navmesh Metrics

When `navmesh` is specified, the report includes walkability analysis metrics:

| Metric | Description |
|--------|-------------|
| `navmesh` | Object containing navmesh analysis metrics (see below) |

The `navmesh` object includes:

| Metric | Description |
|--------|-------------|
| `walkable_face_count` | Number of faces classified as walkable |
| `non_walkable_face_count` | Number of faces classified as non-walkable |
| `walkable_percentage` | Percentage of faces that are walkable (0-100) |
| `stair_candidates` | Number of potential stair surfaces detected (only if stair_detection enabled) |

### Baking Metrics

When `baking` is specified, the report includes baked texture map metrics:

| Metric | Description |
|--------|-------------|
| `baking` | Object containing baking metrics (see below) |

The `baking` object includes:

| Metric | Description |
|--------|-------------|
| `baked_maps` | Array of baked map info (type, path, resolution) |
| `ray_distance` | Ray distance used for baking |
| `margin` | Margin (dilation) in pixels used |

Each entry in `baked_maps` includes:

| Metric | Description |
|--------|-------------|
| `type` | Bake type (normal, ao, curvature, combined) |
| `path` | Output filename of the baked texture |
| `resolution` | [width, height] of the baked texture |

## Post-Generation Verification

Use `speccade verify` to validate metrics against constraints:

```bash
speccade verify --report output.report.json --constraints constraints.json
```

See [../README.md](../README.md) for constraint definitions.

## See Also

- [Character Specs](character.md) - Skeletal meshes with armatures
- [Animation Specs](animation.md) - Skeletal animations
