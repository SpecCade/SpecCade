# SpecCade Starlark Standard Library - Mesh Functions

[← Back to Index](stdlib-reference.md)

> **SSOT:** For complete parameter details, use `speccade stdlib dump --format json`
> or see the Rust types in `crates/speccade-spec/src/recipe/mesh/`.

## Primitives

| Function | Description |
|----------|-------------|
| `mesh_primitive(primitive, dimensions)` | Base primitive (cube, sphere, cylinder, cone, torus, plane, ico_sphere) |
| `mesh_recipe(primitive, dimensions, modifiers)` | Complete mesh recipe params |

## Modifiers

| Function | Description |
|----------|-------------|
| `bevel_modifier(width, segments, angle_limit)` | Bevel edges |
| `subdivision_modifier(levels, render_levels)` | Subdivision surface |
| `decimate_modifier(ratio)` | Polygon reduction |
| `edge_split_modifier(angle)` | Split sharp edges |
| `mirror_modifier(axis_x, axis_y, axis_z)` | Mirror geometry |
| `array_modifier(count, offset)` | Array copies |
| `solidify_modifier(thickness, offset)` | Shell/thickness |

[← Back to Index](stdlib-reference.md)
