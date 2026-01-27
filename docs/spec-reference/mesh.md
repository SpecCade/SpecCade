# Static Mesh Specs

> **SSOT:** Rust types in `crates/speccade-spec/src/recipe/mesh/`.
> Golden specs: `golden/speccade/specs/static_mesh/`.

| Property | Value |
|----------|-------|
| Asset Type | `static_mesh` |
| Recipe Kind | `static_mesh.blender_primitives_v1` |
| Output Format | `glb` |
| Determinism | Tier 2 (metric validation) |
| Coordinate System | Z-up, Y-forward. Dimensions: [X width, Y depth, Z height] |

## Recipe Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `base_primitive` | string | Yes | cube, sphere, cylinder, cone, torus, plane, ico_sphere |
| `dimensions` | [f64; 3] | Yes | [X, Y, Z] in meters |
| `modifiers` | array | No | Ordered modifier list |
| `uv_projection` | string/object | No | UV method: box, cylinder, sphere, smart, lightmap |
| `normals` | object | No | Preset: flat, smooth, auto_smooth, weighted_normals, hard_edge_by_angle |
| `material_slots` | array | No | Materials (name, base_color, metallic, roughness, emissive) |
| `export` | object | No | GLB export flags |
| `constraints` | object | No | max_triangles, max_materials, max_vertices |
| `lod_chain` | object | No | Multi-LOD export |
| `collision_mesh` | object | No | Collision geometry (convex_hull, simplified_mesh, box) |
| `navmesh` | object | No | Walkability analysis metadata |
| `baking` | object | No | Texture baking (normal, ao, curvature, combined) |

## Modifiers

| Type | Key Params |
|------|-----------|
| `bevel` | width, segments, angle_limit |
| `subdivision` | levels, render_levels |
| `decimate` | ratio |
| `mirror` | axis_x, axis_y, axis_z |
| `array` | count, offset [x,y,z] |
| `solidify` | thickness, offset |
| `edge_split` | angle (degrees) |

## UV Projection

Simple: `"uv_projection": "smart"`. Extended form adds `angle_limit`, `cube_size`, `texel_density`, `uv_margin`, `lightmap_uv`.

## LOD Chain

```json
"lod_chain": {
  "levels": [
    {"level": 0, "target_tris": null},
    {"level": 1, "target_tris": 500}
  ],
  "decimate_method": "collapse"
}
```

Methods: `collapse` (organic), `planar` (architectural).

## Collision Mesh

| Type | Description |
|------|-------------|
| `convex_hull` | Fast, wraps mesh (default) |
| `simplified_mesh` | Decimated, preserves concavity |
| `box` | AABB, fastest |

## Baking

| Bake Type | Description |
|-----------|-------------|
| `normal` | Tangent-space normal map |
| `ao` | Ambient occlusion |
| `curvature` | Convex/concave edges |
| `combined` | Full lighting |

Params: `bake_types`, `ray_distance`, `margin`, `resolution`, `high_poly_source`.

## Output Metrics

Generation produces reports with: vertex_count, face_count, triangle_count, quad_count, manifold, uv_island_count, uv_coverage, texel_density, bounding_box, material_slot_count. Plus per-LOD, collision, navmesh, and baking metrics when enabled.

## See Also

- [Character Specs](character.md) — Skeletal meshes
- [Animation Specs](animation.md) — Skeletal animations
- [Starlark stdlib mesh functions](../stdlib-mesh.md)
